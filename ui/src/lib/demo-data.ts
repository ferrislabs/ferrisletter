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
];
