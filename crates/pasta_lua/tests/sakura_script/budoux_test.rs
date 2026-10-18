//! Integration tests for budoux line breaker via Lua API.
//!
//! Tests SAKURA.break_lines() called from Lua, verifying
//! correct line breaking with various inputs and edge cases.

use crate::common::create_sakura_test_runtime;

// ============================================
// Task 5.1: Lua API break_lines tests
// ============================================

#[test]
fn test_break_lines_basic_japanese() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("今日はいい天気ですね", {6})
            "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.contains("\\n"),
        "Expected line break in result: {}",
        result
    );
}

#[test]
fn test_break_lines_with_wait_tags() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local input = "こ" .. [==[\_w[50]]==] .. "れ" .. [==[\_w[50]]==] .. "は" .. [==[\_w[50]]==] .. "テ" .. [==[\_w[50]]==] .. "ス" .. [==[\_w[50]]==] .. "ト"
            return SAKURA.break_lines(input, {6})
            "#,
        )
        .eval()
        .unwrap();

    assert!(result.contains("\\n"), "Expected line break: {}", result);
    // All 5 wait tags must be preserved
    assert_eq!(
        result.matches(r"\_w[50]").count(),
        5,
        "All wait tags must be preserved: {}",
        result
    );
}

#[test]
fn test_break_lines_nil_text_returns_empty() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines(nil, {10})
            "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_break_lines_empty_table_returns_input() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("テスト", {})
            "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "テスト");
}

#[test]
fn test_break_lines_nil_widths_returns_input() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("テスト", nil)
            "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "テスト");
}

#[test]
fn test_break_lines_multiple_widths() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("私は今日学校へ行って友達と一緒に勉強をしました", {4, 6})
            "#,
        )
        .eval()
        .unwrap();

    assert!(result.contains("\\n"), "Expected line breaks: {}", result);
}

#[test]
fn test_break_lines_no_break_when_fits() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("短い", {20})
            "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "短い");
}

// ============================================
// talk_to_script + budoux actor integration
// ============================================

use crate::common::create_sakura_test_runtime_with_config;
use pasta_lua::loader::TalkConfig;

/// Config with visible normal waits (100ms → effective 50ms → \_w[50])
fn budoux_talk_config() -> TalkConfig {
    TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    }
}

#[test]
fn test_talk_to_script_with_budoux_actor_inserts_line_breaks() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { budoux = {6} }
            return SAKURA.talk_to_script(actor, "今日はいい天気ですね")
            "#,
        )
        .eval()
        .unwrap();

    // Should contain wait tags (from wait insertion)
    assert!(
        result.contains(r"\_w["),
        "Expected wait tags in result: {}",
        result
    );
    // Should contain line breaks (from budoux)
    assert!(
        result.contains("\\n"),
        "Expected budoux line break in result: {}",
        result
    );
}

#[test]
fn test_talk_to_script_without_budoux_actor_no_line_breaks() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { name = "test" }
            return SAKURA.talk_to_script(actor, "今日はいい天気ですね")
            "#,
        )
        .eval()
        .unwrap();

    // Should have wait tags but NO budoux line breaks
    assert!(result.contains(r"\_w["), "Expected wait tags: {}", result);
    assert!(
        !result.contains("\\n"),
        "Expected no line breaks without budoux config: {}",
        result
    );
}

#[test]
fn test_talk_to_script_nil_actor_no_line_breaks() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "今日はいい天気ですね")
            "#,
        )
        .eval()
        .unwrap();

    assert!(result.contains(r"\_w["), "Expected wait tags: {}", result);
    assert!(
        !result.contains("\\n"),
        "Expected no line breaks with nil actor: {}",
        result
    );
}

#[test]
fn test_talk_to_script_budoux_preserves_wait_tags() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { budoux = {6} }
            return SAKURA.talk_to_script(actor, "こんにちは。元気ですか")
            "#,
        )
        .eval()
        .unwrap();

    // Period wait must survive budoux processing
    assert!(
        result.contains(r"\_w["),
        "Wait tags must be preserved through budoux: {}",
        result
    );
}

