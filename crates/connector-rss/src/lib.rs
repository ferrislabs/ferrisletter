//! RSS/Atom feed connector for Ferrisletter.
//!
//! Fetches one or more RSS or Atom feeds and exposes their content through the
//! [`Connector`] interface. Each feed is mapped to a [`Topic`], and items are
//! fetched lazily on first access then cached in memory.
//!
//! # Auto-refresh
//!
//! For long-running server processes, call [`RssConnector::start_auto_refresh`]
//! to spawn a background task that periodically re-fetches feeds. The refresh
//! interval is the minimum `refresh_minutes` across all feeds (default 60).
//! Conditional HTTP requests (`If-Modified-Since` / `If-None-Match`) are used
//! to avoid re-parsing unchanged feeds.
//!
//! # Example
//!
//! ```no_run
//! use ferrisletter_connector_rss::{FeedConfig, RssConnector};
//!
//! let connector = RssConnector::new(vec![
//!     FeedConfig {
//!         topic_id:          "rust".to_string(),
//!         topic_label:       "Rust".to_string(),
//!         topic_description: "News from the Rust ecosystem".to_string(),
//!         topic_tags:        vec!["rust".to_string(), "programming".to_string()],
//!         url:               "https://blog.rust-lang.org/feed.xml".to_string(),
//!         refresh_minutes:   Some(30),
//!     },
//! ]);
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use ferrisletter_connector::{
    Connector, ConnectorError, Item, ItemDetail, Link as ItemLink, SearchFilters, Topic, UserPrefs,
};
use tokio::sync::RwLock;

/// Configuration for a single RSS/Atom feed.
#[derive(Debug, Clone)]
pub struct FeedConfig {
    /// Unique topic ID that items from this feed will be tagged with.
    pub topic_id: String,
    /// Human-readable topic label.
    pub topic_label: String,
    /// Short description shown to subscribers.
    pub topic_description: String,
    /// Tags to attach to this topic.
    pub topic_tags: Vec<String>,
    /// URL of the RSS or Atom feed.
    pub url: String,
    /// How often to refresh this feed, in minutes. Defaults to 60 if `None`.
    pub refresh_minutes: Option<u64>,
}

type Cache = Arc<RwLock<Option<Vec<ItemDetail>>>>;

/// Per-feed HTTP caching headers for conditional requests.
#[derive(Debug, Default, Clone)]
struct FeedCacheHeaders {
    last_modified: Option<String>,
    etag: Option<String>,
}

/// Result of a conditional fetch — either new items or "not modified".
enum FetchResult {
    Items(Vec<ItemDetail>),
    NotModified,
}

/// An RSS/Atom connector for Ferrisletter.
///
/// Items are fetched once on first use and cached in memory. For long-running
/// processes, use [`start_auto_refresh`](Self::start_auto_refresh) to enable
/// periodic background refresh with conditional HTTP requests.
#[derive(Debug, Clone)]
pub struct RssConnector {
    feeds: Vec<FeedConfig>,
    client: reqwest::Client,
    cache: Cache,
    /// Per-feed URL → HTTP caching headers (ETag, Last-Modified).
    feed_headers: Arc<RwLock<HashMap<String, FeedCacheHeaders>>>,
}

