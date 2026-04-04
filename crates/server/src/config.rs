//! Configuration file parsing for the Ferrisletter server.
//!
//! Looks for config in this order:
//! 1. `--config <path>` CLI argument
//! 2. `FERRISLETTER_CONFIG` environment variable
//! 3. `./ferrisletter.toml` in the current directory
//!
//! Falls back to built-in defaults (stdio transport + embedded sample data)
//! if no config file is found.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub transport: TransportConfig,

    #[serde(default)]
    pub connector: ConnectorConfig,

    #[serde(default)]
    pub admin: AdminConfig,

    #[serde(default)]
    pub ui: UiConfig,
}

// --- Admin REST API ---

#[derive(Debug, Deserialize, Clone)]
pub struct AdminConfig {
    /// Enable the management REST API.
    #[serde(default)]
    pub enabled: bool,
    /// API key required in `Authorization: Bearer <key>` header.
    #[serde(default)]
    pub api_key: String,
    /// Address to bind the admin HTTP server on.
    #[serde(default = "default_admin_bind")]
    pub bind_addr: String,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            bind_addr: default_admin_bind(),
        }
    }
}

fn default_admin_bind() -> String {
    "127.0.0.1:3001".to_string()
}

// --- MCP App UI ---

/// Enables the MCP App UI (mcpui.dev spec).
///
/// When enabled the server registers `ui://ferrisletter/app` as an MCP
/// resource and includes `_meta.ui.resourceUri` in every tool call result,
/// so that supporting clients (Claude Desktop, etc.) render the digest panel
/// automatically.
#[derive(Debug, Default, Deserialize, Clone)]
pub struct UiConfig {
    /// Register the UI resource and annotate tool results with its URI.
    #[serde(default)]
    pub enabled: bool,
}

// --- Transport ---

#[derive(Debug, Deserialize)]
pub struct TransportConfig {
    #[serde(default)]
    pub mode: TransportMode,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Public base URL used for OAuth metadata (e.g. https://abc.ngrok-free.app).
    /// Required when mode = "sse" and connecting from claude.ai.
    #[serde(default)]
    pub public_url: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            mode: TransportMode::Stdio,
            host: default_host(),
            port: default_port(),
            public_url: None,
        }
    }
}

#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    #[default]
    Stdio,
    Sse,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    3000
}

// --- Connector ---

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConnectorConfig {
    /// Load content from a static JSON file.
    Static { path: PathBuf },
    /// Aggregate one or more RSS/Atom feeds.
    Rss { feeds: Vec<FeedConfig> },
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        // No config — use embedded sample data (path = "").
        ConnectorConfig::Static {
            path: PathBuf::new(),
        }
    }
}

/// Configuration for a single RSS/Atom feed.
#[derive(Debug, Deserialize, Clone)]
pub struct FeedConfig {
    pub topic_id: String,
    pub topic_label: String,
    pub topic_description: String,
    #[serde(default)]
    pub topic_tags: Vec<String>,
    pub url: String,
    /// How often to refresh this feed, in minutes. Defaults to 60 if omitted.
    #[serde(default)]
    pub refresh_minutes: Option<u64>,
}

// --- Loading ---

/// Errors that can occur when loading a config file.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config file '{path}': {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

/// Resolve the config file path from CLI args or environment, then load it.
///
/// Returns `None` if no config file is specified and `./ferrisletter.toml`
/// does not exist — the caller should fall back to built-in defaults.
pub fn load(cli_path: Option<&str>) -> Result<Option<Config>, ConfigError> {
    let path = resolve_path(cli_path);
    match path {
        Some(p) => load_file(&p).map(Some),
        None => Ok(None),
    }
}

fn resolve_path(cli_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = cli_path {
        return Some(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("FERRISLETTER_CONFIG") {
        return Some(PathBuf::from(p));
    }
    let default = PathBuf::from("ferrisletter.toml");
    if default.exists() {
        return Some(default);
    }
    None
}

fn load_file(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| ConfigError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stdio_static_config() {
        let toml = r#"
            [transport]
            mode = "stdio"

            [connector]
            type = "static"
            path = "data/newsletter.json"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.transport.mode, TransportMode::Stdio);
        assert!(matches!(config.connector, ConnectorConfig::Static { .. }));
    }

    #[test]
    fn parses_sse_rss_config() {
        let toml = r#"
            [transport]
            mode = "sse"
            host = "0.0.0.0"
            port = 8080

            [connector]
            type = "rss"

            [[connector.feeds]]
            topic_id    = "rust"
            topic_label = "Rust"
            topic_description = "Rust news"
            topic_tags  = ["rust"]
            url         = "https://blog.rust-lang.org/feed.xml"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.transport.mode, TransportMode::Sse);
        assert_eq!(config.transport.port, 8080);
        if let ConnectorConfig::Rss { feeds } = &config.connector {
            assert_eq!(feeds.len(), 1);
            assert_eq!(feeds[0].topic_id, "rust");
        } else {
            panic!("expected RSS connector");
        }
    }

    #[test]
    fn defaults_to_stdio_when_transport_omitted() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.transport.mode, TransportMode::Stdio);
    }
}
