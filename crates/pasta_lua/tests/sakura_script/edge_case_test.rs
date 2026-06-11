//! Edge case tests for sakura script tokenization, wait insertion, and
//! Lua API argument handling.
//!
//! Covers gaps not exercised by basic_test / output_test / budoux_test:
//! - Dangling backslash / unclosed bracket tokenization
//! - CharSets::classify priority for overlapping character sets
//! - LineStartProhibited-only pending runs (no own wait value)
//! - Pending punctuation flush triggered by SakuraScript / LineEndProhibited
//! - register() with None config (hardcoded defaults)
//! - Non-table actor / budoux / widths argument fallbacks

use crate::common;

use common::{create_sakura_test_runtime, create_sakura_test_runtime_with_config};
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script::tokenizer::{CharSets, Token, TokenKind, Tokenizer};
use pasta_lua::sakura_script::wait_inserter::{WaitValues, insert_waits};

fn default_tokenizer() -> Tokenizer {
    Tokenizer::new(&TalkConfig::default()).unwrap()
}

/// Same wait values as the wait_inserter unit tests:
/// effective waits are normal=50, period=950, comma=450, strong=450, leader=150.
fn test_wait_values() -> WaitValues {
    WaitValues {
        normal: 100,
        period: 1000,
        comma: 500,
        strong: 500,
        leader: 200,
    }
}

// ============================================
// Tokenizer: dangling backslash / bracket edge cases
// ============================================

#[test]
fn test_tokenize_lone_backslash_at_end_is_general() {
    // A trailing backslash matches no tag pattern and must fall through
    // to single-character classification (General), not be dropped or panic.
    let tokens = default_tokenizer().tokenize("あ\\");

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::General);
    assert_eq!(tokens[0].text, "あ");
    assert_eq!(tokens[1].kind, TokenKind::General);
    assert_eq!(tokens[1].text, "\\");
}

#[test]
fn test_tokenize_backslash_before_non_tag_char_is_general() {
    // Backslash followed by a character outside the tag charset is not a tag.
    let tokens = default_tokenizer().tokenize("\\あ");

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::General);
    assert_eq!(tokens[0].text, "\\");
    assert_eq!(tokens[1].kind, TokenKind::General);
    assert_eq!(tokens[1].text, "あ");
}

#[test]
fn test_tokenize_double_backslash_then_tag() {
    // r"\\h": the first backslash cannot start a tag (second char is `\`),
    // so it is a General char; the remaining r"\h" is a tag.
    let tokens = default_tokenizer().tokenize(r"\\h");

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::General);
    assert_eq!(tokens[0].text, "\\");
    assert_eq!(tokens[1].kind, TokenKind::SakuraScript);
    assert_eq!(tokens[1].text, r"\h");
}

#[test]
fn test_tokenize_unclosed_bracket_not_part_of_tag() {
    // r"\s[0" — the bracket part is optional in the pattern and requires a
    // closing `]`, so only r"\s" is a tag. The stray `[` is then classified
    // by char sets: ASCII `[` is in default chars_line_end_prohibited
    // ("（［｛「『([{｢") → LineEndProhibited, and `0` is General.
    let tokens = default_tokenizer().tokenize(r"\s[0");

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
    assert_eq!(tokens[0].text, r"\s");
    assert_eq!(tokens[1].kind, TokenKind::LineEndProhibited);
    assert_eq!(tokens[1].text, "[");
    assert_eq!(tokens[2].kind, TokenKind::General);
    assert_eq!(tokens[2].text, "0");
}

#[test]
fn test_tokenize_empty_bracket_param() {
    // `[^\]]*` allows an empty parameter: r"\s[]" is one complete tag.
    let tokens = default_tokenizer().tokenize(r"\s[]");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
    assert_eq!(tokens[0].text, r"\s[]");
}

#[test]
fn test_tokenize_bang_tag_with_comma_params() {
    // r"\![open,inputbox]" — `!` is in the tag name charset and commas are
    // allowed inside the bracket parameter.
    let tokens = default_tokenizer().tokenize(r"\![open,inputbox]");

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
    assert_eq!(tokens[0].text, r"\![open,inputbox]");
}

// ============================================
// CharSets::classify priority for overlapping sets
// ============================================

#[test]
fn test_classify_priority_leader_wins_over_line_start_prohibited() {
    // '･' (U+FF65) appears in BOTH default chars_leader ("･・‥…") and
    // chars_line_start_prohibited ("...｣､･ｰﾞﾟ"). classify() checks Leader
    // first, so Leader must win.
    let char_sets = CharSets::from_config(&TalkConfig::default());

    assert_eq!(char_sets.classify('･'), TokenKind::Leader);
}

#[test]
fn test_classify_halfwidth_comma_is_line_start_prohibited() {
    // '､' (U+FF64) is in chars_line_start_prohibited but NOT in
    // chars_comma ("、，,"), so it must classify as LineStartProhibited.
    let char_sets = CharSets::from_config(&TalkConfig::default());

    assert_eq!(char_sets.classify('､'), TokenKind::LineStartProhibited);
}

// ============================================
// Wait inserter: LineStartProhibited pending-run edge cases
// ============================================

#[test]
fn test_line_start_prohibited_only_run_gets_no_wait() {
    // A run consisting solely of LineStartProhibited has no own wait value
    // (pending max starts at 0), so 0 - 50 <= 0 means no tag is emitted.
    let tokens = vec![
        Token::new(TokenKind::LineStartProhibited, "」"),
        Token::new(TokenKind::LineStartProhibited, "』"),
    ];

    let result = insert_waits(&tokens, &test_wait_values());

    assert_eq!(result, "」』");
}

