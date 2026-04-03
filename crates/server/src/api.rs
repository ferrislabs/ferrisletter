//! Management REST API for live connector CRUD.
//!
//! Runs on a separate bind address from the MCP SSE transport.
//! All routes require `Authorization: Bearer <api_key>` unless the api_key is empty.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use ferrisletter_connector::BoxedConnector;
use ferrisletter_connector_rss::{FeedConfig as RssFeedConfig, RssConnector};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared connector handle — allows hot-swapping the connector.
// ---------------------------------------------------------------------------

pub type ConnectorHandle = Arc<RwLock<Arc<BoxedConnector>>>;

// ---------------------------------------------------------------------------
// Data records
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicRecord {
    pub id: String,
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedRecord {
    pub id: String,
    pub topic_id: String,
    pub url: String,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct ApiState {
    pub topics: RwLock<Vec<TopicRecord>>,
    pub feeds: RwLock<Vec<FeedRecord>>,
    pub connector_handle: ConnectorHandle,
    pub api_key: String,
}

impl ApiState {
    pub fn new(
        topics: Vec<TopicRecord>,
        feeds: Vec<FeedRecord>,
        connector_handle: ConnectorHandle,
        api_key: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            topics: RwLock::new(topics),
            feeds: RwLock::new(feeds),
            connector_handle,
            api_key,
        })
    }
}

/// Rebuild the live connector from the current topics + feeds.
async fn rebuild(state: &Arc<ApiState>) {
    let topics = state.topics.read().await;
    let feeds = state.feeds.read().await;

    let rss_feeds: Vec<RssFeedConfig> = feeds
        .iter()
        .filter_map(|f| {
            let topic = topics.iter().find(|t| t.id == f.topic_id)?;
            Some(RssFeedConfig {
                topic_id: topic.id.clone(),
                topic_label: topic.label.clone(),
                topic_description: topic.description.clone(),
                topic_tags: topic.tags.clone(),
                url: f.url.clone(),
                refresh_minutes: None,
            })
        })
        .collect();

    let connector = Arc::new(BoxedConnector::new(RssConnector::new(rss_feeds)));
    *state.connector_handle.write().await = connector;
    tracing::info!("connector reloaded ({} feed(s))", feeds.len());
}

// ---------------------------------------------------------------------------
// Auth middleware (extractor)
// ---------------------------------------------------------------------------

struct ApiKey;

