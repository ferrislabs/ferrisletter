//! Ferrisletter MCP server library.
//!
//! This crate can be used both as a standalone binary (`ferrisletter-server`)
//! and as a library. External crates (e.g. Lattice) can import the server logic
//! to reuse MCP tool handling, transport setup, and config types.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//! use ferrisletter_connector::BoxedConnector;
//! use ferrisletter_server::{BoxedFavoriteStore, FerrislletterServer, InMemoryFavoriteStore, transport};
//!
//! # async fn run() -> anyhow::Result<()> {
//! // Create your connector (any implementation of the Connector trait).
//! # let my_connector: Arc<BoxedConnector> = todo!();
//! let handle = Arc::new(RwLock::new(my_connector));
//!
//! // Create a favorites store.
//! let favorites = Arc::new(BoxedFavoriteStore::new(InMemoryFavoriteStore::new(None)));
//!
//! // Build the MCP server.
//! let server = FerrislletterServer::new(handle, /* ui_enabled */ true, favorites);
//!
//! // Start over stdio transport.
//! transport::serve_stdio(server).await?;
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod auth;
pub mod config;
pub mod favorites;
pub mod health;
pub mod server;
#[cfg(feature = "telemetry")]
pub mod telemetry;
pub mod theme;
pub mod transport;
pub mod users;

pub use auth::{AuthProvider, AuthUser, BoxedAuthProvider, NoAuthProvider, OidcAuthProvider};
pub use config::{
    AdminConfig, AuthConfig, Config, ConnectorConfig, OidcConfig, TelemetryConfig, TransportMode,
    UiConfig,
};
pub use favorites::{BoxedFavoriteStore, FavoriteStore, InMemoryFavoriteStore};
pub use health::ServerState;
pub use server::{FerrislletterServer, UI_RESOURCE_URI};
pub use theme::{DisplayPreferences, Theme, ThemeInfo, ThemeRegistry};
pub use transport::{serve_sse, serve_stdio};
pub use users::{BoxedUserStore, InMemoryUserStore, User, UserProfile, UserStore};
