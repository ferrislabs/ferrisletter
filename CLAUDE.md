# Ferrisletter

A conversational newsletter platform powered by the Model Context Protocol (MCP). Replaces traditional newsletters with interactive conversations — users get compact headline cards in their LLM client, expand topics of interest, search content, and ask for recaps.

## Project Structure

```
ferrisletter/
├── crates/
│   ├── server/              # Rust MCP server (main binary)
│   ├── connector/           # Connector trait + types (public SDK)
│   ├── connector-rss/       # RSS/Atom feed connector
│   └── connector-static/    # Static JSON connector
├── ui/                      # React MCP App UI (digest panel, rendered inline in Claude Desktop)
├── admin/                   # React admin dashboard (CRUD topics/feeds)
├── test-mcpui/              # Test harness for MCP UI protocol
├── website/                 # Astro 5 landing page + docs
├── design/                  # Architecture & design docs
├── examples/                # Example TOML configurations
└── vendor/                  # Vendored dependencies (rmcp)
```

## Tech Stack

- **Backend**: Rust (edition 2024), rmcp SDK, Axum
- **UI**: React 19, Tailwind CSS v4, Radix UI, `@modelcontextprotocol/ext-apps` SDK
- **Build**: Vite + vite-plugin-singlefile (produces single HTML bundle embedded in server binary)
- **Transport**: Stdio (Claude Desktop) or SSE/HTTP (remote/claude.ai)

## MCP Tools

1. `ferrisletter_list_topics` — Discover available topics
2. `ferrisletter_get_latest` — Get latest items as compact headlines
3. `ferrisletter_get_item` — Fetch full content of a single item
4. `ferrisletter_search` — Keyword search with filters
5. `ferrisletter_recap` — Summarize items since a given date

## MCP App UI

The UI follows the MCP Apps spec (`@modelcontextprotocol/ext-apps`):
- Server registers resource at `ui://ferrisletter/app` (HTML bundle)
- Every tool result includes `_meta.ui.resourceUri` pointing to this resource
- Claude Desktop auto-renders the React panel alongside tool results
- Uses `useApp` hook from SDK for lifecycle (init, tool results, theme, teardown)
- Falls back to demo mode when not connected to an MCP host

### UI Architecture
- `ui/src/App.tsx` — Root component, uses `useApp` hook, manages state
- `ui/src/lib/mcp.ts` — SDK integration: `McpAppContext`, `inferToolResult()`, tool call helpers
- `ui/src/views/CompactIssue.tsx` — Main digest view (topics, items, expand on click)
- `ui/src/lib/demo-data.ts` — Sample data for demo/standalone mode

## Key Config Files

- `examples/ferrisletter-demo.toml` — Full demo with RSS feeds + UI enabled (used with Claude Desktop)
- `examples/ferrisletter-sse.toml` — SSE transport + admin API
- `examples/ferrisletter-ngrok.toml` — Remote SSE via ngrok for claude.ai

## Running

### Claude Desktop (stdio)
Config at `~/Library/Application Support/Claude/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "ferrisletter": {
      "command": "/Users/luisrubiera/Projects/luis/ferrisletter/target/release/ferrisletter-server",
      "args": ["--config", "/Users/luisrubiera/Projects/luis/ferrisletter/examples/ferrisletter-demo.toml"]
    }
  }
}
```
Build: `cargo build --release -p ferrisletter-server`
UI must be built first: `cd ui && npm install && npm run build`

### Local SSE + basic-host (for testing MCP App rendering)
```bash
cargo run -p ferrisletter-server -- --config examples/ferrisletter-sse.toml
# In another terminal:
cd /tmp/mcp-ext-apps/examples/basic-host && SERVERS='["http://localhost:3000/mcp"]' npm run start
```

### UI dev (standalone demo mode)
```bash
cd ui && npm run dev
```

## Build Commands

```bash
# Build UI (must run before cargo build if UI changed)
cd ui && npm run build

# Build server (embeds ui/dist/index.html at compile time)
cargo build --release -p ferrisletter-server

# Type-check UI
cd ui && npx tsc -p tsconfig.app.json --noEmit
```

## Important Notes

- The UI HTML bundle is embedded in the Rust binary at compile time via `build.rs` — rebuild the UI before rebuilding the server when UI changes are made
- `[ui] enabled = true` in the TOML config is required for the MCP App to work; without it the server only returns text
- The `tool_ok()` helper in `server.rs` always adds `_meta.ui.resourceUri` — the `ui_enabled` flag controls whether `list_tools` annotates tool definitions and whether `list_resources` advertises the resource
- Claude Desktop logs are at `~/Library/Logs/Claude/mcp-server-ferrisletter.log`
