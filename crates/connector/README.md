# ferrisletter-connector

Connector SDK for [Ferrisletter](https://github.com/ferrislabs/ferrisletter), a conversational newsletter platform powered by the Model Context Protocol (MCP).

This crate provides the `Connector` trait that content sources must implement, along with types for topics, items, search filters, and error handling. It also includes a plugin system (`ConnectorFactory` + `ConnectorRegistry`) for runtime connector discovery.

## Implementing a connector

```rust
use ferrisletter_connector::{
    Connector, ConnectorError, Item, ItemDetail, SearchFilters, Topic, UserPrefs,
};
use chrono::{DateTime, Utc};

struct MyConnector { /* ... */ }

impl Connector for MyConnector {
    async fn list_topics(&self) -> Result<Vec<Topic>, ConnectorError> {
        todo!()
    }

    async fn get_latest_items(&self, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError> {
        todo!()
    }

    async fn get_item_detail(&self, id: &str) -> Result<ItemDetail, ConnectorError> {
        todo!()
    }

    async fn search(&self, query: &str, filters: &SearchFilters) -> Result<Vec<Item>, ConnectorError> {
        todo!()
    }

    async fn get_recap(&self, since: DateTime<Utc>, prefs: &UserPrefs) -> Result<Vec<Item>, ConnectorError> {
        todo!()
    }
}
```

## Registering a factory

To integrate with the Ferrisletter server's plugin system, implement `ConnectorFactory`:

```rust
use ferrisletter_connector::{BoxedConnector, ConnectorError, ConnectorFactory, ConnectorRegistry};

struct MyConnectorFactory;

impl ConnectorFactory for MyConnectorFactory {
    fn connector_type(&self) -> &str { "my-source" }

    fn create(&self, config: &toml::Value) -> Result<BoxedConnector, ConnectorError> {
        // Parse config and build your connector.
        todo!()
    }
}

// Register with the server's registry:
let mut registry = ConnectorRegistry::new();
registry.register(MyConnectorFactory);
```

## License

MIT
