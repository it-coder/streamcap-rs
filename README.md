# StreamCap RS

Rust 直播流录制工具 — 基于 [streamget-rs](https://github.com/it-coder/streamget-rs.git) + Tauri / Axum，支持桌面客户端和 B/S 服务器双模式部署。

## 功能特性

- **多平台直播流解析** — 抖音、B站、Twitch、YouTube、虎牙、快手、斗鱼、TikTok
- **自动监控录制** — 定时轮询检测开播，自动开始/停止录制
- **FFmpeg 录制** — 支持多画质（原画/超清/高清/标清）、多格式（MP4/TS/MKV/FLV/MOV）
- **分段录制** — 可配置分段时长，避免单文件过大
- **格式转换** — 录制后可选自动转码（MP4/MKV/MOV/FLV/TS），流复制无重编码，可配置是否删除原文件
- **目录规则** — 按平台/主播/日期/标题自动归类
- **代理支持** — 可配置 HTTP 代理访问直播流
- **实时状态推送** — 桌面模式 Tauri 事件 / 服务器模式 WebSocket
- **双模式部署** — 桌面客户端打包 + B/S 服务器部署，共享同一套业务逻辑

## 技术栈

| 层 | 技术 |
|---|------|
| 后端 | Rust + Tokio + streamget-rs + FFmpeg |
| 桌面壳 | Tauri v2 |
| 服务器 | Axum 0.7 + tower-http |
| 前端 | React 18 + TypeScript + Ant Design + Vite |

## 前置要求

- **Rust** (edition 2021, 建议 1.75+)
- **Node.js** 18+ 和 npm
- **FFmpeg** — 已安装并可在 PATH 中访问
- **Tauri CLI v2**（仅桌面模式需要）— `cargo install tauri-cli --version "^2"`

## 快速开始

### 方式一：桌面客户端模式（默认）

```bash
# 安装前端依赖
cd frontend && npm install && cd ..

# 开发模式
cargo tauri dev

# 打包
cargo tauri build
```

打包产物在 `src-tauri/target/release/bundle/`。

### 方式二：B/S 服务器模式

```bash
# 1. 构建前端
cd frontend && npm install && npm run build && cd ..

# 2. 编译并运行服务器
cargo run --bin streamcap-server --no-default-features --features server -- --port 8080

# 3. 浏览器访问
# http://localhost:8080
```

### 方式三：B/S 开发模式（前后端分离热更新）

```bash
# 终端 1：启动后端服务器
cargo run --bin streamcap-server --no-default-features --features server -- --port 8080

# 终端 2：启动前端 Vite dev server（自动代理 /api 和 /ws 到 8080）
cd frontend && npm run dev
# 浏览器访问 http://localhost:1420
```

### 方式四：Docker 部署

```bash
# 方式 A：拉取 GHCR 预构建镜像（推荐）
docker pull ghcr.io/<owner>/streamcap-rs:latest

# 方式 B：本地构建
docker build -t streamcap-rs .

# 运行（录制文件挂载到宿主机）
docker run -d \
  -p 8080:8080 \
  -v /path/to/recordings:/app/recordings \
  ghcr.io/<owner>/streamcap-rs:latest
```

Docker 镜像内置 FFmpeg，无需额外安装。多平台支持 `linux/amd64` + `linux/arm64`。

## 服务器命令行参数

```
streamcap-server [OPTIONS]

OPTIONS:
    --port <PORT>          监听端口 (默认 8080)
    --static-dir <DIR>     前端静态文件目录 (默认 frontend/dist)
```

## 项目结构

```
streamcap-rs/
├── src/
│   ├── main.rs                  # 桌面模式入口 (Tauri)
│   ├── bin/
│   │   └── server.rs            # 服务器模式入口 (Axum)
│   ├── lib.rs                   # 库入口，模块声明
│   ├── broadcaster.rs           # EventBroadcaster trait + Tauri/Ws 实现
│   ├── config.rs                # AppState 配置管理 + 持久化
│   ├── models.rs                # 数据模型 (RecordingConfig, AppSettings 等)
│   ├── stream/
│   │   ├── mod.rs
│   │   └── resolver.rs          # 直播流 URL 解析 (streamget-rs)
│   ├── recording/
│   │   ├── mod.rs
│   │   ├── manager.rs           # RecordingManager 核心状态机
│   │   ├── downloader.rs        # 流下载器
│   │   └── ffmpeg.rs            # FFmpeg 子进程管理
│   ├── commands/                # Tauri 命令 (仅 desktop feature)
│   │   ├── mod.rs
│   │   ├── recording.rs
│   │   └── settings.rs
│   └── server/                  # Axum 服务器 (仅 server feature)
│       ├── mod.rs               # 路由定义 + ServerState
│       ├── handlers.rs          # REST API handlers
│       └── ws.rs                # WebSocket 事件推送
├── frontend/
│   ├── src/
│   │   ├── api/
│   │   │   ├── provider.ts      # ApiProvider 接口 + 自动模式检测
│   │   │   ├── tauri-provider.ts # Tauri IPC 实现
│   │   │   ├── http-provider.ts  # HTTP + WebSocket 实现
│   │   │   └── tauri.ts         # re-export provider (向后兼容)
│   │   ├── hooks/
│   │   │   ├── useRecordings.ts
│   │   │   ├── useSettings.ts
│   │   │   └── useFfmpegStatus.ts
│   │   ├── components/
│   │   │   └── ShutdownOverlay.tsx
│   │   ├── types/
│   │   │   └── index.ts
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── vite.config.ts           # Vite 配置 (含 B/S 代理)
│   └── package.json
├── capabilities/
│   └── default.json             # Tauri v2 权限配置
├── .github/workflows/
│   ├── ci.yml                   # CI 检查 (cargo check + clippy + 前端构建)
│   ├── main.yml                 # 桌面客户端打包发布 (tag 触发)
│   └── docker-build.yml         # Docker 镜像构建推送 (tag 触发)
├── build.rs                     # Tauri build script
├── tauri.conf.json              # Tauri 配置
├── Cargo.toml                   # Rust 依赖 + features
├── Dockerfile                   # Docker 多阶段构建
└── .dockerignore
```

## 架构设计

### 双模式共享业务层

```
                    ┌─────────────────────────────────┐
                    │       共享业务逻辑层             │
                    │  RecordingManager / AppState     │
                    │  stream::resolver / ffmpeg       │
                    └──────────┬──────────┬────────────┘
                               │          │
              ┌────────────────┘          └────────────────┐
              ▼                                            ▼
    ┌──────────────────┐                        ┌──────────────────┐
    │   桌面模式        │                        │   服务器模式      │
    │   TauriBroadcaster│                        │   WsBroadcaster   │
    │   Tauri IPC       │                        │   Axum HTTP+WS    │
    │   src/main.rs     │                        │   src/bin/server  │
    └──────────────────┘                        └──────────────────┘
```

### EventBroadcaster 解耦

`RecordingManager` 通过 `EventBroadcaster` trait 推送事件，不直接依赖 Tauri 或 Axum：

- **TauriBroadcaster** — 调用 `AppHandle.emit()` 推送到桌面前端
- **WsBroadcaster** — 通过 `tokio::sync::broadcast` fan-out 到所有 WebSocket 客户端

### 前端 Provider 自动切换

前端运行时检测 `window.__TAURI__` 自动选择通信方式，UI 组件层无需感知：

- **TauriApiProvider** — `invoke()` + `listen()` (桌面模式)
- **HttpApiProvider** — `fetch()` + `WebSocket` (B/S 模式)

## REST API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/recordings` | 获取所有录制任务 |
| POST | `/api/recordings` | 添加录制任务 |
| PUT | `/api/recordings/:id` | 更新录制任务 |
| DELETE | `/api/recordings/:id` | 删除录制任务 |
| GET | `/api/recordings/:id/status` | 获取单个任务状态 |
| POST | `/api/recordings/:id/monitor` | 启用监控 |
| DELETE | `/api/recordings/:id/monitor` | 停止监控 |
| POST | `/api/recordings/:id/recording` | 手动开始录制 |
| DELETE | `/api/recordings/:id/recording` | 手动停止录制 |
| GET | `/api/settings` | 获取设置 |
| PUT | `/api/settings` | 更新设置 |
| GET | `/api/ffmpeg/check` | 检查 FFmpeg 是否可用 |

### WebSocket 事件

连接 `ws://<host>/ws/events`，接收 JSON 消息：

```jsonc
// 录制状态变更
{ "type": "recording_status", "data": { "id": "...", "is_recording": true, ... } }

// 服务器关闭通知
{ "type": "app:shutdown", "data": { "stage": "start", "activeCount": 2, "message": "..." } }
```

## Cargo Features

| Feature | 说明 | 依赖 |
|---------|------|------|
| `desktop` (默认) | Tauri 桌面客户端模式 | tauri, tauri-plugin-* |
| `server` | B/S 服务器模式 | axum, tower-http |

```bash
# 编译桌面模式（默认）
cargo build --release

# 编译服务器模式
cargo build --release --no-default-features --features server --bin streamcap-server
```

## 配置说明

录制任务和设置持久化在用户数据目录下的 JSON 文件中：

- **桌面模式**: `~/.streamcap-rs/recordings.json` 和 `settings.json`
- **服务器模式**: 运行目录下的 `recordings.json` 和 `settings.json`（或通过环境变量指定）

### 设置项

| 字段 | 说明 | 默认值 |
|------|------|--------|
| `default_quality` | 默认画质 | OD (原画) |
| `default_format` | 默认格式 | TS |
| `output_dir` | 输出根目录 | ~/Downloads |
| `loop_interval_seconds` | 轮询间隔(秒) | 180 |
| `recording_space_threshold_gb` | 磁盘空间阈值(GB) | 0 (不检查) |
| `enable_proxy` | 启用代理 | false |
| `proxy_url` | 代理地址 | null |
| `folder_by_platform` | 按平台分文件夹 | false |
| `folder_by_anchor` | 按主播分文件夹 | false |
| `folder_by_date` | 按日期分文件夹 | true |
| `folder_by_title` | 按标题分文件夹 | false |
| `segment_duration_seconds` | 分段时长(秒) | 1800 (30分钟) |
| `enable_conversion` | 录制后启用格式转换 | false |
| `conversion_format` | 转换目标格式 | MP4 |
| `delete_original_after_conversion` | 转换后删除原文件 | true |

## 两种模式对比

| | 桌面模式 | B/S 模式 |
|---|---------|---------|
| 通信方式 | Tauri IPC | HTTP + WebSocket |
| 前端 | 嵌入二进制 | 服务器静态托管 |
| 关闭提示 | ShutdownOverlay 遮罩 | WS 推送 shutdown 事件 |
| FFmpeg | 本机 PATH | 服务器端 PATH |
| 文件路径 | 用户选择 | 服务器端配置 |
| 多用户 | 单用户 | 多浏览器共享同一任务列表 |
| 部署方式 | 安装包 | Docker / 二进制 |

## CI/CD

| Workflow | 触发条件 | 说明 |
|----------|---------|------|
| `ci.yml` | push / PR → main | `cargo check` + `cargo clippy`（desktop + server 两种 feature）、前端 `npm run build` |
| `main.yml` | tag `v*.*.*` | Tauri 桌面客户端多平台打包（macOS ARM/Intel、Linux、Windows），自动创建 GitHub Release |
| `docker-build.yml` | tag `v*.*.*` / 手动 | 多平台 Docker 镜像构建（amd64 + arm64），推送到 GHCR |

### 拉取 Docker 镜像

```bash
# 拉取最新版
docker pull ghcr.io/<owner>/streamcap-rs:latest

# 拉取指定版本
docker pull ghcr.io/<owner>/streamcap-rs:1.0.0
```

## License

MIT