#[test]
fn test_talk_to_script_budoux_vs_no_budoux_same_input() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    // without budoux (budoux キーなし)
    let without: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script({ name = "test" }, "今日はいい天気ですね")
            "#,
        )
        .eval()
        .unwrap();

    // with budoux
    let with_bx: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script({ budoux = {6} }, "今日はいい天気ですね")
            "#,
        )
        .eval()
        .unwrap();

    // 結果が異なること
    assert_ne!(without, with_bx, "budoux あり/なしで出力が同一になっている");
    // なし側に \n がないこと
    assert!(
        !without.contains("\\n"),
        "budoux なし側に改行が混入: {}",
        without
    );
    // あり側に \n があること
    assert!(
        with_bx.contains("\\n"),
        "budoux あり側に改行がない: {}",
        with_bx
    );
    // どちらもウェイトタグを含むこと
    assert!(
        without.contains(r"\_w["),
        "budoux なし側にウェイトタグがない: {}",
        without
    );
    assert!(
        with_bx.contains(r"\_w["),
        "budoux あり側にウェイトタグがない: {}",
        with_bx
    );
}

#[test]
fn test_talk_to_script_budoux_multi_width_array() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            -- 複数要素の幅配列: 1行目=4, 2行目以降=8 (繰り返し)
            local actor = { budoux = {4, 8} }
            return SAKURA.talk_to_script(actor, "私は今日学校へ行って友達と一緒に勉強をしました")
            "#,
        )
        .eval()
        .unwrap();

    // 複数要素配列でも改行が生成されること
    assert!(
        result.contains("\\n"),
        "複数幅配列で改行が生成されること: {}",
        result
    );
    // ウェイトタグが保持されること
    assert!(
        result.contains(r"\_w["),
        "ウェイトタグが保持されること: {}",
        result
    );
}

#[test]
fn test_talk_to_script_budoux_last_value_repeats() {
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());

    // {4, 4} と {4, 4, 4} は最後の値が同じ(4)なので同一結果になること
    let result_two: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(
                { budoux = {4, 4} },
                "私は今日学校へ行って友達と一緒に勉強をしました"
            )
            "#,
        )
        .eval()
        .unwrap();

    let result_three: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(
                { budoux = {4, 4, 4} },
                "私は今日学校へ行って友達と一緒に勉強をしました"
            )
            "#,
        )
        .eval()
        .unwrap();

    assert_eq!(
        result_two, result_three,
        "budoux={{4,4}} と budoux={{4,4,4}} は同一結果であること（最後の値が繰り返されるため）:\n{}\nvs\n{}",
        result_two, result_three
    );
}

// ============================================
// SSP にそろえたタグの読み取り: 囲みとエスケープの途中に改行が入らない
// （paragraph-break-tag-only-talk 6.5）
// Requirement: 8.2, 9.2
// ============================================

