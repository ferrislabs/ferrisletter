mod api;
mod config;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json,
    extract::Query,
    response::Redirect,
    routing::{get, post},
};
use ferrisletter_connector::BoxedConnector;
use ferrisletter_connector_rss::{FeedConfig as RssFeedConfig, RssConnector};
use ferrisletter_connector_static::StaticConnector;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use server::FerrislletterServer;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};

use crate::api::{ApiState, ConnectorHandle, FeedRecord, TopicRecord};
use crate::config::{ConnectorConfig, TransportMode};

/// Embedded sample data — used when no config or data file is provided.
const SAMPLE_DATA: &str = include_str!("../data/sample.json");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // MCP stdio transport uses stdout for the protocol — send logs to stderr.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("ferrisletter v{} starting", env!("CARGO_PKG_VERSION"));

    // `--config <path>` CLI arg takes precedence over everything else.
    let cli_config: Option<String> = std::env::args().skip_while(|a| a != "--config").nth(1);

    let cfg = config::load(cli_config.as_deref())
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .unwrap_or_default();

    // Seed the API state from the config so the REST API reflects the initial feeds.
    let (initial_topics, initial_feeds) = extract_api_records(&cfg.connector);

    // Build the initial connector.
    let initial_connector: Arc<BoxedConnector> =
        build_connector(&cfg.connector, cli_config.as_deref()).await?;

    // Wrap in a hot-swappable handle.
    let connector_handle: ConnectorHandle = Arc::new(RwLock::new(initial_connector));

    // Build the API state (shared between REST API and used to rebuild the connector).
    let api_state = ApiState::new(
        initial_topics,
        initial_feeds,
        connector_handle.clone(),
        cfg.admin.api_key.clone(),
    );

    // Spawn the admin REST API if enabled.
    if cfg.admin.enabled {
        let addr: SocketAddr = cfg
            .admin
            .bind_addr
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid admin bind_addr: {}", cfg.admin.bind_addr))?;
        let state = api_state.clone();
        tokio::spawn(async move { api::serve(state, addr).await });
        tracing::info!(
            addr = %cfg.admin.bind_addr,
            auth = !cfg.admin.api_key.is_empty(),
            "admin REST API enabled"
        );
    }

    if cfg.ui.enabled {
        tracing::info!(
            resource = server::UI_RESOURCE_URI,
            "MCP App UI enabled (mcpui.dev)"
        );
    }

    let mcp_server = FerrislletterServer::new(connector_handle, cfg.ui.enabled);

    // Start MCP transport.
    match cfg.transport.mode {
        TransportMode::Sse => {
            let addr: SocketAddr =
                format!("{}:{}", cfg.transport.host, cfg.transport.port).parse()?;
            tracing::info!(%addr, "serving MCP over streamable HTTP");

            let ct = CancellationToken::new();
            let service: StreamableHttpService<FerrislletterServer, LocalSessionManager> =
                StreamableHttpService::new(
                    {
                        let s = mcp_server.clone();
                        move || Ok(s.clone())
                    },
                    Default::default(),
                    StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
                );

            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);

            let mut router = axum::Router::new()
                .nest_service("/mcp", service.clone())
                .fallback_service(service);

            // Stub OAuth 2.0 endpoints so claude.ai can connect without real auth.
            if let Some(public_url) = cfg.transport.public_url.clone() {
                tracing::info!(%public_url, "OAuth stub enabled");
                router = add_oauth_stub(router, public_url);
            }

            let router = router.layer(cors);
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await?;
        }
        TransportMode::Stdio => {
            tracing::info!("serving MCP over stdio");
            let service = mcp_server
                .serve(stdio())
                .await
                .inspect_err(|e| tracing::error!("failed to start server: {e}"))?;
            service
                .waiting()
                .await
                .inspect_err(|e| tracing::error!("server error: {e}"))?;
        }
    }

    Ok(())
}

/// Extract initial TopicRecord + FeedRecord lists from the config connector.
fn extract_api_records(connector: &ConnectorConfig) -> (Vec<TopicRecord>, Vec<FeedRecord>) {
    match connector {
        ConnectorConfig::Rss { feeds } => {
            // Deduplicate topics by id (multiple feeds can share a topic).
            let mut topics: Vec<TopicRecord> = Vec::new();
            let mut feed_records: Vec<FeedRecord> = Vec::new();

            for f in feeds {
                if !topics.iter().any(|t: &TopicRecord| t.id == f.topic_id) {
                    topics.push(TopicRecord {
                        id: f.topic_id.clone(),
                        label: f.topic_label.clone(),
                        description: f.topic_description.clone(),
                        tags: f.topic_tags.clone(),
                    });
                }
                feed_records.push(FeedRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    topic_id: f.topic_id.clone(),
                    url: f.url.clone(),
                });
            }
            (topics, feed_records)
        }
        _ => (Vec::new(), Vec::new()),
    }
}

