FROM rust:1.85.0 AS builder

WORKDIR /usr/src/app

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="just-coin-price"
LABEL org.opencontainers.image.description="A simple API server to retrieve coin prices from multiple vendors in a consistent API interface."
LABEL org.opencontainers.image.authors="Lee Dogeon"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/moreal/just-coin-price"
LABEL org.opencontainers.image.version="0.0.0"

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/just-coin-price /app/

RUN addgroup --system app && \
    adduser --system --ingroup app app && \
    chown -R app:app /app
USER app

EXPOSE 3000

CMD ["/app/just-coin-price"]
