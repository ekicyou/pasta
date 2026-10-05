//! Call の実行の正しさ（文脈の復元・選択肢のジャンプ先・動的コールの失敗）の SHIORI 経由 E2E テスト
//!
//! 仕様 `call-execution-correctness`（requirements 8.1–8.3）。
//!
//! フィクスチャ `tests/fixtures/call_execution_correctness/` の辞書を読み込み、シーンを SHIORI
//! リクエストのイベント ID（シーン関数フォールバック）で起動して、応答のさくらスクリプトと
//! ゴーストのログの警告を確かめる。フィクスチャは埋め込みの標準ランタイムと本番 entry.lua を通す
//! （理由は pasta.toml のコメント）。シーンを足すときは `dic/` に `.pasta` を置くだけでよい。

mod common;

use common::copy_fixture_to_temp;
use common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};

/// 1 つのゴーストに、順にリクエストを送り、各応答の Value とゴーストのログファイルの中身を返す。
///
/// `requests` の各要素は `GET SHIORI/3.0` 行・`Charset`・`Sender` に続くヘッダ行（`\r\n` 区切り）。
/// 例: `"ID: OnFoo"`、`"ID: OnChoiceSelectEx\r\nReference0: …"`。各応答が 200 であることを確かめる。
/// ロガーは non_blocking で書くため、ゴーストを drop して書き出しを待ってからログを読む。
fn run(requests: &[&str]) -> (Vec<String>, String) {
    let temp = copy_fixture_to_temp("call_execution_correctness");
    let values = {
        let mut shiori = PastaShiori::default();
        assert!(
            shiori
                .load(0, temp.path().as_os_str())
                .expect("SHIORI load should not error"),
            "SHIORI load should return true"
        );
        requests
            .iter()
            .map(|headers| {
                let raw = shiori
                    .request(format!(
                        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\n{headers}\r\n\r\n"
                    ))
                    .expect("request should succeed");
                let resp = ShioriResponse::parse(&raw).expect("response should parse");
                assert_eq!(
                    resp.status_code, 200,
                    "{headers} must not be 500: {} {:?}",
                    resp.status_text, resp.value
                );
                resp.value.expect("Value header must exist")
            })
            .collect()
    };
    let log = std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log"))
        .unwrap_or_default();
    (values, log)
}

/// イベント ID 1 つを送り、応答の Value とログを返す
fn fire(event_id: &str) -> (String, String) {
    let (mut values, log) = run(&[&format!("ID: {event_id}")]);
    (values.remove(0), log)
}

/// Lua 側（`@pasta_log`）が出した警告の行
fn lua_warnings(log: &str) -> Vec<&str> {
    log.lines()
        .filter(|l| l.contains(" WARN pasta_lua::runtime::log:"))
        .collect()
}

/// 土台: 最小の辞書（1 シーン 1 行）が SHIORI 経由で話し、Lua 側の警告は出ない
#[test]
fn test_minimal_scene_talks() {
    let (value, log) = fire("OnMinimal");
    assert_eq!(value, r"\p[0]こんにちは\e");
    assert!(
        log.contains("SHIORI.load called successfully"),
        "log must be captured:\n{log}"
    );
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}
