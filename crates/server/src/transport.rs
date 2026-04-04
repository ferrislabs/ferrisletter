//! Transport helpers for running the Ferrisletter MCP server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    Json,
    extract::Query,
    response::Redirect,
    routing::{get, post},
};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};

use crate::server::FerrislletterServer;

/// Default graceful shutdown timeout.
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the SSE/HTTP transport.
pub struct SseConfig {
    /// Optional public base URL for OAuth stub endpoints (e.g. `https://abc.ngrok-free.app`).
    pub public_url: Option<String>,
    /// Graceful shutdown timeout. Defaults to 30 seconds.
    pub shutdown_timeout: Option<Duration>,
}

/// Start the MCP server over stdio transport.
///
/// Handles SIGINT (Ctrl+C) for clean shutdown.
pub async fn serve_stdio(server: FerrislletterServer) -> anyhow::Result<()> {
    tracing::info!("serving MCP over stdio");
    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("failed to start server: {e}"))?;

    tokio::select! {
        result = service.waiting() => {
            result.inspect_err(|e| tracing::error!("server error: {e}"))?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received SIGINT, shutting down");
        }
    }

    Ok(())
}

/// Start the MCP server over SSE/HTTP transport.
///
/// Listens for SIGTERM and SIGINT to initiate graceful shutdown, draining
/// in-flight connections before exiting.
pub async fn serve_sse(
    server: FerrislletterServer,
    addr: SocketAddr,
    config: &SseConfig,
) -> anyhow::Result<()> {
    tracing::info!(%addr, "serving MCP over streamable HTTP");

    let ct = CancellationToken::new();
    let shutdown_timeout = config.shutdown_timeout.unwrap_or(DEFAULT_SHUTDOWN_TIMEOUT);

    // Listen for SIGTERM/SIGINT to trigger graceful shutdown.
    let shutdown_ct = ct.clone();
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to register SIGTERM handler");

        tokio::select! {
            _ = sigterm.recv() => {
                tracing::info!("received SIGTERM, initiating graceful shutdown");
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("received SIGINT, initiating graceful shutdown");
            }
        }

        shutdown_ct.cancel();
    });

    let service: StreamableHttpService<FerrislletterServer, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let s = server.clone();
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
    if let Some(public_url) = config.public_url.clone() {
        tracing::info!(%public_url, "OAuth stub enabled");
        router = add_oauth_stub(router, public_url);
    }

    let router = router.layer(cors);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            ct.cancelled_owned().await;
            tracing::info!(timeout = ?shutdown_timeout, "draining connections...");
        })
        .await?;

    tracing::info!("shutdown complete");
    Ok(())
}

/// Add stub OAuth 2.0 endpoints so hosting clients (e.g. claude.ai) that
/// require OAuth discovery can connect without a real auth server.
///
/// Every authorization attempt succeeds immediately — this is intentionally
/// insecure and only suitable for local/dev use.
fn add_oauth_stub(router: axum::Router, base: String) -> axum::Router {
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