/// Build the initial BoxedConnector from config.
async fn build_connector(
    connector_cfg: &ConnectorConfig,
    cli_config: Option<&str>,
) -> anyhow::Result<Arc<BoxedConnector>> {
    match connector_cfg {
        ConnectorConfig::Static { path } if !path.as_os_str().is_empty() => {
            tracing::info!(path = %path.display(), "loading static connector");
            Ok(Arc::new(BoxedConnector::new(
                StaticConnector::from_file(path)
                    .map_err(|e| anyhow::anyhow!("failed to load '{}': {e}", path.display()))?,
            )))
        }
        ConnectorConfig::Rss { feeds } => {
            tracing::info!(feeds = feeds.len(), "loading RSS connector");
            let rss_feeds = feeds
                .iter()
                .map(|f| RssFeedConfig {
                    topic_id: f.topic_id.clone(),
                    topic_label: f.topic_label.clone(),
                    topic_description: f.topic_description.clone(),
                    topic_tags: f.topic_tags.clone(),
                    url: f.url.clone(),
                })
                .collect();
            Ok(Arc::new(BoxedConnector::new(RssConnector::new(rss_feeds))))
        }
        _ => {
            // No config or empty static path — try env var then embedded sample.
            let _ = cli_config; // unused in this branch
            match std::env::var("FERRISLETTER_DATA") {
                Ok(path) => {
                    tracing::info!(path, "loading static connector from FERRISLETTER_DATA");
                    Ok(Arc::new(BoxedConnector::new(
                        StaticConnector::from_file(&path)
                            .map_err(|e| anyhow::anyhow!("failed to load '{path}': {e}"))?,
                    )))
                }
                Err(_) => {
                    tracing::info!("using embedded sample data");
                    Ok(Arc::new(BoxedConnector::new(
                        StaticConnector::from_json(SAMPLE_DATA)
                            .expect("embedded sample data must be valid"),
                    )))
                }
            }
        }
    }
}

/// Add stub OAuth 2.0 endpoints so hosting clients (e.g. claude.ai) that
/// require OAuth discovery can connect without a real auth server.
///
/// Every authorization attempt succeeds immediately — this is intentionally
/// insecure and only suitable for local/dev use.
fn add_oauth_stub(router: axum::Router, base: String) -> axum::Router {
    use std::collections::HashMap;

    let b = base.clone();
    let protected_resource = move || {
        let b = b.clone();
        async move {
            Json(serde_json::json!({
                "resource": b,
                "authorization_servers": [b]
            }))
        }
    };

    let b = base.clone();
    let auth_server_meta = move || {
        let b = b.clone();
        async move {
            Json(serde_json::json!({
                "issuer": b,
                "authorization_endpoint": format!("{b}/authorize"),
                "token_endpoint": format!("{b}/token"),
                "registration_endpoint": format!("{b}/register"),
                "response_types_supported": ["code"],
                "grant_types_supported": ["authorization_code", "client_credentials"],
                "code_challenge_methods_supported": ["S256"]
            }))
        }
    };

    let register = || async {
        Json(serde_json::json!({
            "client_id": "ferrisletter",
            "client_secret": "stub",
            "redirect_uris": [],
            "grant_types": ["authorization_code"],
            "response_types": ["code"]
        }))
    };

    let authorize = |Query(params): Query<HashMap<String, String>>| async move {
        let redirect_uri = params.get("redirect_uri").cloned().unwrap_or_default();
        let state = params.get("state").cloned().unwrap_or_default();
        Redirect::temporary(&format!("{redirect_uri}?code=stub-code&state={state}"))
    };

    let token = || async {
        Json(serde_json::json!({
            "access_token": "stub-token",
            "token_type": "Bearer",
            "expires_in": 86400
        }))
    };

    router
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(auth_server_meta),
        )
        .route("/register", post(register))
        .route("/authorize", get(authorize))
        .route("/token", post(token))
}
