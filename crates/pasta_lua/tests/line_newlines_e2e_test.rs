//! actor-line-newlines の結合テスト（task 5.2）。
//!
//! ゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! DSL の変換（行の頭の印）・SHIORI の act（`[ghost] line_newlines` の読み込み）・組み立てを
//! 本物のまま、`SHIORI.request` の出力を設定の有無で比べる（`chain_talk_interrupt_test.rs` と同じ作り）。
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// 行の改行を試す辞書（research.md の例と、ぱすた開発セッションの実例を縮めたもの）
const DIC: &str = "\
％ぱすた
　＠笑顔：\\s[5]
　＠注目：\\s[6]
　＠横目：\\s[7]
　＠驚き：\\s[8]
　＠目閉じ：\\s[9]
　＠左：\\_l[-283,0]
　＠改行：\\n[180]
％おじさん
　＠改行：「\\n[180]\\_u[0x3000]」

＊OnTestPoem
　ぱすた：＠笑顔　今日が最後の日なら。
　　　　：＠注目　みんなに会ってみたいかも。
　ぱすた：＠横目　おじさんとか仕事中かな？
　ぱすた：＠左　男の子は帰ったかな？
　　　　：手を振ったら、

＊OnTestRewrite
　ぱすた：ひとつめ。
　ぱすた：ふたつめ。

＊OnTestCall
　ぱすた：窓の下の公園、
　＞季節
　ぱすた：また今度。

＊季節
　ぱすた：春の花。

＊OnTestNetaddict
　ぱすた：ネットの話。
　　　　：＠改行　まだ続く。
　ぱすた：＠左
　＞系統
　ぱすた：おしまい。

＊系統
　ぱすた：＠目閉じ　「系統の話」

＊OnTestClear
　ぱすた：前の吹き出し。
　ぱすた：\\c
　＞起動_朝

＊起動_朝
　ぱすた：おはよう。

＊OnTestOjisan
　おじさん：対策をね。
　　　　　：＠改行　今日は暑い。
　ぱすた：やあ。
　　　　：涼しいよ。

＊OnTestEmptyLine
　ぱすた：なら。
　　　　：＠笑顔
　ぱすた：次の話。

＊OnTestTwoEmptyLines
　ぱすた：なら。
　　　　：＠笑顔
　　　　：＠驚き
　ぱすた：次の話。
";

/// 最後の表が `[actor."ぱすた"]` なので、追記の頭のキーはぱすたの設定になる（1.7 の確認に使う）
const ACTORS: &str = "[actor.\"おじさん\"]\nspot = 1\n\n[actor.\"ぱすた\"]\nspot = 0\n";

/// 指定の `pasta.toml` の追記で一時ゴーストを作って読み込む。
fn load_ghost(base: &Path, extra_toml: &str) -> PastaLuaRuntime {
    let pasta_file = base.join("dic/test/line_newlines.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, DIC).unwrap();
    std::fs::write(
        base.join("pasta.toml"),
        format!("[loader]\ndebug_mode = true\n\n{ACTORS}{extra_toml}"),
    )
    .unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }

    PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load")
}

/// `SHIORI.request` に GET のリクエストを渡し、200 の Value を返す。
fn value(runtime: &PastaLuaRuntime, id: &str) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", "local").unwrap();
    req.set("dic", lua.create_table().unwrap()).unwrap();
    req.set("reference", lua.create_table().unwrap()).unwrap();

    let shiori: Table = lua
        .load(r#"return require("pasta.shiori.entry")"#)
        .eval()
        .unwrap();
    let response = shiori
        .get::<Function>("request")
        .unwrap()
        .call::<String>(req)
        .expect("SHIORI.request must return a response string");
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 200 OK"),
        "{id}: 200 expected: {response:?}"
    );
    response
        .lines()
        .find_map(|l| l.strip_prefix("Value: "))
        .unwrap_or_else(|| panic!("{id}: Value line expected: {response:?}"))
        .to_string()
}

