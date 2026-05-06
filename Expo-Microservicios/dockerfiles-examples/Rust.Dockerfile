# Etapa 1: Builder
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Etapa 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Importante para apps que consumen APIs externas (HTTPS)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mi_binario .

CMD ["./mi_binario"]