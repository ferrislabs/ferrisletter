# Decisions Locked — Pre-Implementation Summary

## What The AI Letter Actually Is

We are NOT building a newsletter platform for anyone to publish.
We ARE building a **content service**: we scrape the internet, tag and
organize content, and serve it on-demand through MCP when subscribers ask.

The mental model: a **personalized, conversational news feed** — not a
traditional newsletter. Content is pre-processed and ready. The LLM
serves it based on what the subscriber wants, when they want it.

---

## Locked Decisions

### 1. Content Model: We Are the Publisher

- We scrape the internet for content (news, articles, releases, etc.)
- We apply tags/categories to scraped content
- Content is stored, indexed, and ready to serve
- When a subscriber asks "what's new in AI today?", the MCP server
  pulls pre-tagged content and presents it
- This is NOT "today's digest" pushed — it's reactive, on-demand,
  shaped by the subscriber's question

**Implication:** We need a content pipeline (scraper → processor → tagger → storage).

### 2. Auth: AuthZEN with PDP/PEP

- Authorization follows the AuthZEN standard
- PDP (Policy Decision Point): evaluates access requests
- PEP (Policy Enforcement Point): enforces at the API/MCP layer
- Possibly using FerrisKey (Rust-based, open source, Apache 2.0)
  - Currently v0.4.2, early access
  - Sub-10ms auth latency, ~10MB binary
  - NOTE: it's pre-release. We may need a fallback.

**Implication:** Auth is decoupled from business logic. Policies are
centralized, not scattered in code.

### 3. Architecture: Fully Remote, We Host Everything

- MCP server: hosted remotely (NOT local install)
- Backend API: hosted remotely
- Database: hosted (PostgreSQL)
- MCP server is stateless with caching layer
- Subscribers connect via remote MCP transport (HTTP/SSE)

**Implication:** Zero install friction for subscribers — just add a URL
to their MCP client config. But we own infrastructure costs and ops.

### 4. Tenancy: Multi-tenant, One Server

- One MCP server instance serves all subscribers
- Users differentiated by auth identity
- Different subscribers can have different preferences for the same content

### 5. Tech Stack

- Backend: **Go**
- MCP UI: **Svelte 5**
- Database: **PostgreSQL**
- Auth: **AuthZEN PDP/PEP** (FerrisKey or alternative)
- Hosting: our infrastructure
- Content pipeline: TBD (Go services? Python for scraping?)

### 6. Client Support

- Only clients that support scheduling (ChatGPT, Claude)
- No fallback for clients without scheduling — this is a POC
- If scheduled task setup from tool output is unreliable, that's a
  finding, not a failure

### 7. Scale / Ops

- Rate limiting: Cloudflare or load balancer in front
- Caching: MCP server caches aggressively
- Not optimizing for 1000s of users in v1 — this is a POC

---

## The Read-Tracking Problem

> "If it's directly conversation, how are we going to mark as read?"

This is a real problem. In email, "read" is binary: opened or not.
In a conversational model, it's nuanced.

### Proposed: Three-State Model

| State | Trigger | Meaning |
|-------|---------|---------|
| **new** | Issue published | Content exists, subscriber hasn't seen it |
| **delivered** | `check_new_issues` returns it | Shown to subscriber (compact view) |
| **read** | Subscriber expands or interacts | Subscriber engaged with the content |

**How it works:**
- When the scheduled task fires and calls `check_new_issues`, every
  returned item moves from `new` → `delivered`
- When the subscriber says "tell me more about X" or clicks expand
  in the MCP UI, that item moves from `delivered` → `read`
- "I haven't checked in 2 weeks" — the server knows which items
  are still `new` (never delivered) vs `delivered` (shown but not read)

**The MCP UI can help:** the rendered card UI can include interaction
tracking. When a topic card is expanded in the iframe, it calls a tool
or sends an event back to mark it as read. MCP Apps support bidirectional
communication (iframe → server via JSON-RPC postMessage).

