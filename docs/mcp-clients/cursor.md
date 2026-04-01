# Ferrisletter with Cursor

## Prerequisites

1. [Cursor](https://cursor.com) 0.43 or later (MCP support)
2. Ferrisletter server built:

```bash
cargo build --release -p ferrisletter-server
```

## Configuration

Open **Cursor Settings → MCP** (or edit `.cursor/mcp.json` in your project, or `~/.cursor/mcp.json` globally).

### With embedded sample data

```json
{
  "mcpServers": {
    "ferrisletter": {
      "command": "/absolute/path/to/ferrisletter-server"
    }
  }
}
```

### With your own data file

```json
{
  "mcpServers": {
    "ferrisletter": {
      "command": "/absolute/path/to/ferrisletter-server",
      "env": {
        "FERRISLETTER_DATA": "/absolute/path/to/your-newsletter.json"
      }
    }
  }
}
```

## Reload

Use **Cursor: Reload MCP Servers** from the command palette after saving.

## Try it in Agent mode

Switch to Agent mode and ask:

```
What topics does ferrisletter have?
```

```
Show me the latest AI news.
```