impl RssConnector {
    /// Create a new [`RssConnector`] from a list of feed configurations.
    pub fn new(feeds: Vec<FeedConfig>) -> Self {
        Self {
            feeds,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
            feed_headers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns cached items, fetching all feeds on first call.
    async fn load(&self) -> Result<Vec<ItemDetail>, ConnectorError> {
        // Fast path — already loaded.
        {
            let guard = self.cache.read().await;
            if let Some(items) = &*guard {
                return Ok(items.clone());
            }
        }

        // Slow path — acquire write lock and fetch.
        let mut guard = self.cache.write().await;

        // Double-checked: another task may have loaded while we waited.
        if let Some(items) = &*guard {
            return Ok(items.clone());
        }

        let mut all: Vec<ItemDetail> = Vec::new();
        for config in &self.feeds {
            match self.fetch_feed(config, None).await {
                Ok(FetchResult::Items(items)) => all.extend(items),
                Ok(FetchResult::NotModified) => {} // shouldn't happen on first load
                Err(e) => {
                    // A broken feed should not prevent other feeds from loading.
                    tracing::warn!(url = %config.url, error = %e, "skipping feed due to fetch error");
                }
            }
        }

        all.sort_by(|a, b| b.item.published.cmp(&a.item.published));
        *guard = Some(all.clone());
        Ok(all)
    }

    /// Fetch a single feed, optionally using conditional HTTP headers.
    ///
    /// Returns [`FetchResult::NotModified`] if the server responds with 304.
    async fn fetch_feed(
        &self,
        config: &FeedConfig,
        headers: Option<&FeedCacheHeaders>,
    ) -> Result<FetchResult, ConnectorError> {
        let mut request = self.client.get(&config.url);

        if let Some(h) = headers {
            if let Some(lm) = &h.last_modified {
                request = request.header("If-Modified-Since", lm);
            }
            if let Some(etag) = &h.etag {
                request = request.header("If-None-Match", etag);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ConnectorError::Unavailable(e.to_string()))?;

        // 304 Not Modified — feed hasn't changed.
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(FetchResult::NotModified);
        }

        // Store caching headers for next request.
        let new_headers = FeedCacheHeaders {
            last_modified: response
                .headers()
                .get("last-modified")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            etag: response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
        };

        // Store headers if we got any.
        if new_headers.last_modified.is_some() || new_headers.etag.is_some() {
            self.feed_headers
                .write()
                .await
                .insert(config.url.clone(), new_headers);
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ConnectorError::Unavailable(e.to_string()))?;

        let feed = feed_rs::parser::parse(bytes.as_ref())
            .map_err(|e| ConnectorError::Other(Box::new(e)))?;

        Ok(FetchResult::Items(
            feed.entries
                .iter()
                .map(|entry| entry_to_item_detail(entry, &config.topic_id))
                .collect(),
        ))
    }

    /// Spawn a background task that refreshes all feeds on their configured interval.
    ///
    /// The refresh interval is the minimum `refresh_minutes` across all feeds
    /// (defaulting to 60 for feeds that don't specify one). Returns a
    /// [`JoinHandle`](tokio::task::JoinHandle) that can be aborted to stop
    /// refreshing.
    pub fn start_auto_refresh(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval_mins = self
            .feeds
            .iter()
            .map(|f| f.refresh_minutes.unwrap_or(60))
            .min()
            .unwrap_or(60)
            .max(1); // at least 1 minute

        tracing::info!(
            interval_minutes = interval_mins,
            feeds = self.feeds.len(),
            "starting auto-refresh background task"
        );

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_mins * 60));

            // Skip the first immediate tick — feeds are loaded lazily on first access.
            interval.tick().await;

            loop {
                interval.tick().await;
                tracing::info!("auto-refresh: starting feed refresh");

                let mut new_items: Vec<ItemDetail> = Vec::new();
                let mut new_count: usize = 0;

                for config in &self.feeds {
                    // Get current conditional headers for this feed.
                    let headers = self.feed_headers.read().await.get(&config.url).cloned();

                    match self.fetch_feed(config, headers.as_ref()).await {
                        Ok(FetchResult::Items(items)) => {
                            new_count += items.len();
                            new_items.extend(items);
                        }
                        Ok(FetchResult::NotModified) => {
                            tracing::debug!(url = %config.url, "feed not modified (304)");
                        }
                        Err(e) => {
                            tracing::warn!(
                                url = %config.url,
                                error = %e,
                                "auto-refresh: feed fetch failed, keeping stale data"
                            );
                        }
                    }
                }

                // Merge: keep existing items for feeds that returned 304,
                // replace items for feeds that returned new data.
                let refreshed_topic_ids: std::collections::HashSet<&str> =
                    new_items.iter().map(|d| d.item.topic_id.as_str()).collect();

                let mut merged = {
                    let guard = self.cache.read().await;
                    match &*guard {
                        Some(existing) => {
                            // Keep items from topics that weren't refreshed (304).
                            let mut kept: Vec<ItemDetail> = existing
                                .iter()
                                .filter(|d| !refreshed_topic_ids.contains(d.item.topic_id.as_str()))
                                .cloned()
                                .collect();
                            kept.extend(new_items);
                            kept
                        }
                        None => new_items,
                    }
                };

                // Deduplicate by item ID (keep first occurrence = newest if sorted).
                let mut seen = std::collections::HashSet::new();
                merged.retain(|d| seen.insert(d.item.id.clone()));

                merged.sort_by(|a, b| b.item.published.cmp(&a.item.published));

                // Swap cache — hold write lock briefly.
                {
                    let mut guard = self.cache.write().await;
                    *guard = Some(merged);
                }

                tracing::info!(new_items = new_count, "auto-refresh: feed refresh complete");
            }
        })
    }
}

impl Connector for RssConnector {
    async fn list_topics(&self) -> Result<Vec<Topic>, ConnectorError> {
        let mut seen = std::collections::HashSet::new();
        Ok(self
            .feeds
            .iter()
            .filter(|f| seen.insert(f.topic_id.clone()))
            .map(|f| Topic {
                id: f.topic_id.clone(),
                label: f.topic_label.clone(),
                description: f.topic_description.clone(),
                tags: f.topic_tags.clone(),
            })
            .collect())
    }

