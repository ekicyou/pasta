//! 起動シーケンスの致命分類に関する統合テスト。
//!
//! `main` / `pasta.shiori.entry` / `pasta.scene_dic` のロード失敗はいずれも致命であり、
//! `PastaLoader::load` が `Err` を返し、その表示文字列に「起動モジュール名」と
//! 「根本原因」の両方が含まれることを検証する
//! （要件 4.1 / 4.5 / 4.9 / 5.1 / 5.2 / 5.5）。

use crate::common;

use common::copy_fixture_to_temp;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

/// `minimal` フィクスチャの一時ゴーストを作り、`scripts/` 配下へ壊れたスクリプトを置く。
///
/// `scripts` は既定の検索パスで自己展開済み内蔵スクリプトより優先されるため、
/// ここへ置いたファイルが起動モジュールの実体を上書きする。
fn ghost_with_script(relative: &str, source: &str) -> TempDir {
    let temp = copy_fixture_to_temp("minimal");
    let path = temp.path().join("scripts").join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    temp
}

/// ロードが `Err` で終わることを確認し、その表示文字列を返す。
fn load_error_message(temp: &TempDir) -> String {
    match PastaLoader::load(temp.path()) {
        Ok(_) => panic!("起動モジュールのロード失敗は致命であるべきだが Ok が返った"),
        Err(e) => e.to_string(),
    }
}

/// 表示文字列に起動モジュール名の文脈と根本原因の両方が含まれることを検証する。
fn assert_context_and_cause(message: &str, module: &str, cause_marker: &str) {
    let context = format!("failed to load startup module '{module}'");
    assert!(
        message.contains(&context),
        "起動モジュール名の文脈 `{context}` が含まれていない: {message}"
    );
    assert!(
        message.contains(cause_marker),
        "根本原因 `{cause_marker}` が含まれていない: {message}"
    );
}

/// 要件 5.2: 利用者初期化スクリプト（`main`）の構文エラーは致命。
#[test]
fn broken_main_lua_is_fatal_with_context_and_cause() {
    // `a b` の形は LuaJIT の構文エラーとなり、メッセージに後続トークンが出る。
    let temp = ghost_with_script("main.lua", "PASTA_BROKEN_MAIN_A PASTA_BROKEN_MAIN_B\n");
    let message = load_error_message(&temp);
    assert_context_and_cause(&message, "main", "PASTA_BROKEN_MAIN_B");
}

/// 要件 4.1 / 4.5: SHIORI 応答モジュールの実行時エラーは致命。
#[test]
fn entry_runtime_error_is_fatal_with_context_and_cause() {
    let temp = ghost_with_script(
        "pasta/shiori/entry.lua",
        r#"error("PASTA_ENTRY_RUNTIME_MARKER")"#,
    );
    let message = load_error_message(&temp);
    assert_context_and_cause(&message, "pasta.shiori.entry", "PASTA_ENTRY_RUNTIME_MARKER");

    // 要件 4.7: 根本原因は複数行・スタックトレース込みで欠落なく残る。
    // `require_startup_module` の error ログの `error` フィールドはこの原因部分そのものを
    // 出力するため、ここで全文が保たれていることが確認できればログ側も同一である。
    let (context_line, cause) = message
        .split_once('\n')
        .expect("原因は 2 行目以降に続くべき");
    assert!(
        context_line.ends_with("failed to load startup module 'pasta.shiori.entry'"),
        "1 行目は起動モジュール名の文脈であるべき: {message}"
    );
    assert!(
        cause.contains("stack traceback:") && cause.lines().count() >= 3,
        "原因はスタックトレースを含む複数行として残るべき: {message}"
    );
}

/// 要件 4.9: 入れ子の `require` によるモジュール未検出も同じ経路で致命になる。
#[test]
fn entry_requiring_missing_module_is_fatal_with_context_and_cause() {
    let temp = ghost_with_script(
        "pasta/shiori/entry.lua",
        r#"require("pasta_missing_module_marker")"#,
    );
    let message = load_error_message(&temp);
    assert_context_and_cause(&message, "pasta.shiori.entry", "pasta_missing_module_marker");
}

/// 要件 5.1: シーン辞書モジュールの失敗は従来どおり致命（分類は共通ヘルパ経由）。
#[test]
fn scene_dic_failure_is_fatal_with_context_and_cause() {
    let temp = ghost_with_script("pasta/scene_dic.lua", r#"error("PASTA_SCENE_DIC_MARKER")"#);
    let message = load_error_message(&temp);
    assert_context_and_cause(&message, "pasta.scene_dic", "PASTA_SCENE_DIC_MARKER");
}

/// 正常なゴーストは従来どおりロードに成功する（致命化の巻き添えが無いことの確認）。
#[test]
fn intact_ghost_still_loads() {
    let temp = copy_fixture_to_temp("minimal");
    PastaLoader::load(temp.path()).expect("正常なゴーストのロードは成功するべき");
}
