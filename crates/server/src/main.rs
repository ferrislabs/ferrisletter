use std::net::SocketAddr;
use std::sync::Arc;

use ferrisletter_connector::BoxedConnector;
use ferrisletter_connector_rss::{FeedConfig as RssFeedConfig, RssConnector};
use ferrisletter_connector_static::StaticConnector;
use ferrisletter_server::api::{ApiState, ConnectorHandle, FeedRecord, TopicRecord};
use ferrisletter_server::config::{ConnectorConfig, TransportMode};
use ferrisletter_server::server::FerrislletterServer;
use ferrisletter_server::transport::{self, SseConfig};
use ferrisletter_server::{config, server};
use tokio::sync::RwLock;

/// Embedded sample data — used when no config or data file is provided.
const SAMPLE_DATA: &str = include_str!("../data/sample.json");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--config <path>` CLI arg takes precedence over everything else.
    let cli_config: Option<String> = std::env::args().skip_while(|a| a != "--config").nth(1);

    let cfg = config::load(cli_config.as_deref())
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .unwrap_or_default();

    // Initialise tracing — with optional OpenTelemetry layer.
    // MCP stdio transport uses stdout for the protocol, so logs go to stderr.
    init_tracing(&cfg)?;

    tracing::info!("ferrisletter v{} starting", env!("CARGO_PKG_VERSION"));

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
        tokio::spawn(async move { ferrisletter_server::api::serve(state, addr).await });
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
            let sse_config = SseConfig {
                public_url: cfg.transport.public_url,
            };
            transport::serve_sse(mcp_server, addr, &sse_config).await?;
        }
        TransportMode::Stdio => {
            transport::serve_stdio(mcp_server).await?;
        }
    }

    #[cfg(feature = "telemetry")]
    if cfg.telemetry.enabled {
        ferrisletter_server::telemetry::shutdown();
    }

    Ok(())
}

/// Initialise the tracing subscriber, optionally layering OpenTelemetry on top.
fn init_tracing(
    #[allow(unused_variables)] cfg: &ferrisletter_server::Config,
) -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());

    #[cfg(feature = "telemetry")]
    if cfg.telemetry.enabled {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let otel_layer = ferrisletter_server::telemetry::init(&cfg.telemetry)
            .map_err(|e| anyhow::anyhow!("failed to initialise OpenTelemetry: {e}"))?;

        let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        tracing::info!(
            endpoint = %cfg.telemetry.endpoint,
            service = %cfg.telemetry.service_name,
            "OpenTelemetry enabled"
        );
        return Ok(());
    }

    // Default: plain fmt subscriber to stderr.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .init();

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
                    refresh_minutes: f.refresh_minutes,
                })
                .collect();
            let connector = Arc::new(RssConnector::new(rss_feeds));
            let boxed = Arc::new(BoxedConnector::new(connector.as_ref().clone()));
            // Spawn auto-refresh background task.
            let _refresh_handle = connector.start_auto_refresh();
            Ok(boxed)
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
