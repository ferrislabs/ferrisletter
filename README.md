# Ferrisletter

[![CI](https://github.com/ferrislabs/ferrisletter/actions/workflows/ci.yml/badge.svg)](https://github.com/ferrislabs/ferrisletter/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 2024 edition](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)

**Newsletters that talk back.**

A conversational newsletter platform powered by [MCP](https://modelcontextprotocol.io). Your LLM delivers the news — you just talk back. Built in Rust. Open source.

## What is this?

Traditional newsletters are long emails you never finish. Ferrisletter replaces them with a **conversation**. Your news is delivered as compact headlines in your LLM client. Expand what interests you. Ask for a recap of what you missed. Skip the rest.

## How it works

1. Your LLM client connects to the Ferrisletter MCP server
2. A scheduled task delivers your digest: compact topic cards
3. Expand any topic for the full story
4. Missed a week? Ask for a recap

## Project Structure

```
ferrisletter/
├── crates/
│   ├── server/              # MCP server
│   ├── connector/           # Connector trait + types (public SDK)
│   ├── connector-rss/       # RSS/Atom feed connector
│   └── connector-static/    # Local JSON/Markdown connector
├── ui/                      # React MCP UI components
├── website/                 # Landing page + docs (Explainer)
├── design/                  # Architecture & design documents
├── docs/                    # User-facing documentation
└── examples/                # Example configurations
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Protocol | MCP (Model Context Protocol) |
| Backend | Rust |
| UI | React (MCP Apps) |
| Auth | AuthZEN + FerrisKey |
| Website | Astro 5 + Explainer |

## Quickstart

```bash
# Build the server
cargo build --release -p ferrisletter-server

# Run with the demo RSS config (stdio transport)
./target/release/ferrisletter-server --config examples/ferrisletter-demo.toml
```

## Demo — Claude Desktop in 1 command

The fastest way to see Ferrisletter working end-to-end with real RSS feeds:

```bash
./scripts/setup-claude-desktop.sh
```

This builds the release binary and wires it into Claude Desktop's MCP config automatically. Restart Claude Desktop, then try:

- *"What's new in AI this week?"*
- *"Catch me up on Rust since last Monday"*
- *"Find anything about agents or MCP"*
- *"Summarise the top open-source news"*

The demo config ([`examples/ferrisletter-demo.toml`](examples/ferrisletter-demo.toml)) includes:

| Topic | Feeds |
|-------|-------|
| AI & LLMs | Simon Willison's blog · HN filtered (LLM/Claude/OpenAI, ≥50 pts) |
| Rust | blog.rust-lang.org · This Week in Rust |
| Open Source | GitHub Blog |

### Test it in Claude Code

If you already have the repo open in Claude Code, the MCP tools are available immediately without restarting anything:

```
ferrisletter_list_topics     — discover available topics
ferrisletter_get_latest      — headlines + summaries
ferrisletter_search          — keyword search across all feeds
ferrisletter_recap           — time-range digest (e.g. "since last Monday")
ferrisletter_get_item        — full text of a single item
```

### Custom config

Point the server at your own TOML:

```bash
./scripts/setup-claude-desktop.sh --config /path/to/my-feeds.toml
```

See [`examples/ferrisletter-demo.toml`](examples/ferrisletter-demo.toml) for the format.

## Status

Early development — proof of concept.

## License

MIT — see [LICENSE](LICENSE).

---

Part of [Ferrislabs](https://github.com/ferrislabs), by the team behind [FerrisKey](https://ferriskey.rs).
