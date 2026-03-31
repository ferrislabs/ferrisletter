# 05 — Monorepo Structure

## Repository Layout

**Public monorepo:** `github.com/the-ai-letter/ai-letter`

Everything a contributor or user needs in one place: the MCP server, connector SDK, built-in connectors, UI components, website, and docs.

**Private repo (separate):** `github.com/the-ai-letter/service` — backend API, scraping pipeline, infra. Not part of the monorepo.

---

## Directory Structure

```
ai-letter/
│
├── cmd/
│   └── ai-letter/
│       └── main.go              # MCP server entrypoint
│
├── internal/                    # Private Go packages (not importable)
│   ├── server/                  # MCP protocol handling, tool registration
│   ├── conversation/            # Conversation flow logic, recap, search
│   ├── preferences/             # User preference management
│   └── cache/                   # Response caching layer
│
├── pkg/                         # Public Go packages (importable by third parties)
│   └── connector/
│       ├── connector.go         # Connector interface definition
│       ├── types.go             # Topic, Item, ItemDetail, UserPrefs, etc.
│       └── registry.go          # Connector registration + discovery
│
├── connectors/                  # Built-in connector implementations
│   ├── ailetter/                # Our service connector (calls hosted API)
│   │   └── ailetter.go
│   ├── rss/                     # RSS/Atom feed connector
│   │   └── rss.go
│   └── static/                  # Local JSON/Markdown connector
│       └── static.go
│
├── ui/                          # Svelte 5 MCP UI components
│   ├── src/
│   │   ├── views/
│   │   │   ├── CompactIssue.svelte    # Headline cards (default view)
│   │   │   ├── ExpandedTopic.svelte   # Full article view
│   │   │   ├── RecapView.svelte       # Multi-issue recap
│   │   │   └── SettingsPanel.svelte   # Preference configuration
│   │   ├── components/                # Shared UI primitives
│   │   └── lib/                       # Client SDK, MCP App bridge
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
│
├── website/                     # Landing page (already exists)
│   ├── src/
│   ├── astro.config.mjs
│   └── package.json
│
├── design/                      # Architecture & design docs (already exists)
│   ├── 00-vision.md
│   ├── 01-revised-vision.md
│   ├── 02-delivery-model.md
│   ├── 03-decisions-locked.md
│   ├── 04-open-core-model.md
│   └── 05-monorepo-structure.md
│
├── docs/                        # User-facing documentation
│   ├── getting-started.md
│   ├── configuration.md
│   ├── building-a-connector.md
│   └── api-reference.md
│
├── examples/                    # Example configs and usage
│   ├── claude-config.json       # Example MCP config for Claude
│   ├── chatgpt-config.json      # Example MCP config for ChatGPT
│   └── rss-feeds.json           # Example RSS connector config
│
├── .github/
│   └── workflows/
│       ├── ci.yml               # Go test + lint + build
│       ├── ui.yml               # Svelte build + checks
│       └── pages.yml            # Website deploy to GitHub Pages
│
├── go.mod
├── go.sum
├── Makefile                     # build, test, lint, run
├── Dockerfile                   # MCP server container image
├── README.md
├── LICENSE                      # MIT
└── CONTRIBUTING.md
```

---

## Why This Layout

### `cmd/` + `internal/` + `pkg/`

Standard Go project layout. The key distinction:

- `pkg/connector/` is **importable by anyone** — third-party connector authors import this to implement the interface
- `internal/` is **private** — Go enforces this at the compiler level, no one outside the module can import it

This means the connector SDK is a stable public API, while server internals can change freely.

### `connectors/` at the root

Not inside `internal/` because:
- They serve as **reference implementations** for connector authors
- The `ailetter/` connector is open source (it's just an HTTP client — the proprietary part is the API it calls)
- Easy to find, easy to copy as a starting point

### `ui/` as a separate package

The Svelte UI is built independently and bundled into the Go binary at build time (embedded via `go:embed`). This keeps:
- Frontend devs working in `ui/` with standard Node tooling
- Go devs not needing to touch JavaScript
- The final binary self-contained — no separate UI deployment

### `website/` stays in the monorepo

It's the project's public face. Having it in the same repo means:
- One PR can update docs + website together
- GitHub Pages deploys from the same CI
- Contributors see the whole picture

### `design/` vs `docs/`

- `design/` = internal architecture decisions (for us and contributors)
- `docs/` = user-facing documentation (getting started, configuration, how to build a connector)

---

## Build & Development

### Makefile targets

```makefile
# Build the MCP server binary
build:
	cd ui && npm run build
	go build -o bin/ai-letter ./cmd/ai-letter

# Run locally
run:
	go run ./cmd/ai-letter --config config.yaml

# Run tests
test:
	go test ./...

# Lint
lint:
	golangci-lint run
	cd ui && npm run check

# Build Docker image
docker:
	docker build -t ai-letter .

# Dev mode: run server + UI hot reload
dev:
	cd ui && npm run dev &
	go run ./cmd/ai-letter --dev
```

### CI Pipelines

1. **ci.yml** — On every PR: `go test`, `go vet`, `golangci-lint`, build binary
2. **ui.yml** — On every PR touching `ui/`: `npm ci`, `npm run check`, `npm run build`
3. **pages.yml** — On main push touching `website/`: build Astro, deploy to GitHub Pages

---

## Private Service Repo

```
service/                         # github.com/the-ai-letter/service (private)
├── cmd/
│   └── service/
│       └── main.go              # API server entrypoint
├── internal/
│   ├── api/                     # HTTP handlers
│   ├── scraper/                 # Content scraping pipeline
│   ├── tagger/                  # LLM-assisted content tagging
│   ├── storage/                 # PostgreSQL repositories
│   └── auth/                    # API key validation, FerrisKey integration
├── migrations/                  # Database migrations
├── deploy/                      # Docker Compose, Helm charts
├── go.mod
├── Dockerfile
└── README.md
```

This repo imports `github.com/the-ai-letter/ai-letter/pkg/connector` to ensure the API responses match the connector types exactly.

---

## GitHub Configuration

### Repository settings
- **Default branch:** `main`
- **Branch protection:** require PR reviews, require CI pass
- **Topics:** `mcp`, `newsletter`, `llm`, `open-source`, `go`, `svelte`

### GitHub Pages
- Source: GitHub Actions (from `website/` build output)
- Custom domain: TBD

### Issue labels
- `connector` — connector-related issues
- `ui` — MCP UI component issues
- `server` — core MCP server issues
- `docs` — documentation
- `good first issue` — onboarding
- `help wanted` — community contributions welcome
