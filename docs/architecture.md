# Architecture

## Overview

Ferrisletter is a Rust workspace that exposes newsletter content as an MCP server. LLM clients (Claude Desktop, Cursor, Zed, etc.) connect to the server and interact with the content via five tools.

```
┌─────────────────────────────────────┐
│           LLM Client                │
│  (Claude Desktop / Cursor / Zed)    │
└────────────────┬────────────────────┘
                 │ MCP (stdio or SSE)
┌────────────────▼────────────────────┐
│         ferrisletter-server         │
│                                     │
│  FerrislletterServer (rmcp)         │
│  ┌─────────────────────────────┐    │
│  │       5 MCP tools           │    │
│  │  list_topics · get_latest   │    │
│  │  get_item · search · recap  │    │
│  └──────────────┬──────────────┘    │
│                 │ Connector trait   │
│  ┌──────────────▼──────────────┐    │
│  │       BoxedConnector        │    │
│  └──────────────┬──────────────┘    │
└─────────────────┼───────────────────┘
          ┌───────┴────────┐
          │                │
   ┌──────▼──────┐  ┌──────▼──────┐
   │   Static    │  │     RSS     │
   │  Connector  │  │  Connector  │
   │ (JSON file) │  │ (live feeds)│
   └─────────────┘  └─────────────┘
```

## Crate layout

| Crate | Role |
|---|---|
| `ferrisletter-connector` | `Connector` trait + `BoxedConnector` type-eraser (SDK) |
| `ferrisletter-connector-static` | Loads content from a static JSON file |
| `ferrisletter-connector-rss` | Aggregates one or more RSS/Atom feeds, with lazy cache |
| `ferrisletter-server` | MCP server binary — wires transport, config, and connector |

## Key design decisions

### Connector trait (AFIT)

The `Connector` trait uses **native async fn in trait** (Rust 2024 edition — no `async_trait` crate). Because AFIT traits are not object-safe, a `BoxedConnector` type-eraser is provided in the SDK crate. It uses an internal `ErasedConnector` trait with `BoxFuture` return types, allowing the server to hold `Arc<BoxedConnector>` without generics.

### Transport

The server supports two transports, selected via config:

- **stdio** (default) — communicates over stdin/stdout; logs go to stderr. Works with any MCP client that spawns a child process.
- **SSE** — serves over HTTP using Server-Sent Events. Used for remote/containerised deployments.

### Configuration

Config is resolved in priority order:

1. `--config <path>` CLI argument
2. `FERRISLETTER_CONFIG` environment variable
3. `./ferrisletter.toml` in the working directory
4. Built-in defaults (stdio + embedded sample data)

See `examples/ferrisletter.toml` for an annotated reference.

### RSS connector caching

The RSS connector fetches all feeds once on first use and caches the result behind an `Arc<RwLock<Option<Vec<ItemDetail>>>>`. Subsequent calls hit the cache. The server process is expected to be short-lived (LLM client spawns it); for long-running SSE deployments, a TTL-based refresh should be added.

## Data model

```
Topic          — id, label, description, tags
Item           — id, topic_id, headline, summary, tags, source, published, read_time
ItemDetail     — Item + body + links
UserPrefs      — topic_ids (filter by topic)
SearchFilters  — tags, since, topic_id
```

## MCP tools

| Tool | Description |
|---|---|
| `ferrisletter_list_topics` | List all available topics |
| `ferrisletter_get_latest` | Get latest items, optionally filtered by topic |
| `ferrisletter_get_item` | Get full detail for a single item |
| `ferrisletter_search` | Full-text + tag search across all items |
| `ferrisletter_recap` | Get all items published since a given date |