/// (イベント, 設定なしの出力 = 今の版, `line_newlines = 1.8` の出力)
const CASES: [(&str, &str, &str); 8] = [
    // 2.1・2.2・4.1: 継続行と書き直した行の前に入り、頭が `\_l` の書き直し行には入らない
    (
        "OnTestPoem",
        r"\p[0]\s[5]今日が最後の日なら。\_w[950]\s[6]みんなに会ってみたいかも。\_w[950]\s[7]おじさんとか仕事中かな？\_w[450]\_l[-283,0]男の子は帰ったかな？\_w[450]手を振ったら、\_w[450]\e",
        r"\p[0]\s[5]今日が最後の日なら。\_w[950]\s[6]\n[180]みんなに会ってみたいかも。\_w[950]\s[7]\n[180]おじさんとか仕事中かな？\_w[450]\_l[-283,0]男の子は帰ったかな？\_w[450]\n[180]手を振ったら、\_w[450]\e",
    ),
    // 2.2: `ぱすた：A` → `ぱすた：B`
    (
        "OnTestRewrite",
        r"\p[0]ひとつめ。\_w[950]ふたつめ。\_w[950]\e",
        r"\p[0]ひとつめ。\_w[950]\n[180]ふたつめ。\_w[950]\e",
    ),
    // 2.5: Call 先の最初の行と、戻った後の行に入る
    (
        "OnTestCall",
        r"\p[0]窓の下の公園、\_w[450]春の花。\_w[950]また今度。\_w[950]\e",
        r"\p[0]窓の下の公園、\_w[450]\n[180]春の花。\_w[950]\n[180]また今度。\_w[950]\e",
    ),
    // 4.1・4.2・4.5: `＠改行` の継続行・`＠左` の行・`\_l` で終わった後の Call 先の頭には入らない
    // （戻った後の `おしまい。` は前の出力が見える字で終わるので入る）
    (
        "OnTestNetaddict",
        r"\p[0]ネットの話。\_w[950]\n[180]まだ続く。\_w[950]\_l[-283,0]\s[9]「系統の話」おしまい。\_w[950]\e",
        r"\p[0]ネットの話。\_w[950]\n[180]まだ続く。\_w[950]\_l[-283,0]\s[9]「系統の話」\n[180]おしまい。\_w[950]\e",
    ),
    // 4.2: `ぱすた：\c` の後の Call 先の頭の行には入らない
    (
        "OnTestClear",
        r"\p[0]前の吹き出し。\_w[950]\cおはよう。\_w[950]\e",
        r"\p[0]前の吹き出し。\_w[950]\cおはよう。\_w[950]\e",
    ),
    // 4.5: 相手役の継続行 `＠改行　…` は設定の前後で同じ。話し手が替わった行には入らない（2.2）
    (
        "OnTestOjisan",
        r"\p[1]対策をね。\_w[950]\n[180]\_u[0x3000]今日は暑い。\_w[950]\p[0]やあ。\_w[950]涼しいよ。\_w[950]\e",
        r"\p[1]対策をね。\_w[950]\n[180]\_u[0x3000]今日は暑い。\_w[950]\p[0]やあ。\_w[950]\n[180]涼しいよ。\_w[950]\e",
    ),
    // 2.4: 字の無い行も 1 列と数える
    (
        "OnTestEmptyLine",
        r"\p[0]なら。\_w[950]\s[5]次の話。\_w[950]\e",
        r"\p[0]なら。\_w[950]\s[5]\n[180]\n[180]次の話。\_w[950]\e",
    ),
    // 2.4: 字の無い行が 2 つ続けば、それぞれが 1 列（空の列が 2 つ）
    (
        "OnTestTwoEmptyLines",
        r"\p[0]なら。\_w[950]\s[5]\s[8]次の話。\_w[950]\e",
        r"\p[0]なら。\_w[950]\s[5]\n[180]\s[8]\n[180]\n[180]次の話。\_w[950]\e",
    ),
];

/// 指定の追記で読み込み、全イベントの出力が期待（設定なし or 設定あり）と同じことを確かめる。
fn assert_outputs(extra_toml: &str, with_newlines: bool) {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), extra_toml);
    for (id, without, with) in CASES {
        let expected = if with_newlines { with } else { without };
        assert_eq!(value(&runtime, id), expected, "{id} ({extra_toml:?})");
    }
}

/// 2.1・2.2・2.4・2.5・4.1・4.2・4.5: `line_newlines = 1.8` で同じ話し手の行の前に `\n[180]` が入る。
#[test]
fn line_newlines_inserts_newline_before_same_speaker_lines() {
    assert_outputs("[ghost]\nline_newlines = 1.8\n", true);
}

/// 6.1: 設定なしは今の版と同じ出力。
#[test]
fn absent_setting_keeps_current_output() {
    assert_outputs("", false);
}

/// 6.1: `line_newlines = 0` は設定なしと同じ。
#[test]
fn zero_setting_keeps_current_output() {
    assert_outputs("[ghost]\nline_newlines = 0\n", false);
}

/// 1.7: アクターごとの `line_newlines` は読まない。
#[test]
fn actor_line_newlines_is_ignored() {
    // 追記は `[actor."ぱすた"]` の表に入る（ACTORS の並びを参照）
    assert_outputs("line_newlines = 1.8\n", false);
}