#[test]
fn test_line_start_prohibited_extends_run_without_raising_max() {
    // Comma starts the pending run (max=500); the following 」 extends the
    // run but contributes no wait of its own → flush emits comma's wait.
    let tokens = vec![
        Token::new(TokenKind::Comma, "、"),
        Token::new(TokenKind::LineStartProhibited, "」"),
    ];

    let result = insert_waits(&tokens, &test_wait_values());

    assert_eq!(result, r"、」\_w[450]");
}

#[test]
fn test_line_start_prohibited_first_then_strong_uses_strong_wait() {
    // 」 opens the pending run with max=0; the following ！ raises the max
    // to strong (500) → flush emits 450.
    let tokens = vec![
        Token::new(TokenKind::LineStartProhibited, "」"),
        Token::new(TokenKind::Strong, "！"),
    ];

    let result = insert_waits(&tokens, &test_wait_values());

    assert_eq!(result, r"」！\_w[450]");
}

// ============================================
// Wait inserter: pending flush triggered by non-punctuation tokens
// ============================================

#[test]
fn test_sakura_script_tag_flushes_pending_punctuation() {
    // The pending period must be flushed (with its wait) BEFORE the tag is
    // appended, not merged across it.
    let tokens = vec![
        Token::new(TokenKind::Period, "。"),
        Token::new(TokenKind::SakuraScript, r"\h"),
        Token::new(TokenKind::General, "あ"),
    ];

    let result = insert_waits(&tokens, &test_wait_values());

    assert_eq!(result, r"。\_w[950]\hあ\_w[50]");
}

#[test]
fn test_line_end_prohibited_flushes_pending_punctuation() {
    let tokens = vec![
        Token::new(TokenKind::Period, "。"),
        Token::new(TokenKind::LineEndProhibited, "「"),
        Token::new(TokenKind::General, "あ"),
    ];

    let result = insert_waits(&tokens, &test_wait_values());

    assert_eq!(result, r"。\_w[950]「あ\_w[50]");
}

// ============================================
// register() with None config — hardcoded defaults
// ============================================

#[test]
fn test_register_with_none_config_uses_defaults() {
    let lua = mlua::Lua::new();
    let module = pasta_lua::sakura_script::register(&lua, None).unwrap();

    let talk_to_script: mlua::Function = module.get("talk_to_script").unwrap();
    let result: String = talk_to_script
        .call((mlua::Value::Nil, "あ。"))
        .unwrap();

    // Default normal=50 → effective 0 (no tag); period=1000 → \_w[950]
    assert_eq!(result, r"あ。\_w[950]");
}

// ============================================
// Lua API: non-table actor / budoux / widths fallbacks
// ============================================

#[test]
fn test_actor_string_value_falls_back_to_config() {
    let config = TalkConfig {
        script_wait_normal: 100, // effective: 50
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // A non-table, non-nil actor (string) must use config defaults.
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script("actor-name", "あ")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, r"あ\_w[50]");
}

#[test]
fn test_actor_wait_below_threshold_suppresses_tags() {
    // Actor-level override below the 50ms floor → effective wait <= 0 → no tag,
    // even though the config value (100) would have produced one.
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { script_wait_normal = 30 }
            return SAKURA.talk_to_script(actor, "あ")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "あ");
}

#[test]
fn test_actor_budoux_empty_table_no_line_breaks() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { budoux = {} }
            return SAKURA.talk_to_script(actor, "今日はいい天気ですね")
        "#,
        )
        .eval()
        .unwrap();

    assert!(
        !result.contains("\\n"),
        "Empty budoux array must not insert line breaks: {}",
        result
    );
}

#[test]
fn test_actor_budoux_non_table_value_ignored() {
    let lua = create_sakura_test_runtime();

    // budoux = 6 (number, not a table) must be ignored — no line breaks.
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { budoux = 6 }
            return SAKURA.talk_to_script(actor, "今日はいい天気ですね")
        "#,
        )
        .eval()
        .unwrap();

    assert!(
        !result.contains("\\n"),
        "Non-table budoux value must not insert line breaks: {}",
        result
    );
}

#[test]
fn test_actor_budoux_non_numeric_entry_raises_error() {
    let lua = create_sakura_test_runtime();

    // Documented behavior: a budoux array entry that cannot convert to an
    // integer propagates as a Lua error (talk_to_script does not swallow it).
    let result = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = { budoux = {true} }
            return SAKURA.talk_to_script(actor, "今日はいい天気ですね")
        "#,
        )
        .eval::<String>();

    assert!(
        result.is_err(),
        "Non-numeric budoux entry should raise a Lua error"
    );
}

#[test]
fn test_break_lines_non_table_widths_returns_input() {
    let lua = create_sakura_test_runtime();

    // widths as string (non-table, non-nil) → input returned unchanged.
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("今日はいい天気ですね", "abc")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "今日はいい天気ですね");
}

#[test]
fn test_break_lines_tags_only_input_unchanged() {
    let lua = create_sakura_test_runtime();

    // Input consisting solely of sakura script tags has no plain chars to
    // measure → returned unchanged.
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.break_lines("\\h\\s[0]", {4})
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, r"\h\s[0]");
}
