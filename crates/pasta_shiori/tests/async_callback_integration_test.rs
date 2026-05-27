//! SHIORI非同期コールバック統合テスト
//!
//! SHIORIプロトコルレベルでの非同期コールバック（get_property）機構を検証する。
//! 各テストシナリオはマルチラウンドのリクエスト・レスポンスサイクルとして構成される。
//!
//! 検証要件: 2.1, 2.2, 2.3, 3.1, 3.2, 5.2, 6.1, 6.2, 6.3

mod common;

use common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// テスト環境ヘルパー
// ============================================================================

/// 非同期コールバックテスト用の環境構築
///
/// `copy_fixture_to_temp` でサポート＋フィクスチャをコピーした後、
/// production の `pasta_scripts/` を `pasta_lua` クレートからコピーする。
/// これにより EVENT, CALLBACK, SHIORI_ACT 等の本番モジュールが利用可能になる。
struct AsyncCallbackEnv {
    shiori: PastaShiori,
    _temp_dir: TempDir,
}

impl AsyncCallbackEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // 1. Copy production pasta_scripts/ from pasta_lua crate
        //    These provide EVENT, CALLBACK, SHIORI_ACT, etc.
        let pasta_scripts_src = manifest_dir
            .parent()
            .expect("parent dir")
            .join("pasta_lua")
            .join("pasta_scripts");
        let pasta_scripts_dst = temp_dir.path().join("pasta_scripts");
        std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
        copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

        // 2. Copy fixture files (entry.lua override + pasta.toml)
        //    These go into scripts/ which takes priority over pasta_scripts/
        let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
        copy_dir_recursive(&fixture_src, temp_dir.path()).expect("copy fixture");

        let mut shiori = PastaShiori::default();
        let loaded = shiori
            .load(0, temp_dir.path().as_os_str())
            .expect("SHIORI load should not error");
        assert!(loaded, "SHIORI load should return true");

        Self {
            shiori,
            _temp_dir: temp_dir,
        }
    }

    fn request(&mut self, text: &str) -> ShioriResponse {
        let normalized = normalize_request(text);
        let raw = self
            .shiori
            .request(&normalized)
            .expect("SHIORI request should not error");
        ShioriResponse::parse(&raw).expect("Response should be parseable")
    }
}

fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut result = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    result.push_str("\r\n\r\n");
    result
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

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

// ============================================================================
// ヘルパー関数
// ============================================================================

/// Value 文字列から OnPastaCallBack{N} のイベントIDを抽出する
fn extract_callback_id(value: &str) -> String {
    let start = value
        .find("OnPastaCallBack")
        .expect("Value should contain OnPastaCallBack");
    let rest = &value[start..];
    let end = rest
        .find(|c: char| c == ',' || c == ']')
        .expect("OnPastaCallBack should be followed by ',' or ']'");
    rest[..end].to_string()
}

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
