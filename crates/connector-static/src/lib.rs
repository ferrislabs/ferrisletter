//! Static JSON connector for Ferrisletter.
//!
//! Loads all content from a single JSON file at startup and serves it entirely
//! from memory. Useful for testing, demos, and simple personal newsletters.
//!
//! # Data format
//!
//! ```json
//! {
//!   "topics": [
//!     {
//!       "id": "tech",
//!       "label": "Technology",
//!       "description": "Tech news and releases",
//!       "tags": ["rust", "ai", "open-source"]
//!     }
//!   ],
//!   "items": [
//!     {
//!       "id": "item-1",
//!       "topic_id": "tech",
//!       "headline": "Rust 2024 edition released",
//!       "summary": "The new edition brings AFIT and more.",
//!       "tags": ["rust", "programming"],
//!       "source": "blog.rust-lang.org",
//!       "published": "2024-02-01T10:00:00Z",
//!       "body": "Full article text goes here.",
//!       "links": [{ "url": "https://blog.rust-lang.org", "label": "Read more" }],
//!       "read_time": "3 min"
//!     }
//!   ]
//! }
//! ```

use std::path::Path;

use chrono::{DateTime, Utc};
use ferrisletter_connector::{
    BoxedConnector, Connector, ConnectorError, ConnectorFactory, Item, ItemDetail, SearchFilters,
    Topic, UserPrefs,
};

/// Error returned when constructing a [`StaticConnector`] from disk or a string.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse JSON: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, serde::Deserialize)]
struct StaticData {
    topics: Vec<Topic>,
    items: Vec<ItemDetail>,
}

/// A connector that serves content from a static JSON file loaded at startup.
///
/// All data is held in memory — no async I/O happens after construction.
/// Use [`StaticConnector::from_file`] to load from disk, or
/// [`StaticConnector::from_json`] to parse an embedded string.
#[derive(Debug)]
pub struct StaticConnector {
    data: StaticData,
}

impl StaticConnector {
    /// Load a [`StaticConnector`] from a JSON file on disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parse a [`StaticConnector`] from a JSON string.
    ///
    /// Useful for embedding data at compile time with `include_str!`.
    pub fn from_json(json: &str) -> Result<Self, LoadError> {
        let data: StaticData = serde_json::from_str(json)?;
        Ok(Self { data })
    }
}

impl Connector for StaticConnector {
    async fn list_topics(&self) -> Result<Vec<Topic>, ConnectorError> {
        Ok(self.data.topics.clone())
    }

