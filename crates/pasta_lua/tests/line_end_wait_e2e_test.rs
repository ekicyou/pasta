//! 行末の待ちの結合テスト。
//!
//! ゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! DSL の変換・待ちの挿入・組み立てを本物のまま、`SHIORI.request` の出力を比べる
//! （`line_newlines_e2e_test.rs` と同じ作り）。`PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// 行末のキーを書かない ghost_dev の待ちの値
const TALK: &str = "\
[talk]
script_wait_normal = 90
script_wait_period = 1200
script_wait_comma = 600
script_wait_strong = 750
script_wait_leader = 300
";

const ACTORS: &str = "[actor.\"ぱすた\"]\nspot = 0\n";

/// 行末の形を並べた辞書
const DIC: &str = "\
％ぱすた
　＠笑顔：\\s[5]
　＠左：\\_l[-283,0]
　＠間：\\_w[500]

＊OnTestQuestion
　ぱすた：雨？降るかな？

＊OnTestStrongClose
　ぱすた：「行くよ。」
　ぱすた：本当！？

＊OnTestNormalEnd
　ぱすた：こんにちは

＊OnTestLeaderEnd
　ぱすた：そうね‥‥

＊OnTestSuEnd
　ぱすた：「そうです」
　ぱすた：「そうです」と言った。

＊OnTestSurfaceAfterEnd
　ぱすた：雨？＠笑顔

＊OnTestAuthorWait
　ぱすた：雨？\\_w[500]

＊OnTestWordWait
　ぱすた：雨？＠間

＊OnTestLeftContinuation
　ぱすた：ひとつめ。
　　　　：＠左　ふたつめ。

＊OnTestSurfaceOnly
　ぱすた：なら。
　　　　：＠笑顔
　ぱすた：次の話。

＊OnTestCall
　ぱすた：窓の下の公園、
　＞季節
　ぱすた：また今度。

＊季節
　ぱすた：春の花。

＊OnTestBackslashEnd
　ぱすた：あ\\\\

＊OnTestOpenBracketEnd
　ぱすた：言うね「

＊OnTestTrailingSpace
　ぱすた：雨？\u{3000}
";

