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

### Content
1. `ferrisletter_list_topics` — Discover available topics
2. `ferrisletter_get_latest` — Get latest items as compact headlines
3. `ferrisletter_get_item` — Fetch full content of a single item
4. `ferrisletter_search` — Keyword search with filters
5. `ferrisletter_recap` — Summarize items since a given date

### Favorites
6. `ferrisletter_add_favorite` — Save an article to favorites
7. `ferrisletter_remove_favorite` — Remove an article from favorites
8. `ferrisletter_list_favorites` — List saved favorites with full item details

### User state (requires a UserStore)
9. `ferrisletter_setup_preferences` — Set topics, tags, summary length, arbitrary key-value prefs
10. `ferrisletter_get_preferences` — Return the user's full profile
11. `ferrisletter_get_my_feed` — Personalized, read-aware feed
12. `ferrisletter_mark_read` — Mark item IDs as read

### Themes
13. `ferrisletter_list_themes` — List themes registered by the host (Ferrisletter itself ships zero)

### Other
14. `ferrisletter_setup_delivery` — Suggest a cron expression + prompt for scheduled delivery

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
- `ui/src/views/FavoritesPanel.tsx` — Favorites view (list saved articles, empty state)
- `ui/src/lib/demo-data.ts` — Sample data for demo/standalone mode

## Favorites System

Server-level feature (not in the connector trait) — works with any connector.

- **Storage**: `FavoriteStore` trait in `crates/server/src/favorites.rs`
- **Default impl**: `InMemoryFavoriteStore` with JSON file persistence at `~/.config/ferrisletter/favorites.json`
- **Type erasure**: `BoxedFavoriteStore` for external implementations (e.g. Lattice's database backend)
- **User keying**: favorites are per-user (keyed by `user_id`, `"anonymous"` for stdio mode)
- **Item resolution**: `list_favorites` stores only item IDs; resolves to full `Item` objects via the connector at query time

## User State System

Same pattern as favorites — trait + in-memory/file default + type-erased wrapper.

- **Storage**: `UserStore` trait in `crates/server/src/users.rs`
- **Default impl**: `InMemoryUserStore` with JSON file persistence at `~/.config/ferrisletter/users.json`
- **Type erasure**: `BoxedUserStore` for external implementations (e.g. Lattice's PostgreSQL backend)
- **Scope**: identity (email/name), topic subscriptions, tag subscriptions, key-value preferences, read tracking
- **Stateless mode**: the 4 user-aware tools return `invalid_request` if the server was built without a `UserStore`

## Theme Registry

Ferrisletter ships with **zero built-in themes** — only the pattern.

- **`Theme` trait**: `name() / description() / css()` in `crates/server/src/theme.rs`
- **`ThemeRegistry`**: registration-order-preserving map of name → `Box<dyn Theme>`
- **`DisplayPreferences`**: generic struct with `theme`, `custom_instructions`, and an `extra` JSON blob for product-specific fields (Lattice stores `summary_words`, `separator_style`, `layout_density` there)
- **Downstream integration**: `FerrislletterServer::with_themes(Arc<ThemeRegistry>)` replaces the default empty registry
- **Exposure**: `ferrisletter_list_themes` returns registered themes to MCP clients

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
