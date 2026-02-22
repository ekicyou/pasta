//! Integration tests for sakura script output and configuration.
//!
//! Covers: output examples (Requirement 7), TalkConfig TOML integration,
//! and comprehensive wait value verification.

use crate::common;

use common::{create_sakura_test_runtime_with_config};
use pasta_lua::loader::TalkConfig;

// ============================================
// Requirement 7: Output Examples
// ============================================

#[test]
fn test_requirement_7_1_consecutive_punctuation() {
    // 」」」！？。、 with script_wait_period=1000 -> 」」」！？。、\_w[950]
    let config = TalkConfig {
        script_wait_period: 1000,
        script_wait_comma: 500,
        script_wait_strong: 500,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "」」」！？。、")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, r"」」」！？。、\_w[950]");
}

#[test]
fn test_requirement_7_2_general_text() {
    // こんにちは with script_wait_normal=100 -> \_w[50] after each char
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "こんにちは")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, r"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]");
}

#[test]
fn test_requirement_7_3_sakura_script_preserved() {
    // \h\s[0]こんにちは -> sakura tags preserved, wait on text only
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "\\h\\s[0]こんにちは")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(
        result,
        r"\h\s[0]こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]"
    );
}

// ============================================
// TalkConfig TOML Integration
// ============================================

#[test]
fn test_talk_config_from_toml_full() {
    // PastaConfig uses its own parser, tested via config.rs unit tests
    // Here we test TalkConfig can be deserialized directly
    let toml_str = r#"
script_wait_normal = 80
script_wait_period = 1200
script_wait_comma = 600
script_wait_strong = 700
script_wait_leader = 250
chars_period = "。."
chars_comma = "、,"
chars_strong = "！？"
chars_leader = "…"
chars_line_start_prohibited = "」』"
chars_line_end_prohibited = "「『"
"#;

    let talk: TalkConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(talk.script_wait_normal, 80);
    assert_eq!(talk.script_wait_period, 1200);
    assert_eq!(talk.script_wait_comma, 600);
    assert_eq!(talk.script_wait_strong, 700);
    assert_eq!(talk.script_wait_leader, 250);
    assert_eq!(talk.chars_period, "。.");
    assert_eq!(talk.chars_comma, "、,");
    assert_eq!(talk.chars_strong, "！？");
    assert_eq!(talk.chars_leader, "…");
    assert_eq!(talk.chars_line_start_prohibited, "」』");
    assert_eq!(talk.chars_line_end_prohibited, "「『");
}

#[test]
fn test_talk_config_partial_override() {
    let toml_str = r#"
script_wait_normal = 80
"#;

    let talk: TalkConfig = toml::from_str(toml_str).unwrap();

    // Only normal is overridden
    assert_eq!(talk.script_wait_normal, 80);
    // Others use defaults
    assert_eq!(talk.script_wait_period, 1000);
    assert_eq!(talk.script_wait_comma, 500);
    assert_eq!(talk.chars_period, "｡。．.");
}

#[test]
fn test_talk_config_default() {
    let config = TalkConfig::default();

    assert_eq!(config.script_wait_normal, 50);
    assert_eq!(config.script_wait_period, 1000);
    assert_eq!(config.script_wait_comma, 500);
    assert_eq!(config.script_wait_strong, 500);
    assert_eq!(config.script_wait_leader, 200);
    assert_eq!(config.chars_period, "｡。．.");
    assert_eq!(config.chars_comma, "、，,");
    assert_eq!(config.chars_strong, "？！!?");
    assert_eq!(config.chars_leader, "･・‥…");
}

// ============================================
// Comprehensive Wait Value Tests
// ============================================

/// Test that all wait types (normal, comma, period, strong, leader) are correctly applied.
/// Input: "あ、い。う！え‥‥お、、、。"
/// This covers: General, Comma, Period, Strong, Leader, and consecutive punctuation.
#[test]
fn test_all_wait_types_comprehensive() {
    let config = TalkConfig {
        script_wait_normal: 100, // effective: 50
        script_wait_comma: 200,  // effective: 150
        script_wait_period: 300, // effective: 250
        script_wait_strong: 400, // effective: 350
        script_wait_leader: 250, // effective: 200
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // Test individual punctuation types with distinct wait values
    // あ(General) 、(Comma) い(General) 。(Period) う(General) ！(Strong) え(General) ‥‥(Leader×2) お(General) 、、、。(Consecutive: max=Period)
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ、い。う！え‥‥お、、、。")
        "#,
        )
        .eval()
        .unwrap();

    // Expected breakdown:
    // あ -> \_w[50] (normal: 100-50)
    // 、 -> \_w[150] (comma: 200-50)
    // い -> \_w[50]
    // 。 -> \_w[250] (period: 300-50)
    // う -> \_w[50]
    // ！ -> \_w[350] (strong: 400-50)
    // え -> \_w[50]
    // ‥ -> \_w[200] (leader: 250-50)
    // ‥ -> \_w[200]
    // お -> \_w[50]
    // 、、、。 -> consecutive, max is period(300), so \_w[250]
    assert_eq!(
        result,
        r"あ\_w[50]、\_w[150]い\_w[50]。\_w[250]う\_w[50]！\_w[350]え\_w[50]‥\_w[200]‥\_w[200]お\_w[50]、、、。\_w[250]"
    );
}

/// Test each punctuation type individually to verify correct wait value selection.
#[test]
fn test_individual_punctuation_wait_values() {
    let config = TalkConfig {
        script_wait_normal: 100, // effective: 50
        script_wait_comma: 200,  // effective: 150
        script_wait_period: 300, // effective: 250
        script_wait_strong: 400, // effective: 350
        script_wait_leader: 250, // effective: 200
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // Test comma individually
    let comma_result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ、")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(comma_result, r"あ\_w[50]、\_w[150]");

    // Test period individually
    let period_result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ。")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(period_result, r"あ\_w[50]。\_w[250]");

    // Test strong (exclamation) individually
    let strong_result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ！")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(strong_result, r"あ\_w[50]！\_w[350]");

    // Test strong (question) individually
    let question_result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ？")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(question_result, r"あ\_w[50]？\_w[350]");

    // Test leader individually
    let leader_result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ…")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(leader_result, r"あ\_w[50]…\_w[200]");
}

/// Test mixed consecutive punctuation to verify max wait selection.
#[test]
fn test_consecutive_mixed_punctuation_max_selection() {
    let config = TalkConfig {
        script_wait_comma: 200,  // effective: 150
        script_wait_period: 300, // effective: 250
        script_wait_strong: 400, // effective: 350
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // Comma followed by period: max is period
    let result1: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "、。")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result1, r"、。\_w[250]");

    // Strong followed by comma: max is strong
    let result2: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "！、")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result2, r"！、\_w[350]");

    // Multiple comma: max is still comma
    let result3: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "、、、")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result3, r"、、、\_w[150]");
}
