//! Ferrisletter Connector SDK
//!
//! This crate defines the [`Connector`] trait that all content sources must implement.
//! Third-party connector authors should depend on this crate and implement the trait
//! to integrate their content source with Ferrisletter.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod error;
pub use error::ConnectorError;

/// A content source that provides news items to Ferrisletter.
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    /// Returns the available content categories.
    async fn list_topics(&self) -> Result<Vec<Topic>, ConnectorError>;

    /// Returns recent items, filtered by user preferences.
    async fn get_latest_items(&self, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError>;

    /// Returns the full content for a specific item.
    async fn get_item_detail(&self, id: &str) -> Result<ItemDetail, ConnectorError>;

    /// Finds items matching a query across all past content.
    async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<Item>, ConnectorError>;

    /// Returns a summary of items since a given point in time.
    async fn get_recap(
        &self,
        since: DateTime<Utc>,
        prefs: &UserPrefs,
    ) -> Result<Vec<Item>, ConnectorError>;
}

/// A content category within a newsletter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub label: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// A single news item in compact form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub topic_id: String,
    pub headline: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub source: String,
    pub published: DateTime<Utc>,
}

/// A fully expanded news item with body content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetail {
    #[serde(flatten)]
    pub item: Item,
    pub body: String,
    pub links: Vec<Link>,
    pub read_time: String,
}

/// A related link attached to an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub label: String,
}

/// Subscriber preferences for content filtering and presentation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    pub topics_of_interest: Vec<String>,
    pub summary_length: SummaryLength,
    pub max_items: Option<usize>,
    pub language: Option<String>,
}

/// How detailed the compact view should be.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryLength {
    Brief,
    #[default]
    Standard,
    Detailed,
}

/// Filters for searching across past content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub topic_ids: Vec<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
}
