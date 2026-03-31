# The AI Letter - Revised Vision (v2)

## The Pitch

Newsletters are broken. They're long emails you never finish, piling up in your inbox, making you feel guilty. The AI Letter fixes this.

Instead of a wall of text, you get a **conversation**. Your newsletter is a few lines per topic. Interested? Expand it. Missed a week? Ask for a recap. Want to go deeper on one subject? Just ask. The LLM is your personal content concierge.

No email. No inbox. No guilt. Just a chat.

---

## How It Actually Works

### For a Publisher

```
You: "Create a newsletter about AI research, published weekly"
AI Letter: "Done. What topics should each issue cover?"
You: "Latest papers, industry news, open source releases, one opinion piece"
AI Letter: [renders a preview of the newsletter structure]
You: "Make it more casual, shorter summaries"
AI Letter: [re-renders with adjusted tone and length]
You: "Publish this week's issue"
AI Letter: [generates issue, notifies subscribers]
```

### For a Subscriber

```
You: "What's new in AI Digest this week?"
AI Letter: [renders compact card UI — 5 topics, 1-2 lines each]
You: "Tell me more about the new Mamba architecture paper"
AI Letter: [expands that section with details, links, context]
You: "I haven't checked in for 2 weeks, what did I miss?"
AI Letter: [renders a recap of the last 3 issues, highlights only]
You: "Show me everything about open source releases from the last month"
AI Letter: [cross-issue search, filtered view]
```

This is the core UX. The newsletter is **alive** — not a PDF, not an email, but an interactive knowledge base you talk to.

---

## Revised Architecture

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Distribution | MCP-only + auto-generated website | Niche audience, MCP-native experience. Website for discovery/SEO. |
| Storage | Remote server (API + DB) | Multi-device, multi-user, persistent preferences, subscription management |
| Tenancy | Multi-tenant | One server hosts many newsletters, users are different per newsletter |
| Backend | Go or Kotlin | User preference. See analysis below. |
| MCP UI | Svelte 5 or Solid.js | Modern, lightweight, fast iframe rendering |

### System Architecture

```
┌─────────────────────────────────────────┐
│  Subscriber's LLM Client               │
│  (Claude, ChatGPT, VS Code, etc.)      │
└──────────────┬──────────────────────────┘
               │ MCP Protocol
┌──────────────▼──────────────────────────┐
│  AI Letter MCP Server                   │
│  (installed locally by each user)       │
│                                         │
│  Tools:                                 │
│  - subscribe / unsubscribe              │
│  - list_issues / read_issue             │
│  - expand_topic / recap                 │
│  - search_across_issues                 │
│  - create_newsletter (publisher)        │
│  - draft_issue / publish (publisher)    │
│  - configure_preferences                │
│                                         │
│  UI Resources:                          │
│  - issue_compact (summary cards)        │
│  - issue_expanded (full article)        │
│  - newsletter_browse (discovery)        │
│  - settings_panel                       │
└──────────────┬──────────────────────────┘
               │ HTTPS API
┌──────────────▼──────────────────────────┐
│  AI Letter Backend API                  │
│  (Go or Kotlin — hosted service)        │
│                                         │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │ Auth        │  │ Newsletter CRUD │   │
│  │ (API keys   │  │ (create, edit,  │   │
│  │  or OAuth)  │  │  publish)       │   │
│  └─────────────┘  └─────────────────┘   │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │ Subscript.  │  │ User Prefs      │   │
│  │ Management  │  │ (length, tone,  │   │
│  └─────────────┘  │  topics, freq)  │   │
│                    └─────────────────┘   │
│  ┌──────────────────────────────────┐   │
│  │ Database (PostgreSQL)            │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Auto-Generated Website                 │
│  (static, for discovery + archive)      │
│  - Newsletter landing pages             │
│  - Public issue archive                 │
│  - "Connect via MCP" CTA               │
└─────────────────────────────────────────┘
```

