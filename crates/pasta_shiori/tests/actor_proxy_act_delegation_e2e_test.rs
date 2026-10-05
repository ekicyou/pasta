//! アクション行から呼ぶ組み込み関数（`yield`・`チェイントーク`・`ゴースト終了`）の SHIORI 経由 E2E テスト
//!
//! 仕様 `actor-proxy-act-delegation` task 1.3（requirements 1.1–1.7, 4.1, 4.2, 4.4）。
//!
//! フィクスチャ `tests/fixtures/actor_proxy_act_delegation/` のシーンを SHIORI リクエストの
//! イベント ID（シーン関数フォールバック）で起動し、500 にならないこと、継続トークの残りが
//! 次の OnTalk の機会に出ること、`\-` が行の外の `＞ゴースト終了` と同じ形で出ることを固定する。
//! 各シーンは登録済みのアクター（さくら）と未登録のアクター（未登録さん）の両方の行で用意する（1.7）。
//! フィクスチャは埋め込みの標準ランタイムと本番 entry.lua を通す（理由は pasta.toml のコメント）。

mod common;

use common::response::ShioriResponse;
use common::test_env::ShioriTestEnv;

const FIXTURE: &str = "actor_proxy_act_delegation";

/// イベント ID を指定して GET を送る
fn get(env: &mut ShioriTestEnv, event_id: &str) -> ShioriResponse {
    env.request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nID: {event_id}\n"
    ))
    .expect("request should succeed")
}

/// X-Pasta-Time を固定した OnSecondChange（仮想ディスパッチャ経由の OnTalk 判定）を送る
fn on_second_change(env: &mut ShioriTestEnv, time: &str) -> ShioriResponse {
    env.request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nStatus: idle\nID: OnSecondChange\nReference0: 1\nX-Pasta-Time: {time}\n"
    ))
    .expect("request should succeed")
}

/// 200 OK であることを確かめて Value を返す
fn ok_value(resp: ShioriResponse, what: &str) -> String {
    assert_eq!(
        resp.status_code,
        200,
        "{what} must not be 500: {} {:?} reason={:?}",
        resp.status_text,
        resp.value,
        resp.header("X-Error-Reason")
    );
    resp.value.expect("Value header must exist")
}

/// 新しい環境でイベントを 1 回起動し、200 の Value を返す
fn fire(event_id: &str) -> String {
    let mut env = ShioriTestEnv::new(FIXTURE);
    let resp = get(&mut env, event_id);
    ok_value(resp, event_id)
}

/// `前＠…後` のシーン：最初の応答は「前」まで、「後」は次の OnTalk の機会の応答に出る（1.1, 1.5, 1.6）
fn assert_yield_continues(event_id: &str) {
    let mut env = ShioriTestEnv::new(FIXTURE);

    // 次回トーク時刻を 12:05:10 に設定（talk 間隔はフィクスチャで 10 秒に固定）
    let init = on_second_change(&mut env, "2025-07-15T12:05:00Z");
    assert_eq!(init.status_code, 204, "{event_id}: init OnSecondChange");

    let first = ok_value(get(&mut env, event_id), event_id);
    assert!(first.contains("前"), "{event_id}: first={first}");
    assert!(!first.contains("後"), "{event_id}: first={first}");
    assert!(!first.contains("table:"), "{event_id}: first={first}");

    // 次の OnTalk の機会 → 中断点の後ろ（同じ行の「後」）が出る
    let resumed = on_second_change(&mut env, "2025-07-15T12:05:10Z");
    let resumed = ok_value(resumed, &format!("{event_id} resumed"));
    assert!(resumed.contains("後"), "{event_id}: resumed={resumed}");
    assert!(!resumed.contains("前"), "{event_id}: resumed={resumed}");
    assert!(!resumed.contains("table:"), "{event_id}: resumed={resumed}");
}

