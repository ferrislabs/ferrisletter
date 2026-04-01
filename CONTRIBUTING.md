# Contributing to Ferrisletter

Thanks for your interest in contributing! This guide covers everything you need to get started.

## Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2021)
- [lefthook](https://github.com/evilmartians/lefthook) for git hooks

## First-time setup

```bash
# Clone the repo
git clone https://github.com/ferrislabs/ferrisletter.git
cd ferrisletter

# Install lefthook (macOS)
brew install lefthook

# Wire up the git hooks — run once per clone
lefthook install
```

After `lefthook install`, a **pre-push** gate runs automatically before every `git push`:

| Check | Command |
|---|---|
| Formatting | `cargo fmt --all -- --check` |
| Lints | `cargo clippy --workspace -- -D warnings` |
| Tests | `cargo test --workspace` |

The gate only runs when `.rs` files are part of the push, so pure documentation or config-only changes don't trigger a full build.

## Development workflow

```bash
# Build the workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Check lints
cargo clippy --workspace -- -D warnings

# Auto-fix formatting
cargo fmt --all
```

## Project structure

```
crates/
  connector/          # Connector trait + BoxedConnector (SDK)
  connector-static/   # Static JSON connector
  connector-rss/      # RSS/Atom connector
  server/             # MCP server binary
examples/             # Sample config and data files
website/              # Ferrislabs marketing site (Astro)
docs/                 # MCP client setup guides
```

## Opening a pull request

1. Branch off `main`: `git checkout -b feat/your-feature`
2. Make your changes and commit
3. Push — the pre-push gate will run automatically
4. Open a PR against `main` and fill in the template

Please link the relevant GitHub issue in your PR description.
