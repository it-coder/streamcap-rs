//! Tauri 应用入口点

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use streamcap_rs::{commands, config, RecordingManager};
use tauri::Manager;
use tracing::{error, info};

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
            // AppHandle 在此可用，创建 RecordingManager 并注册
            let recording_manager = RecordingManager::new(app_state_for_rm, app.handle().clone());
            let rm_for_polling = recording_manager.clone();
            app.manage(recording_manager);

            // 应用启动后自动开启直播检测轮询
            tauri::async_runtime::spawn(async move {
                rm_for_polling.start_polling().await;
            });
            // 4. Tauri v2: 监听窗口关闭事件
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        info!("暂停30s");
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.destroy();
                        }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running streamcap-rs");
}
