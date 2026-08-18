//! Tauri 应用入口点 — 桌面客户端模式

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use streamcap_rs::{
    broadcaster::TauriBroadcaster, commands, config, cleanup, RecordingManager, ShutdownPayload,
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tracing::info;

/// 优雅停止所有录制并退出应用
async fn graceful_quit(app: tauri::AppHandle) {
    let rm = app
        .state::<Arc<RecordingManager>>()
        .inner()
        .clone();
    let active = rm.active_count().await;

    // 通知前端：开始关闭流程
    rm.broadcast_shutdown(&ShutdownPayload::start(active));

    // 执行优雅停止
    let stopped = rm.shutdown_all().await;

    // 通知前端：停止完成
    rm.broadcast_shutdown(&ShutdownPayload::done(stopped));

    // 给前端 600ms 渲染完成提示
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    info!("录制已停止，退出应用");
    std::process::exit(0);
}

/// 创建系统托盘：菜单（显示/隐藏/立即清理/退出）+ 动态提示（活动录制数）
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("缺少默认窗口图标，无法创建托盘");

    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "显示主界面", true, None::<&str>)?,
            &MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?,
            &MenuItem::with_id(app, "cleanup", "立即清理", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("StreamCap RS — 直播录制")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "cleanup" => {
                let st = app
                    .state::<Arc<config::AppState>>()
                    .inner()
                    .clone();
                tauri::async_runtime::spawn(async move {
                    let _ = cleanup::run_cleanup(&st).await;
                });
            }
            "quit" => {
                let app = app.clone();
                tauri::async_runtime::spawn(graceful_quit(app));
            }
            _ => {}
        })
        .build(app)?;

    // 定时刷新托盘提示（活动录制数）；避免事件监听生命周期问题，采用轮询
    let app_for_loop = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if let Some(tray) = app_for_loop.tray_by_id("main-tray") {
                let state = app_for_loop.state::<Arc<config::AppState>>();
                let active = state
                    .recordings
                    .read()
                    .iter()
                    .filter(|r| r.is_recording)
                    .count();
                let tip = if active > 0 {
                    format!("StreamCap RS — 录制中 ({})", active)
                } else {
                    "StreamCap RS — 直播录制".to_string()
                };
                let _ = tray.set_tooltip(Some(&tip));
            }
        }
    });

    Ok(())
}

/// 按已保存设置应用开机自启
fn apply_autolaunch(app: &tauri::AppHandle) {
    let state = app.state::<Arc<config::AppState>>();
    let enabled = state.settings.read().auto_launch;
    let manager = app.autolaunch();
    let _ = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
}

fn main() {
    tracing_subscriber::fmt::init();

    let app_state = config::AppState::new();
    let app_state_for_rm = app_state.clone();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
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

            // 系统托盘
            let app_handle = app.handle().clone();
            setup_tray(&app_handle)?;

            // 开机自启（按已保存设置）
            apply_autolaunch(&app_handle);

            // 窗口关闭时：最小化到托盘 或 优雅退出
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();

                        let state = app_handle.state::<Arc<config::AppState>>();
                        let minimize = state.settings.read().minimize_to_tray;

                        if minimize {
                            info!("窗口关闭请求，最小化到系统托盘");
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        } else {
                            info!("窗口关闭请求，正在停止所有录制...");
                            let app_handle = app_handle.clone();
                            tauri::async_runtime::spawn(graceful_quit(app_handle));
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
            commands::settings::get_version,
            commands::files::list_files,
            commands::files::open_file,
            commands::files::read_file_base64,
            commands::history::list_history,
            commands::history::delete_history,
            commands::postprocess::start_postprocess,
            commands::postprocess::get_postprocess,
            commands::cleanup::run_cleanup,
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building streamcap-rs");

    app.run(|app, event| match event {
        // macOS: 点击 Dock 图标（窗口已隐藏到托盘时）→ 显示并聚焦主窗口
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        // 用户显式退出（如 macOS Cmd+Q）：优雅停止录制后再退出
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
            let app = app.clone();
            tauri::async_runtime::spawn(graceful_quit(app));
        }
        _ => {}
    });
}
