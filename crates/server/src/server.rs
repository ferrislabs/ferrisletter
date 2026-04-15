//! Ferrisletter MCP server — tool definitions and handler.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::favorites::BoxedFavoriteStore;
use crate::users::BoxedUserStore;
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
    /// Favorites store (shared, type-erased).
    favorites: Arc<BoxedFavoriteStore>,
    /// User state store — `None` in stateless mode.
    users: Option<Arc<BoxedUserStore>>,
    tool_router: ToolRouter<Self>,
}

impl FerrislletterServer {
    pub fn new(
        connector: ConnectorHandle,
        ui_enabled: bool,
        favorites: Arc<BoxedFavoriteStore>,
    ) -> Self {
        Self::with_user_store(connector, ui_enabled, favorites, None)
    }

    /// Build a server with a user store (enables personalized tools).
    pub fn with_user_store(
        connector: ConnectorHandle,
        ui_enabled: bool,
        favorites: Arc<BoxedFavoriteStore>,
        users: Option<Arc<BoxedUserStore>>,
    ) -> Self {
        Self {
            connector,
            ui_enabled,
            client_supports_ui: Arc::new(AtomicBool::new(false)),
            favorites,
            users,
            tool_router: Self::tool_router(),
        }
    }

    /// Borrow the user store or return a helpful error if not configured.
    fn require_user_store(&self) -> Result<&Arc<BoxedUserStore>, ErrorData> {
        self.users.as_ref().ok_or_else(|| {
            ErrorData::invalid_request(
                "user store not configured on this server (stateless mode)".to_string(),
                None,
            )
        })
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

// --- Delivery setup helpers ---

const DEFAULT_DELIVERY_PROMPT: &str = "\
Check my newsletter for new items since yesterday. \
Call ferrisletter_recap with since set to 24 hours ago. \
Summarize the headlines — for each item include the title, source, and a one-line summary. \
If there are more than 10 items, focus on the top 5 most interesting ones.";

fn generate_cron(frequency: &str, time: &str, day: &str) -> String {
    let parts: Vec<&str> = time.split(':').collect();
    let hour = parts
        .first()
        .and_then(|h| h.parse::<u32>().ok())
        .unwrap_or(9);
    let minute = parts
        .get(1)
        .and_then(|m| m.parse::<u32>().ok())
        .unwrap_or(0);

    match frequency {
        "weekdays" => format!("{minute} {hour} * * 1-5"),
        "weekly" => {
            let dow = match day.to_lowercase().as_str() {
                "monday" | "mon" => 1,
                "tuesday" | "tue" => 2,
                "wednesday" | "wed" => 3,
                "thursday" | "thu" => 4,
                "friday" | "fri" => 5,
                "saturday" | "sat" => 6,
                "sunday" | "sun" => 0,
                _ => 1,
            };
            format!("{minute} {hour} * * {dow}")
        }
        _ => format!("{minute} {hour} * * *"), // daily
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetupPreferencesParams {
    /// Topic IDs to subscribe to (e.g. ["rust", "ai"]).
    pub topics: Option<Vec<String>>,
    /// Tags to subscribe to (e.g. ["mcp", "async"]).
    pub tags: Option<Vec<String>>,
    /// Summary length preference: "brief", "standard", or "detailed".
    pub summary_length: Option<String>,
    /// Arbitrary key-value preferences for extensibility.
    pub preferences: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMyFeedParams {
    /// Maximum number of items to return.
    pub max_items: Option<usize>,
    /// Only return unread items.
    pub unread_only: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkReadParams {
    /// Item IDs to mark as read.
    pub item_ids: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddFavoriteParams {
    /// Item ID to save as a favorite.
    pub item_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveFavoriteParams {
    /// Item ID to remove from favorites.
    pub item_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListFavoritesParams {
    /// Maximum number of favorites to return.
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetupDeliveryParams {
    /// Delivery frequency: "daily", "weekdays", or "weekly".
    pub frequency: Option<String>,
    /// Preferred delivery time (HH:MM, 24h format). Defaults to "09:00".
    pub time: Option<String>,
    /// Preferred day for weekly delivery (e.g. "monday"). Only used when frequency is "weekly".
    pub day: Option<String>,
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

    /// Help set up scheduled delivery of the newsletter digest.
    #[tool(
        description = "Help set up scheduled delivery of the newsletter digest. \
        Returns a suggested cron expression and prompt for the LLM client's scheduler. \
        Supports daily, weekday, or weekly delivery."
    )]
    #[tracing::instrument(skip(self), name = "tool:setup_delivery")]
    async fn ferrisletter_setup_delivery(
        &self,
        Parameters(params): Parameters<SetupDeliveryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let freq = params.frequency.as_deref().unwrap_or("daily");
        let time = params.time.as_deref().unwrap_or("09:00");
        let day = params.day.as_deref().unwrap_or("monday");

        let cron = generate_cron(freq, time, day);
        let prompt = DEFAULT_DELIVERY_PROMPT;

        let resp = serde_json::json!({
            "suggested_cron": cron,
            "suggested_prompt": prompt,
            "instructions": "To set this up in Claude, create a scheduled task with the cron expression and prompt above."
        });

        let json = serde_json::to_string_pretty(&resp)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, 1, "delivery setup"));
        }
        Ok(tool_ok_text(json))
    }

    /// Save an article to favorites.
    #[tool(description = "Save an article to favorites. \
        Pass the item_id as returned by ferrisletter_get_latest or ferrisletter_search.")]
    #[tracing::instrument(skip(self), name = "tool:add_favorite")]
    async fn ferrisletter_add_favorite(
        &self,
        Parameters(params): Parameters<AddFavoriteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = "anonymous";
        self.favorites
            .add_favorite(user_id, &params.item_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let count = self
            .favorites
            .count_favorites(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let resp = serde_json::json!({
            "status": "saved",
            "item_id": params.item_id,
            "total_favorites": count,
        });
        let json = serde_json::to_string_pretty(&resp)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, count, "favorites"));
        }
        Ok(tool_ok_text(format!(
            "Saved to favorites. You now have {count} favorite(s)."
        )))
    }

    /// Remove an article from favorites.
    #[tool(description = "Remove an article from favorites. \
        Pass the item_id to unfavorite.")]
    #[tracing::instrument(skip(self), name = "tool:remove_favorite")]
    async fn ferrisletter_remove_favorite(
        &self,
        Parameters(params): Parameters<RemoveFavoriteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = "anonymous";
        self.favorites
            .remove_favorite(user_id, &params.item_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let count = self
            .favorites
            .count_favorites(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let resp = serde_json::json!({
            "status": "removed",
            "item_id": params.item_id,
            "total_favorites": count,
        });
        let json = serde_json::to_string_pretty(&resp)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, count, "favorites"));
        }
        Ok(tool_ok_text(format!(
            "Removed from favorites. You now have {count} favorite(s)."
        )))
    }

    /// List saved favorites.
    #[tool(
        description = "List saved favorites. Returns full item details for each favorited article. \
        Use limit to cap the number of results."
    )]
    #[tracing::instrument(skip(self), name = "tool:list_favorites")]
    async fn ferrisletter_list_favorites(
        &self,
        Parameters(params): Parameters<ListFavoritesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = "anonymous";
        let entries = self
            .favorites
            .list_favorites(user_id, params.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        // Resolve item IDs to full Items via the connector.
        let conn = self.conn().await;
        let mut items: Vec<Item> = Vec::new();
        for entry in &entries {
            match conn.get_item_detail(&entry.item_id).await {
                Ok(detail) => items.push(detail.item),
                Err(_) => {
                    // Item may have been removed from the feed — include a stub.
                    items.push(Item {
                        id: entry.item_id.clone(),
                        topic_id: String::new(),
                        headline: format!("[unavailable] {}", entry.item_id),
                        summary: String::new(),
                        tags: Vec::new(),
                        source: String::new(),
                        published: entry.saved_at,
                    });
                }
            }
        }

        let count = items.len();
        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "favorites"));
        }

