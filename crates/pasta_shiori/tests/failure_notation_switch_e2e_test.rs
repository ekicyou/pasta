//! `[logging] balloon_warning` で失敗表記の有無だけが切り替わることの SHIORI 経由 E2E テスト
//!
//! 仕様 `failure-output-unification` task 4.1（requirements 6.4, 7.1–7.5）。
//!
//! フィクスチャ `tests/fixtures/failure_notation_switch/` の pasta.toml の末尾に
//! `[logging] balloon_warning` を書き足して（書かない・false・型違い）同じシーンを起動し、
//! 出さない設定の出力が出す設定の出力から表記（`【…】`）を除いたものと一致し、
//! ログの警告が同じで、表記しか出ないシーンが 204 になることを確かめる。

mod common;

use common::copy_fixture_to_temp;
use common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};

/// `balloon_warning` の行（None なら書かない）を足したゴーストでイベントを起こし、
/// 応答と Lua 側のログの警告（文言だけ）を返す。
/// ロガーは non_blocking で書くため、ゴーストを drop して書き出しを待ってからログを読む。
fn fire(event_id: &str, balloon_warning: Option<&str>) -> (ShioriResponse, Vec<String>) {
    let temp = copy_fixture_to_temp("failure_notation_switch");
    if let Some(value) = balloon_warning {
        let path = temp.path().join("pasta.toml");
        let mut toml = std::fs::read_to_string(&path).unwrap();
        toml.push_str(&format!("\n[logging]\nballoon_warning = {value}\n"));
        std::fs::write(&path, toml).unwrap();
    }
    let resp = {
        let mut shiori = PastaShiori::default();
        assert!(
            shiori
                .load(0, temp.path().as_os_str())
                .expect("SHIORI load should not error"),
            "SHIORI load should return true"
        );
        let raw = shiori
            .request(format!(
                "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nID: {event_id}\r\n\r\n"
            ))
            .expect("request should succeed");
        ShioriResponse::parse(&raw).expect("response should parse")
    };
    let log = std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log"))
        .unwrap_or_default();
    assert!(
        log.contains("SHIORI.load called successfully"),
        "log must be captured:\n{log}"
    );
    let warns = log
        .lines()
        .filter_map(|l| l.split_once(" WARN pasta_lua::runtime::log: "))
        // 呼び出し元の位置（一時ディレクトリのパスを含む）は除く
        .map(|(_, rest)| rest.split(" lua_source=").next().unwrap().to_string())
        .collect();
    (resp, warns)
}

/// 200 OK の Value を返す
fn value(resp: ShioriResponse) -> String {
    assert_eq!(
        resp.status_code, 200,
        "{} {:?}",
        resp.status_text, resp.value
    );
    resp.value.expect("Value header must exist")
}

/// `【…】` をすべて取り除く
fn strip_notations(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '【' => depth += 1,
            '】' => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// 出さない設定の出力は、出す設定の出力から表記を除いたものと同じで、ログの警告は同じ（6.4・7.3・7.4）
#[test]
fn test_disabled_output_equals_enabled_without_notations() {
    let (on, on_warns) = fire("OnFailureMix", None);
    let (off, off_warns) = fire("OnFailureMix", Some("false"));
    let on = value(on);
    let off = value(off);
    // 関数・変数・未登録アクター・数にできない値の 4 つの表記が出る
    assert_eq!(on.matches('【').count(), 4, "{on}");
    assert!(!off.contains('【'), "{off}");
    assert_eq!(off, strip_notations(&on));
    assert_eq!(on_warns.len(), 4, "{on_warns:#?}");
    assert_eq!(off_warns, on_warns);
}

/// 型違いの値は書かれていないときと同じ（表記を出す）（7.2・7.5）
#[test]
fn test_mistyped_value_falls_back_to_showing() {
    let (on, _) = fire("OnFailureMix", None);
    let (mistyped, _) = fire("OnFailureMix", Some("\"no\""));
    assert_eq!(value(mistyped), value(on));
}

/// 表記しか出ないシーン（未登録アクターの値の無い変数だけ）は、出さない設定で 204（7.3）
#[test]
fn test_notation_only_scene_is_204_when_disabled() {
    let (on, _) = fire("OnFailureOnly", None);
    assert!(value(on).contains("【未登録アクター：ぱすたー】"));
    let (off, warns) = fire("OnFailureOnly", Some("false"));
    assert_eq!(off.status_code, 204, "{} {:?}", off.status_text, off.value);
    assert_eq!(warns.len(), 2, "{warns:#?}");
}
