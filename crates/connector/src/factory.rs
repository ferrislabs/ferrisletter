//! Connector plugin system — factory trait and registry for runtime discovery.

use std::collections::HashMap;

use crate::{BoxedConnector, ConnectorError};

/// A factory that creates a [`BoxedConnector`] from TOML configuration.
///
/// Implement this trait for each connector type to register it with the
/// [`ConnectorRegistry`]. The server discovers available connectors through
/// the registry at startup and uses the matching factory to build the
/// active connector from the config file.
///
/// # Example
///
/// ```rust,ignore
/// use ferrisletter_connector::{BoxedConnector, ConnectorError, ConnectorFactory};
///
/// pub struct MyConnectorFactory;
///
/// impl ConnectorFactory for MyConnectorFactory {
///     fn connector_type(&self) -> &str { "my-source" }
///
///     fn create(&self, config: &toml::Value) -> Result<BoxedConnector, ConnectorError> {
///         // Parse your config fields from the TOML value and build the connector.
///         todo!()
///     }
/// }
/// ```
pub trait ConnectorFactory: Send + Sync {
    /// Unique name identifying this connector type (e.g. `"rss"`, `"static"`).
    ///
    /// This must match the `type` field in the TOML `[connector]` section.
    fn connector_type(&self) -> &str;

    /// Create a connector instance from a TOML config table.
    ///
    /// The `config` value is the full `[connector]` table from the config file,
    /// including the `type` key. Implementations should extract the fields they
    /// need and ignore unknown keys for forward compatibility.
    fn create(&self, config: &toml::Value) -> Result<BoxedConnector, ConnectorError>;
}

/// A registry of [`ConnectorFactory`] implementations, keyed by type name.
///
/// # Example
///
/// ```rust,ignore
/// use ferrisletter_connector::ConnectorRegistry;
///
/// let mut registry = ConnectorRegistry::new();
/// registry.register(RssConnectorFactory);
/// registry.register(StaticConnectorFactory);
///
/// // Later, create a connector from config:
/// let connector = registry.create("rss", &config_value)?;
/// ```
pub struct ConnectorRegistry {
    factories: HashMap<String, Box<dyn ConnectorFactory>>,
}

impl ConnectorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a connector factory.
    ///
    /// # Panics
    ///
    /// Panics if a factory with the same [`ConnectorFactory::connector_type`]
    /// is already registered.
    pub fn register<F: ConnectorFactory + 'static>(&mut self, factory: F) {
        let name = factory.connector_type().to_string();
        if self.factories.contains_key(&name) {
            panic!("duplicate connector factory registered for type '{name}'");
        }
        self.factories.insert(name, Box::new(factory));
    }

    /// Create a connector by type name and config.
    ///
    /// Returns [`ConnectorError::Other`] if the type name is not registered.
    pub fn create(
        &self,
        type_name: &str,
        config: &toml::Value,
    ) -> Result<BoxedConnector, ConnectorError> {
        let factory = self.factories.get(type_name).ok_or_else(|| {
            ConnectorError::Other(format!("unknown connector type: '{type_name}'").into())
        })?;
        factory.create(config)
    }

    /// List all registered connector type names.
    pub fn available(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial factory for testing the registry.
    struct DummyFactory;

    impl ConnectorFactory for DummyFactory {
        fn connector_type(&self) -> &str {
            "dummy"
        }

        fn create(&self, _config: &toml::Value) -> Result<BoxedConnector, ConnectorError> {
            Err(ConnectorError::Other("not implemented".into()))
        }
    }

    #[test]
    fn register_and_list() {
        let mut reg = ConnectorRegistry::new();
        reg.register(DummyFactory);
        let names = reg.available();
        assert_eq!(names, vec!["dummy"]);
    }

    #[test]
    fn create_unknown_type_returns_error() {
        let reg = ConnectorRegistry::new();
        let result = reg.create("nope", &toml::Value::Table(Default::default()));
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "duplicate connector factory")]
    fn duplicate_registration_panics() {
        let mut reg = ConnectorRegistry::new();
        reg.register(DummyFactory);
        reg.register(DummyFactory);
    }

    #[test]
    fn create_delegates_to_factory() {
        let mut reg = ConnectorRegistry::new();
        reg.register(DummyFactory);
        // DummyFactory always returns Err, so we verify it was called.
        let result = reg.create("dummy", &toml::Value::Table(Default::default()));
        assert!(result.is_err());
    }
}
