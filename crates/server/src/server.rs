//! Ferrisletter MCP server — tool definitions and handler.

use std::sync::Arc;

use chrono::DateTime;
use ferrisletter_connector::{BoxedConnector, Connector, SearchFilters, UserPrefs};
use rmcp::{
    ServerHandler,
    model::{Implementation, ServerCapabilities, ServerInfo},
    schemars, tool,
};
use serde::Deserialize;

use crate::api::ConnectorHandle;

/// The Ferrisletter MCP server.
///
/// Holds a handle to the active connector so that live hot-reload via the
/// management REST API is reflected in subsequent tool calls without a restart.
#[derive(Clone)]
pub struct FerrislletterServer {
    connector: ConnectorHandle,
}

impl FerrislletterServer {
    pub fn new(connector: ConnectorHandle) -> Self {
        Self { connector }
    }

    /// Borrow the active connector for one request.
    async fn conn(&self) -> Arc<BoxedConnector> {
        self.connector.read().await.clone()
    }
}

// --- Tool parameter types ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetLatestParams {
    /// Topic IDs to filter by. Leave empty or omit for all topics.
    pub topics: Option<Vec<String>>,
    /// Maximum number of items to return.
    pub max_items: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetItemParams {
    /// Item ID as returned by `ferrisletter_get_latest` or `ferrisletter_search`.
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Keywords to match in headlines, summaries, and tags.
    pub query: String,
    /// Topic IDs to filter by.
    pub topics: Option<Vec<String>>,
    /// Tags to filter by.
    pub tags: Option<Vec<String>>,
    /// Only return items published after this date (ISO 8601, e.g. `2024-01-01T00:00:00Z`).
    pub since: Option<String>,
    /// Only return items published before this date (ISO 8601).
    pub until: Option<String>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecapParams {
    /// Recap items published after this date (ISO 8601, e.g. `2024-01-01T00:00:00Z`).
    pub since: String,
    /// Topic IDs to filter by. Leave empty or omit for all topics.
    pub topics: Option<Vec<String>>,
    /// Maximum number of items to return.
    pub max_items: Option<usize>,
}

// --- Tools ---

#[tool(tool_box)]
impl FerrislletterServer {
    /// List available newsletter topics.
    #[tool(
        description = "List available newsletter topics and their descriptions. \
        Call this first to discover what content is available."
    )]
    async fn ferrisletter_list_topics(&self) -> Result<String, String> {
        let topics = self
            .conn()
            .await
            .list_topics()
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&topics).map_err(|e| e.to_string())
    }

    /// Get the latest items from the newsletter.
    #[tool(
        description = "Get the latest newsletter items as compact headlines and summaries. \
        Filter by topic or limit the count. \
        Use `ferrisletter_get_item` to read the full text of anything interesting."
    )]
    async fn ferrisletter_get_latest(
        &self,
        #[tool(aggr)] params: GetLatestParams,
    ) -> Result<String, String> {
        let prefs = UserPrefs {
            topics_of_interest: params.topics.unwrap_or_default(),
            max_items: params.max_items,
            ..Default::default()
        };
        let items = self
            .conn()
            .await
            .get_latest_items(&prefs)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
    }

    /// Get the full content of a specific item.
    #[tool(
        description = "Get the full body text, links, and metadata for a specific newsletter item \
        by its ID. IDs come from `ferrisletter_get_latest` or `ferrisletter_search`."
    )]
    async fn ferrisletter_get_item(
        &self,
        #[tool(aggr)] params: GetItemParams,
    ) -> Result<String, String> {
        let detail = self
            .conn()
            .await
            .get_item_detail(&params.id)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&detail).map_err(|e| e.to_string())
    }

    /// Search newsletter content.
    #[tool(
        description = "Search newsletter content by keyword, topic, tags, or date range. \
        An empty query with filters acts as a pure filter. \
        Returns compact item summaries — use `ferrisletter_get_item` to expand one."
    )]
    async fn ferrisletter_search(
        &self,
        #[tool(aggr)] params: SearchParams,
    ) -> Result<String, String> {
        let parse_dt = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| format!("invalid datetime '{s}': {e}"))
        };

        let filters = SearchFilters {
            topic_ids: params.topics.unwrap_or_default(),
            tags: params.tags.unwrap_or_default(),
            since: params.since.as_deref().map(parse_dt).transpose()?,
            until: params.until.as_deref().map(parse_dt).transpose()?,
            limit: params.limit,
        };

        let items = self
            .conn()
            .await
            .search(&params.query, &filters)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
    }

    /// Get a recap of items published since a given date.
    #[tool(
        description = "Summarise what happened in the newsletter since a given date. \
        Perfect for 'what did I miss this week?' queries. \
        Returns compact headlines — use `ferrisletter_get_item` to dig into anything."
    )]
    async fn ferrisletter_recap(
        &self,
        #[tool(aggr)] params: RecapParams,
    ) -> Result<String, String> {
        let since = DateTime::parse_from_rfc3339(&params.since)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| format!("invalid datetime '{}': {e}", params.since))?;

        let prefs = UserPrefs {
            topics_of_interest: params.topics.unwrap_or_default(),
            max_items: params.max_items,
            ..Default::default()
        };

        let items = self
            .conn()
            .await
            .get_recap(since, &prefs)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&items).map_err(|e| e.to_string())
    }
}

#[tool(tool_box)]
impl ServerHandler for FerrislletterServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "ferrisletter".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some(
                "Ferrisletter is a conversational newsletter platform. \
                Start with `ferrisletter_list_topics` to see what's available, \
                then `ferrisletter_get_latest` to browse headlines. \
                Expand anything interesting with `ferrisletter_get_item`, \
                search across past content with `ferrisletter_search`, \
                or catch up on what you missed with `ferrisletter_recap`."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
