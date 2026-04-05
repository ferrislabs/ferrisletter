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
//! use ferrisletter_server::{FerrislletterServer, transport};
//!
//! # async fn run() -> anyhow::Result<()> {
//! // Create your connector (any implementation of the Connector trait).
//! # let my_connector: Arc<BoxedConnector> = todo!();
//! let handle = Arc::new(RwLock::new(my_connector));
//!
//! // Build the MCP server.
//! let server = FerrislletterServer::new(handle, /* ui_enabled */ true);
//!
//! // Start over stdio transport.
//! transport::serve_stdio(server).await?;
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod config;
pub mod health;
pub mod server;
#[cfg(feature = "telemetry")]
pub mod telemetry;
pub mod transport;

pub use config::{AdminConfig, Config, ConnectorConfig, TelemetryConfig, TransportMode, UiConfig};
pub use health::ServerState;
pub use server::{FerrislletterServer, UI_RESOURCE_URI};
pub use transport::{serve_sse, serve_stdio};
