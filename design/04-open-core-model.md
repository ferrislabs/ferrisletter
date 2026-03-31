# 04 — Open Core Model

## The Split

| Open Source (the platform) | Closed Source (the service) |
|---|---|
| MCP Server | Hosted backend API |
| UI Components (Svelte 5) | Curated news database |
| Connector Interface | Scraping + tagging pipeline |
| Conversation Logic | Content curation models |
| Preference handling | Hosted infrastructure |
| Example connectors (RSS, static) | — |

The open source project is a **complete, useful product** on its own. Anyone can build a connector, plug in their own content, and run a conversational newsletter.

Our closed source part is the **service**, not the software — the curated content database, scraping infrastructure, and hosted API. The connector that talks to our API can be open source; the API key gates access to the service.

---

## Connector Interface

A connector is the boundary between the platform and a content source. The MCP server handles everything else: conversation flow, UI rendering, scheduling, preferences.

```go
// Connector is the interface any content source must implement.
type Connector interface {
    // ListTopics returns the available content categories.
    ListTopics() ([]Topic, error)

    // GetLatestItems returns recent items, filtered by user preferences.
    GetLatestItems(prefs UserPrefs) ([]Item, error)

    // GetItemDetail returns the full content for a specific item.
    GetItemDetail(id string) (*ItemDetail, error)

    // Search finds items matching a query across all past content.
    Search(query string, filters SearchFilters) ([]Item, error)

    // GetRecap returns a summary of items since a given point in time.
    GetRecap(since time.Time, prefs UserPrefs) ([]Item, error)
}
```

### Types

```go
type Topic struct {
    ID          string
    Label       string   // e.g. "Research Papers"
    Description string   // e.g. "Latest notable papers from arxiv, conferences"
    Tags        []string // for filtering
}

type Item struct {
    ID        string
    TopicID   string
    Headline  string    // one-liner shown in compact view
    Summary   string    // 1-2 sentences, shown in compact view
    Tags      []string
    Source    string    // original source URL
    Published time.Time
}

type ItemDetail struct {
    Item
    Body     string   // full content in Markdown
    Links    []Link   // related links
    ReadTime string   // e.g. "3 min"
}

type Link struct {
    URL   string
    Label string
}

type UserPrefs struct {
    TopicsOfInterest []string // topic IDs to include
    SummaryLength    string   // "brief" | "standard" | "detailed"
    MaxItems         int      // max items per request
    Language         string   // "en", "fr", etc.
}

type SearchFilters struct {
    TopicIDs []string
    Since    *time.Time
    Until    *time.Time
    Tags     []string
    Limit    int
}
```

---

## Example Connectors

### 1. AI Letter Connector (our service)

Calls our hosted API. Requires an API key. Content is curated by our pipeline.

```go
type AILetterConnector struct {
    apiKey  string
    baseURL string
    client  *http.Client
}
```

### 2. RSS Connector (open source, bundled)

Reads from one or more RSS/Atom feeds. Uses the LLM to tag and summarize entries.

```go
type RSSConnector struct {
    feeds []FeedConfig
}

type FeedConfig struct {
    URL     string
    TopicID string
    Tags    []string
}
```

### 3. Static/JSON Connector (open source, bundled)

Reads from a local JSON file or directory. Useful for testing, demos, and simple personal newsletters.

```go
type StaticConnector struct {
    dataDir string
}
```

---

## What This Enables

- **Us:** We sell access to curated content via API keys. The software is free; the service is the product.
- **Community:** Anyone can build a connector for any source — Hacker News, arxiv, Reddit, Substack imports, internal company feeds.
- **Self-hosters:** Full platform works without our service. Bring your own content.
- **Contributors:** Clear contribution boundary. PRs to the platform or new connectors welcome. No ambiguity about what's open.

---

## Revenue Model (future)

- Free tier: limited API calls, delayed content
- Paid tier: full access, real-time content, priority topics
- Enterprise: custom connectors, private content sources, SLA

---

## Open Questions

1. **Connector registration:** How does the MCP server discover and load connectors? Plugin system? Config file pointing to a binary/module?
2. **Multiple connectors:** Can a user have multiple connectors active at once? e.g. AI Letter + personal RSS feed, merged into one conversation.
3. **Connector auth:** Each connector handles its own auth (API key, OAuth, etc.), or is there a standard auth wrapper?
4. **Content freshness:** Should the connector interface include a `LastUpdated()` or `HasNewContent()` method for efficient polling?
