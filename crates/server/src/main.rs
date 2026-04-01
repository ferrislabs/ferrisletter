mod server;

use std::sync::Arc;

use ferrisletter_connector::BoxedConnector;
use ferrisletter_connector_static::StaticConnector;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use server::FerrislletterServer;

/// Embedded sample data — used when `FERRISLETTER_DATA` is not set.
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

    // Load connector — from file if FERRISLETTER_DATA is set, otherwise use embedded sample.
    let connector: Arc<BoxedConnector> = match std::env::var("FERRISLETTER_DATA") {
        Ok(path) => {
            tracing::info!(path, "loading data from file");
            Arc::new(BoxedConnector::new(
                StaticConnector::from_file(&path)
                    .map_err(|e| anyhow::anyhow!("failed to load '{path}': {e}"))?,
            ))
        }
        Err(_) => {
            tracing::info!("no FERRISLETTER_DATA set — using embedded sample data");
            Arc::new(BoxedConnector::new(
                StaticConnector::from_json(SAMPLE_DATA)
                    .expect("embedded sample data must be valid"),
            ))
        }
    };

    let server = FerrislletterServer::new(connector);

    tracing::info!("serving over stdio");
    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("failed to start server: {e}"))?;

    service
        .waiting()
        .await
        .inspect_err(|e| tracing::error!("server error: {e}"))?;

    Ok(())
}