/// `pasta.toml` の `[loader]` の後ろと辞書を受け取り、一時ゴーストを作って読み込む。
fn load_ghost(base: &Path, toml: &str, dic: &str) -> PastaLuaRuntime {
    let pasta_file = base.join("dic/test/line_end_wait.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, dic).unwrap();
    std::fs::write(
        base.join("pasta.toml"),
        format!("[loader]\ndebug_mode = true\n\n{toml}"),
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

/// 読み込んだゴーストで、全イベントの出力が期待と同じことを確かめる。
fn assert_outputs(toml: &str, dic: &str, cases: &[(&str, &str)]) {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), toml, dic);
    for (id, expected) in cases {
        assert_eq!(value(&runtime, id), *expected, "{id} ({toml:?})");
    }
}

/// (イベント, 設定なしの出力, `line_newlines = 1.8` の出力)
const CASES: [(&str, &str, &str); 14] = [
    (
        "OnTestQuestion",
        r"\p[0]雨\_w[40]？\_w[700]降\_w[40]る\_w[40]か\_w[40]な\_w[40]？\_w[700]\e",
        r"\p[0]雨\_w[40]？\_w[700]降\_w[40]る\_w[40]か\_w[40]な\_w[40]？\_w[700]\e",
    ),
    (
        "OnTestStrongClose",
        r"\p[0]「行\_w[40]く\_w[40]よ\_w[40]。」\_w[1150]本\_w[40]当\_w[40]！？\_w[700]\e",
        r"\p[0]「行\_w[40]く\_w[40]よ\_w[40]。」\_w[1150]\n[180]本\_w[40]当\_w[40]！？\_w[700]\e",
    ),
    (
        "OnTestNormalEnd",
        r"\p[0]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]\e",
        r"\p[0]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]\e",
    ),
    (
        "OnTestLeaderEnd",
        r"\p[0]そ\_w[40]う\_w[40]ね\_w[40]‥\_w[250]‥\_w[250]\e",
        r"\p[0]そ\_w[40]う\_w[40]ね\_w[40]‥\_w[250]‥\_w[250]\e",
    ),
    (
        "OnTestSuEnd",
        r"\p[0]「そ\_w[40]う\_w[40]で\_w[40]す\_w[40]」「そ\_w[40]う\_w[40]で\_w[40]す\_w[40]」と\_w[40]言\_w[40]っ\_w[40]た\_w[40]。\_w[1150]\e",
        r"\p[0]「そ\_w[40]う\_w[40]で\_w[40]す\_w[40]」\n[180]「そ\_w[40]う\_w[40]で\_w[40]す\_w[40]」と\_w[40]言\_w[40]っ\_w[40]た\_w[40]。\_w[1150]\e",
    ),
    (
        "OnTestSurfaceAfterEnd",
        r"\p[0]雨\_w[40]？\_w[700]\s[5]\e",
        r"\p[0]雨\_w[40]？\_w[700]\s[5]\e",
    ),
    (
        "OnTestAuthorWait",
        r"\p[0]雨\_w[40]？\_w[700]\_w[500]\e",
        r"\p[0]雨\_w[40]？\_w[700]\_w[500]\e",
    ),
    (
        "OnTestWordWait",
        r"\p[0]雨\_w[40]？\_w[700]\_w[500]\e",
        r"\p[0]雨\_w[40]？\_w[700]\_w[500]\e",
    ),
    (
        "OnTestLeftContinuation",
        r"\p[0]ひ\_w[40]と\_w[40]つ\_w[40]め\_w[40]。\_w[1150]\_l[-283,0]ふ\_w[40]た\_w[40]つ\_w[40]め\_w[40]。\_w[1150]\e",
        r"\p[0]ひ\_w[40]と\_w[40]つ\_w[40]め\_w[40]。\_w[1150]\_l[-283,0]ふ\_w[40]た\_w[40]つ\_w[40]め\_w[40]。\_w[1150]\e",
    ),
    (
        "OnTestSurfaceOnly",
        r"\p[0]な\_w[40]ら\_w[40]。\_w[1150]\s[5]次\_w[40]の\_w[40]話\_w[40]。\_w[1150]\e",
        r"\p[0]な\_w[40]ら\_w[40]。\_w[1150]\s[5]\n[180]\n[180]次\_w[40]の\_w[40]話\_w[40]。\_w[1150]\e",
    ),
    (
        "OnTestCall",
        r"\p[0]窓\_w[40]の\_w[40]下\_w[40]の\_w[40]公\_w[40]園\_w[40]、\_w[550]春\_w[40]の\_w[40]花\_w[40]。\_w[1150]ま\_w[40]た\_w[40]今\_w[40]度\_w[40]。\_w[1150]\e",
        r"\p[0]窓\_w[40]の\_w[40]下\_w[40]の\_w[40]公\_w[40]園\_w[40]、\_w[550]\n[180]春\_w[40]の\_w[40]花\_w[40]。\_w[1150]\n[180]ま\_w[40]た\_w[40]今\_w[40]度\_w[40]。\_w[1150]\e",
    ),
    (
        "OnTestBackslashEnd",
        r"\p[0]あ\_w[40]\\\e",
        r"\p[0]あ\_w[40]\\\e",
    ),
    (
        "OnTestOpenBracketEnd",
        r"\p[0]言\_w[40]う\_w[40]ね\_w[40]「\e",
        r"\p[0]言\_w[40]う\_w[40]ね\_w[40]「\e",
    ),
    // 行の終わりの空白は字として残り、ふつうの字の待ちが付く
    (
        "OnTestTrailingSpace",
        "\\p[0]雨\\_w[40]？\\_w[700]\u{3000}\\_w[40]\\e",
        "\\p[0]雨\\_w[40]？\\_w[700]\u{3000}\\_w[40]\\e",
    ),
];

/// 行末のキーを書かないゴーストは今の出力のまま。
#[test]
fn without_line_end_keys_keeps_current_output() {
    let cases: Vec<_> = CASES
        .iter()
        .map(|(id, without, _)| (*id, *without))
        .collect();
    assert_outputs(&format!("{TALK}\n{ACTORS}"), DIC, &cases);
}

/// `line_newlines = 1.8` を書いたゴーストも今の出力のまま。
#[test]
fn without_line_end_keys_keeps_current_output_with_line_newlines() {
    let cases: Vec<_> = CASES.iter().map(|(id, _, with)| (*id, *with)).collect();
    assert_outputs(
        &format!("[ghost]\nline_newlines = 1.8\n\n{TALK}\n{ACTORS}"),
        DIC,
        &cases,
    );
}

const EOL_DIC: &str = "\
％ぱすた
　＠笑顔：\\s[5]
　＠間：\\_w[500]

％妹

＊OnIntro
　ぱすた：明日は晴れ？それとも、雨？
　　　　：じゃあ、行ってきます。

＊OnStrongClose
　ぱすた：そうだよね？」
　ぱすた：本当！？

＊OnPlain
　ぱすた：こんにちは

＊OnLeader
　ぱすた：お母さんに怒られた‥‥

＊OnEndQuote
　ぱすた：行ってきます」
　ぱすた：ます」と言う

＊OnTag
　ぱすた：行ってきます。＠笑顔

＊OnAuthor
　ぱすた：雨？\\_w[500]
　ぱすた：雨？\\_w[0]
　ぱすた：雨？\\_w[500]\\s[5]
　ぱすた：雨？\\_w[300]\\_w[200]
　ぱすた：雨？＠間
　ぱすた：晴れ？\\_w[500]それとも
　ぱすた：雨？\\s[5]\\_w[500]

＊OnSeamAfterTag
　ぱすた：行ってきます。＠笑顔
　ぱすた：次。

＊OnBare
　ぱすた：雨？

＊OnActors
　妹：こんにちは？
　ぱすた：こんにちは？
";

fn eol_toml(extra: &str) -> String {
    format!("{TALK}{extra}\n[actor.\"ぱすた\"]\nspot = 0\n\n[actor.\"妹\"]\nspot = 1\n")
}

#[test]
fn line_end_keys_change_only_line_ends() {
    assert_outputs(
        &eol_toml("script_wait_strong_eol = 1200\n"),
        EOL_DIC,
        &[
            (
                "OnIntro",
                r"\p[0]明\_w[40]日\_w[40]は\_w[40]晴\_w[40]れ\_w[40]？\_w[700]そ\_w[40]れ\_w[40]と\_w[40]も\_w[40]、\_w[550]雨\_w[40]？\_w[1150]じ\_w[40]ゃ\_w[40]あ\_w[40]、\_w[550]行\_w[40]っ\_w[40]て\_w[40]き\_w[40]ま\_w[40]す\_w[40]。\_w[1150]\e",
            ),
            (
                "OnStrongClose",
                r"\p[0]そ\_w[40]う\_w[40]だ\_w[40]よ\_w[40]ね\_w[40]？」\_w[1150]本\_w[40]当\_w[40]！？\_w[1150]\e",
            ),
            (
                "OnAuthor",
                r"\p[0]雨\_w[40]？\_w[500]雨\_w[40]？\_w[0]雨\_w[40]？\_w[500]\s[5]雨\_w[40]？\_w[300]\_w[200]雨\_w[40]？\_w[500]晴\_w[40]れ\_w[40]？\_w[700]\_w[500]そ\_w[40]れ\_w[40]と\_w[40]も\_w[40]雨\_w[40]？\_w[1150]\s[5]\_w[500]\e",
            ),
        ],
    );
}

#[test]
fn zero_or_fifty_removes_line_end_wait() {
    for v in [50, 0] {
        assert_outputs(
            &eol_toml(&format!("script_wait_strong_eol = {v}\n")),
            EOL_DIC,
            &[("OnBare", r"\p[0]雨\_w[40]？\e")],
        );
    }
}

#[test]
fn removed_manual_wait_equals_line_end_key() {
    assert_outputs(
        &eol_toml("script_wait_strong_eol = 1250\n"),
        EOL_DIC,
        &[("OnBare", r"\p[0]雨\_w[40]？\_w[1200]\e")],
    );
}

#[test]
fn plain_leader_and_close_line_ends() {
    assert_outputs(
        &eol_toml("script_wait_normal_eol = 1250\n"),
        EOL_DIC,
        &[
            (
                "OnPlain",
                r"\p[0]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[1200]\e",
            ),
            (
                "OnEndQuote",
                r"\p[0]行\_w[40]っ\_w[40]て\_w[40]き\_w[40]ま\_w[40]す」\_w[1200]ま\_w[40]す\_w[40]」と\_w[40]言\_w[40]う\_w[1200]\e",
            ),
            (
                "OnAuthor",
                r"\p[0]雨\_w[40]？\_w[700]\_w[500]雨\_w[40]？\_w[700]\_w[0]雨\_w[40]？\_w[700]\_w[500]\s[5]雨\_w[40]？\_w[700]\_w[300]\_w[200]雨\_w[40]？\_w[700]\_w[500]晴\_w[40]れ\_w[40]？\_w[700]\_w[500]そ\_w[40]れ\_w[40]と\_w[40]も\_w[1200]雨\_w[40]？\_w[700]\s[5]\_w[500]\e",
            ),
        ],
    );
    assert_outputs(
        &eol_toml("script_wait_leader_eol = 1250\n"),
        EOL_DIC,
        &[(
            "OnLeader",
            r"\p[0]お\_w[40]母\_w[40]さ\_w[40]ん\_w[40]に\_w[40]怒\_w[40]ら\_w[40]れ\_w[40]た\_w[40]‥\_w[250]‥\_w[1200]\e",
        )],
    );
}

#[test]
fn line_end_wait_goes_before_trailing_tag() {
    assert_outputs(
        &eol_toml("script_wait_period_eol = 1250\n"),
        EOL_DIC,
        &[(
            "OnTag",
            r"\p[0]行\_w[40]っ\_w[40]て\_w[40]き\_w[40]ま\_w[40]す\_w[40]。\_w[1200]\s[5]\e",
        )],
    );
}

#[test]
fn actor_line_end_key_applies_to_that_actor_only() {
    assert_outputs(
        &eol_toml("script_wait_strong_eol = 1200\n").replace(
            "[actor.\"妹\"]\nspot = 1\n",
            "[actor.\"妹\"]\nspot = 1\nscript_wait_strong_eol = 900\n",
        ),
        EOL_DIC,
        &[(
            "OnActors",
            r"\p[1]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]？\_w[850]\p[0]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]？\_w[1150]\e",
        )],
    );
    assert_outputs(
        &eol_toml("").replace(
            "[actor.\"妹\"]\nspot = 1\n",
            "[actor.\"妹\"]\nspot = 1\nscript_wait_strong_eol = 1200\n",
        ),
        EOL_DIC,
        &[(
            "OnActors",
            r"\p[1]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]？\_w[1150]\p[0]こ\_w[40]ん\_w[40]に\_w[40]ち\_w[40]は\_w[40]？\_w[700]\e",
        )],
    );
}

const BUDOUX_DIC: &str = "\
％ぱすた
　＠笑顔：\\s[5]
　＠左：\\_l[-283,0]
　＠間：\\_w[500]

＊OnBudoux
　ぱすた：私は今日学校へ行って友達と一緒に勉強をしました。＠笑顔
　ぱすた：明日は、公園で遊ぶ約束をしたよ？\\_w[500]
　ぱすた：雨が降ったら図書館で本を読もうね！＠間
　ぱすた：ひとつめの話はここまで。
　　　　：＠左　ふたつめは帰り道の話。
";

fn budoux_toml(ghost: &str, eol: &str) -> String {
    format!("{ghost}{TALK}{eol}\n[actor.\"ぱすた\"]\nspot = 0\nbudoux = [8]\n")
}

const LINE_NEWLINES: &str = "[ghost]\nline_newlines = 1.8\n\n";

const ALL_EOL: &str =
    "script_wait_strong_eol = 1250\nscript_wait_period_eol = 1250\nscript_wait_normal_eol = 1250\n";

#[test]
fn line_end_wait_with_line_newlines_and_budoux() {
    assert_outputs(
        &budoux_toml(LINE_NEWLINES, ALL_EOL),
        BUDOUX_DIC,
        &[(
            "OnBudoux",
            r"\p[0]私\_w[40]は\_w[40]\n今\_w[40]日\_w[40]学\_w[40]校\_w[40]へ\_w[40]\n行\_w[40]っ\_w[40]て\_w[40]\n友\_w[40]達\_w[40]と\_w[40]\n一\_w[40]緒\_w[40]に\_w[40]\n勉\_w[40]強\_w[40]を\_w[40]\nし\_w[40]ま\_w[40]し\_w[40]た\_w[40]。\_w[1200]\s[5]\n[180]明\_w[40]日\_w[40]は\_w[40]、\_w[550]\n公\_w[40]園\_w[40]で\_w[40]\n遊\_w[40]ぶ\_w[40]約\_w[40]束\_w[40]を\_w[40]\nし\_w[40]た\_w[40]よ\_w[40]？\_w[500]\n[180]雨\_w[40]が\_w[40]\n降\_w[40]っ\_w[40]た\_w[40]ら\_w[40]\n図\_w[40]書\_w[40]館\_w[40]で\_w[40]本\_w[40]を\_w[40]\n読\_w[40]も\_w[40]う\_w[40]ね\_w[40]！\_w[500]\n[180]ひ\_w[40]と\_w[40]つ\_w[40]め\_w[40]の\_w[40]\n話\_w[40]は\_w[40]\nこ\_w[40]こ\_w[40]ま\_w[40]で\_w[40]。\_w[1200]\_l[-283,0]ふ\_w[40]た\_w[40]\nつ\_w[40]め\_w[40]は\_w[40]\n帰\_w[40]り\_w[40]道\_w[40]の\_w[40]\n話\_w[40]。\_w[1200]\e",
        )],
    );
}

/// 出力を、待ちの並び（`\_w[…]` の続き）で区切った字の切れ目と、その後ろの待ちの並びに分ける。
fn split_waits(script: &str) -> Vec<(&str, String)> {
    let mut parts = Vec::new();
    let mut rest = script;
    while let Some(i) = rest.find(r"\_w[") {
        let text = &rest[..i];
        rest = &rest[i..];
        let mut waits = String::new();
        while rest.starts_with(r"\_w[") {
            let end = rest.find(']').unwrap() + 1;
            waits.push_str(&rest[..end]);
            rest = &rest[end..];
        }
        parts.push((text, waits));
    }
    parts.push((rest, String::new()));
    parts
}

#[test]
fn line_end_keys_keep_budoux_breaks_and_mid_line_waits() {
    for ghost in ["", LINE_NEWLINES] {
        let outputs: Vec<String> = ["", ALL_EOL]
            .iter()
            .map(|eol| {
                let temp = tempfile::TempDir::new().unwrap();
                value(
                    &load_ghost(temp.path(), &budoux_toml(ghost, eol), BUDOUX_DIC),
                    "OnBudoux",
                )
            })
            .collect();
        let (without, with) = (split_waits(&outputs[0]), split_waits(&outputs[1]));
        let texts = |p: &[(&str, String)]| p.iter().map(|(t, _)| t.to_string()).collect::<Vec<_>>();
        assert_eq!(texts(&without), texts(&with), "{ghost:?}");

        // 待ちの後ろが次の行の頭（タグ・行の改行・辞書の行の最初の字。行の改行がなければ budoux の改行が前に付く）なら、その待ちは行末の待ち
        let mut line_ends = 0;
        for (i, ((_, a), (_, b))) in without.iter().zip(&with).enumerate() {
            let next = without[i + 1..].first().map_or("", |(t, _)| *t);
            let at_line_end = [
                "\\s[5]", "\\n[180]", "\\_l[", "\\e", "雨", "ひ", "\\n雨", "\\nひ",
            ]
            .iter()
            .any(|h| next.starts_with(h));
            if at_line_end {
                line_ends += 1;
            } else {
                assert_eq!(a, b, "{ghost:?}: mid-line wait #{i}");
            }
        }
        assert_eq!(line_ends, 5, "{ghost:?}");
    }
}

#[test]
fn line_end_wait_goes_before_trailing_tag_with_line_newlines() {
    assert_outputs(
        &format!(
            "[ghost]
line_newlines = 1.8

{}",
            eol_toml(
                "script_wait_period_eol = 1250
"
            )
        ),
        EOL_DIC,
        &[(
            "OnSeamAfterTag",
            r"\p[0]行\_w[40]っ\_w[40]て\_w[40]き\_w[40]ま\_w[40]す\_w[40]。\_w[1200]\s[5]\n[180]次\_w[40]。\_w[1200]\e",
        )],
    );
}
