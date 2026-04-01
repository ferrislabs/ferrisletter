mod config;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use ferrisletter_connector::BoxedConnector;
use ferrisletter_connector_rss::{FeedConfig as RssFeedConfig, RssConnector};
use ferrisletter_connector_static::StaticConnector;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use server::FerrislletterServer;

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

    let cfg = config::load(cli_config.as_deref()).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Build connector from config.
    let connector: Arc<BoxedConnector> = match cfg.as_ref().map(|c| &c.connector) {
        Some(ConnectorConfig::Static { path }) if !path.as_os_str().is_empty() => {
            tracing::info!(path = %path.display(), "loading static connector");
            Arc::new(BoxedConnector::new(
                StaticConnector::from_file(path)
                    .map_err(|e| anyhow::anyhow!("failed to load '{}': {e}", path.display()))?,
            ))
        }
        Some(ConnectorConfig::Rss { feeds }) => {
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
            Arc::new(BoxedConnector::new(RssConnector::new(rss_feeds)))
        }
        // No config or empty static path — fall back to env var or embedded sample.
        _ => match std::env::var("FERRISLETTER_DATA") {
            Ok(path) => {
                tracing::info!(path, "loading static connector from FERRISLETTER_DATA");
                Arc::new(BoxedConnector::new(
                    StaticConnector::from_file(&path)
                        .map_err(|e| anyhow::anyhow!("failed to load '{path}': {e}"))?,
                ))
            }
            Err(_) => {
                tracing::info!("using embedded sample data");
                Arc::new(BoxedConnector::new(
                    StaticConnector::from_json(SAMPLE_DATA)
                        .expect("embedded sample data must be valid"),
                ))
            }
        },
    };

    let server = FerrislletterServer::new(connector);

    // Start transport.
    let mode = cfg
        .map(|c| c.transport.mode)
        .unwrap_or(TransportMode::Stdio);

    match mode {
        TransportMode::Sse => {
            let cfg2 = config::load(cli_config.as_deref())
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .unwrap_or_default();
            let addr: SocketAddr =
                format!("{}:{}", cfg2.transport.host, cfg2.transport.port).parse()?;
            tracing::info!(%addr, "serving over SSE");
            let ct = rmcp::transport::sse_server::SseServer::serve(addr)
                .await?
                .with_service(move || server.clone());
            tokio::signal::ctrl_c().await?;
            ct.cancel();
        }
        TransportMode::Stdio => {
            tracing::info!("serving over stdio");
            let service = server
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
