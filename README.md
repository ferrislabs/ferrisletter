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

# Run it (uses embedded sample data)
./target/release/ferrisletter-server
```

Point your MCP client at the binary. See the setup guides:

- [Claude Desktop](docs/mcp-clients/claude-desktop.md)
- [Cursor](docs/mcp-clients/cursor.md)
- [Zed](docs/mcp-clients/zed.md)

To use your own content, set `FERRISLETTER_DATA` to a JSON file — see [`examples/sample-newsletter.json`](examples/sample-newsletter.json) for the format.

## Status

Early development — proof of concept.

## License

MIT — see [LICENSE](LICENSE).

---

Part of [Ferrislabs](https://github.com/ferrislabs), by the team behind [FerrisKey](https://ferriskey.rs).