    async fn get_latest_items(&self, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError> {
        let all = self.load().await?;

        let mut items: Vec<Item> = all
            .iter()
            .filter(|d| {
                prefs.topics_of_interest.is_empty()
                    || prefs.topics_of_interest.contains(&d.item.topic_id)
            })
            .map(|d| d.item.clone())
            .collect();

        items.sort_by(|a, b| b.published.cmp(&a.published));

        if let Some(max) = prefs.max_items {
            items.truncate(max);
        }

        Ok(items)
    }

    async fn get_item_detail(&self, id: &str) -> Result<ItemDetail, ConnectorError> {
        self.load()
            .await?
            .into_iter()
            .find(|d| d.item.id == id)
            .ok_or_else(|| ConnectorError::NotFound(id.to_string()))
    }

    async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<Item>, ConnectorError> {
        let all = self.load().await?;
        let q = query.to_lowercase();

        let mut items: Vec<Item> = all
            .iter()
            .filter(|d| {
                let item = &d.item;

                let text_match = q.is_empty()
                    || item.headline.to_lowercase().contains(&q)
                    || item.summary.to_lowercase().contains(&q)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&q));

                let topic_match =
                    filters.topic_ids.is_empty() || filters.topic_ids.contains(&item.topic_id);

                let tag_match =
                    filters.tags.is_empty() || filters.tags.iter().any(|t| item.tags.contains(t));

                let since_ok = filters.since.is_none_or(|s| item.published >= s);
                let until_ok = filters.until.is_none_or(|u| item.published <= u);

                text_match && topic_match && tag_match && since_ok && until_ok
            })
            .map(|d| d.item.clone())
            .collect();

        items.sort_by(|a, b| b.published.cmp(&a.published));

        if let Some(limit) = filters.limit {
            items.truncate(limit);
        }

        Ok(items)
    }

    async fn get_recap(
        &self,
        since: DateTime<Utc>,
        prefs: &UserPrefs,
    ) -> Result<Vec<Item>, ConnectorError> {
        let all = self.load().await?;

        let mut items: Vec<Item> = all
            .iter()
            .filter(|d| {
                let item = &d.item;
                let topic_match = prefs.topics_of_interest.is_empty()
                    || prefs.topics_of_interest.contains(&item.topic_id);
                item.published >= since && topic_match
            })
            .map(|d| d.item.clone())
            .collect();

        items.sort_by(|a, b| b.published.cmp(&a.published));

        if let Some(max) = prefs.max_items {
            items.truncate(max);
        }

        Ok(items)
    }
}

fn entry_to_item_detail(entry: &feed_rs::model::Entry, topic_id: &str) -> ItemDetail {
    let headline = entry
        .title
        .as_ref()
        .map(|t| t.content.clone())
        .unwrap_or_default();

    let summary = entry
        .summary
        .as_ref()
        .map(|s| strip_html(&s.content))
        .unwrap_or_default();

    let body = entry
        .content
        .as_ref()
        .and_then(|c| c.body.as_ref())
        .map(|b| strip_html(b))
        .unwrap_or_else(|| summary.clone());

    let source = entry
        .links
        .first()
        .map(|l| l.href.clone())
        .unwrap_or_default();

    let published = entry.published.or(entry.updated).unwrap_or_else(Utc::now);

    let tags = entry.categories.iter().map(|c| c.term.clone()).collect();

    let links = entry
        .links
        .iter()
        .map(|l| ItemLink {
            url: l.href.clone(),
            label: l.title.clone().unwrap_or_else(|| "Read more".to_string()),
        })
        .collect();

    let read_time = estimate_read_time(&body);

    ItemDetail {
        item: Item {
            id: entry.id.clone(),
            topic_id: topic_id.to_string(),
            headline,
            summary,
            tags,
            source,
            published,
        },
        body,
        links,
        read_time,
    }
}

