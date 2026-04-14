#!/usr/bin/env bash
# setup-claude-desktop.sh — add Ferrisletter to Claude Desktop in one command.
#
# Usage:
#   ./scripts/setup-claude-desktop.sh
#   ./scripts/setup-claude-desktop.sh --config /path/to/my.toml
#
# Requirements: cargo, jq (for merging into existing claude_desktop_config.json)

set -euo pipefail

# --------------------------------------------------------------------------
# Helper — must be defined before it is called
# --------------------------------------------------------------------------

print_manual_config() {
  echo ""
  echo "Add this to your claude_desktop_config.json:"
  echo ""
  echo "{"
  echo "  \"mcpServers\": {"
  echo "    \"ferrisletter\": {"
  echo "      \"command\": \"$BINARY\","
  echo "      \"args\": [\"--config\", \"$FERRISLETTER_CONFIG\"]"
  echo "    }"
  echo "  }"
  echo "}"
}

# --------------------------------------------------------------------------
# Resolve paths
# --------------------------------------------------------------------------

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_ARG="${1:-}"
CONFIG_PATH="${2:-}"

if [[ "$CONFIG_ARG" == "--config" && -n "$CONFIG_PATH" ]]; then
  FERRISLETTER_CONFIG="$CONFIG_PATH"
else
  FERRISLETTER_CONFIG="$REPO_ROOT/examples/ferrisletter-demo.toml"
fi

# --------------------------------------------------------------------------
# Build the release binary
# --------------------------------------------------------------------------

echo "→ Building ferrisletter-server (release)…"
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" -p ferrisletter-server
BINARY="$REPO_ROOT/target/release/ferrisletter-server"
echo "  ✓ $BINARY"

# --------------------------------------------------------------------------
# Locate Claude Desktop config file
# --------------------------------------------------------------------------

_OS="$(uname -s)"
if [[ "$_OS" == Darwin* ]]; then
  CLAUDE_CONFIG="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
elif [[ "$_OS" == Linux* ]]; then
  CLAUDE_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/Claude/claude_desktop_config.json"
else
  echo "Unsupported OS ($_OS). Add the config manually:" >&2
  print_manual_config
  exit 1
fi

mkdir -p "$(dirname "$CLAUDE_CONFIG")"

# --------------------------------------------------------------------------
# Merge into existing config (or create a new one)
# --------------------------------------------------------------------------

ENTRY=$(cat <<JSON
{
  "command": "$BINARY",
  "args": ["--config", "$FERRISLETTER_CONFIG"]
}
JSON
)

if [ -f "$CLAUDE_CONFIG" ]; then
  if command -v jq &>/dev/null; then
    echo "→ Merging into existing ${CLAUDE_CONFIG}…"
    tmp=$(mktemp)
    jq --argjson entry "$ENTRY" '.mcpServers = (.mcpServers // {}) | .mcpServers.ferrisletter = $entry' "$CLAUDE_CONFIG" > "$tmp"
    mv "$tmp" "$CLAUDE_CONFIG"
  else
    echo "⚠  jq not found — printing config to merge manually." >&2
    print_manual_config
    exit 0
  fi
else
  echo "→ Creating ${CLAUDE_CONFIG}…"
  cat > "$CLAUDE_CONFIG" <<JSON
{
  "mcpServers": {
    "ferrisletter": $ENTRY
  }
}
JSON
fi

# --------------------------------------------------------------------------
# Done
# --------------------------------------------------------------------------

echo ""
echo "✓ Ferrisletter added to Claude Desktop"
echo ""
echo "  Binary : $BINARY"
echo "  Config : $FERRISLETTER_CONFIG"
echo "  Topics : AI & LLMs · Rust · Open Source"
echo ""
echo "  Restart Claude Desktop, then try:"
echo "    \"What's new in AI this week?\""
echo "    \"Catch me up on Rust since last Monday\""
echo "    \"Find anything about agents or MCP\""
echo ""
