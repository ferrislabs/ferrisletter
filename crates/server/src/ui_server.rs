//! Static HTTP server that serves the embedded MCP App UI bundle.

use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use tower_http::cors::{Any, CorsLayer};

/// The single-file UI bundle produced by `npm run build` in `ui/`.
/// Embedded at compile time via build.rs.
pub const UI_BUNDLE: &str = include_str!(concat!(env!("OUT_DIR"), "/ui_bundle.html"));

async fn serve_index() -> impl IntoResponse {
    Html(UI_BUNDLE)
}

pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new().route("/", get(serve_index)).layer(cors)
}

pub async fn serve(bind_addr: &str) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!("MCP App UI serving on http://{}", bind_addr);
    axum::serve(listener, router()).await?;
    Ok(())
}
