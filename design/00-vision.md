# The AI Letter - Vision & Design Discussion

## One-Liner

A chat-native newsletter platform — create, customize, and consume newsletters entirely through LLM conversations, rendered via MCP UI.

---

## Core Concept

The AI Letter is an MCP server that lets anyone run a newsletter without writing a single line of frontend code. Everything — authoring, styling, subscribing, reading — happens through natural language in any MCP-capable client (Claude, VS Code Copilot, ChatGPT, Goose, etc.).

The MCP server exposes tools for newsletter management and renders rich, interactive newsletter UIs directly in the chat using MCP Apps (the official MCP UI standard).

---

## Actors

| Actor | Description |
|-------|-------------|
| **Publisher** | Creates and manages a newsletter via chat |
| **Subscriber** | Reads and interacts with newsletters via chat |

Both actors interact exclusively through MCP tools — no web dashboard, no admin panel.

---

## How It Works (High Level)

### For Publishers
1. Connect the AI Letter MCP server to their LLM client
2. Create a newsletter: `"Create a new newsletter called 'Weekly AI Digest'"`
3. Customize look & feel: `"Use a dark theme with blue accents"`
4. Write an issue: `"Draft a new issue about transformer architectures"`
5. Publish: `"Publish this issue"` — renders a beautiful newsletter UI in-chat and delivers to subscribers

### For Subscribers
1. Connect to a publisher's AI Letter MCP server
2. Browse: `"Show me the latest issues"` — sees rich card UI
3. Read: `"Open issue #12"` — full newsletter rendered in-chat
4. Manage: `"Unsubscribe from Weekly AI Digest"`

---

## Critical Questions We Need to Answer

### 1. Distribution Model

**The elephant in the room:** If everything is MCP-based, how do subscribers actually receive newsletters?

Options:
- **A) MCP-only (pull model):** Subscribers connect to the publisher's MCP server and pull issues on demand. Simple, but requires active effort.
- **B) MCP + Email:** Generate an email version automatically from the same content. Broader reach, but now we need email infrastructure.
- **C) MCP + Static Web:** Auto-generate a static site (like a blog) from published issues. Best of both worlds?
- **D) MCP + Notification hooks:** Push notifications through webhooks/integrations when new issues drop.

**My recommendation:** Start with **A** (MCP-only) for v1, design the content model to support B/C/D later. Trying to solve distribution on day one adds massive scope.

### 2. Content Storage

Where does newsletter content live?

Options:
- **A) Local filesystem (JSON/Markdown):** Simple, portable, git-friendly. Publisher owns their data.
- **B) SQLite:** More structured, supports queries, still single-file portable.
- **C) Cloud storage:** Firebase, Supabase, etc. Enables multi-device, but adds dependency.

**My recommendation:** **A (filesystem with Markdown + frontmatter)** for v1. Newsletters are fundamentally documents. Markdown is the lingua franca. Git-friendly = open-source friendly.

### 3. Customization Model

What does "fully customizable" mean concretely?

Layers of customization:
- **Theme:** Colors, fonts, spacing (CSS variables)
- **Layout:** Header style, content arrangement, footer
- **Components:** Which blocks appear (hero image, author bio, social links, table of contents)
- **Branding:** Logo, name, tagline
- **Content types:** Articles, links roundups, Q&A, curated lists

**My recommendation:** Template system with theme overrides. Ship 3-5 polished templates. Each template is a self-contained HTML/CSS/JS bundle that the MCP server injects content into.

### 4. Multi-tenant vs Single-tenant

Is one MCP server = one newsletter, or can one server host multiple?

**My recommendation:** **Single-tenant for v1.** One server instance = one newsletter. Simpler mental model, easier to self-host, aligns with "your newsletter, your server." Multi-tenant can come later.

### 5. The "No Frontend" Constraint

This is both the killer feature and the biggest risk:

**Strengths:**
- Truly novel — no one is doing newsletters this way
- Zero friction for technical users who already use MCP clients
- Content creation through conversation is genuinely powerful (LLM helps write, edit, structure)
- The LLM IS the UI — personalized, adaptive, accessible

**Risks:**
- MCP client adoption is still growing — limits potential audience
- Some tasks are genuinely harder in chat (visual layout tweaking, drag-and-drop reordering)
- Subscriber onboarding friction: "install an MCP client to read my newsletter" is a tough sell
- Accessibility: not everyone wants/can use an LLM to read content

