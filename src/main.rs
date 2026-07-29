//! Tauri 应用入口点

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use streamcap_rs::{commands, config, RecordingManager};

fn main() {
    tracing_subscriber::fmt::init();

    let app_state = config::AppState::new();
    let recording_manager = RecordingManager::new(app_state.clone());
    let rm_for_polling = recording_manager.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .manage(recording_manager)
        .setup(move |_app| {
            // 应用启动后自动开启直播检测轮询
            tauri::async_runtime::spawn(async move {
                rm_for_polling.start_polling().await;
            });
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
