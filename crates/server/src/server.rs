//! Ferrisletter MCP server — tool definitions and handler.

use std::sync::Arc;

use chrono::DateTime;
use ferrisletter_connector::{BoxedConnector, Connector, SearchFilters, UserPrefs};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        AnnotateAble, CallToolResult, Content, Implementation, ListResourcesResult, Meta,
        PaginatedRequestParams, RawResource, ReadResourceRequestParams, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;

use crate::api::ConnectorHandle;

/// MCP resource URI for the embedded App UI (mcpui.dev spec).
pub const UI_RESOURCE_URI: &str = "ui://ferrisletter/app";

/// The HTML bundle embedded at compile time by build.rs.
const UI_BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/ui_bundle.html"));

/// The Ferrisletter MCP server.
///
/// Holds a handle to the active connector so that live hot-reload via the
/// management REST API is reflected in subsequent tool calls without a restart.
#[derive(Clone)]
pub struct FerrislletterServer {
    connector: ConnectorHandle,
    /// Whether the MCP App UI resource is enabled.
    pub ui_enabled: bool,
    tool_router: ToolRouter<Self>,
}

impl FerrislletterServer {
    pub fn new(connector: ConnectorHandle, ui_enabled: bool) -> Self {
        Self {
            connector,
            ui_enabled,
            tool_router: Self::tool_router(),
        }
    }

    /// Borrow the active connector for one request.
    async fn conn(&self) -> Arc<BoxedConnector> {
        self.connector.read().await.clone()
    }
}

/// Build `_meta: { "ui": { "resourceUri": "ui://ferrisletter/app" } }` for
/// tool call results, following the mcpui.dev spec.
fn ui_meta() -> Meta {
    let mut meta = Meta::new();
    meta.insert(
        "ui".to_string(),
        serde_json::json!({ "resourceUri": UI_RESOURCE_URI }),
    );
    meta
}

/// Wrap serialised JSON in a successful `CallToolResult` with UI metadata.
fn tool_ok(json: String) -> CallToolResult {
    CallToolResult::success(vec![Content::text(json)]).with_meta(Some(ui_meta()))
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

#[tool_router]
impl FerrislletterServer {
    /// List available newsletter topics.
    #[tool(
        description = "List available newsletter topics and their descriptions. \
        Call this first to discover what content is available."
    )]
    async fn ferrisletter_list_topics(&self) -> Result<CallToolResult, ErrorData> {
        let topics = self
            .conn()
            .await
            .list_topics()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&topics)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(tool_ok(json))
    }

    /// Get the latest items from the newsletter.
    #[tool(
        description = "Get the latest newsletter items as compact headlines and summaries. \
        Filter by topic or limit the count. \
        Use `ferrisletter_get_item` to read the full text of anything interesting."
    )]
    async fn ferrisletter_get_latest(
        &self,
        Parameters(params): Parameters<GetLatestParams>,
    ) -> Result<CallToolResult, ErrorData> {
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
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&items)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(tool_ok(json))
    }

    /// Get the full content of a specific item.
    #[tool(
        description = "Get the full body text, links, and metadata for a specific newsletter item \
        by its ID. IDs come from `ferrisletter_get_latest` or `ferrisletter_search`."
    )]
    async fn ferrisletter_get_item(
        &self,
        Parameters(params): Parameters<GetItemParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let detail = self
            .conn()
            .await
            .get_item_detail(&params.id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&detail)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(tool_ok(json))
    }

    /// Search newsletter content.
    #[tool(
        description = "Search newsletter content by keyword, topic, tags, or date range. \
        An empty query with filters acts as a pure filter. \
        Returns compact item summaries — use `ferrisletter_get_item` to expand one."
    )]
    async fn ferrisletter_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let parse_dt = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| {
                    ErrorData::invalid_params(format!("invalid datetime '{s}': {e}"), None)
                })
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
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&items)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(tool_ok(json))
    }

    /// Get a recap of items published since a given date.
    #[tool(
        description = "Summarise what happened in the newsletter since a given date. \
        Perfect for 'what did I miss this week?' queries. \
        Returns compact headlines — use `ferrisletter_get_item` to dig into anything."
    )]
    async fn ferrisletter_recap(
        &self,
        Parameters(params): Parameters<RecapParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let since = DateTime::parse_from_rfc3339(&params.since)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| {
                ErrorData::invalid_params(format!("invalid datetime '{}': {e}", params.since), None)
            })?;

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
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&items)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(tool_ok(json))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FerrislletterServer {
    fn get_info(&self) -> ServerInfo {
        let caps = if self.ui_enabled {
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build()
        } else {
            ServerCapabilities::builder().enable_tools().build()
        };
        ServerInfo::new(caps)
            .with_server_info(Implementation::new(
                "ferrisletter",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Ferrisletter is a conversational newsletter platform. \
                Start with `ferrisletter_list_topics` to see what's available, \
                then `ferrisletter_get_latest` to browse headlines. \
                Expand anything interesting with `ferrisletter_get_item`, \
                search across past content with `ferrisletter_search`, \
                or catch up on what you missed with `ferrisletter_recap`.",
            )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        if !self.ui_enabled {
            return Ok(ListResourcesResult::default());
        }
        let mut raw = RawResource::new(UI_RESOURCE_URI, "Ferrisletter");
        raw.description = Some("Interactive newsletter digest".into());
        raw.mime_type = Some("text/html".into());
        Ok(ListResourcesResult {
            meta: None,
            resources: vec![raw.no_annotation()],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        if !self.ui_enabled {
            return Err(ErrorData::invalid_params("UI not enabled", None));
        }
        if request.uri != UI_RESOURCE_URI {
            return Err(ErrorData::invalid_params(
                format!("unknown resource '{}'", request.uri),
                None,
            ));
        }
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(UI_BUNDLE, UI_RESOURCE_URI).with_mime_type("text/html"),
        ]))
    }
}
