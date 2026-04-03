import type { Topic, Item } from "@/types";

/** Shown when no MCP server URL is configured. */
export const DEMO_TOPICS: Topic[] = [
  {
    id: "rust",
    label: "Rust",
    description: "News and releases from the Rust ecosystem",
    tags: ["rust", "programming", "systems"],
  },
  {
    id: "ai",
    label: "AI & LLMs",
    description: "Large language models, agents, and AI tooling",
    tags: ["ai", "llm", "agents", "mcp"],
  },
  {
    id: "open-source",
    label: "Open Source",
    description: "Notable open-source releases and community news",
    tags: ["open-source", "github", "community"],
  },
];

export const DEMO_ITEMS: Item[] = [
  {
    id: "demo-1",
    topic_id: "rust",
    headline: "Rust 2024 edition ships with async fn in traits",
    summary:
      "The 2024 edition lands native async fn in trait support, eliminating the need for the async-trait crate in most cases. Existing code migrates automatically via rustfix.",
    tags: ["rust", "async", "edition"],
    source: "blog.rust-lang.org",
    published: new Date().toISOString(),
    read_time: "4 min",
  },
  {
    id: "demo-2",
    topic_id: "rust",
    headline: "cargo-semver-checks now covers 95% of breaking changes",
    summary:
      "The popular linting tool that catches semver violations before you publish has expanded its rule set to cover trait implementations and type aliases.",
    tags: ["rust", "cargo", "semver"],
    source: "predr.ag",
    published: new Date(Date.now() - 86400000).toISOString(),
    read_time: "3 min",
  },
  {
    id: "demo-3",
    topic_id: "ai",
    headline: "MCP becomes the de-facto standard for LLM tool use",
    summary:
      "Anthropic's Model Context Protocol is now supported by Claude, Cursor, Zed, and a growing list of open-source clients — establishing a common language for AI tool integration.",
    tags: ["mcp", "ai", "tooling"],
    source: "modelcontextprotocol.io",
    published: new Date(Date.now() - 172800000).toISOString(),
    read_time: "5 min",
  },
  {
    id: "demo-4",
    topic_id: "ai",
    headline: "Agents that write their own tools are gaining traction",
    summary:
      "Several research teams are publishing results showing LLMs can reliably generate, test, and register new MCP tools at runtime — a step toward truly autonomous agents.",
    tags: ["ai", "agents", "research"],
    source: "arxiv.org",
    published: new Date(Date.now() - 259200000).toISOString(),
    read_time: "6 min",
  },
  {
    id: "demo-5",
    topic_id: "open-source",
    headline: "lefthook hits 10k GitHub stars",
    summary:
      "The fast, language-agnostic git-hook manager from Evil Martians crossed a milestone this week, driven by adoption in Rust and Go projects looking for a committed-config alternative to Husky.",
    tags: ["tooling", "git", "open-source"],
    source: "github.com",
    published: new Date(Date.now() - 345600000).toISOString(),
    read_time: "2 min",
  },
  {
    id: "demo-6",
    topic_id: "rust",
    headline: "Bevy 0.16 brings a new scene system and required components — this is a really long headline to test tooltip truncation behavior in the compact view",
    summary:
      "The popular Rust game engine overhauls its scene format and introduces required components, making entity bundles simpler and more ergonomic.",
    tags: ["rust", "gamedev", "bevy"],
    source: "bevyengine.org",
    published: new Date(Date.now() - 432000000).toISOString(),
    read_time: "7 min",
  },
  {
    id: "demo-7",
    topic_id: "rust",
    headline: "Embassy reaches 1.0 for embedded async Rust",
    summary:
      "The async runtime for embedded systems hits its stable release, supporting STM32, nRF, and RP2040 families with zero-alloc async/await.",
    tags: ["rust", "embedded", "async"],
    source: "embassy.dev",
    published: new Date(Date.now() - 518400000).toISOString(),
    read_time: "5 min",
  },
  {
    id: "demo-8",
    topic_id: "ai",
    headline: "Claude Code launches hooks for pre/post-command automation",
    summary:
      "Anthropic adds a hooks system to Claude Code, letting developers run custom scripts before or after specific actions — enabling linting, testing, and deployment pipelines.",
    tags: ["ai", "claude", "tooling"],
    source: "anthropic.com",
    published: new Date(Date.now() - 604800000).toISOString(),
    read_time: "3 min",
  },
  {
    id: "demo-9",
    topic_id: "ai",
    headline: "OpenAI open-sources a lightweight MCP client library",
    summary:
      "A minimal TypeScript client for the Model Context Protocol, designed for quick integration into existing apps without the full SDK overhead.",
    tags: ["ai", "mcp", "open-source"],
    source: "github.com/openai",
    published: new Date(Date.now() - 691200000).toISOString(),
    read_time: "4 min",
  },
  {
    id: "demo-10",
    topic_id: "open-source",
    headline: "Zed editor adds native MCP App panel support",
    summary:
      "The GPU-accelerated editor now renders MCP App UIs inline, joining Claude Desktop as a host that supports the interactive newsletter experience.",
    tags: ["editor", "mcp", "open-source"],
    source: "zed.dev",
    published: new Date(Date.now() - 777600000).toISOString(),
    read_time: "3 min",
  },
  {
    id: "demo-11",
    topic_id: "open-source",
    headline: "SQLite 3.50 adds vector search as a built-in module",
    summary:
      "No extensions needed — the world's most deployed database now ships with vector similarity search out of the box, powered by DiskANN indexes.",
    tags: ["database", "sqlite", "vector"],
    source: "sqlite.org",
    published: new Date(Date.now() - 864000000).toISOString(),
    read_time: "4 min",
  },
  {
    id: "demo-12",
    topic_id: "rust",
    headline: "Tokio 2.0 roadmap published with structured concurrency",
    summary:
      "The Tokio team outlines plans for structured concurrency, scoped tasks, and a simplified runtime builder — targeting a release by late 2026.",
    tags: ["rust", "async", "tokio"],
    source: "tokio.rs",
    published: new Date(Date.now() - 950400000).toISOString(),
    read_time: "6 min",
  },
  {
    id: "demo-13",
    topic_id: "ai",
    headline: "Google DeepMind publishes Gemini tool-use benchmark results",
    summary:
      "New benchmarks comparing tool-use accuracy across Claude, GPT-4, and Gemini show convergence in capabilities but divergent strategies for complex multi-step tasks.",
    tags: ["ai", "benchmark", "research"],
    source: "deepmind.google",
    published: new Date(Date.now() - 1036800000).toISOString(),
    read_time: "8 min",
  },
  {
    id: "demo-14",
    topic_id: "open-source",
    headline: "Deno 2.2 achieves full Node.js compatibility",
    summary:
      "With the latest release, Deno can run virtually any npm package unmodified — closing the last major gap with Node.js while keeping its security-first defaults.",
    tags: ["javascript", "runtime", "open-source"],
    source: "deno.com",
    published: new Date(Date.now() - 1123200000).toISOString(),
    read_time: "5 min",
  },
];