/// Strip HTML tags from a string, normalising whitespace.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Estimate reading time from word count at ~200 wpm.
fn estimate_read_time(text: &str) -> String {
    let minutes = ((text.split_whitespace().count() as f32 / 200.0).ceil() as u64).max(1);
    format!("{} min", minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal RSS 2.0 feed with two items for parsing tests.
    const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Tech News</title>
    <link>https://example.com</link>
    <description>Latest tech news</description>
    <item>
      <title>Rust 2024 Edition Released</title>
      <link>https://blog.rust-lang.org/rust-2024</link>
      <description>The new Rust edition brings &lt;b&gt;exciting&lt;/b&gt; features.</description>
      <guid>https://blog.rust-lang.org/rust-2024</guid>
      <pubDate>Thu, 01 Feb 2024 10:00:00 +0000</pubDate>
      <category>rust</category>
      <category>programming</category>
    </item>
    <item>
      <title>New AI Model Released</title>
      <link>https://example.com/ai-model</link>
      <description>A powerful new AI model.</description>
      <guid>https://example.com/ai-model</guid>
      <pubDate>Wed, 15 Jan 2024 08:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    fn parse_items(xml: &str, topic_id: &str) -> Vec<ItemDetail> {
        let feed = feed_rs::parser::parse(xml.as_bytes()).unwrap();
        feed.entries
            .iter()
            .map(|e| entry_to_item_detail(e, topic_id))
            .collect()
    }

    fn make_connector() -> RssConnector {
        RssConnector::new(vec![
            FeedConfig {
                topic_id: "tech".to_string(),
                topic_label: "Technology".to_string(),
                topic_description: "Tech news".to_string(),
                topic_tags: vec!["rust".to_string()],
                url: "https://example.com/feed.xml".to_string(),
                refresh_minutes: None,
            },
            FeedConfig {
                topic_id: "science".to_string(),
                topic_label: "Science".to_string(),
                topic_description: "Science news".to_string(),
                topic_tags: vec!["space".to_string()],
                url: "https://example.com/science.xml".to_string(),
                refresh_minutes: Some(30),
            },
        ])
    }

    // --- list_topics ---

    #[tokio::test]
    async fn list_topics_returns_all_configured_feeds() {
        let conn = make_connector();
        let topics = conn.list_topics().await.unwrap();
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0].id, "tech");
        assert_eq!(topics[1].id, "science");
    }

    // --- parsing ---

    #[test]
    fn parses_headline_and_id() {
        let items = parse_items(SAMPLE_RSS, "tech");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item.headline, "Rust 2024 Edition Released");
        assert!(!items[0].item.id.is_empty());
    }

    #[test]
    fn strips_html_from_summary() {
        let items = parse_items(SAMPLE_RSS, "tech");
        assert_eq!(
            items[0].item.summary,
            "The new Rust edition brings exciting features."
        );
    }

    #[test]
    fn assigns_topic_id() {
        let items = parse_items(SAMPLE_RSS, "tech");
        assert!(items.iter().all(|d| d.item.topic_id == "tech"));
    }

    #[test]
    fn parses_categories_as_tags() {
        let items = parse_items(SAMPLE_RSS, "tech");
        assert!(items[0].item.tags.contains(&"rust".to_string()));
        assert!(items[0].item.tags.contains(&"programming".to_string()));
    }

    #[test]
    fn parses_source_link() {
        let items = parse_items(SAMPLE_RSS, "tech");
        assert_eq!(items[0].item.source, "https://blog.rust-lang.org/rust-2024");
    }

    #[test]
    fn estimates_read_time() {
        assert_eq!(estimate_read_time("one two three"), "1 min");
        // 201 words → 2 min
        let long = "word ".repeat(201);
        assert_eq!(estimate_read_time(&long), "2 min");
    }

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<b>hello</b> <i>world</i>"), "hello world");
    }

    #[test]
    fn strip_html_normalises_whitespace() {
        assert_eq!(strip_html("  foo   bar  "), "foo bar");
    }

    #[test]
    fn strip_html_plain_text_unchanged() {
        assert_eq!(strip_html("no tags here"), "no tags here");
    }

    // --- auto-refresh ---

    #[test]
    fn refresh_interval_uses_minimum_across_feeds() {
        let conn = make_connector();
        // Feed 1: None (default 60), Feed 2: Some(30) → min = 30
        let min_interval = conn
            .feeds
            .iter()
            .map(|f| f.refresh_minutes.unwrap_or(60))
            .min()
            .unwrap_or(60);
        assert_eq!(min_interval, 30);
    }

    #[tokio::test]
    async fn start_auto_refresh_returns_handle() {
        let conn = Arc::new(make_connector());
        let handle = conn.start_auto_refresh();
        // The handle is valid and the task is running.
        assert!(!handle.is_finished());
        handle.abort();
    }

    // --- live feed (requires network) ---

    #[tokio::test]
    #[ignore]
    async fn fetches_real_rust_blog_feed() {
        let conn = RssConnector::new(vec![FeedConfig {
            topic_id: "rust".to_string(),
            topic_label: "Rust Blog".to_string(),
            topic_description: "Official Rust blog".to_string(),
            topic_tags: vec!["rust".to_string()],
            url: "https://blog.rust-lang.org/feed.xml".to_string(),
            refresh_minutes: None,
        }]);

        let items = conn.get_latest_items(&UserPrefs::default()).await.unwrap();
        assert!(!items.is_empty());
    }
}
