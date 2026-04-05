//! Ferrisletter MCP server — tool definitions and handler.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::DateTime;
use ferrisletter_connector::{
    BoxedConnector, Connector, Item, ItemDetail, Link, SearchFilters, Topic, UserPrefs,
};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        AnnotateAble, CallToolRequestParams, CallToolResult, Content, ExtensionCapabilities,
        Implementation, ListResourcesResult, ListToolsResult, Meta, PaginatedRequestParams,
        RawResource, ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo, Tool,
    },
    schemars,
    service::{NotificationContext, RequestContext},
    tool, tool_router,
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
    /// Whether the MCP App UI resource is enabled in config.
    pub ui_enabled: bool,
    /// Whether the connected client supports the MCP-UI extension.
    /// Set during the MCP initialization handshake.
    client_supports_ui: Arc<AtomicBool>,
    tool_router: ToolRouter<Self>,
}

impl FerrislletterServer {
    pub fn new(connector: ConnectorHandle, ui_enabled: bool) -> Self {
        Self {
            connector,
            ui_enabled,
            client_supports_ui: Arc::new(AtomicBool::new(false)),
            tool_router: Self::tool_router(),
        }
    }

    /// Borrow the active connector for one request.
    async fn conn(&self) -> Arc<BoxedConnector> {
        self.connector.read().await.clone()
    }

    /// Whether the current session should use UI-annotated responses.
    ///
    /// True only when UI is enabled in config AND the client advertised
    /// support for `io.modelcontextprotocol/ui` during initialization.
    fn should_use_ui(&self) -> bool {
        self.ui_enabled && self.client_supports_ui.load(Ordering::Relaxed)
    }
}

/// Build `_meta.ui` for tool call results (mcpui.dev / SEP-1865 spec).
fn ui_result_meta() -> Meta {
    let mut meta = Meta::new();
    meta.insert(
        "ui".to_string(),
        serde_json::json!({ "resourceUri": UI_RESOURCE_URI }),
    );
    meta
}

/// Build `_meta.ui` for tool definitions in `list_tools` (mcpui.dev / SEP-1865 spec).
fn ui_tool_meta() -> Meta {
    let mut meta = Meta::new();
    meta.insert(
        "ui".to_string(),
        serde_json::json!({
            "resourceUri": UI_RESOURCE_URI,
            "visibility": ["model", "app"]
        }),
    );
    meta
}

/// Wrap serialised JSON in a successful `CallToolResult` — no UI metadata.
fn tool_ok_text(json: String) -> CallToolResult {
    CallToolResult::success(vec![Content::text(json)])
}

/// Wrap serialised JSON in a successful `CallToolResult` with UI metadata.
/// The note tells the model the data is already rendered in the UI panel;
/// the JSON is in a separate content block so the app can still parse it.
fn tool_ok_ui(json: String, item_count: usize, label: &str) -> CallToolResult {
    let note = format!(
        "Rendered in companion UI — {item_count} {label}. \
         Do NOT repeat or summarise individual items to the user."
    );
    CallToolResult::success(vec![Content::text(note), Content::text(json)])
        .with_meta(Some(ui_result_meta()))
}

// --- Rich text formatting for non-UI clients ---

/// Format a list of topics as human-readable text.
fn format_topics_text(topics: &[Topic]) -> String {
    let mut out = format!("{} topic(s) available:\n", topics.len());
    for t in topics {
        out.push_str(&format!("\n- **{}**: {}", t.label, t.description));
        if !t.tags.is_empty() {
            out.push_str(&format!(" ({})", t.tags.join(", ")));
        }
    }
    out
}

/// Format a list of items as human-readable text.
fn format_items_text(items: &[Item], label: &str) -> String {
    if items.is_empty() {
        return format!("No {label} found.");
    }
    let mut out = format!("{} {}:\n", items.len(), label);
    for (i, item) in items.iter().enumerate() {
        let date = item.published.format("%Y-%m-%d");
        out.push_str(&format!("\n{}. **{}**", i + 1, item.headline));
        out.push_str(&format!("\n   {} | {}", item.source, date));
        if !item.summary.is_empty() {
            // Truncate long summaries for the compact list view.
            let summary = if item.summary.len() > 200 {
                format!("{}...", &item.summary[..200])
            } else {
                item.summary.clone()
            };
            out.push_str(&format!("\n   {summary}"));
        }
        if !item.tags.is_empty() {
            out.push_str(&format!("\n   Tags: {}", item.tags.join(", ")));
        }
    }
    out
}

