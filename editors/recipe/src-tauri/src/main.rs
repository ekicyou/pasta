#![windows_subsystem = "windows"]

const WEBVIEW2_MISSING: &str = "pasta-recipe を動かすには Microsoft Edge WebView2 ランタイムが要ります。https://developer.microsoft.com/microsoft-edge/webview2/ から入手してください。";

/// WebView2 ランタイムの確かめの結果から、出して終わるべきメッセージを決める（1.4）。
/// 版が空の文字列で返る場合も「無い」と扱う。
fn webview2_missing_message<E>(version: &Result<String, E>) -> Option<&'static str> {
    match version {
        Ok(v) if !v.is_empty() => None,
        _ => Some(WEBVIEW2_MISSING),
    }
}

fn main() {
    if let Some(msg) = webview2_missing_message(&tauri::webview_version()) {
        show_error(msg);
        std::process::exit(1);
    }
    pasta_recipe::run()
}

/// 窓を作る前なので、OS のメッセージ箱で知らせる。
fn show_error(msg: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (text, caption) = (wide(msg), wide("pasta-recipe"));
    // SAFETY: どちらも NUL で終わる UTF-16 の列で、呼び出しの間生きている
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview2_present_starts_normally() {
        let found: Result<String, ()> = Ok("131.0.2903.70".into());
        assert_eq!(webview2_missing_message(&found), None);
    }

    #[test]
    fn webview2_missing_tells_where_to_get_it() {
        let missing: Result<String, ()> = Err(());
        let msg = webview2_missing_message(&missing).expect("無いときはメッセージを出す");
        assert!(msg.contains("WebView2"), "{msg}");
        assert!(
            msg.contains("https://developer.microsoft.com/microsoft-edge/webview2/"),
            "{msg}"
        );
    }

    #[test]
    fn webview2_empty_version_counts_as_missing() {
        let empty: Result<String, ()> = Ok(String::new());
        assert_eq!(webview2_missing_message(&empty), Some(WEBVIEW2_MISSING));
    }
}
