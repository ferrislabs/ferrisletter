//! Health and readiness probes for container orchestrators.
//!
//! Provides `/healthz` (liveness) and `/readyz` (readiness) HTTP endpoints
//! for use with Kubernetes, Docker Compose, fly.io, etc.
//!
//! These endpoints are only active when using the SSE/HTTP transport.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Shared server state for health probes.
///
/// Tracks whether the server is ready to serve requests (i.e. the connector
/// has been loaded successfully). Pass this as Axum state to the readiness
/// endpoint.
#[derive(Clone)]
pub struct ServerState {
    connector_loaded: Arc<AtomicBool>,
}

impl ServerState {
    /// Create a new [`ServerState`] with readiness set to `false`.
    pub fn new() -> Self {
        Self {
            connector_loaded: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Mark the server as ready (connector loaded successfully).
    pub fn set_ready(&self) {
        self.connector_loaded.store(true, Ordering::Relaxed);
    }

    /// Check whether the server is ready to serve requests.
    pub fn is_ready(&self) -> bool {
        self.connector_loaded.load(Ordering::Relaxed)
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Liveness probe — returns 200 if the process is running.
///
/// Mapped to `GET /healthz`.
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// Readiness probe — returns 200 only when the server is ready to serve.
///
/// Mapped to `GET /readyz`. Returns 503 Service Unavailable if the connector
/// has not yet been loaded.
pub async fn readyz(
    axum::extract::State(state): axum::extract::State<ServerState>,
) -> impl IntoResponse {
    if state.is_ready() {
        (StatusCode::OK, Json(serde_json::json!({"status": "ready"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not_ready"})),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_state_starts_not_ready() {
        let state = ServerState::new();
        assert!(!state.is_ready());
    }

    #[test]
    fn server_state_becomes_ready() {
        let state = ServerState::new();
        state.set_ready();
        assert!(state.is_ready());
    }

    #[test]
    fn server_state_clone_shares_readiness() {
        let state = ServerState::new();
        let cloned = state.clone();
        state.set_ready();
        assert!(cloned.is_ready());
    }
}
