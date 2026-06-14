//! SHIORI非同期コールバック統合テスト — 単発シナリオ / プロトコル準拠クラスタ
//!
//! SHIORIプロトコルレベルでの非同期コールバック（get_property）機構を検証する。
//! 各テストシナリオはマルチラウンドのリクエスト・レスポンスサイクルとして構成される。
//! このファイルは単発の get → callback シナリオとコールバックID/プロトコル準拠を扱う。
//!
//! 検証要件: 2.1, 2.2, 2.3, 3.1, 3.2, 5.2, 6.1, 6.2, 6.3

mod common;

#[path = "common/async_callback_support.rs"]
mod async_callback_support;

use async_callback_support::AsyncCallbackEnv;

// ============================================================================
// Scenario 1: シンプルなプロパティ取得（2ラウンド）
// Req 2.1: get_propertyがコルーチン内で呼び出し可能
// Req 2.2: コールバックイベントIDが\![get,property,...]に含まれる
// Req 2.3: コールバック到着時にコルーチンがresumeされ値が返る
// Req 6.1: get_propertyが\![get,property,{id},{name}]タグを生成
// ============================================================================

/// Round 1: GET OnTestSimple → get_property タグを含むレスポンス
/// Round 2: GET OnPastaCallBack1 + Reference0 → プロパティ値を含む最終レスポンス
#[test]
fn test_scenario1_simple_property_get() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: OnTestSimple を送信
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestSimple
"#,
    );

    assert_eq!(resp1.status_code, 200, "Round 1 should return 200 OK");

    let value1 = resp1.value.as_ref().expect("Round 1 should have Value");
    // Req 6.1: get タグが含まれること
    assert!(
        value1.contains("\\![get,property,OnPastaCallBack"),
        "Round 1 Value should contain get property tag, got: {value1}"
    );
    // Req 6.2: baseware.version がタグに含まれること
    assert!(
        value1.contains("baseware.version"),
        "Round 1 Value should contain property name, got: {value1}"
    );
    // コールバック待ち中はプロパティ値が含まれないこと（まだ解決していない）
    assert!(
        !value1.contains("version="),
        "Round 1 should NOT contain resolved value (still waiting for callback), got: {value1}"
    );

    // Round 2: コールバックを送信（Reference0 にプロパティ値を設定）
    let resp2 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
Reference0: 2.6.77
"#,
    );

    assert_eq!(resp2.status_code, 200, "Round 2 should return 200 OK");

    let value2 = resp2.value.as_ref().expect("Round 2 should have Value");
    // Req 2.3: コルーチンがresumeされプロパティ値が反映される
    assert!(
        value2.contains("version=2.6.77"),
        "Round 2 Value should contain property value, got: {value2}"
    );
    // 最終レスポンスは \\e を含む
    assert!(
        value2.contains("\\e"),
        "Round 2 Value should contain \\e (final response), got: {value2}"
    );
}

// ============================================================================
// Scenario 2: トーク蓄積 + プロパティ取得
// get_property のトークン退避・復元により、get タグのみが先行 yield され、
// 蓄積トークンはコールバック後の最終出力に含まれる。
// ============================================================================

/// Round 1: GET OnTestAccumulate → get タグのみ（トークンは退避済み）
/// Round 2: コールバック → 蓄積トーク + 最終トーク
#[test]
fn test_scenario2_talk_accumulation_with_property_get() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: OnTestAccumulate を送信
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestAccumulate
"#,
    );

    assert_eq!(resp1.status_code, 200, "Round 1 should return 200 OK");

    let value1 = resp1.value.as_ref().expect("Round 1 should have Value");
    // トークン退避により、get タグのみがyieldされる（プレフィックストークは含まれない）
    assert!(
        !value1.contains("\\p[0]checking..."),
        "Round 1 should NOT contain prefix talk (tokens saved), got: {value1}"
    );
    assert!(
        value1.contains("\\![get,property,OnPastaCallBack"),
        "Round 1 should contain get property tag, got: {value1}"
    );

    // Round 2: コールバック
    let resp2 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
Reference0: 3.14
"#,
    );

    assert_eq!(resp2.status_code, 200, "Round 2 should return 200 OK");

    let value2 = resp2.value.as_ref().expect("Round 2 should have Value");
    // トークン復元により、プレフィックストーク + 最終トークが同じレスポンスに含まれる
    assert!(
        value2.contains("checking..."),
        "Round 2 should contain restored prefix talk, got: {value2}"
    );
    assert!(
        value2.contains("result=3.14"),
        "Round 2 should contain property value result, got: {value2}"
    );
    assert!(
        value2.contains("\\e"),
        "Round 2 should contain \\e (final), got: {value2}"
    );
}