/// budoux を設定したアクターで `talk_to_script` を呼ぶ（既定値: 通常の字のウェイトは出ない）
fn talk_with_budoux(input: &str) -> String {
    let lua = create_sakura_test_runtime();
    let talk_to_script: mlua::Function = lua
        .load(r#"return require("@pasta_sakura_script").talk_to_script"#)
        .eval()
        .unwrap();
    let actor = lua.create_table().unwrap();
    actor
        .set("budoux", lua.create_sequence_from([4]).unwrap())
        .unwrap();
    talk_to_script.call((actor, input)).unwrap()
}

#[test]
fn test_talk_to_script_budoux_no_break_inside_literal_region() {
    // 囲みは幅 0 の 1 単位として前の字に付いて運ばれ、中の字の間に改行は入らない（9.2）
    assert_eq!(
        talk_with_budoux(r"今日は\_?いい天気ですね\_?明日も晴れるでしょう"),
        r"今日は\_?いい天気ですね\_?\n明日も\n晴れるでしょう"
    );
}

#[test]
fn test_talk_to_script_budoux_no_break_right_after_non_unit_backslash() {
    // 単位にならない `\` の直後に改行を挿むと `\\n` ができるので、改行は `\` の前に入れる（7.10）
    let out = talk_with_budoux(r"今日は\いい天気ですね");
    assert!(!out.contains(r"\\"), "`\\` が作られた: {out}");
    assert_eq!(out.replace(r"\n", ""), r"今日は\いい天気ですね");
}

#[test]
fn test_talk_to_script_budoux_no_break_inside_percent_escape() {
    // `\%` は幅 0 の 1 単位として前の字に付いて運ばれ、`\%` の前にも間にも改行は入らない（8.2）
    assert_eq!(
        talk_with_budoux(r"今日は\%いい天気ですね"),
        r"今日は\%\nいい\n天気ですね"
    );
}

// ============================================
// 列の状態の表と新しいアクター設定（vertical-balloon-lines 2.4）
// Requirement: 1.1, 1.2, 1.3, 1.4, 1.6, 3.5, 3.6, 3.7, 6.3, 6.5, 6.6
// ============================================

/// Lua のコードを評価して文字列を返す
fn eval_str(code: &str) -> String {
    create_sakura_test_runtime().load(code).eval().unwrap()
}

const LONG_TALK: &str = "私は今日学校へ行って友達と一緒に勉強をしました";

#[test]
fn test_talk_to_script_two_args_same_as_nil_column() {
    // 2 つの引数の呼び方は、3 つ目に nil を渡したのと同じ・戻り値は文字列 1 つ（6.5）
    let lua = create_sakura_test_runtime_with_config(&budoux_talk_config());
    let (two, three, count): (String, String, i64) = lua
        .load(format!(
            r##"
            local SAKURA = require "@pasta_sakura_script"
            local actor = {{ budoux = {{6}} }}
            local two = SAKURA.talk_to_script(actor, "{LONG_TALK}")
            local three = SAKURA.talk_to_script(actor, "{LONG_TALK}", nil)
            return two, three, select("#", SAKURA.talk_to_script(actor, "{LONG_TALK}", {{}}))
            "##
        ))
        .eval()
        .unwrap();
    assert_eq!(two, three);
    assert_eq!(count, 1, "戻り値は 1 つ");
}

#[test]
fn test_budoux_newlines_ratio_makes_newline_with_percent() {
    // 1.8 → \n[180]、1.0・無し → \n（1.1・1.2・1.3）
    let run = |newlines: &str| {
        eval_str(&format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script({{ budoux = {{6}}, budoux_newlines = {newlines} }}, "{LONG_TALK}")
            "#
        ))
    };
    let with_180 = run("1.8");
    assert!(with_180.contains(r"\n[180]"), "{with_180}");
    let plain = run("nil");
    assert!(plain.contains(r"\n") && !plain.contains(r"\n["), "{plain}");
    assert_eq!(run("1.0"), plain);
    assert_eq!(with_180.replace(r"\n[180]", r"\n"), plain);
}

#[test]
fn test_column_table_is_written_back() {
    // 成功した後に line・width・jump を書き戻す（6.3）
    let (out, line, width, jump): (String, i64, i64, bool) = create_sakura_test_runtime()
        .load(format!(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local col = {{}}
            local out = SAKURA.talk_to_script({{ budoux = {{6}} }}, "{LONG_TALK}", col)
            return out, col.line, col.width, col.jump
            "#
        ))
        .eval()
        .unwrap();
    let breaks = out.matches(r"\n").count() as i64;
    assert!(breaks > 0, "{out}");
    assert_eq!(line, breaks + 1);
    let last = out.rsplit(r"\n").next().unwrap();
    assert_eq!(width, 2 * last.chars().count() as i64, "{out}");
    assert!(!jump);
}

#[test]
fn test_column_table_start_state_is_read() {
    // 渡した表を始めの状態として読む。字数を持ち越して最初の語の前で割る
    let (out, line): (String, i64) = create_sakura_test_runtime()
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local col = { line = 2, width = 10 }
            local out = SAKURA.talk_to_script({ budoux = {40, 12} }, "今日は", col)
            return out, col.line
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(out, r"\n今日は");
    assert_eq!(line, 3);
}

#[test]
fn test_column_table_not_written_on_error() {
    // 変換が失敗したときは表を書き換えない
    let (ok, line, width, jump): (bool, i64, i64, bool) = create_sakura_test_runtime()
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local col = { line = 3, width = 5, jump = true }
            local ok = pcall(SAKURA.talk_to_script, { budoux = {6}, budoux_newlines = "abc" }, "今日はいい天気ですね", col)
            return ok, col.line, col.width, col.jump
            "#,
        )
        .eval()
        .unwrap();
    assert!(!ok);
    assert_eq!((line, width, jump), (3, 5, true));
}

