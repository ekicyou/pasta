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
