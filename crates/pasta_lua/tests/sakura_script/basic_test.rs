//! Integration tests for sakura script basic functionality.
//!
//! Covers: Lua module exposure, actor parameter reference,
//! token decomposition, wait insertion rules, and error handling.

use crate::common;

use common::{create_sakura_test_runtime, create_sakura_test_runtime_with_config};
use pasta_lua::loader::TalkConfig;

// ============================================
// Requirement 1: Lua Module Exposure
// ============================================

#[test]
fn test_require_pasta_sakura_script() {
    let lua = create_sakura_test_runtime();

    let result: mlua::Table = lua
        .load(r#"return require "@pasta_sakura_script""#)
        .eval()
        .unwrap();

    assert!(result.contains_key("talk_to_script").unwrap());
    assert!(result.contains_key("_VERSION").unwrap());
    assert!(result.contains_key("_DESCRIPTION").unwrap());
}

#[test]
fn test_talk_to_script_basic() {
    let lua = create_sakura_test_runtime();

    // Default config: script_wait_normal = 50 (effective: 0, no wait)
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "あ")
        "#,
        )
        .eval()
        .unwrap();

    // With default normal=50, effective wait is 0, so no wait tag
    assert_eq!(result, "あ");
}

#[test]
fn test_talk_to_script_with_custom_normal_wait() {
    let config = TalkConfig {
        script_wait_normal: 100, // effective: 50
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

// ============================================
// Requirement 2: Actor Parameter Reference
// ============================================

#[test]
fn test_actor_wait_override() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = {
                script_wait_normal = 100,  -- effective: 50
                script_wait_period = 500   -- effective: 450
            }
            return SAKURA.talk_to_script(actor, "あ。")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, r"あ\_w[50]。\_w[450]");
}

#[test]
fn test_actor_partial_override() {
    // Actor only overrides normal, others use defaults
    let config = TalkConfig {
        script_wait_period: 1000, // default
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = {
                script_wait_normal = 100  -- only override normal
            }
            return SAKURA.talk_to_script(actor, "あ。")
        "#,
        )
        .eval()
        .unwrap();

    // normal: 100-50=50, period: 1000-50=950
    assert_eq!(result, r"あ\_w[50]。\_w[950]");
}

#[test]
fn test_actor_non_numeric_fallback() {
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // Actor has non-numeric value, should fallback to config
    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            local actor = {
                script_wait_normal = "not a number"  -- invalid type
            }
            return SAKURA.talk_to_script(actor, "あ")
        "#,
        )
        .eval()
        .unwrap();

    // Falls back to config value (100 - 50 = 50)
    assert_eq!(result, r"あ\_w[50]");
}

// ============================================
// Requirement 4: Token Decomposition
// ============================================

#[test]
fn test_sakura_script_tag_preserved() {
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

    // Sakura script tags should be preserved without wait
    assert_eq!(
        result,
        r"\h\s[0]こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]"
    );
}

#[test]
fn test_complex_sakura_tag() {
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "\\_w[500]あ")
        "#,
        )
        .eval()
        .unwrap();

    // \_w[500] is a sakura script tag, preserved
    assert_eq!(result, r"\_w[500]あ\_w[50]");
}

// ============================================
// Requirement 5: Wait Insertion Rules
// ============================================

#[test]
fn test_consecutive_punctuation_max_wait() {
    let config = TalkConfig {
        script_wait_period: 1000,
        script_wait_comma: 500,
        script_wait_strong: 500,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    // Requirement 7.1: 」」」！？。、 -> 」」」！？。、\_w[950]
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
fn test_leader_per_char_wait() {
    let config = TalkConfig {
        script_wait_leader: 200, // effective: 150
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "……")
        "#,
        )
        .eval()
        .unwrap();

    // Each leader character gets its own wait
    assert_eq!(result, r"…\_w[150]…\_w[150]");
}

#[test]
fn test_line_end_prohibited_no_wait() {
    let config = TalkConfig {
        script_wait_normal: 100,
        ..Default::default()
    };
    let lua = create_sakura_test_runtime_with_config(&config);

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "「あ")
        "#,
        )
        .eval()
        .unwrap();

    // 「 is line-end prohibited, no wait after it
    assert_eq!(result, r"「あ\_w[50]");
}

// ============================================
// Requirement 6: Error Handling
// ============================================

#[test]
fn test_nil_talk_returns_empty() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, nil)
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_empty_talk_returns_empty() {
    let lua = create_sakura_test_runtime();

    let result: String = lua
        .load(
            r#"
            local SAKURA = require "@pasta_sakura_script"
            return SAKURA.talk_to_script(nil, "")
        "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "");
}
