//! Tauri 应用入口点 — 桌面客户端模式

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use streamcap_rs::{broadcaster::TauriBroadcaster, commands, config, RecordingManager};
use tauri::Manager;
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    let app_state = config::AppState::new();
    let app_state_for_rm = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .setup(move |app| {
            // 创建 TauriBroadcaster 并传入 RecordingManager
            let broadcaster = Arc::new(TauriBroadcaster::new(app.handle().clone()));
            let recording_manager = RecordingManager::new(app_state_for_rm, broadcaster);
            let rm_for_polling = recording_manager.clone();
            app.manage(recording_manager);

            // 应用启动后自动开启直播检测轮询
            tauri::async_runtime::spawn(async move {
                rm_for_polling.start_polling().await;
            });

            // 窗口关闭时优雅停止所有录制
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        info!("窗口关闭请求，正在停止所有录制...");

                        let rm = app_handle
                            .state::<Arc<RecordingManager>>()
                            .inner()
                            .clone();

                        tauri::async_runtime::spawn(async move {
                            let active = rm.active_count().await;

                            // 通知前端：开始关闭流程
                            rm.broadcast_shutdown(
                                &streamcap_rs::ShutdownPayload::start(active),
                            );

                            // 执行优雅停止
                            let stopped = rm.shutdown_all().await;

                            // 通知前端：停止完成
                            rm.broadcast_shutdown(
                                &streamcap_rs::ShutdownPayload::done(stopped),
                            );

                            // 给前端 600ms 渲染完成提示
                            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                            info!("录制已停止，退出应用");
                            std::process::exit(0);
                        });
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::recording::add_recording,
            commands::recording::remove_recording,
            commands::recording::update_recording,
            commands::recording::list_recordings,
            commands::recording::start_monitor,
            commands::recording::stop_monitor,
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::get_recording_status,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::check_ffmpeg,
            commands::settings::get_version,
            commands::files::list_files,
            commands::files::open_file,
            commands::files::read_file_base64,
        ])
        .run(tauri::generate_context!())
        .expect("error while running streamcap-rs");
}