**My honest take:** The "no frontend" approach is powerful for *publishers* but limiting for *subscribers*. I'd strongly consider auto-generating a static web archive of published issues (even if it's not the primary experience). This doesn't violate the "no frontend to build" promise — it's auto-generated.

---

## Architecture Sketch

```
┌─────────────────────────────────────┐
│          LLM Client                 │
│  (Claude, VS Code, ChatGPT, etc.)  │
└──────────────┬──────────────────────┘
               │ MCP Protocol
               │ (tools + ui resources)
┌──────────────▼──────────────────────┐
│      AI Letter MCP Server           │
│                                     │
│  ┌───────────┐  ┌────────────────┐  │
│  │   Tools   │  │  UI Resources  │  │
│  │           │  │                │  │
│  │ - create  │  │ - newsletter   │  │
│  │ - write   │  │ - issue view   │  │
│  │ - publish │  │ - issue list   │  │
│  │ - config  │  │ - settings     │  │
│  │ - list    │  │                │  │
│  │ - read    │  │                │  │
│  └───────────┘  └────────────────┘  │
│                                     │
│  ┌──────────────────────────────┐   │
│  │     Content Store            │   │
│  │  (Markdown + frontmatter)    │   │
│  │                              │   │
│  │  /newsletter.json  (config)  │   │
│  │  /themes/          (CSS)     │   │
│  │  /issues/          (content) │   │
│  │  /assets/          (images)  │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

---

## Tech Stack (Proposed)

| Layer | Choice | Rationale |
|-------|--------|-----------|
| MCP Server | TypeScript + `@modelcontextprotocol/sdk` | Best ecosystem support, typed, widely adopted |
| UI Rendering | MCP Apps (`@mcp-ui/server`) | Official standard for in-chat UI |
| UI Framework | Preact or vanilla JS | Lightweight, fast load in iframe sandbox |
| Content Format | Markdown + YAML frontmatter | Universal, git-friendly, human-readable |
| Templating | Handlebars or custom | Simple, logic-less, safe for user templates |
| Package Manager | pnpm | Fast, disk-efficient, monorepo-friendly |
| Build | tsup or esbuild | Fast TS compilation |

---

## Project Structure (Proposed)

```
ai-letter/
├── packages/
│   ├── server/          # MCP server (core)
│   │   ├── src/
│   │   │   ├── tools/       # MCP tool handlers
│   │   │   ├── resources/   # MCP UI resources
│   │   │   ├── content/     # Content management
│   │   │   └── themes/      # Theme engine
│   │   └── package.json
│   ├── templates/       # Built-in newsletter templates
│   │   ├── minimal/
│   │   ├── magazine/
│   │   └── digest/
│   └── cli/             # Optional CLI for setup
├── website/             # Project website
├── examples/            # Example newsletters
├── design/              # Design documents (this)
└── README.md
```

---

## Open Design Questions

1. **How should images/assets be handled?** Base64 inline? Local file references? External URLs?
2. **Should we support scheduled publishing?** (cron-like, or manual only)
3. **What analytics (if any)?** Read counts? Open tracking feels antithetical to the ethos.
4. **How do subscribers discover newsletters?** Registry? Directory? Word of mouth?
5. **Versioning of issues?** Can publishers edit after publishing?
6. **Collaboration?** Can multiple people author a newsletter?
7. **Content moderation?** If we build a registry, who moderates?
8. **Monetization path?** Paid subscriptions? Tips? Or purely free/open?

---

## What I Think We Should Build for v1

A focused, opinionated v1:

1. **Single MCP server** that manages one newsletter
2. **5-7 tools:** create newsletter, configure theme, draft issue, publish issue, list issues, read issue, get config
3. **3 UI resources:** issue reader, issue list, newsletter settings
4. **3 built-in templates:** minimal, magazine, digest
5. **Markdown content** stored on local filesystem
6. **Theme customization** via natural language (mapped to CSS variables)
7. **Beautiful defaults** — it should look great out of the box

No email, no multi-tenant, no analytics, no paid subscriptions. Just a rock-solid, beautiful, MCP-native newsletter that works.

---

## Next Steps

- [ ] Align on distribution model
- [ ] Align on content storage format
- [ ] Define the exact tool API (names, params, returns)
- [ ] Design the template/theme system
- [ ] Design the UI resources (wireframes)
- [ ] Set up the monorepo
