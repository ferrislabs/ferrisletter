use std::net::SocketAddr;
use std::sync::Arc;

use ferrisletter_connector::{BoxedConnector, ConnectorRegistry};
use ferrisletter_connector_rss::RssConnectorFactory;
use ferrisletter_connector_static::StaticConnectorFactory;
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

    // Health/readiness state for container orchestrators.
    let server_state = ferrisletter_server::ServerState::new();

    // Build the connector registry and create the initial connector.
    let registry = default_registry();
    tracing::info!(connectors = ?registry.available(), "connector registry initialised");

    let initial_connector: Arc<BoxedConnector> = build_connector(&cfg.connector, &registry)?;

    // Mark server as ready now that the connector is loaded.
    server_state.set_ready();

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
                shutdown_timeout: cfg
                    .transport
                    .shutdown_timeout_seconds
                    .map(std::time::Duration::from_secs),
            };
            transport::serve_sse(mcp_server, addr, &sse_config, server_state).await?;
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

/// Build the default [`ConnectorRegistry`] with all built-in connector types.
fn default_registry() -> ConnectorRegistry {
    let mut registry = ConnectorRegistry::new();
    registry.register(RssConnectorFactory);
    registry.register(StaticConnectorFactory);
    registry
}

/// Build the initial BoxedConnector from config using the plugin registry.
fn build_connector(
    connector_cfg: &ConnectorConfig,
    registry: &ConnectorRegistry,
) -> anyhow::Result<Arc<BoxedConnector>> {
    match connector_cfg {
        // Empty static path = no real config → fallback to env var / embedded sample.
        ConnectorConfig::Static { path } if path.as_os_str().is_empty() => {
            match std::env::var("FERRISLETTER_DATA") {
                Ok(path) => {
                    tracing::info!(path, "loading static connector from FERRISLETTER_DATA");
                    let toml_val = toml::Value::Table({
                        let mut t = toml::map::Map::new();
                        t.insert("type".into(), toml::Value::String("static".into()));
                        t.insert("path".into(), toml::Value::String(path));
                        t
                    });
                    Ok(Arc::new(registry.create("static", &toml_val)?))
                }
                Err(_) => {
                    tracing::info!("using embedded sample data");
                    Ok(Arc::new(BoxedConnector::new(
                        ferrisletter_connector_static::StaticConnector::from_json(SAMPLE_DATA)
                            .expect("embedded sample data must be valid"),
                    )))
                }
            }
        }
        // Real config — serialize to TOML value and dispatch through the registry.
        _ => {
            let type_name = match connector_cfg {
                ConnectorConfig::Static { .. } => "static",
                ConnectorConfig::Rss { .. } => "rss",
            };

            tracing::info!(
                connector_type = type_name,
                "building connector via registry"
            );

            // Re-serialize the typed config to a toml::Value for the factory.
            let toml_value = connector_config_to_toml(connector_cfg);
            Ok(Arc::new(registry.create(type_name, &toml_value)?))
        }
    }
}

/// Convert a typed [`ConnectorConfig`] into a [`toml::Value`] table for factory dispatch.
fn connector_config_to_toml(cfg: &ConnectorConfig) -> toml::Value {
    let mut table = toml::map::Map::new();
    match cfg {
        ConnectorConfig::Static { path } => {
            table.insert("type".into(), toml::Value::String("static".into()));
            table.insert(
                "path".into(),
                toml::Value::String(path.display().to_string()),
            );
        }
        ConnectorConfig::Rss { feeds } => {
            table.insert("type".into(), toml::Value::String("rss".into()));
            let feeds_arr: Vec<toml::Value> = feeds
                .iter()
                .map(|f| {
                    let mut ft = toml::map::Map::new();
                    ft.insert("topic_id".into(), toml::Value::String(f.topic_id.clone()));
                    ft.insert(
                        "topic_label".into(),
                        toml::Value::String(f.topic_label.clone()),
                    );
                    ft.insert(
                        "topic_description".into(),
                        toml::Value::String(f.topic_description.clone()),
                    );
                    ft.insert(
                        "topic_tags".into(),
                        toml::Value::Array(
                            f.topic_tags
                                .iter()
                                .map(|t| toml::Value::String(t.clone()))
                                .collect(),
                        ),
                    );
                    ft.insert("url".into(), toml::Value::String(f.url.clone()));
                    if let Some(mins) = f.refresh_minutes {
                        ft.insert("refresh_minutes".into(), toml::Value::Integer(mins as i64));
                    }
                    toml::Value::Table(ft)
                })
                .collect();
            table.insert("feeds".into(), toml::Value::Array(feeds_arr));
        }
    }
    toml::Value::Table(table)
}
