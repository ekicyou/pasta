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

/// 起動失敗 error ログを捕捉するためのテストローカルな書き込み先。
struct CaptureWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for CaptureWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self {
        CaptureWriter(std::sync::Arc::clone(&self.0))
    }
}

/// 要件 4.7 / 5.3: 起動モジュールの失敗が構造化フィールド付き error ログとして
/// 欠落なく記録されることを固定する。
///
/// `book/src/reference/startup.md` の切り分け手順は「ログを `fatal=true` で検索する」
/// ことを利用者に案内しているため、フィールド名と描画表記そのものが契約である。
/// フィールドのリネームや `error` の単一行化が起きた場合、このテストが落ちる。
///
/// 捕捉に `tracing-test` の `logs_contain` を使わないのは、あれが捕捉バッファを行単位に
/// 分割し「スパン名（= テスト関数名）を含む行」だけを走査するためで、複数行フィールドの
/// 2 行目以降（`stack traceback:` 以下）が観測できず要件 4.7 の「欠落なく」を
/// 検証できないことを実測で確認したため。ここではテストスレッドにだけ装着する
/// fmt subscriber で生の描画結果を捕捉する。
/// （`X-ERROR-REASON` は意図的に単一行である点と対照的であることに注意。）
#[test]
fn startup_failure_log_carries_module_fatal_and_multiline_cause() {
    let temp = ghost_with_script(
        "pasta/shiori/entry.lua",
        r#"error("PASTA_LOG_CONTRACT_MARKER")"#,
    );

    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(CaptureWriter(std::sync::Arc::clone(&buffer)))
        .with_ansi(false)
        .with_max_level(tracing::Level::ERROR)
        .finish();
    // `with_default` はカレントスレッドにのみ subscriber を装着するため、
    // 他テストのグローバル subscriber と干渉しない。
    let result = tracing::subscriber::with_default(subscriber, || PastaLoader::load(temp.path()));
    assert!(result.is_err(), "起動モジュールのロード失敗は致命であるべき");

    let logged = String::from_utf8(buffer.lock().unwrap().clone()).expect("ログは UTF-8");

    // 要件 5.3: 失敗した起動モジュール名が `module` フィールドとして識別できる。
    assert!(
        logged.contains(r#"module="pasta.shiori.entry""#),
        "`module` フィールドに起動モジュール名が無い: {logged}"
    );
    // 要件 5.3: 致命かどうかが `fatal` フィールドで識別できる（マニュアルの grep 対象）。
    // フィールド区切りの空白込みで照合する。空白を外すと `is_fatal=true` のような
    // 別名フィールドでも通ってしまい、リネームを検出できない。
    assert!(
        logged.contains(" fatal=true"),
        "`fatal=true` フィールドが無い: {logged}"
    );

    // 要件 4.7: `error` フィールドは根本原因を複数行・スタックトレース込みで保持する。
    let cause = logged
        .split_once("error=")
        .expect("`error` フィールドが無い")
        .1;
    assert!(
        cause.contains("PASTA_LOG_CONTRACT_MARKER"),
        "`error` フィールドに根本原因が無い: {logged}"
    );
    assert!(
        cause.contains("stack traceback:") && cause.lines().count() >= 3,
        "`error` フィールドが単一行化・切り詰められている: {logged}"
    );
}
