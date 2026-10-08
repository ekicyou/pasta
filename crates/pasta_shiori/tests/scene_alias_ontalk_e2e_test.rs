//! `＊会話` だけのゴーストがランダムトークを発行する SHIORI E2E テスト
//!
//! 仕様 `scene-name-alias` task 6.4（requirements 1.2, 1.5, 5.6, 10.2）。
//!
//! フィクスチャ `tests/fixtures/scene_alias_ontalk/` は `＊会話` を 2 つだけ持ち `＊OnTalk` を持たない。
//! SHIORI として読み込み、OnBoot の後に X-Pasta-Time で時刻を固定した OnSecondChange を送って
//! 仮想ディスパッチャの次回トーク時刻を進め、OnTalk の機会に `＊会話` の台詞が出ることを確かめる。
//! 同じゴーストの pasta.toml に空の `[scene.alias]` を書き足した版では OnTalk が発行されない。

mod common;

use common::copy_fixture_to_temp;
use common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};
use tempfile::TempDir;

const FIXTURE: &str = "scene_alias_ontalk";
const LINES: [&str; 2] = ["ひとつめの雑談だよ。", "ふたつめの雑談だよ。"];

/// フィクスチャを一時ディレクトリへコピーし、必要なら pasta.toml に追記してから読み込む
fn load(extra_toml: &str) -> (PastaShiori, TempDir) {
    let temp = copy_fixture_to_temp(FIXTURE);
    if !extra_toml.is_empty() {
        let toml = temp.path().join("pasta.toml");
        let mut text = std::fs::read_to_string(&toml).unwrap();
        text.push_str(extra_toml);
        std::fs::write(&toml, text).unwrap();
    }
    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, temp.path().as_os_str()).unwrap());
    (shiori, temp)
}

fn request(shiori: &mut PastaShiori, headers: &str) -> ShioriResponse {
    let raw = shiori
        .request(format!(
            "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\n{headers}\r\n"
        ))
        .expect("request should succeed");
    ShioriResponse::parse(&raw).expect("response should parse")
}

fn on_boot(shiori: &mut PastaShiori) -> ShioriResponse {
    request(shiori, "ID: OnBoot\r\nReference0: master\r\n")
}

/// X-Pasta-Time を固定した OnSecondChange（仮想ディスパッチャ経由の OnTalk 判定）
fn on_second_change(shiori: &mut PastaShiori, time: &str) -> ShioriResponse {
    request(
        shiori,
        &format!("Status: idle\r\nID: OnSecondChange\r\nReference0: 1\r\nX-Pasta-Time: {time}\r\n"),
    )
}

/// 既定の別名表：`＊会話` の 2 シーンが OnTalk の候補として順に発行される（1.2, 10.2）。
/// 発行条件は従来どおり（初回は次回トーク時刻の設定だけ・間隔未到達では出ない。1.5）。
#[test]
fn default_alias_table_issues_kaiwa_as_random_talk() {
    let (mut shiori, _temp) = load("");
    on_boot(&mut shiori);

    // 初回：次回トーク時刻（12:05:10）を決めるだけで発行しない
    let init = on_second_change(&mut shiori, "2025-07-15T12:05:00Z");
    assert_eq!(init.status_code, 204, "init: {:?}", init.value);
    // 間隔未到達：発行しない
    let early = on_second_change(&mut shiori, "2025-07-15T12:05:05Z");
    assert_eq!(early.status_code, 204, "early: {:?}", early.value);

    // 2 回の OnTalk の機会で、2 つの `＊会話` がシャッフル＆順次消費で 1 つずつ出る
    let mut seen = Vec::new();
    for time in ["2025-07-15T12:05:10Z", "2025-07-15T12:05:20Z"] {
        let resp = on_second_change(&mut shiori, time);
        assert_eq!(resp.status_code, 200, "{time}: {:?}", resp.value);
        let value = resp.value.expect("Value header must exist");
        let line = LINES
            .iter()
            .find(|l| value.contains(**l))
            .unwrap_or_else(|| panic!("{time}: no ＊会話 line in {value}"));
        seen.push(*line);
    }
    seen.sort();
    let mut expected = LINES.to_vec();
    expected.sort();
    assert_eq!(seen, expected, "both ＊会話 scenes must be issued");
}

/// 空の別名表：`＊会話` は「会話」のままで OnTalk は発行されない（5.6）
#[test]
fn empty_alias_table_does_not_issue_ontalk() {
    let (mut shiori, _temp) = load("\n[scene.alias]\n");
    on_boot(&mut shiori);

    let init = on_second_change(&mut shiori, "2025-07-15T12:05:00Z");
    assert_eq!(init.status_code, 204, "init: {:?}", init.value);
    for time in ["2025-07-15T12:05:10Z", "2025-07-15T12:05:20Z"] {
        let resp = on_second_change(&mut shiori, time);
        assert_eq!(resp.status_code, 204, "{time}: {:?}", resp.value);
    }

    // 対照：シーン自体は「会話」の名前で読み込まれている（イベント ID でのシーン関数フォールバック）
    let direct = request(&mut shiori, "ID: 会話\r\n");
    assert_eq!(direct.status_code, 200, "direct: {:?}", direct.value);
    let value = direct.value.expect("Value header must exist");
    assert!(LINES.iter().any(|l| value.contains(l)), "direct: {value}");
}