#[test]
fn test_column_advances_without_budoux() {
    // アクターが nil・budoux の無いアクターでも列の状態は進み、出力は変わらない（決定 3）
    let lua = create_sakura_test_runtime();
    let (n_out, n_line, n_width, c_line, c_jump): (String, i64, i64, i64, bool) = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local col = {}
            local n_out = SAKURA.talk_to_script(nil, "\\n", col)
            local n_line, n_width = col.line, col.width
            local col2 = { line = 4, width = 3, jump = true }
            SAKURA.talk_to_script(nil, "\\c", col2)
            return n_out, n_line, n_width, col2.line, col2.jump
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(n_out, r"\n");
    assert_eq!((n_line, n_width), (2, 0));
    assert_eq!((c_line, c_jump), (1, false));

    let (with_col, without, line, width): (String, String, i64, i64) = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { name = "test" }
            local col = {}
            local with_col = SAKURA.talk_to_script(actor, "あいう\\nかき", col)
            return with_col, SAKURA.talk_to_script(actor, "あいう\\nかき"), col.line, col.width
            "#,
        )
        .eval()
        .unwrap();
    assert_eq!(with_col, without);
    assert_eq!((line, width), (2, 4));
}

#[test]
fn test_new_keys_not_read_without_budoux() {
    // budoux が無い・空なら budoux_newlines・budoux_jump を読まない（1.4・3.6）
    let out = eval_str(&format!(
        r#"
        local SAKURA = require "@pasta_sakura_script"
        return SAKURA.talk_to_script({{ budoux_newlines = "abc", budoux_jump = {{"x"}} }}, "{LONG_TALK}", {{}})
        "#
    ));
    assert_eq!(out, LONG_TALK);
    let empty = eval_str(&format!(
        r#"
        local SAKURA = require "@pasta_sakura_script"
        return SAKURA.talk_to_script({{ budoux = {{}}, budoux_newlines = "abc" }}, "{LONG_TALK}")
        "#
    ));
    assert_eq!(empty, LONG_TALK);
}

#[test]
fn test_break_lines_recounts_after_existing_newline() {
    // 元からある \n の後を数え直す（6.6）。数え直さないと「天気」の前で割る
    let out = eval_str(
        r#"
        local SAKURA = require "@pasta_sakura_script"
        return SAKURA.break_lines("今日は\\n天気", {8})
        "#,
    );
    assert_eq!(out, r"今日は\n天気");
}

#[test]
fn test_jump_widths_conformance_with_jump_tag() {
    // budoux = [40]・budoux_jump = [40, 28] で \_l[-295,0] を挟む長い台詞（3.7）
    let part = "今日はとても良い天気なので、公園まで散歩に出かけようと思います。途中で友達に会ったら、一緒にお茶でも飲みに行きたいですね。";
    let out = eval_str(&format!(
        r#"
        local SAKURA = require "@pasta_sakura_script"
        local actor = {{ budoux = {{40}}, budoux_jump = {{40, 28}} }}
        return SAKURA.talk_to_script(actor, "{part}" .. [==[\_l[-295,0]]==] .. "{part}{part}", {{}})
        "#
    ));
    let (before, after) = out.split_once(r"\_l[-295,0]").expect(&out);
    let waits = regex::Regex::new(r"\\_w\[\d+\]").unwrap();
    let cols = |s: &str| -> Vec<usize> {
        s.split(r"\n")
            .map(|c| waits.replace_all(c, "").chars().count())
            .collect()
    };
    let before = cols(before);
    let after = cols(after);
    assert!(before.len() >= 3 && after.len() >= 4, "{out}");
    for n in &before {
        assert!(*n <= 20, "列が 20 字を越えた {before:?}: {out}");
    }
    assert!(after[0] <= 20, "{after:?}: {out}");
    for n in &after[1..] {
        assert!(*n <= 14, "列が 14 字を越えた {after:?}: {out}");
    }
}
