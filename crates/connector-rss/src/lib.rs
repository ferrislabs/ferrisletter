//! RSS/Atom feed connector for Ferrisletter.
//!
//! Fetches one or more RSS or Atom feeds and exposes their content through the
//! [`Connector`] interface. Each feed is mapped to a [`Topic`], and items are
//! fetched lazily on first access then cached in memory for the lifetime of the
//! connector.
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
//!     },
//! ]);
//! ```

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
}

type Cache = Arc<RwLock<Option<Vec<ItemDetail>>>>;

/// An RSS/Atom connector for Ferrisletter.
///
/// Items are fetched once on first use and cached in memory. To force a
/// refresh, create a new [`RssConnector`].
#[derive(Debug, Clone)]
pub struct RssConnector {
    feeds: Vec<FeedConfig>,
    client: reqwest::Client,
    cache: Cache,
}

impl RssConnector {
    /// Create a new [`RssConnector`] from a list of feed configurations.
    pub fn new(feeds: Vec<FeedConfig>) -> Self {
        Self {
            feeds,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
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
            match self.fetch_feed(config).await {
                Ok(items) => all.extend(items),
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

    async fn fetch_feed(&self, config: &FeedConfig) -> Result<Vec<ItemDetail>, ConnectorError> {
        let bytes = self
            .client
            .get(&config.url)
            .send()
            .await
            .map_err(|e| ConnectorError::Unavailable(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| ConnectorError::Unavailable(e.to_string()))?;

        let feed = feed_rs::parser::parse(bytes.as_ref())
            .map_err(|e| ConnectorError::Other(Box::new(e)))?;

        Ok(feed
            .entries
            .iter()
            .map(|entry| entry_to_item_detail(entry, &config.topic_id))
            .collect())
    }
}

impl Connector for RssConnector {
    async fn list_topics(&self) -> Result<Vec<Topic>, ConnectorError> {
        Ok(self
            .feeds
            .iter()
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
            },
            FeedConfig {
                topic_id: "science".to_string(),
                topic_label: "Science".to_string(),
                topic_description: "Science news".to_string(),
                topic_tags: vec!["space".to_string()],
                url: "https://example.com/science.xml".to_string(),
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
        }]);

        let items = conn.get_latest_items(&UserPrefs::default()).await.unwrap();
        assert!(!items.is_empty());
    }
}