/// Format a single item detail as human-readable text.
fn format_detail_text(detail: &ItemDetail) -> String {
    let item = &detail.item;
    let date = item.published.format("%Y-%m-%d %H:%M UTC");
    let mut out = format!("**{}**\n", item.headline);
    out.push_str(&format!(
        "Source: {} | Published: {} | Read time: {}\n",
        item.source, date, detail.read_time
    ));
    if !item.tags.is_empty() {
        out.push_str(&format!("Tags: {}\n", item.tags.join(", ")));
    }
    out.push_str(&format!("\n{}", detail.body));
    if !detail.links.is_empty() {
        out.push_str("\n\nLinks:");
        for Link { url, label } in &detail.links {
            out.push_str(&format!("\n- [{label}]({url})"));
        }
    }
    out
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
    #[tracing::instrument(skip(self), name = "tool:list_topics")]
    async fn ferrisletter_list_topics(&self) -> Result<CallToolResult, ErrorData> {
        let topics = self
            .conn()
            .await
            .list_topics()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let count = topics.len();
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&topics)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "topics"));
        }
        let text = format_topics_text(&topics);
        Ok(tool_ok_text(text))
    }

    /// Get the latest items from the newsletter.
    #[tool(
        description = "Get the latest newsletter items as compact headlines and summaries. \
        Filter by topic or limit the count. \
        Use `ferrisletter_get_item` to read the full text of anything interesting."
    )]
    #[tracing::instrument(skip(self), name = "tool:get_latest")]
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
        let count = items.len();
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "items"));
        }
        let text = format_items_text(&items, "items");
        Ok(tool_ok_text(text))
    }

    /// Get the full content of a specific item.
    #[tool(
        description = "Get the full body text, links, and metadata for a specific newsletter item \
        by its ID. IDs come from `ferrisletter_get_latest` or `ferrisletter_search`."
    )]
    #[tracing::instrument(skip(self), name = "tool:get_item")]
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
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&detail)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, 1, "item"));
        }
        let text = format_detail_text(&detail);
        Ok(tool_ok_text(text))
    }

    /// Search newsletter content.
    #[tool(
        description = "Search newsletter content by keyword, topic, tags, or date range. \
        An empty query with filters acts as a pure filter. \
        Returns compact item summaries — use `ferrisletter_get_item` to expand one."
    )]
    #[tracing::instrument(skip(self), name = "tool:search")]
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
        let count = items.len();
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "results"));
        }
        let text = format_items_text(&items, "results");
        Ok(tool_ok_text(text))
    }

    /// Get a recap of items published since a given date.
    #[tool(
        description = "Summarise what happened in the newsletter since a given date. \
        Perfect for 'what did I miss this week?' queries. \
        Returns compact headlines — use `ferrisletter_get_item` to dig into anything."
    )]
    #[tracing::instrument(skip(self), name = "tool:recap")]
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
        let count = items.len();
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "items"));
        }
        let text = format_items_text(&items, "items");
        Ok(tool_ok_text(text))
    }
}

impl ServerHandler for FerrislletterServer {
    fn get_info(&self) -> ServerInfo {
        let caps = if self.ui_enabled {
            let mut exts = ExtensionCapabilities::new();
            exts.insert(
                "io.modelcontextprotocol/ui".to_string(),
                serde_json::from_value(serde_json::json!({})).unwrap(),
            );
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_extensions_with(exts)
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

    /// Detect client UI capability after the MCP handshake completes.
    async fn on_initialized(&self, context: NotificationContext<rmcp::RoleServer>) {
        let supports_ui = context
            .peer
            .peer_info()
            .and_then(|info| info.capabilities.extensions.as_ref())
            .map(|ext| ext.contains_key("io.modelcontextprotocol/ui"))
            .unwrap_or(false);

        self.client_supports_ui
            .store(supports_ui, Ordering::Relaxed);

        if supports_ui {
            tracing::info!("client supports MCP-UI extension");
        } else {
            tracing::info!("client does not support MCP-UI, using text responses");
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tcc = ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools: Vec<Tool> = self
            .tool_router
            .list_all()
            .into_iter()
            .map(|mut t| {
                if self.should_use_ui() {
                    t.meta = Some(ui_tool_meta());
                }
                t
            })
            .collect();
        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
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
        raw.mime_type = Some("text/html;profile=mcp-app".into());
        raw.meta = Some({
            let mut meta = Meta::new();
            meta.insert(
                "ui".to_string(),
                serde_json::json!({
                    "prefersBorder": true
                }),
            );
            meta
        });
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
            ResourceContents::text(UI_BUNDLE, UI_RESOURCE_URI)
                .with_mime_type("text/html;profile=mcp-app"),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_topics_produces_readable_output() {
        let topics = vec![
            Topic {
                id: "rust".into(),
                label: "Rust".into(),
                description: "Rust programming news".into(),
                tags: vec!["rust".into(), "programming".into()],
            },
            Topic {
                id: "ai".into(),
                label: "AI".into(),
                description: "Artificial intelligence updates".into(),
                tags: vec![],
            },
        ];
        let text = format_topics_text(&topics);
        assert!(text.contains("2 topic(s)"));
        assert!(text.contains("**Rust**"));
        assert!(text.contains("rust, programming"));
        assert!(text.contains("**AI**"));
    }

    #[test]
    fn format_items_produces_numbered_list() {
        let items = vec![Item {
            id: "1".into(),
            topic_id: "rust".into(),
            headline: "Rust 2024 ships".into(),
            summary: "The new edition is here.".into(),
            tags: vec!["rust".into()],
            source: "blog.rust-lang.org".into(),
            published: chrono::Utc::now(),
        }];
        let text = format_items_text(&items, "items");
        assert!(text.contains("1 items"));
        assert!(text.contains("1. **Rust 2024 ships**"));
        assert!(text.contains("blog.rust-lang.org"));
        assert!(text.contains("Tags: rust"));
    }

    #[test]
    fn format_items_empty_returns_no_items() {
        let text = format_items_text(&[], "results");
        assert_eq!(text, "No results found.");
    }

    #[test]
    fn format_detail_includes_body_and_links() {
        let detail = ItemDetail {
            item: Item {
                id: "1".into(),
                topic_id: "rust".into(),
                headline: "Big Release".into(),
                summary: "Summary here".into(),
                tags: vec!["release".into()],
                source: "example.com".into(),
                published: chrono::Utc::now(),
            },
            body: "Full article body text.".into(),
            links: vec![Link {
                url: "https://example.com".into(),
                label: "Read more".into(),
            }],
            read_time: "3 min".into(),
        };
        let text = format_detail_text(&detail);
        assert!(text.contains("**Big Release**"));
        assert!(text.contains("Read time: 3 min"));
        assert!(text.contains("Full article body text."));
        assert!(text.contains("[Read more](https://example.com)"));
    }
}