/// アクション行のゴースト終了が `\-` を含み、行の外の `＞ゴースト終了` のシーンと同じ出力になる（1.2–1.6）
fn assert_close_same_as_outside(event_id: &str, outside_id: &str) {
    let value = fire(event_id);
    assert!(value.contains(r"\-"), "{event_id}: {value}");
    assert!(!value.contains("table:"), "{event_id}: {value}");
    assert_eq!(value, fire(outside_id), "{event_id} vs {outside_id}");
}

/// 比較に使う行の外のシーンそのもの：ミリ秒なしは `\-`、ミリ秒ありは待ちに続く `\-` を出す
/// （修正前の実装でも通る比較基準）
#[test]
fn test_outside_close_ghost_is_the_baseline() {
    for (id, tail) in [
        ("OnCloseOutsidePlain", r"\-"),
        ("OnCloseOutsideMs", r"\_w[500]\-"),
        ("OnUnregCloseOutsidePlain", r"\-"),
        ("OnUnregCloseOutsideMs", r"\_w[500]\-"),
    ] {
        let value = fire(id);
        assert!(value.contains(tail), "{id}: {value}");
    }
}

/// シーンごとに 1 テストを作る（修正前の実装でどのシーンが落ちるかを個別に見られるようにする）
macro_rules! yield_tests {
    ($($name:ident => $id:literal,)*) => {
        $( #[test] fn $name() { assert_yield_continues($id); } )*
    };
}

macro_rules! close_tests {
    ($($name:ident => $id:literal vs $outside:literal,)*) => {
        $( #[test] fn $name() { assert_close_same_as_outside($id, $outside); } )*
    };
}

// ＠yield・＠チェイントーク（単語参照と関数呼び出しの形）と、値が yield の ＠＄x・＠＄x（）（1.1, 1.4, 1.7, 4.1, 4.2）
yield_tests! {
    test_yield_word => "OnYieldWord",
    test_chain_word => "OnChainWord",
    test_yield_call => "OnYieldCall",
    test_chain_call => "OnChainCall",
    test_yield_var => "OnYieldVarWord",
    test_yield_var_call => "OnYieldVarCall",
    test_unreg_yield_word => "OnUnregYieldWord",
    test_unreg_chain_word => "OnUnregChainWord",
    test_unreg_yield_call => "OnUnregYieldCall",
    test_unreg_chain_call => "OnUnregChainCall",
    test_unreg_yield_var => "OnUnregYieldVarWord",
    test_unreg_yield_var_call => "OnUnregYieldVarCall",
}

// ＠ゴースト終了・＠ゴースト終了（）・＠ゴースト終了（500）と、値がゴースト終了の ＠＄x・＠＄x（…）
// （1.2, 1.3, 1.4, 1.7, 4.1, 4.2）
close_tests! {
    test_close_word => "OnCloseWord" vs "OnCloseOutsidePlain",
    test_close_call => "OnCloseCallPlain" vs "OnCloseOutsidePlain",
    test_close_call_ms => "OnCloseCallMs" vs "OnCloseOutsideMs",
    test_close_var => "OnCloseVarWord" vs "OnCloseOutsidePlain",
    test_close_var_call => "OnCloseVarCallPlain" vs "OnCloseOutsidePlain",
    test_close_var_call_ms => "OnCloseVarCallMs" vs "OnCloseOutsideMs",
    test_unreg_close_word => "OnUnregCloseWord" vs "OnUnregCloseOutsidePlain",
    test_unreg_close_call => "OnUnregCloseCallPlain" vs "OnUnregCloseOutsidePlain",
    test_unreg_close_call_ms => "OnUnregCloseCallMs" vs "OnUnregCloseOutsideMs",
    test_unreg_close_var => "OnUnregCloseVarWord" vs "OnUnregCloseOutsidePlain",
    test_unreg_close_var_call => "OnUnregCloseVarCallPlain" vs "OnUnregCloseOutsidePlain",
    test_unreg_close_var_call_ms => "OnUnregCloseVarCallMs" vs "OnUnregCloseOutsideMs",
}
