# Ferrisletter

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

## Status

Early development — proof of concept.

## License

MIT — see [LICENSE](LICENSE).

---

Part of [Ferrislabs](https://github.com/ferrislabs), by the team behind [FerrisKey](https://ferriskey.rs).
