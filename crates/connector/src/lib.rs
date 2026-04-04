//! Ferrisletter Connector SDK
//!
//! This crate defines the [`Connector`] trait that all content sources must implement.
//! Third-party connector authors should depend on this crate and implement the trait
//! to integrate their content source with Ferrisletter.

use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod error;
pub use error::ConnectorError;

mod factory;
pub use factory::{ConnectorFactory, ConnectorRegistry};

/// A content source that provides news items to Ferrisletter.
pub trait Connector: Send + Sync {
    /// Returns the available content categories.
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, ConnectorError>> + Send;

    /// Returns recent items, filtered by user preferences.
    fn get_latest_items(
        &self,
        prefs: &UserPrefs,
    ) -> impl Future<Output = Result<Vec<Item>, ConnectorError>> + Send;

    /// Returns the full content for a specific item.
    fn get_item_detail(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<ItemDetail, ConnectorError>> + Send;

    /// Finds items matching a query across all past content.
    fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> impl Future<Output = Result<Vec<Item>, ConnectorError>> + Send;

    /// Returns a summary of items since a given point in time.
    fn get_recap(
        &self,
        since: DateTime<Utc>,
        prefs: &UserPrefs,
    ) -> impl Future<Output = Result<Vec<Item>, ConnectorError>> + Send;
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

// --- Type erasure ---

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Object-safe backing trait — keeps `Connector` methods callable through `dyn`.
trait ErasedConnector: Send + Sync {
    fn list_topics_erased(&self) -> BoxFuture<'_, Result<Vec<Topic>, ConnectorError>>;
    fn get_latest_items_erased<'a>(
        &'a self,
        prefs: &'a UserPrefs,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>>;
    fn get_item_detail_erased<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ItemDetail, ConnectorError>>;
    fn search_erased<'a>(
        &'a self,
        query: &'a str,
        filters: &'a SearchFilters,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>>;
    fn get_recap_erased<'a>(
        &'a self,
        since: DateTime<Utc>,
        prefs: &'a UserPrefs,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>>;
}

impl<C: Connector + Send + Sync> ErasedConnector for C {
    fn list_topics_erased(&self) -> BoxFuture<'_, Result<Vec<Topic>, ConnectorError>> {
        Box::pin(Connector::list_topics(self))
    }
    fn get_latest_items_erased<'a>(
        &'a self,
        prefs: &'a UserPrefs,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>> {
        Box::pin(Connector::get_latest_items(self, prefs))
    }
    fn get_item_detail_erased<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ItemDetail, ConnectorError>> {
        Box::pin(Connector::get_item_detail(self, id))
    }
    fn search_erased<'a>(
        &'a self,
        query: &'a str,
        filters: &'a SearchFilters,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>> {
        Box::pin(Connector::search(self, query, filters))
    }
    fn get_recap_erased<'a>(
        &'a self,
        since: DateTime<Utc>,
        prefs: &'a UserPrefs,
    ) -> BoxFuture<'a, Result<Vec<Item>, ConnectorError>> {
        Box::pin(Connector::get_recap(self, since, prefs))
    }
}

/// A type-erased [`Connector`] that can be stored as `Arc<BoxedConnector>`.
///
/// Useful when the concrete connector type is not known at compile time
/// (e.g. selected from config).
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use ferrisletter_connector::{BoxedConnector, Connector};
///
/// fn start(connector: Arc<BoxedConnector>) { /* ... */ }
///
/// let conn = StaticConnector::from_json(data)?;
/// start(Arc::new(BoxedConnector::new(conn)));
/// ```
pub struct BoxedConnector(Box<dyn ErasedConnector>);

impl BoxedConnector {
    /// Wrap any [`Connector`] for type-erased use.
    pub fn new<C: Connector + Send + Sync + 'static>(connector: C) -> Self {
        Self(Box::new(connector))
    }
}

impl Connector for BoxedConnector {
    // list_topics takes only &self — RPIT works fine here.
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, ConnectorError>> + Send {
        self.0.list_topics_erased()
    }

    // Methods with additional reference params use `async fn` to let the compiler
    // handle lifetime capture automatically, avoiding E0700.
    async fn get_latest_items(&self, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError> {
        self.0.get_latest_items_erased(prefs).await
    }
    async fn get_item_detail(&self, id: &str) -> Result<ItemDetail, ConnectorError> {
        self.0.get_item_detail_erased(id).await
    }
    async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<Item>, ConnectorError> {
        self.0.search_erased(query, filters).await
    }
    async fn get_recap(
        &self,
        since: DateTime<Utc>,
        prefs: &UserPrefs,
    ) -> Result<Vec<Item>, ConnectorError> {
        self.0.get_recap_erased(since, prefs).await
    }
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
