//! SHIORI非同期コールバック統合テスト — チェーントーク / 複数プロパティ / トーク干渉クラスタ
//!
//! SHIORIプロトコルレベルでの非同期コールバック（get_property）機構を検証する。
//! 各テストシナリオはマルチラウンドのリクエスト・レスポンスサイクルとして構成される。
//! このファイルはチェーントーク遷移・複数プロパティマッピング・トークン退避/復元の
//! 干渉回避・タイムアウト sweep を扱う。
//!
//! 検証要件: 2.1, 2.2, 2.3, 3.1, 3.2, 5.2, 6.1, 6.2, 6.3

mod common;

#[path = "common/async_callback_support.rs"]
mod async_callback_support;

use async_callback_support::{extract_callback_id, AsyncCallbackEnv};

// ============================================================================
// Scenario 3: チェーントーク → コールバック待ち遷移（3ラウンド）
// 通常の chain talk yield → コルーチン resume → get_property → callback
// ============================================================================

/// Round 1: GET OnTestChain → 通常チェーントーク（\e付き）
/// Round 2: GET OnResumeChain → co_scene resume → get_property タグ
/// Round 3: GET OnPastaCallBack{N} → プロパティ値を含む最終レスポンス
#[test]
fn test_scenario3_chain_talk_to_callback_transition() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: Chain talk yield（通常の \e）
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestChain
"#,
    );
    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");
    let v1 = resp1.value.as_ref().expect("R1 should have Value");
    assert!(
        v1.contains("起動しました\\e"),
        "R1 should contain chain talk with \\e, got: {v1}"
    );

    // Round 2: Resume chain talk → get_property → get タグ
    let resp2 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnResumeChain
"#,
    );
    assert_eq!(resp2.status_code, 200, "R2 should return 200 OK");
    let v2 = resp2.value.as_ref().expect("R2 should have Value");
    assert!(
        v2.contains("\\![get,property,OnPastaCallBack"),
        "R2 should contain get property tag, got: {v2}"
    );
    // コールバック待ち中はプロパティ値が含まれないこと
    assert!(
        !v2.contains("ver="),
        "R2 should NOT contain resolved value (still waiting for callback), got: {v2}"
    );

    // Round 3: Callback response
    let cb_id = extract_callback_id(v2);
    let resp3 = env.request(&format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {cb_id}\r\nReference0: 2.6.77\r\n\r\n"
    ));
    assert_eq!(resp3.status_code, 200, "R3 should return 200 OK");
    let v3 = resp3.value.as_ref().expect("R3 should have Value");
    assert!(
        v3.contains("ver=2.6.77"),
        "R3 should contain property value, got: {v3}"
    );
    assert!(
        v3.contains("\\e"),
        "R3 should contain \\e (final response), got: {v3}"
    );
}

// ============================================================================
// Scenario: 複数プロパティの Reference マッピング
// get_property({name1, name2}) → Reference0, Reference1 が正しくマッピング
// ============================================================================

/// Round 1: GET OnTestMultiProp → 複数プロパティ名を含む get タグ
/// Round 2: コールバック + Reference0, Reference1 → 各プロパティ値
#[test]
fn test_multiple_property_reference_mapping() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: 複数プロパティの get_property
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestMultiProp
"#,
    );
    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");
    let v1 = resp1.value.as_ref().expect("R1 should have Value");
    assert!(
        v1.contains("width"),
        "R1 get tag should contain 'width', got: {v1}"
    );
    assert!(
        v1.contains("height"),
        "R1 get tag should contain 'height', got: {v1}"
    );

    // Round 2: コールバック（Reference0=100, Reference1=200）
    let cb_id = extract_callback_id(v1);
    let resp2 = env.request(&format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {cb_id}\r\nReference0: 100\r\nReference1: 200\r\n\r\n"
    ));
    assert_eq!(resp2.status_code, 200, "R2 should return 200 OK");
    let v2 = resp2.value.as_ref().expect("R2 should have Value");
    assert!(
        v2.contains("w=100"),
        "R2 width should be mapped from Reference0, got: {v2}"
    );
    assert!(
        v2.contains("h=200"),
        "R2 height should be mapped from Reference1, got: {v2}"
    );
}