    async fn get_latest_items(&self, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError> {
        let mut items: Vec<Item> = self
            .data
            .items
            .iter()
            .filter(|detail| {
                prefs.topics_of_interest.is_empty()
                    || prefs.topics_of_interest.contains(&detail.item.topic_id)
            })
            .map(|detail| detail.item.clone())
            .collect();

        // Newest first.
        items.sort_by(|a, b| b.published.cmp(&a.published));

        if let Some(max) = prefs.max_items {
            items.truncate(max);
        }

        Ok(items)
    }

    async fn get_item_detail(&self, id: &str) -> Result<ItemDetail, ConnectorError> {
        self.data
            .items
            .iter()
            .find(|detail| detail.item.id == id)
            .cloned()
            .ok_or_else(|| ConnectorError::NotFound(id.to_string()))
    }

    async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<Item>, ConnectorError> {
        let q = query.to_lowercase();

        let mut items: Vec<Item> = self
            .data
            .items
            .iter()
            .filter(|detail| {
                let item = &detail.item;

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
            .map(|detail| detail.item.clone())
            .collect();

        // Newest first.
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
        let mut items: Vec<Item> = self
            .data
            .items
            .iter()
            .filter(|detail| {
                let item = &detail.item;
                let topic_match = prefs.topics_of_interest.is_empty()
                    || prefs.topics_of_interest.contains(&item.topic_id);
                item.published >= since && topic_match
            })
            .map(|detail| detail.item.clone())
            .collect();

        // Newest first.
        items.sort_by(|a, b| b.published.cmp(&a.published));

        if let Some(max) = prefs.max_items {
            items.truncate(max);
        }

        Ok(items)
    }
}

/// Factory for creating [`StaticConnector`] instances from TOML configuration.
///
/// Expects the `[connector]` table to contain a `path` string pointing to a
/// JSON data file.
///
/// # Config example
///
/// ```toml
/// [connector]
/// type = "static"
/// path = "data/newsletter.json"
/// ```
pub struct StaticConnectorFactory;

impl ConnectorFactory for StaticConnectorFactory {
    fn connector_type(&self) -> &str {
        "static"
    }

    fn create(&self, config: &toml::Value) -> Result<BoxedConnector, ConnectorError> {
        let path = config.get("path").and_then(|v| v.as_str()).unwrap_or("");

        if path.is_empty() {
            return Err(ConnectorError::Other(
                "static connector requires a non-empty 'path'".into(),
            ));
        }

        StaticConnector::from_file(path)
            .map(BoxedConnector::new)
            .map_err(|e| ConnectorError::Other(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const SAMPLE: &str = r#"
    {
      "topics": [
        {
          "id": "tech",
          "label": "Technology",
          "description": "Tech news",
          "tags": ["rust", "ai"]
        },
        {
          "id": "science",
          "label": "Science",
          "description": "Science news",
          "tags": ["space", "biology"]
        }
      ],
      "items": [
        {
          "id": "item-1",
          "topic_id": "tech",
          "headline": "Rust 2024 edition released",
          "summary": "The new edition brings AFIT and more.",
          "tags": ["rust", "programming"],
          "source": "blog.rust-lang.org",
          "published": "2024-02-01T10:00:00Z",
          "body": "Full article text.",
          "links": [{"url": "https://blog.rust-lang.org", "label": "Read more"}],
          "read_time": "3 min"
        },
        {
          "id": "item-2",
          "topic_id": "science",
          "headline": "New exoplanet discovered",
          "summary": "Astronomers found a potentially habitable world.",
          "tags": ["space", "astronomy"],
          "source": "nasa.gov",
          "published": "2024-01-15T08:00:00Z",
          "body": "Details about the exoplanet.",
          "links": [],
          "read_time": "2 min"
        }
      ]
    }
    "#;

    fn make_connector() -> StaticConnector {
        StaticConnector::from_json(SAMPLE).expect("valid sample JSON")
    }

    #[tokio::test]
    async fn list_topics_returns_all() {
        let conn = make_connector();
        let topics = conn.list_topics().await.unwrap();
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0].id, "tech");
        assert_eq!(topics[1].id, "science");
    }

    #[tokio::test]
    async fn get_latest_items_no_filter_returns_all_sorted_newest_first() {
        let conn = make_connector();
        let items = conn.get_latest_items(&UserPrefs::default()).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "item-1"); // 2024-02-01
        assert_eq!(items[1].id, "item-2"); // 2024-01-15
    }

    #[tokio::test]
    async fn get_latest_items_topic_filter() {
        let conn = make_connector();
        let prefs = UserPrefs {
            topics_of_interest: vec!["tech".to_string()],
            ..Default::default()
        };
        let items = conn.get_latest_items(&prefs).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-1");
    }

    #[tokio::test]
    async fn get_latest_items_max_items_truncates() {
        let conn = make_connector();
        let prefs = UserPrefs {
            max_items: Some(1),
            ..Default::default()
        };
        let items = conn.get_latest_items(&prefs).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-1"); // newest wins
    }

    #[tokio::test]
    async fn get_item_detail_found() {
        let conn = make_connector();
        let detail = conn.get_item_detail("item-1").await.unwrap();
        assert_eq!(detail.item.headline, "Rust 2024 edition released");
        assert_eq!(detail.read_time, "3 min");
        assert_eq!(detail.links.len(), 1);
    }

    #[tokio::test]
    async fn get_item_detail_not_found_returns_error() {
        let conn = make_connector();
        let err = conn.get_item_detail("does-not-exist").await.unwrap_err();
        assert!(matches!(err, ConnectorError::NotFound(_)));
    }

    #[tokio::test]
    async fn search_by_keyword_in_headline() {
        let conn = make_connector();
        let items = conn
            .search("rust", &SearchFilters::default())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-1");
    }

    #[tokio::test]
    async fn search_by_keyword_in_tags() {
        let conn = make_connector();
        let items = conn
            .search("astronomy", &SearchFilters::default())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-2");
    }

    #[tokio::test]
    async fn search_empty_query_returns_all() {
        let conn = make_connector();
        let items = conn.search("", &SearchFilters::default()).await.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn search_with_topic_filter() {
        let conn = make_connector();
        let filters = SearchFilters {
            topic_ids: vec!["science".to_string()],
            ..Default::default()
        };
        let items = conn.search("", &filters).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-2");
    }

    #[tokio::test]
    async fn search_with_since_filter() {
        let conn = make_connector();
        let filters = SearchFilters {
            since: Some(Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap()),
            ..Default::default()
        };
        let items = conn.search("", &filters).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-1");
    }

    #[tokio::test]
    async fn search_with_limit() {
        let conn = make_connector();
        let filters = SearchFilters {
            limit: Some(1),
            ..Default::default()
        };
        let items = conn.search("", &filters).await.unwrap();
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn get_recap_filters_by_since() {
        let conn = make_connector();
        let since = Utc.with_ymd_and_hms(2024, 1, 20, 0, 0, 0).unwrap();
        let items = conn.get_recap(since, &UserPrefs::default()).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-1");
    }

    #[tokio::test]
    async fn get_recap_with_topic_filter() {
        let conn = make_connector();
        let since = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let prefs = UserPrefs {
            topics_of_interest: vec!["science".to_string()],
            ..Default::default()
        };
        let items = conn.get_recap(since, &prefs).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-2");
    }

    #[tokio::test]
    async fn from_json_invalid_returns_parse_error() {
        let err = StaticConnector::from_json("not valid json").unwrap_err();
        assert!(matches!(err, LoadError::Parse(_)));
    }
}
