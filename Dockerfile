# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates curl libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /app app

WORKDIR /app

COPY --from=builder /build/target/release/metervalues /app/metervalues
COPY --from=builder /build/static /app/static

RUN chown -R app:app /app

USER app

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:8000/health || exit 1

ENTRYPOINT ["/app/metervalues"]
