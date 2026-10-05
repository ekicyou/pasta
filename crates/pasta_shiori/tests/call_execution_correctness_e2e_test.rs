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

// ---------------------------------------------------------------------------
// 特性化テスト（tasks 1.2）: 変更前のコードで成功し、以後の変更で落ちないことの基準
// 辞書: dic/characterization.pasta
// ---------------------------------------------------------------------------

/// 4.1: 末尾の Call（動的コール `＞＠次（）`、Lua のカウンタ）だけで 10 万回つないでも
/// 深さの上限によるエラーにならず、最後のシーンまで実行される
#[test]
fn test_tail_call_chain_100k_completes() {
    let (value, log) = fire("OnTailChain");
    assert_eq!(value, r"\p[0]完走\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 4.4: 同じグローバルシーンの配下のローカルシーンを Call したときの応答（途中の Call と
/// 末尾の Call）。前方一致するグローバルシーンよりローカルシーンが選ばれる
#[test]
fn test_call_local_scene_in_same_global() {
    let (value, log) = fire("OnSameGlobalCall");
    assert_eq!(value, r"\p[0]［ローカル］締め\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 4.2: 末尾の Call で別のグローバルシーンへ移った後は、移った先のシーンの文脈で解決される
/// （単語参照は移った先のローカル単語、Call は移った先の配下のローカルシーン）
#[test]
fn test_tail_call_to_other_global_resolves_in_callee_context() {
    let (value, log) = fire("OnTailToOther");
    assert_eq!(value, r"\p[0]元先の単語先の後半\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 2.6: 別のグローバルシーンへの途中の Call を含まないシーンが出した選択肢を、既定の自動
/// ルーティングで選んだときのジャンプ先。出したシーンの配下のローカルシーンが先に選ばれ、
/// 無ければグローバルシーンへフォールバックする。`\q` の文字列そのものは確かめない（2.4 で変わる）
#[test]
fn test_choice_routing_without_mid_call() {
    let select = |title: &str, id: &str| {
        format!("ID: OnChoiceSelectEx\r\nReference0: {title}\r\nReference1: {id}")
    };
    for (title, id, expected) in [
        ("甲", "選択肢甲", r"\p[0]メニューの甲\e"),
        ("乙", "選択肢乙", r"\p[0]グローバルの乙\e"),
    ] {
        let (values, log) = run(&["ID: OnChoiceMenu", &select(title, id)]);
        assert!(values[0].contains("どれ？"), "menu: {}", values[0]);
        assert_eq!(values[1], expected, "choice {id}");
        assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
    }
}

/// 4.8: ローカルシーンの最後の行が Call のとき、呼ばれた側の戻り値がそのシーンの戻り値になる。
/// Lua の関数が `act:call` でローカルシーンを呼び、その戻り値をアクション行で出力する
#[test]
fn test_tail_call_return_value_propagates() {
    let (value, log) = fire("OnTailReturn");
    assert_eq!(value, r"\p[0]末尾の戻り値\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}
