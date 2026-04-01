# Ferrisletter with Claude Desktop

## Prerequisites

1. [Claude Desktop](https://claude.ai/download) installed
2. Ferrisletter server built:

```bash
git clone https://github.com/ferrislabs/ferrisletter
cd ferrisletter
cargo build --release -p ferrisletter-server
```

The binary will be at `target/release/ferrisletter-server`.

## Configuration

Edit `claude_desktop_config.json`:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |

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

See [`examples/sample-newsletter.json`](../../examples/sample-newsletter.json) for the data file format.

## Restart Claude Desktop

After saving the config, restart Claude Desktop. You should see a hammer icon (🔨) in the input bar indicating tools are available.

## Try it

```
What newsletter topics are available?
```

```
Show me the latest headlines.
```

```
What happened in the Rust ecosystem this week?
```

```
Give me a recap of everything since Monday.
```
