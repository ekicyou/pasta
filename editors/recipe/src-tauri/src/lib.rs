//! pasta-recipe の Rust の殻。

pub mod check;
pub mod commands;
pub mod files;
pub mod paths;
pub mod recent;
pub mod watch;

use tauri::{Emitter, Manager};

/// 窓を作って動かす。題名は設定では版を埋められないので、ここで `pasta-recipe {版}` にする（1.6）。
pub fn run() {
    tauri::Builder::default()
        // 2 つ目の起動は窓を開かず、パスを今の窓へ送って前に出す（2.11）。最初に登録する決まり
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let path = commands::second_launch_path(&argv, &cwd);
            let _ = app.emit_to("main", "open-request", commands::OpenRequest { path });
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::default())
        .invoke_handler(commands::handler())
        .setup(|app| {
            let title = format!("pasta-recipe {}", app.package_info().version);
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&title)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("pasta-recipe の起動に失敗しました");
}
