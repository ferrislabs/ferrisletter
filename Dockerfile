# syntax=docker/dockerfile:1

# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency compilation separately from source.
# Copy manifests first, build a fake main to warm the cache, then copy source.
COPY Cargo.toml Cargo.lock ./
COPY crates/connector/Cargo.toml       crates/connector/Cargo.toml
COPY crates/connector-static/Cargo.toml crates/connector-static/Cargo.toml
COPY crates/connector-rss/Cargo.toml   crates/connector-rss/Cargo.toml
COPY crates/server/Cargo.toml          crates/server/Cargo.toml

# Stub out each crate so cargo can resolve and build deps.
RUN mkdir -p crates/connector/src crates/connector-static/src \
             crates/connector-rss/src crates/server/src && \
    echo 'pub fn main() {}' > crates/server/src/main.rs && \
    touch crates/connector/src/lib.rs \
          crates/connector-static/src/lib.rs \
          crates/connector-rss/src/lib.rs && \
    cargo build --release -p ferrisletter-server 2>/dev/null || true

# Now copy the real source and rebuild (only changed crates recompile).
COPY crates/ crates/
RUN touch crates/*/src/*.rs && \
    cargo build --release -p ferrisletter-server

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
# distroless/cc includes glibc + libssl — no shell, non-root by default.
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/ferrisletter-server /ferrisletter-server

# Config can be provided via:
#   - FERRISLETTER_CONFIG=/config/ferrisletter.toml (bind-mount the file)
#   - FERRISLETTER_DATA=/data/newsletter.json       (static JSON only)
#   - Neither — falls back to embedded sample data

EXPOSE 3000

ENTRYPOINT ["/ferrisletter-server"]
