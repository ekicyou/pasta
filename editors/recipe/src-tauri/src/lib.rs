//! pasta-recipe の Rust の殻。

pub mod check;
pub mod commands;
pub mod files;
pub mod paths;
pub mod recent;
pub mod watch;

use tauri::Manager;

/// 窓を作って動かす。題名は設定では版を埋められないので、ここで `pasta-recipe {版}` にする（1.6）。
pub fn run() {
    tauri::Builder::default()
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
