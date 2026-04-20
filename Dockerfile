
FROM rust:1.91-bookworm AS builder

WORKDIR /usr/local/src/ferrisletter


COPY Cargo.toml Cargo.lock ./
COPY crates/connector/Cargo.toml        crates/connector/Cargo.toml
COPY crates/connector-static/Cargo.toml crates/connector-static/Cargo.toml
COPY crates/connector-rss/Cargo.toml    crates/connector-rss/Cargo.toml
COPY crates/server/Cargo.toml           crates/server/Cargo.toml
COPY crates/gateway/Cargo.toml          crates/gateway/Cargo.toml
COPY vendor/rmcp/Cargo.toml             vendor/rmcp/Cargo.toml
COPY vendor/rmcp/build.rs               vendor/rmcp/build.rs
RUN mkdir -p crates/connector/src crates/connector-static/src \
             crates/connector-rss/src crates/server/src crates/gateway/src vendor/rmcp/src && \
    echo 'fn main() {}' > crates/server/src/main.rs && \
    echo 'fn main() {}' > crates/gateway/src/main.rs && \
    touch crates/connector/src/lib.rs \
          crates/connector-static/src/lib.rs \
          crates/connector-rss/src/lib.rs \
          vendor/rmcp/src/lib.rs \
          crates/server/src/lib.rs && \
    cargo build --release


COPY crates/ crates/
COPY vendor/ vendor/
RUN touch crates/*/src/*.rs && \
    touch vendor/rmcp/src/lib.rs && \
    cargo build --release


FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    libssl3=3.0.17-1~deb12u2 && \
    rm -rf /var/lib/apt/lists/* && \
    addgroup \
    --system \
    --gid 1000 \
    ferrisletter && \
    adduser \
    --system \
    --no-create-home \
    --disabled-login \
    --uid 1000 \
    --gid 1000 \
    ferrisletter

USER ferrisletter


FROM runtime AS gateway

COPY --from=builder /usr/local/src/ferrisletter/target/release/gateway /usr/local/bin/gateway

EXPOSE 8080

ENTRYPOINT [ "gateway" ]

FROM runtime AS server

COPY --from=builder /usr/local/src/ferrisletter/target/release/ferrisletter-server /usr/local/bin/ferrisletter-server

EXPOSE 8080

ENTRYPOINT [ "ferrisletter-server" ]
