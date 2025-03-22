FROM rust:1.85.0 AS builder

WORKDIR /usr/src/app

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime

LABEL org.opencontainers.image.title="just-coin-price"
LABEL org.opencontainers.image.description="A simple API server to retrieve coin prices from multiple vendors in a consistent API interface."
LABEL org.opencontainers.image.authors="Lee Dogeon"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/moreal/just-coin-price"
LABEL org.opencontainers.image.version="0.0.0"

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/just-coin-price /app/

EXPOSE 3000

CMD ["/app/just-coin-price"]