// ============================================================================
// Scenario: 空 Reference → nil 変換
// Reference0 が空（または欠如）の場合、get_property は nil を返す
// ============================================================================

/// Round 1: GET OnTestSimple → get_property タグ
/// Round 2: コールバック（Reference0 なし）→ nil 値
#[test]
fn test_empty_reference_becomes_nil() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: OnTestSimple を送信
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestSimple
"#,
    );
    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");

    // Round 2: コールバック（Reference0 なし → nil）
    let resp2 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnPastaCallBack1
"#,
    );
    assert_eq!(resp2.status_code, 200, "R2 should return 200 OK");
    let v2 = resp2.value.as_ref().expect("R2 should have Value");
    // Reference なし → refs={} → refs[1]=nil → get_property returns nil → tostring(nil)="nil"
    assert!(
        v2.contains("version=nil"),
        "R2 empty ref should become nil, got: {v2}"
    );
}

// ============================================================================
// Scenario: タイムアウト sweep がコルーチンを解放
// timeout=0 で即時タイムアウト可能 → OnSecondChange で sweep → late callback は 204
// ============================================================================

/// Round 1: GET OnTestTimeout → get_property(timeout=0) → get タグ
/// Round 2: NOTIFY OnSecondChange → sweep がタイムアウト検知
/// Round 3: GET OnPastaCallBack{N}（遅延到着）→ 204（既に sweep 済み）
#[test]
fn test_timeout_sweep_releases_coroutine() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: get_property with timeout=0
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestTimeout
"#,
    );
    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");
    let v1 = resp1.value.as_ref().expect("R1 should have Value");
    assert!(
        v1.contains("\\![get,property,"),
        "R1 should contain get property tag, got: {v1}"
    );
    let cb_id = extract_callback_id(v1);

    // os.time() の粒度は1秒。timeout=0 → timeout_at=os.time() at stage.
    // 1秒スリープで now > timeout_at を保証
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Round 2: OnSecondChange → sweep がタイムアウトを検知
    let resp2 = env.request(
        r#"
NOTIFY SHIORI/3.0
Charset: UTF-8
ID: OnSecondChange
Reference0: 1
"#,
    );
    // sweep は 500 文字列を返すが、EVENT.fire が RES.ok() でラップするため 200 になる
    // （二重ラップは既知の設計上の挙動）
    assert_eq!(
        resp2.status_code, 200,
        "R2 sweep response should be 200 (double-wrapped)"
    );

    // Round 3: Late callback → 既に sweep で削除済み → 204
    let resp3 = env.request(&format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {cb_id}\r\nReference0: late\r\n\r\n"
    ));
    assert_eq!(
        resp3.status_code, 204,
        "Late callback after sweep should return 204, got: {}",
        resp3.status_code
    );
}

// ============================================================================
// Scenario: talk()チェーン内 get_property のトーク生成干渉回避
// トランスパイル後パターン（act.さくら:talk() + get_property）において
// トークン退避・復元が正しく動作し、最終出力が期待通りに連結されること。
// ============================================================================

