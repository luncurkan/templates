FROM rust:1.85-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY migration ./migration
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub mod app; pub mod controllers; pub mod models;" > src/lib.rs
RUN mkdir -p src/controllers src/models/_entities
RUN echo "pub mod health; pub mod items;" > src/controllers/mod.rs
RUN echo "pub mod _entities;" > src/models/mod.rs
RUN echo "pub mod items;" > src/models/_entities/mod.rs
RUN touch src/app.rs src/controllers/health.rs src/controllers/items.rs src/models/_entities/items.rs
RUN cargo build --release || true
RUN rm -rf src target/release/rust-loco-fullstack target/release/deps/rust_loco_fullstack* target/release/deps/librust_loco_fullstack* target/release/.fingerprint/rust-loco-fullstack*

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-loco-fullstack ./app
COPY --from=builder /app/config ./config
COPY --from=builder /app/assets ./assets

ENV LOCO_ENV=production
ENV PORT=3000
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/app

EXPOSE 3000

CMD ["./app", "start"]
