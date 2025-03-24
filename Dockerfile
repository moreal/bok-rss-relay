FROM rust:1.85.1 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /
EXPOSE 3000

COPY --from=builder /app/target/release/bok-rss-relay /
COPY ./static /static

ENTRYPOINT ["/bok-rss-relay"]
