# ===== Stage 1: Build frontend =====
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ===== Stage 2: Build Rust server =====
FROM rust:bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* build.rs ./
COPY src/ ./src/
RUN cargo build --release --no-default-features --features server --bin streamcap-server

# ===== Stage 3: Runtime =====
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/streamcap-server /app/
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
EXPOSE 8080
CMD ["/app/streamcap-server", "--port", "8080", "--static-dir", "frontend/dist"]
