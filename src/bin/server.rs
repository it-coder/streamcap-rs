//! streamcap-server — B/S 模式服务器入口
//!
//! 启动 HTTP + WebSocket 服务器，提供 REST API 和实时事件推送。
//! 前端静态文件由服务器内置提供，浏览器直接访问即可。
//!
//! 用法:
//!   ./streamcap-server                              # 默认端口 8080
//!   ./streamcap-server --port 3000                  # 自定义端口
//!   ./streamcap-server --static-dir /var/www/app    # 自定义静态目录

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use streamcap_rs::broadcaster::WsBroadcaster;
use streamcap_rs::server::{build_router, ServerState};
use streamcap_rs::{config, RecordingManager, ShutdownPayload};

fn parse_args() -> (u16, String) {
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 8080;
    let mut static_dir = "frontend/dist".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse() {
                        port = p;
                    }
                    i += 1;
                }
            }
            "--static-dir" | "-d" => {
                if i + 1 < args.len() {
                    static_dir = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    (port, static_dir)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (port, static_dir) = parse_args();

    // 检查静态目录是否存在
    if !std::path::Path::new(&static_dir).exists() {
        info!(
            "警告: 静态目录 '{}' 不存在，前端页面将不可用。仅 API 模式。",
            static_dir
        );
    }

    // 创建共享状态
    let app_state = config::AppState::new();
    let ws_broadcaster = WsBroadcaster::new();

    // 创建 RecordingManager，传入 WsBroadcaster 作为 EventBroadcaster
    let broadcaster: Arc<dyn streamcap_rs::EventBroadcaster> = ws_broadcaster.clone();
    let recording_manager = RecordingManager::new(app_state.clone(), broadcaster);

    // 启动轮询
    let rm_for_polling = recording_manager.clone();
    tokio::spawn(async move {
        rm_for_polling.start_polling().await;
    });

    // 构建服务器状态和路由
    let server_state = Arc::new(ServerState {
        app_state,
        recording_manager: recording_manager.clone(),
        ws_broadcaster,
        started_at: Instant::now(),
    });

    let app = build_router(server_state.clone(), &static_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("StreamCap 服务器启动: http://localhost:{}", port);
    info!("静态文件目录: {}", static_dir);
    info!("WebSocket 端点: ws://localhost:{}/ws/events", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("无法绑定端口");

    // 启动服务器，等待 Ctrl+C 优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("无法注册 Ctrl+C 信号处理器");
            info!("\n收到关闭信号，正在停止所有录制...");
        })
        .await
        .expect("服务器错误");

    // 服务器已停止，执行录制清理
    let active = recording_manager.active_count().await;
    recording_manager.broadcast_shutdown(&ShutdownPayload::start(active));

    let stopped = recording_manager.shutdown_all().await;

    recording_manager.broadcast_shutdown(&ShutdownPayload::done(stopped));

    info!("已停止 {} 个录制任务，服务器已关闭", stopped);
}
