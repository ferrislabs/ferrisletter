# Ferrisletter with Zed

## Prerequisites

1. [Zed](https://zed.dev) with the Assistant panel enabled
2. Ferrisletter server built:

```bash
cargo build --release -p ferrisletter-server
```

## Configuration

Edit `~/.config/zed/settings.json`:

### With embedded sample data

```json
{
  "context_servers": {
    "ferrisletter": {
      "command": {
        "path": "/absolute/path/to/ferrisletter-server",
        "args": []
      }
    }
  }
}
```

### With your own data file

```json
{
  "context_servers": {
    "ferrisletter": {
      "command": {
        "path": "/absolute/path/to/ferrisletter-server",
        "args": [],
        "env": {
          "FERRISLETTER_DATA": "/absolute/path/to/your-newsletter.json"
        }
      }
    }
  }
}
```

## Try it

Open the Assistant panel and ask:

```
Use ferrisletter to show me today's headlines.
```