### Two Deployable Pieces

1. **MCP Server (npm/go module):** Users install this locally. It connects to their LLM client AND to the remote backend. This is the "app."
2. **Backend API (hosted):** Stores everything. Handles auth, subscriptions, content. We host this (or users self-host).

---

## Backend Language: Go vs Kotlin

| Criteria | Go | Kotlin |
|----------|-----|--------|
| MCP ecosystem | `mcp-go` — mature, well-maintained | `kotlin-mcp-sdk` — exists but younger |
| Open source friendliness | Excellent. Single binary, zero deps. Easy to contribute. | JVM required. Heavier. Gradle complexity. |
| Server performance | Excellent. Goroutines, low memory. | Good. Coroutines, but JVM overhead. |
| Learning curve for contributors | Low. Simple language, easy to read. | Medium. Kotlin is expressive but has more concepts. |
| Web framework | Stdlib `net/http` or Chi/Echo | Ktor or Spring Boot |
| Database | sqlc, pgx, GORM | Exposed, jOOQ, Spring Data |
| Deployment | Single binary, tiny Docker image | JAR + JVM, larger image |
| Typing / Safety | Good but limited (no generics until recently) | Excellent. Null safety, sealed classes, ADTs. |

**My honest recommendation: Go.**

Reasons:
- Open source projects live and die by contributor friction. Go is easier to read, easier to build, easier to deploy. A single binary with no runtime dependencies is a massive advantage.
- The MCP Go SDK is mature.
- For a backend that's mostly CRUD + auth + content serving, Go's simplicity is a feature, not a limitation.
- Smaller Docker images = cheaper to host, faster to deploy.

Kotlin is the better *language*, but Go is the better *choice for this project*.

---

## UI Framework: Svelte 5 vs Solid.js

Both are modern, lightweight, and perfect for MCP UI iframes.

| Criteria | Svelte 5 (Runes) | Solid.js |
|----------|-------------------|----------|
| Bundle size | ~2KB runtime | ~7KB runtime |
| Reactivity | Compile-time (runes) | Fine-grained signals |
| Learning curve | Very low | Low-medium |
| Ecosystem | Large, growing fast | Smaller but devoted |
| MCP UI fit | Excellent — compiles away, tiny output | Excellent — signals are efficient |
| Community | Larger | Smaller |

**My recommendation: Svelte 5.**

