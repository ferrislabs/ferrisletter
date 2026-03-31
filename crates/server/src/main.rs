fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("ferrisletter server starting");

    // TODO: Initialize MCP server with rmcp
    // TODO: Register connector from config
    // TODO: Register tools (list_topics, get_latest, expand, search, recap)
    // TODO: Register UI resources
    // TODO: Start transport (stdio or SSE)

    println!("ferrisletter v{}", env!("CARGO_PKG_VERSION"));
}
