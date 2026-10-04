//! 書き間違い（U18・U19・U20・U22）と `\\` エスケープ（U08）の SHIORI 経由 E2E テスト
//!
//! 仕様 `dsl-codegen-runtime-safety` task 4.2（requirements 4.1–4.3, 5.2, 5.5）。
//!
//! フィクスチャ `tests/fixtures/codegen_runtime_safety/` のシーンを SHIORI リクエストの
//! イベント ID（シーン関数フォールバック）で起動し、500 にならず、書き間違いの箇所を既定の
//! 結果に置き換えたトークが返ることを、最終さくらスクリプトの一致で固定する。
//! フィクスチャは埋め込みの標準ランタイムと本番 entry.lua を通す（理由は pasta.toml のコメント）。

mod common;

use common::test_env::ShioriTestEnv;

/// イベント ID を指定して GET を送り、200 OK であることを確かめて Value を返す
fn fire(event_id: &str) -> String {
    let mut env = ShioriTestEnv::new("codegen_runtime_safety");
    let resp = env
        .request(&format!(
            "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nID: {event_id}\n"
        ))
        .expect("request should succeed");
    assert_eq!(
        resp.status_code, 200,
        "{event_id} must not be 500: {} {:?}",
        resp.status_text, resp.value
    );
    resp.value.expect("Value header must exist")
}

/// U18・U19・U20・U22 を含むシーンが 500 にならず、既定の結果に置き換えたトークを返す（5.2, 5.5）
#[test]
fn test_typo_scene_returns_talk_with_default_results() {
    let value = fire("OnTypoCheck");
    let expected = concat!(
        // U18: 未定義の ＠＊未定義関数（） は何も出力しない
        r"\p[1]前後",
        // U19: 未登録アクターは立ち位置 0 で、目印を付けて話す
        r"\p[0]【未登録アクター：未登録さん】こんにちは",
        // U20: act のメンバー名と同名のアクター talk が普通に話す
        r"\p[2]トークです",
        // U22: 数値にできない算術は値なし → 参照は空文字
        r"\p[1]\n[150]結果、\_w[450]終わり\e",
    );
    assert_eq!(value, expected);
}

/// U08: `C:\\new`・行末の `\\` がウェイト設定ありでも割れず、`\e` を壊さない（4.1–4.3, 5.5）
#[test]
fn test_escaped_backslash_survives_wait_insertion() {
    let value = fire("OnEscapeCheck");
    assert_eq!(
        value,
        r"\p[1]パスは、\_w[450]C:\\new。\_w[950]終わり。\_w[950]\\\e"
    );
}