The compile-time approach means near-zero runtime overhead — perfect for sandboxed iframes that need to load fast. Runes (Svelte 5's reactivity system) are clean and modern. Larger community = more contributors for an open source project.

---

## Content Model

### Newsletter

```yaml
id: "ai-digest"
name: "Weekly AI Digest"
description: "Your weekly dose of AI research, news, and open source"
publisher_id: "user_abc123"
created_at: "2026-03-29T00:00:00Z"

settings:
  default_tone: "casual"           # casual | professional | academic
  default_length: "brief"          # brief | standard | detailed
  topics:
    - label: "Research Papers"
      description: "Latest notable papers from arxiv, conferences"
    - label: "Industry News"
      description: "Funding, launches, acquisitions"
    - label: "Open Source"
      description: "New releases, notable projects"
    - label: "Opinion"
      description: "One editorial take per issue"
  schedule: "weekly"               # weekly | biweekly | monthly | manual
```

### Issue

```yaml
id: "ai-digest-2026-w13"
newsletter_id: "ai-digest"
number: 13
title: "The Week Transformers Got Competition"
published_at: "2026-03-29T10:00:00Z"
status: "published"                # draft | published | archived

items:
  - topic: "Research Papers"
    headline: "Mamba-3 achieves transformer parity on reasoning benchmarks"
    summary: "One-liner shown in compact view"
    body: "Full expanded content in Markdown — shown on demand"
    links:
      - url: "https://arxiv.org/abs/..."
        label: "Paper"
    metadata:
      reading_time: "3 min"

  - topic: "Industry News"
    headline: "Anthropic raises Series D at $60B valuation"
    summary: "..."
    body: "..."
    links: [...]

  # ... more items
```

### User Preferences (per subscriber, per newsletter)

```yaml
user_id: "user_xyz"
newsletter_id: "ai-digest"
subscribed_at: "2026-03-15T00:00:00Z"

preferences:
  summary_length: "brief"          # how long the compact view is
  topics_of_interest: ["Research Papers", "Open Source"]  # filter
  expand_by_default: false         # auto-expand or click-to-expand
  recap_style: "highlights"        # highlights | chronological | grouped
  language: "en"                   # future: i18n
```

---

## Critical Concerns (Revised)

### 1. The Two-Piece Architecture Is Complex

Having both a local MCP server AND a remote backend means:
- Users need to install the MCP server AND get an API key/auth token
- We need to build, host, and maintain a backend service
- Network dependency: no internet = no newsletter
- Data privacy questions: who owns the content? Where is it stored? What jurisdiction?

**This is the biggest engineering risk.** A purely local MCP server is dramatically simpler. But the multi-user, subscription, preferences model requires remote state.

**Mitigation:** Design the MCP server so it can work in "local-only" mode too (publisher creates + reads their own content locally). Remote backend is for the social features (subscriptions, multi-user).

### 2. Content Sourcing Is Underspecified

Where does the actual newsletter content come from?
- Does the publisher write every item manually through chat?
- Does the LLM generate content from provided sources/URLs?
- Does it aggregate from RSS feeds, APIs, etc.?
- Some combination?

This matters enormously for the product. If publishers have to manually write every item, adoption will be low. If the LLM curates from sources, that's powerful but raises quality/accuracy concerns.

**We need to decide this.**

### 3. Authentication Model

Multi-tenant + remote server = we need real auth.
- API keys? Simple but less secure.
- OAuth? Better UX but complex to implement.
- JWT tokens? Standard but needs careful implementation.

For an MCP server, the auth flow is unusual — there's no browser to redirect to. The MCP OAuth spec exists but client support varies.

### 4. Hosting & Cost

If we're hosting the backend:
- Who pays for it? Open source project with hosting costs is a sustainability challenge.
- Do we offer a hosted version + self-host option?
- Database costs scale with number of newsletters and issues.

**My recommendation:** Design for self-hosting first, offer a hosted version later. Docker Compose setup that anyone can deploy.

---

## Revised v1 Scope

### Must Have (v1)

- [ ] Go backend API with PostgreSQL
- [ ] MCP server (Go or TypeScript?) that connects to backend
- [ ] Publisher flow: create newsletter, define topics, draft + publish issues
- [ ] Subscriber flow: subscribe, read compact view, expand topics, recap
- [ ] 3 MCP UI views: compact issue, expanded topic, issue list
- [ ] 2-3 visual themes
- [ ] API key auth (simple)
- [ ] Docker Compose for self-hosting

### Nice to Have (v1.x)

- [ ] Auto-generated static website per newsletter
- [ ] Content sourcing from URLs/RSS (LLM-assisted)
- [ ] Cross-issue search
- [ ] Subscriber preference customization
- [ ] OAuth authentication
- [ ] Hosted version

### Future (v2+)

- [ ] Paid subscriptions
- [ ] Newsletter discovery/registry
- [ ] Analytics (respectful, privacy-first)
- [ ] i18n / multi-language
- [ ] Collaborative authoring
- [ ] Mobile-optimized MCP UI

---

## Next Steps

1. **Decide:** Content sourcing model (manual? AI-assisted? RSS?)
2. **Decide:** MCP server language (Go end-to-end, or TypeScript for MCP + Go for backend?)
3. **Design:** Detailed tool API spec (every tool, params, returns)
4. **Design:** UI wireframes for the 3 core views
5. **Setup:** Monorepo, CI, linting, project scaffolding
