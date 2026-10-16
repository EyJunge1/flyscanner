# ---- frontend ----
FROM node:22-bookworm AS web
WORKDIR /web
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ---- backend (embeds frontend/build as blobs) ----
FROM rust:1-bookworm AS rust
WORKDIR /src
COPY Cargo.toml Cargo.lock* rust-toolchain.toml rustfmt.toml clippy.toml build.rs ./
COPY src ./src
COPY --from=web /web/build ./frontend/build
RUN cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust /src/target/release/flyscanner /usr/local/bin/flyscanner
ENV FLYSCANNER_DB=/data/flyscanner.db
ENV FLYSCANNER_LISTEN=0.0.0.0:8080
VOLUME ["/data"]
EXPOSE 8080 30004
ENTRYPOINT ["flyscanner"]