// ============================================================================
// Scenario 4: コールバック待ち中の無関係イベント
// Req 3.1: コールバック待ち中に他のイベントが処理可能
// Req 3.2: 無関係イベントがコールバック状態に影響しない
// ============================================================================

/// Round 1: GET OnTestWait → get_property タグ（コールバック待ち）
/// Round 2: NOTIFY OnUnrelated → 204 No Content（コールバックに影響しない）
/// Round 3: GET OnPastaCallBack1 → プロパティ値を含む最終レスポンス
#[test]
fn test_scenario4_unrelated_event_during_callback_wait() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: OnTestWait を送信 → コールバック待ちに入る
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestWait
"#,
    );

    assert_eq!(resp1.status_code, 200, "Round 1 should return 200 OK");

    let value1 = resp1.value.as_ref().expect("Round 1 should have Value");
    assert!(
        value1.contains("\\![get,property,OnPastaCallBack"),
        "Round 1 should contain get property tag, got: {value1}"
    );

    // Round 2: 無関係イベントを送信（ハンドラ未登録 → 204）
    let resp2 = env.request(
        r#"
NOTIFY SHIORI/3.0
Charset: UTF-8
ID: OnUnrelatedEvent
"#,
    );

    // Req 3.1: 無関係イベントは正常に処理される（204 No Content）
    assert_eq!(
        resp2.status_code, 204,
        "Round 2 unrelated event should return 204, got: {}",
        resp2.status_code
    );

    // Round 3: コールバックを送信 → ペンディング状態が維持されている
    let resp3 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
Reference0: 9.99
"#,
    );

    // Req 3.2: コールバックが正常に処理される
    assert_eq!(resp3.status_code, 200, "Round 3 should return 200 OK");

    let value3 = resp3.value.as_ref().expect("Round 3 should have Value");
    assert!(
        value3.contains("waited=9.99"),
        "Round 3 should contain property value, got: {value3}"
    );
    assert!(
        value3.contains("\\e"),
        "Round 3 should contain \\e (final), got: {value3}"
    );
}

// ============================================================================
// Scenario: コールバックIDの一意性（連続呼び出し）
// Req 2.2: コールバックIDが一意に生成される
// ============================================================================

/// 連続で2つのget_property呼び出しが異なるコールバックIDを生成すること
#[test]
fn test_callback_id_uniqueness() {
    let mut env = AsyncCallbackEnv::new();

    // 1つ目のget_property
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestSimple
"#,
    );

    let value1 = resp1.value.as_ref().expect("should have Value");
    assert!(
        value1.contains("OnPastaCallBack1"),
        "First callback should be OnPastaCallBack1, got: {value1}"
    );

    // コールバックを完了させる
    let _ = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
Reference0: v1
"#,
    );

    // 2つ目のget_property（新しいコールバックID）
    let resp3 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestSimple
"#,
    );

    let value3 = resp3.value.as_ref().expect("should have Value");
    // Req 2.2: 2つ目は異なるIDになる
    assert!(
        value3.contains("OnPastaCallBack2"),
        "Second callback should be OnPastaCallBack2, got: {value3}"
    );
}

// ============================================================================
// Scenario: 無効なコールバックID（ルーティングされない）
// Req 3.1: 存在しないコールバックIDは通常イベントとして処理
// ============================================================================

/// 存在しないコールバックIDにリクエストを送ると、通常のイベント処理になる
#[test]
fn test_invalid_callback_id_not_routed() {
    let mut env = AsyncCallbackEnv::new();

    // 存在しないコールバックID → REGにハンドラなし → 204
    let resp = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack999
Reference0: bogus
"#,
    );

    assert_eq!(
        resp.status_code, 204,
        "Non-existent callback should return 204, got: {}",
        resp.status_code
    );
}

// ============================================================================
// Scenario: コールバックレスポンスのプロトコル準拠
// Req 6.1, 6.2: レスポンスヘッダーが正しいこと
// ============================================================================

/// コールバックレスポンスに標準ヘッダーが含まれること
#[test]
fn test_callback_response_protocol_compliance() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: get_property
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestSimple
"#,
    );

    // プロトコルヘッダー検証（Round 1）
    assert_eq!(resp1.status_code, 200);
    assert!(
        resp1.header("Charset").is_some(),
        "Response should have Charset header"
    );
    assert!(
        resp1.header("Sender").is_some(),
        "Response should have Sender header"
    );

    // Round 2: コールバック
    let resp2 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
Reference0: test_value
"#,
    );

    // プロトコルヘッダー検証（Round 2 - コールバックレスポンス）
    assert_eq!(resp2.status_code, 200);
    assert!(
        resp2.header("Charset").is_some(),
        "Callback response should have Charset header"
    );
    assert!(
        resp2.header("Sender").is_some(),
        "Callback response should have Sender header"
    );
    assert!(
        resp2.value.is_some(),
        "Callback response should have Value header"
    );
}
