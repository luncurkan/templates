FROM rust:1.85-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src target/release/rust-actix-rest-api target/release/deps/rust_actix_rest_api*

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-actix-rest-api ./app

ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/app
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./app"]
