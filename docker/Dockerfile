# syntax=docker/dockerfile:1.7

# ------------- chef: cache cargo metadata for incremental builds ---------------
FROM rust:1.99-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked --version 0.1.67

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------- builder ---------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release \
    --bin discord_forum_todo \
    --bin migration

# ------------- runtime ---------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 --shell /usr/sbin/nologin bot
USER bot
WORKDIR /home/bot

COPY --from=builder /app/target/release/discord_forum_todo /usr/local/bin/discord_forum_todo
COPY --from=builder /app/target/release/migration /usr/local/bin/migration
COPY --from=builder /app/locales /home/bot/locales

ENV RUST_LOG=info \
    RUST_LOG_FORMAT=json

ENTRYPOINT ["/usr/local/bin/discord_forum_todo"]
