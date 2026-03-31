/// Errors that can occur when interacting with a connector.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("item not found: {0}")]
    NotFound(String),

    #[error("authentication failed: {0}")]
    AuthError(String),

    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("upstream source unavailable: {0}")]
    Unavailable(String),

    #[error("{0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
