//! Transport helpers for running the Ferrisletter MCP server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Json,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};

use crate::auth::{AuthUser, BoxedAuthProvider};
use crate::health::{self, ServerState};
use crate::server::FerrislletterServer;

/// Default graceful shutdown timeout.
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the SSE/HTTP transport.
pub struct SseConfig {
    /// Optional public base URL for OAuth metadata (e.g. `https://abc.ngrok-free.app`).
    pub public_url: Option<String>,
    /// Graceful shutdown timeout. Defaults to 30 seconds.
    pub shutdown_timeout: Option<Duration>,
    /// Whether real auth is enabled (`[auth] enabled = true` in config).
    /// When true, the auth middleware validates tokens and the old OAuth
    /// stub endpoints (`/authorize`, `/token`, `/register`) are **not** mounted.
    pub auth_enabled: bool,
}

/// Start the MCP server over stdio transport.
///
/// Handles SIGINT (Ctrl+C) for clean shutdown.
/// Auth is always anonymous over stdio per the MCP spec.
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
/// When auth is enabled the server:
/// - serves `/.well-known/oauth-protected-resource` from the auth provider
/// - applies auth middleware (bearer token → AuthUser)
///
/// When auth is disabled, the old OAuth stubs are mounted instead.
pub async fn serve_sse(
    server: FerrislletterServer,
    addr: SocketAddr,
    config: &SseConfig,
    server_state: ServerState,
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
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz).with_state(server_state))
        .nest_service("/mcp", service.clone())
        .fallback_service(service);

    let auth = server.auth().clone();

    if config.auth_enabled {
        // Real auth — protected resource metadata + auth middleware.
        tracing::info!("auth middleware enabled");
        let public_url = config.public_url.clone().unwrap_or_default();
        router = add_protected_resource_endpoint(router, auth.clone(), public_url);
        router = router.layer(axum::middleware::from_fn_with_state(
            auth.clone(),
            auth_middleware,
        ));
    } else if let Some(public_url) = config.public_url.clone() {
        // No auth — mount OAuth stubs for claude.ai compatibility.
        tracing::info!(%public_url, "OAuth stub enabled (auth disabled)");
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

// ── Protected Resource Metadata (RFC 9728) ───────────────────────────────

/// Add the `/.well-known/oauth-protected-resource` endpoint.
///
/// Returns the resource identifier, authorization servers, and supported
/// scopes — all sourced from the [`AuthProvider`](crate::auth::AuthProvider).
pub fn add_protected_resource_endpoint(
    router: axum::Router,
    auth: Arc<BoxedAuthProvider>,
    public_url: String,
) -> axum::Router {
    let handler = move |State(auth): State<Arc<BoxedAuthProvider>>| {
        let public_url = public_url.clone();
        async move {
            Json(serde_json::json!({
                "resource": public_url,
                "authorization_servers": auth.authorization_servers(),
                "scopes_supported": auth.scopes_supported(),
            }))
        }
    };

    router.route(
        "/.well-known/oauth-protected-resource",
        get(handler).with_state(auth),
    )
}

// ── Auth Middleware ───────────────────────────────────────────────────────

/// Axum middleware that extracts `Authorization: Bearer <token>`, calls the
/// auth provider, and injects [`AuthUser`] into request extensions.
///
/// - Well-known endpoints (`.well-known/*`) always pass through.
/// - Missing or invalid token → 401 with `WWW-Authenticate` header.
pub async fn auth_middleware(
    State(auth): State<Arc<BoxedAuthProvider>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // Always allow well-known discovery endpoints and health probes.
    if path.starts_with("/.well-known/") || path == "/healthz" || path == "/readyz" {
        request.extensions_mut().insert(AuthUser::anonymous());
        return next.run(request).await;
    }

    // Extract bearer token.
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => {
            return unauthorized_response(&auth);
        }
    };

    // Validate the token.
    match auth.authenticate(token).await {
        Ok(Some(user)) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Ok(None) => unauthorized_response(&auth),
        Err(e) => {
            tracing::warn!("auth provider error: {e}");
            unauthorized_response(&auth)
        }
    }
}

/// Build a 401 response with `WWW-Authenticate` header pointing to the IAM.
fn unauthorized_response(auth: &BoxedAuthProvider) -> Response {
    let servers = auth.authorization_servers();
    let www_auth = if let Some(server) = servers.first() {
        format!("Bearer realm=\"{server}\"")
    } else {
        "Bearer".to_string()
    };

    (
        StatusCode::UNAUTHORIZED,
        [("WWW-Authenticate", www_auth)],
        Json(serde_json::json!({ "error": "unauthorized" })),
    )
        .into_response()
}

// ── OAuth stubs (dev mode) ───────────────────────────────────────────────

/// Add stub OAuth 2.0 endpoints so hosting clients (e.g. claude.ai) that
/// require OAuth discovery can connect without a real auth server.
///
/// Every authorization attempt succeeds immediately — this is intentionally
/// insecure and only suitable for local/dev use.
pub fn add_oauth_stub(router: axum::Router, base: String) -> axum::Router {
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