**Edge cases:**
- Subscriber sees compact view but doesn't expand anything → `delivered`
  (they saw the headlines but didn't engage)
- Subscriber asks "recap last 2 weeks" → items in the recap move to
  `delivered`, not `read` (they got a summary, not the full content)
- Subscriber explicitly says "mark everything as read" → batch update

---

## Revised Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│         Subscriber's LLM Client                 │
│         (ChatGPT / Claude)                      │
│                                                 │
│  Scheduled task: "Every Monday 9am,             │
│  call check_new_issues and present results"     │
└──────────────────┬──────────────────────────────┘
                   │ MCP Protocol (HTTP+SSE, remote)
                   │
┌──────────────────▼──────────────────────────────┐
│          AI Letter MCP Server                   │
│          (hosted, stateless + cache)            │
│                                                 │
│  Tools:                                         │
│  ├─ subscribe(newsletter_id)                    │
│  ├─ unsubscribe(newsletter_id)                  │
│  ├─ check_new_issues(since?)                    │
│  ├─ expand_topic(issue_id, topic_id)            │
│  ├─ recap(days_back, topics?)                   │
│  ├─ search(query, date_range?)                  │
│  ├─ configure_preferences(prefs)                │
│  ├─ setup_delivery(frequency, time)             │
│  └─ mark_read(issue_id, topic_id?)              │
│                                                 │
│  UI Resources:                                  │
│  ├─ compact_issue (headline cards)              │
│  ├─ expanded_topic (full article view)          │
│  ├─ recap_view (multi-issue summary)            │
│  └─ settings_panel                              │
│                                                 │
│  Cache: Redis or in-memory (recent content)     │
└──────────────────┬──────────────────────────────┘
                   │ Internal HTTPS API
                   │
┌──────────────────▼──────────────────────────────┐
│          AI Letter Backend API (Go)             │
│                                                 │
│  ┌────────────┐  ┌───────────────────────────┐  │
│  │ PEP        │  │ Content API               │  │
│  │ (AuthZEN)  │→ │ - issues                  │  │
│  │            │  │ - topics                  │  │
│  └────────────┘  │ - search                  │  │
│       ↓          └───────────────────────────┘  │
│  ┌────────────┐  ┌───────────────────────────┐  │
│  │ PDP        │  │ Subscription API          │  │
│  │ (FerrisKey │  │ - subscribe/unsubscribe   │  │
│  │  or alt.)  │  │ - preferences             │  │
│  └────────────┘  │ - delivery settings       │  │
│                  │ - read tracking            │  │
│                  └───────────────────────────┘  │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │         PostgreSQL                       │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│        Content Pipeline                         │
│                                                 │
│  Scraper → Processor → Tagger → Store           │
│                                                 │
│  - Scrapes sources on schedule                  │
│  - Extracts/cleans content                      │
│  - LLM-assisted tagging + categorization        │
│  - Stores in PostgreSQL, indexed by tag/date    │
└─────────────────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│        Website (auto-generated)                 │
│                                                 │
│  - Newsletter landing page                      │
│  - Public archive of past issues                │
│  - "Connect via MCP" instructions               │
│  - SEO for discovery                            │
└─────────────────────────────────────────────────┘
```

---

## What's Left Before Implementation

### Must answer:

1. **What do we scrape?** Which sources, which topics, what verticals?
   This is an editorial decision. The first newsletter needs a topic.

2. **Content pipeline tech:** Go for scraping too? Or Python
   (better scraping libs: scrapy, newspaper3k, trafilatura)?
   Go for the API, Python for the pipeline is a common split.

3. **FerrisKey viability:** It's v0.4.2 / early access. Do we commit
   to it or use a simpler auth model for v1 and migrate later?

4. **MCP transport:** Remote MCP server over HTTP+SSE — confirm this
   works well with ChatGPT and Claude as clients.

### Can figure out during implementation:

- Exact database schema
- Caching strategy details
- UI component design
- Website tech stack
- CI/CD setup
