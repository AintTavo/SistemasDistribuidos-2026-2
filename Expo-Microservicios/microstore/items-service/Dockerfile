# ── Etapa 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -f src/main.rs

COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# ── Etapa 2: Runtime mínimo ───────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/items-service .

EXPOSE 8002
CMD ["./items-service"]