/// act.さくら:talk() チェーン内で get_property を呼び出した場合、
/// 退避・復元によりプレフィックストークが失われず、
/// プロパティ値と後続テキストが正しく連結されること。
///
/// Round 1: GET OnTestTalkChainProperty → get タグのみ（トークン退避済み）
/// Round 2: コールバック → "名前は{value}です。" がさくらスクリプトとして出力
#[test]
fn test_talk_chain_property_no_interference() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: act.さくら:talk("名前は") + get_property → トークン退避
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestTalkChainProperty
"#,
    );

    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");
    let v1 = resp1.value.as_ref().expect("R1 should have Value");

    // トークン退避により get タグのみ（"名前は" は含まれない）
    assert!(
        v1.contains("\\![get,property,OnPastaCallBack"),
        "R1 should contain get property tag, got: {v1}"
    );
    assert!(
        !v1.contains("名前は"),
        "R1 should NOT contain prefix talk (tokens saved), got: {v1}"
    );

    // Round 2: コールバック送信 → トークン復元 + 最終出力
    let cb_id = extract_callback_id(v1);
    let resp2 = env.request(&format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {cb_id}\r\nReference0: テストゴースト\r\n\r\n"
    ));

    assert_eq!(resp2.status_code, 200, "R2 should return 200 OK");
    let v2 = resp2.value.as_ref().expect("R2 should have Value");

    // トークン復元により全トークが出力に含まれる
    assert!(
        v2.contains("名前は"),
        "R2 should contain restored prefix talk, got: {v2}"
    );
    assert!(
        v2.contains("テストゴースト"),
        "R2 should contain property value, got: {v2}"
    );
    assert!(
        v2.contains("です。"),
        "R2 should contain suffix talk, got: {v2}"
    );
    assert!(
        v2.contains("\\e"),
        "R2 should contain \\e (final), got: {v2}"
    );

    // さくらスクリプト出力順序の検証: "名前は" → "テストゴースト" → "です。"
    let pos_prefix = v2.find("名前は").expect("should find prefix");
    let pos_value = v2.find("テストゴースト").expect("should find value");
    let pos_suffix = v2.find("です。").expect("should find suffix");
    assert!(
        pos_prefix < pos_value && pos_value < pos_suffix,
        "Output order should be: prefix → value → suffix, got: {v2}"
    );

    // スポットタグ \p[0] が出力に含まれる（さくらアクター）
    assert!(
        v2.contains("\\p[0]"),
        "R2 should contain spot tag \\p[0] for さくら actor, got: {v2}"
    );
}

/// 複数アクター間での talk() + get_property の干渉回避テスト。
/// さくら → get_property → うにゅう の切り替えパターンで
/// スポットタグとトーク内容が正しく生成されること。
///
/// Round 1: GET OnTestMultiActorProperty → get タグのみ
/// Round 2: コールバック → さくら:talk + うにゅう:talk が正しいスポットで出力
#[test]
fn test_multi_actor_talk_chain_property_no_interference() {
    let mut env = AsyncCallbackEnv::new();

    // Round 1: act.さくら:talk("確認中...") + get_property → トークン退避
    let resp1 = env.request(
        r#"
GET SHIORI/3.0
Charset: UTF-8
ID: OnTestMultiActorProperty
"#,
    );

    assert_eq!(resp1.status_code, 200, "R1 should return 200 OK");
    let v1 = resp1.value.as_ref().expect("R1 should have Value");

    assert!(
        v1.contains("\\![get,property,OnPastaCallBack"),
        "R1 should contain get property tag, got: {v1}"
    );
    assert!(
        !v1.contains("確認中"),
        "R1 should NOT contain prefix talk (tokens saved), got: {v1}"
    );

    // Round 2: コールバック送信
    let cb_id = extract_callback_id(v1);
    let resp2 = env.request(&format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {cb_id}\r\nReference0: マイゴースト\r\n\r\n"
    ));

    assert_eq!(resp2.status_code, 200, "R2 should return 200 OK");
    let v2 = resp2.value.as_ref().expect("R2 should have Value");

    // さくらのトーク（プレフィックス）が復元されている
    assert!(
        v2.contains("確認中..."),
        "R2 should contain さくら's prefix talk, got: {v2}"
    );
    // うにゅうのトーク（プロパティ値含む）が出力されている
    assert!(
        v2.contains("名前はマイゴーストだよ。"),
        "R2 should contain うにゅう's talk with property value, got: {v2}"
    );

    // さくらスポット(\p[0]) と うにゅうスポット(\p[1]) の両方が含まれる
    assert!(
        v2.contains("\\p[0]"),
        "R2 should contain \\p[0] for さくら, got: {v2}"
    );
    assert!(
        v2.contains("\\p[1]"),
        "R2 should contain \\p[1] for うにゅう, got: {v2}"
    );

    // 出力順序: さくらのトーク → うにゅうのトーク
    let pos_sakura = v2.find("確認中...").expect("should find さくら talk");
    let pos_kero = v2
        .find("名前はマイゴーストだよ。")
        .expect("should find うにゅう talk");
    assert!(
        pos_sakura < pos_kero,
        "さくら's talk should appear before うにゅう's talk, got: {v2}"
    );

    assert!(
        v2.contains("\\e"),
        "R2 should contain \\e (final), got: {v2}"
    );
}