impl ApiKey {
    fn check(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
        if expected.is_empty() {
            return Ok(()); // auth disabled
        }
        let provided = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");
        if provided == expected {
            Ok(())
        } else {
            Err(ApiError::Unauthorized)
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum ApiError {
    Unauthorized,
    NotFound(String),
    Conflict(String),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// ---------------------------------------------------------------------------
// Topic endpoints
// ---------------------------------------------------------------------------

async fn list_topics(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<TopicRecord>>, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;
    Ok(Json(state.topics.read().await.clone()))
}

#[derive(Debug, Deserialize)]
struct CreateTopicBody {
    id: String,
    label: String,
    description: String,
    #[serde(default)]
    tags: Vec<String>,
}

async fn create_topic(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(body): Json<CreateTopicBody>,
) -> Result<(StatusCode, Json<TopicRecord>), ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    if body.id.is_empty() {
        return Err(ApiError::BadRequest("id is required".into()));
    }

    let mut topics = state.topics.write().await;
    if topics.iter().any(|t| t.id == body.id) {
        return Err(ApiError::Conflict(format!(
            "topic '{}' already exists",
            body.id
        )));
    }

    let record = TopicRecord {
        id: body.id,
        label: body.label,
        description: body.description,
        tags: body.tags,
    };
    topics.push(record.clone());
    drop(topics);

    rebuild(&state).await;
    Ok((StatusCode::CREATED, Json(record)))
}

#[derive(Debug, Deserialize)]
struct UpdateTopicBody {
    label: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
}

async fn update_topic(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateTopicBody>,
) -> Result<Json<TopicRecord>, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    let mut topics = state.topics.write().await;
    let topic = topics
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| ApiError::NotFound(format!("topic '{id}' not found")))?;

    if let Some(label) = body.label {
        topic.label = label;
    }
    if let Some(description) = body.description {
        topic.description = description;
    }
    if let Some(tags) = body.tags {
        topic.tags = tags;
    }
    let updated = topic.clone();
    drop(topics);

    rebuild(&state).await;
    Ok(Json(updated))
}

async fn delete_topic(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    let mut topics = state.topics.write().await;
    let before = topics.len();
    topics.retain(|t| t.id != id);
    if topics.len() == before {
        return Err(ApiError::NotFound(format!("topic '{id}' not found")));
    }
    drop(topics);

    // Cascade: remove feeds assigned to this topic.
    state.feeds.write().await.retain(|f| f.topic_id != id);

    rebuild(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Feed (connector) endpoints
// ---------------------------------------------------------------------------

async fn list_connectors(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<FeedRecord>>, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;
    Ok(Json(state.feeds.read().await.clone()))
}

#[derive(Debug, Deserialize)]
struct CreateFeedBody {
    topic_id: String,
    url: String,
}

async fn create_feed(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(body): Json<CreateFeedBody>,
) -> Result<(StatusCode, Json<FeedRecord>), ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    if body.url.is_empty() {
        return Err(ApiError::BadRequest("url is required".into()));
    }

    // Ensure the topic exists.
    {
        let topics = state.topics.read().await;
        if !topics.iter().any(|t| t.id == body.topic_id) {
            return Err(ApiError::NotFound(format!(
                "topic '{}' not found",
                body.topic_id
            )));
        }
    }

    let record = FeedRecord {
        id: Uuid::new_v4().to_string(),
        topic_id: body.topic_id,
        url: body.url,
    };

    state.feeds.write().await.push(record.clone());

    rebuild(&state).await;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn delete_feed(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    let mut feeds = state.feeds.write().await;
    let before = feeds.len();
    feeds.retain(|f| f.id != id);
    if feeds.len() == before {
        return Err(ApiError::NotFound(format!("feed '{id}' not found")));
    }
    drop(feeds);

    rebuild(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Config export endpoint
// ---------------------------------------------------------------------------

async fn export_config(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    ApiKey::check(&headers, &state.api_key)?;

    let topics = state.topics.read().await;
    let feeds = state.feeds.read().await;

    let mut toml = String::from(
        "# ferrisletter.toml — exported by Ferrisletter Admin API\n\
         # Edit and run: ferrisletter-server --config ferrisletter.toml\n\n\
         [transport]\n\
         mode = \"sse\"\n\
         host = \"127.0.0.1\"\n\
         port = 3000\n\n\
         [connector]\n\
         type = \"rss\"\n",
    );

    for feed in feeds.iter() {
        let Some(topic) = topics.iter().find(|t| t.id == feed.topic_id) else {
            continue;
        };
        let tags = topic
            .tags
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");
        toml.push_str(&format!(
            "\n[[connector.feeds]]\n\
             topic_id          = \"{}\"\n\
             topic_label       = \"{}\"\n\
             topic_description = \"{}\"\n\
             topic_tags        = [{}]\n\
             url               = \"{}\"\n",
            topic.id, topic.label, topic.description, tags, feed.url,
        ));
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        toml,
    ))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: Arc<ApiState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/topics", get(list_topics).post(create_topic))
        .route("/api/topics/:id", put(update_topic).delete(delete_topic))
        .route("/api/connectors", get(list_connectors))
        .route("/api/connectors/rss", post(create_feed))
        .route("/api/connectors/rss/:id", delete(delete_feed))
        .route("/api/config", get(export_config))
        .with_state(state)
        .layer(cors)
}

/// Bind and serve the admin REST API.
pub async fn serve(state: Arc<ApiState>, addr: SocketAddr) {
    let app = router(state);
    tracing::info!(%addr, "admin REST API listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind admin API");
    axum::serve(listener, app)
        .await
        .expect("admin API server error");
}
