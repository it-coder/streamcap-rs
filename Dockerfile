# syntax=docker/dockerfile:1
#
# 多阶段构建：
#   1. frontend  —— 在 node 镜像里构建前端静态资源 (frontend/dist)
#   2. builder   —— 在 rust 镜像里编译 server 二进制（Linux，容器内编译，无需宿主机交叉编译）
#   3. runtime   —— 精简 debian 镜像，仅含二进制 + 前端 + ffmpeg 运行时
#
# 构建上下文必须是 streamcap-rs 目录：
#   docker build -t streamcap-rs .
#
# 说明：
#   - streamget-rs 是 git 依赖，builder 阶段需要联网拉取
#   - server 二进制用 --no-default-features --features server 构建，
#     不会触发 desktop/tauri 的 build.rs（已 feature 门控）

# ============================================================
# 1. 前端构建
# ============================================================
FROM node:20-slim AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# ============================================================
# 2. Rust 服务端构建
# ============================================================
FROM rust:1-slim AS builder
WORKDIR /app
# pkg-config + libssl-dev：reqwest 默认使用 native-tls（OpenSSL）
# git + ca-certificates：拉取 streamget-rs 这个 git 依赖
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --no-default-features --features server --bin streamcap-server

# ============================================================
# 3. 运行镜像
# ============================================================
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# ffmpeg：录制必需；libssl3：二进制动态链接 OpenSSL；wget：供 compose healthcheck；ca-certificates：https 请求
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ffmpeg libssl3 wget ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/streamcap-server /app/streamcap-server
COPY --from=frontend /app/frontend/dist /app/frontend/dist

ENV STREAMCAP_DATA_DIR=/app/data
EXPOSE 8080
VOLUME ["/app/data"]

# 静态目录指向打包好的前端；设置与任务持久化到 /app/data（用 volume 挂载）
CMD ["/app/streamcap-server", "--port", "8080", "--static-dir", "/app/frontend/dist"]
