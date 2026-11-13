//! 画面と Rust の唯一の接点（Tauri のコマンドの表と開いているゴーストの状態）。

use pasta_lsp::analysis::{AnalysisEngine, AnalysisResult, TOKEN_MODIFIERS, TOKEN_TYPES};

/// 画面に登録するコマンドの表（4.1 で残りを足す）
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![analyze, legend]
}

/// 画面の入力と別のスレッドで走らせ、パニックを `Err` に写す（7.7・6.5）
async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| format!("解析に失敗しました: {e}"))
}

/// 文書 1 つを解析する。値は拡張の WASM と同じ形のまま返す（13.4）
#[tauri::command]
pub async fn analyze(source: String) -> Result<AnalysisResult, String> {
    run_blocking(move || AnalysisEngine::analyze(&source)).await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Legend {
    token_types: &'static [&'static str],
    token_modifiers: &'static [&'static str],
}

/// セマンティックトークンの凡例（正本は pasta_lsp の 1 か所・6.3）
#[tauri::command]
pub async fn legend() -> Legend {
    Legend {
        token_types: TOKEN_TYPES,
        token_modifiers: TOKEN_MODIFIERS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_lsp::analysis::AnalysisEngine;
    use tauri::async_runtime::block_on;

    /// 全角文字・サロゲートペア・柵の中の Lua・構文の誤り（最上位の地の文）を含む文書
    const SOURCE: &str = "＊挨拶\n```lua\nlocal x = \"😀\"\n```\n　女の子：こんにちは😀＠単語　です。\n　・朝\n　　女の子：おはよう\n\n＊別れ\n壊れた行\n";

    #[test]
    fn analyze_json_is_the_native_serialization() {
        let native = serde_json::to_string(&AnalysisEngine::analyze(SOURCE)).unwrap();
        let app = serde_json::to_string(&block_on(analyze(SOURCE.into())).unwrap()).unwrap();
        assert_eq!(app, native);
        // 誤りの診断・サロゲートペアを含む行のトークンが実際に出ていて、欄の名前と順が拡張と同じ
        assert!(
            app.starts_with(r#"{"tokens":[{"deltaLine":0,"deltaStartCharacter":"#),
            "{app}"
        );
        assert!(
            app.contains(r#"],"diagnostics":[{"range":{"start":{"line":"#),
            "{app}"
        );
        assert!(
            app.contains(r#"],"scenes":[{"name":"挨拶","kind":"global","#),
            "{app}"
        );
    }

    #[test]
    fn panic_in_blocking_work_becomes_err() {
        let err = block_on(run_blocking(|| -> u32 { panic!("試しのパニック") })).unwrap_err();
        assert!(err.contains("解析"), "{err}");
        assert_eq!(block_on(run_blocking(|| 7)), Ok(7));
    }

    #[test]
    fn legend_is_the_rust_one() {
        let json = serde_json::to_value(block_on(legend())).unwrap();
        assert_eq!(json["tokenTypes"][8], "actorName");
        assert_eq!(json["tokenTypes"].as_array().unwrap().len(), 17);
        assert_eq!(
            json["tokenModifiers"],
            serde_json::json!(["declaration", "definition", "global"])
        );
    }
}