        if items.is_empty() {
            return Ok(tool_ok_text(
                "No favorites yet. Save articles you want to revisit with ferrisletter_add_favorite."
                    .to_string(),
            ));
        }

        let mut text = format!("You have {} saved favorite(s):\n", count);
        for (i, item) in items.iter().enumerate() {
            let date = item.published.format("%Y-%m-%d");
            text.push_str(&format!("\n{}. **{}**", i + 1, item.headline));
            if !item.source.is_empty() {
                text.push_str(&format!("\n   {} | {}", item.source, date));
            }
            if let Some(entry) = entries.get(i) {
                let saved = entry.saved_at.format("%Y-%m-%d");
                text.push_str(&format!("\n   Saved on: {saved}"));
            }
        }
        Ok(tool_ok_text(text))
    }

    /// Set or update user preferences (topics, tags, display settings).
    #[tool(description = "Set or update your content preferences. \
        Provide topics and tags you're interested in. \
        Only provided fields are updated — omitted fields keep their current values.")]
    #[tracing::instrument(skip(self), name = "tool:setup_preferences")]
    async fn ferrisletter_setup_preferences(
        &self,
        Parameters(params): Parameters<SetupPreferencesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let users = self.require_user_store()?;
        let user_id = "anonymous";

        // Ensure the user row exists so changes persist.
        users
            .upsert_user(user_id, None, None)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if let Some(topics) = params.topics.as_ref() {
            users
                .set_topics(user_id, topics)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        if let Some(tags) = params.tags.as_ref() {
            users
                .set_tags(user_id, tags)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        if let Some(summary) = params.summary_length.as_ref() {
            users
                .set_preference(user_id, "summary_length", summary)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }
        if let Some(extras) = params.preferences.as_ref() {
            for (k, v) in extras {
                users
                    .set_preference(user_id, k, v)
                    .await
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            }
        }

        // Return the updated profile.
        let profile = users
            .get_profile(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&profile)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, 1, "profile"));
        }
        Ok(tool_ok_text(format!(
            "Preferences updated. Current profile:\n{json}"
        )))
    }

    /// Get the user's current profile and preferences.
    #[tool(
        description = "Get your current profile and content preferences, including \
        subscribed topics, tags, and key-value preferences."
    )]
    #[tracing::instrument(skip(self), name = "tool:get_preferences")]
    async fn ferrisletter_get_preferences(&self) -> Result<CallToolResult, ErrorData> {
        let users = self.require_user_store()?;
        let user_id = "anonymous";

        let profile = users
            .get_profile(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if profile.is_none() {
            return Ok(tool_ok_text(
                "No profile yet. Call ferrisletter_setup_preferences to get started.".to_string(),
            ));
        }

        let json = serde_json::to_string_pretty(&profile)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, 1, "profile"));
        }
        Ok(tool_ok_text(json))
    }

    /// Get a personalized feed filtered by the user's topic/tag subscriptions.
    #[tool(
        description = "Get your personalized feed filtered by your subscribed topics and tags. \
        Items are annotated with read state. Use unread_only=true to skip already-read items."
    )]
    #[tracing::instrument(skip(self), name = "tool:get_my_feed")]
    async fn ferrisletter_get_my_feed(
        &self,
        Parameters(params): Parameters<GetMyFeedParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let users = self.require_user_store()?;
        let user_id = "anonymous";

        let topics = users
            .get_topics(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let tags = users
            .get_tags(user_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let prefs = UserPrefs {
            topics_of_interest: topics.clone(),
            max_items: params.max_items,
            ..Default::default()
        };

        let mut items = self
            .conn()
            .await
            .get_latest_items(&prefs)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        // Tag filter (server-side if we have subscribed tags).
        if !tags.is_empty() {
            items.retain(|it| it.tags.iter().any(|t| tags.contains(t)));
        }

        // Split read / unread.
        let candidate_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
        let mut read_set = std::collections::HashSet::new();
        for id in &candidate_ids {
            if users
                .is_read(user_id, id)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            {
                read_set.insert(id.clone());
            }
        }

        if params.unread_only.unwrap_or(false) {
            items.retain(|i| !read_set.contains(&i.id));
        }

        let count = items.len();
        let unread = items.iter().filter(|i| !read_set.contains(&i.id)).count();

        if self.should_use_ui() {
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            return Ok(tool_ok_ui(json, count, "items"));
        }

        if items.is_empty() {
            return Ok(tool_ok_text(
                "No items in your personalized feed. Try subscribing to more topics with \
                 ferrisletter_setup_preferences."
                    .to_string(),
            ));
        }

        let mut text = format!("Your personalized feed — {count} item(s), {unread} unread:\n");
        for (i, item) in items.iter().enumerate() {
            let date = item.published.format("%Y-%m-%d");
            let marker = if read_set.contains(&item.id) {
                " [read]"
            } else {
                ""
            };
            text.push_str(&format!("\n{}. **{}**{}", i + 1, item.headline, marker));
            text.push_str(&format!("\n   {} | {}", item.source, date));
        }
        Ok(tool_ok_text(text))
    }

    /// Mark items as read.
    #[tool(description = "Mark one or more items as read. Pass the item_ids to mark.")]
    #[tracing::instrument(skip(self), name = "tool:mark_read")]
    async fn ferrisletter_mark_read(
        &self,
        Parameters(params): Parameters<MarkReadParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let users = self.require_user_store()?;
        let user_id = "anonymous";

        users
            .mark_read(user_id, &params.item_ids)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let count = params.item_ids.len();
        let resp = serde_json::json!({
            "status": "ok",
            "marked_read": count,
        });
        let json = serde_json::to_string_pretty(&resp)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if self.should_use_ui() {
            return Ok(tool_ok_ui(json, count, "items"));
        }
        Ok(tool_ok_text(format!("Marked {count} item(s) as read.")))
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
                or catch up on what you missed with `ferrisletter_recap`. \
                Save articles with `ferrisletter_add_favorite` and retrieve them later \
                with `ferrisletter_list_favorites`. \
                Personalize the experience with `ferrisletter_setup_preferences` \
                (topics, tags, display settings) and retrieve a personalized, \
                read-tracked feed via `ferrisletter_get_my_feed`. \
                Track what you've seen with `ferrisletter_mark_read`.",
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
