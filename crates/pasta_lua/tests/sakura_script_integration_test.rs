//! Integration tests for sakura script wait insertion.
//!
//! Tests the full flow from Lua API calls to wait-inserted output.

use mlua::Lua;
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script;

/// Helper to create a Lua runtime with sakura_script module registered.
fn create_test_runtime() -> Lua {
    let lua = Lua::new();
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();

    // Register as @pasta_sakura_script
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    lua
}

/// Helper to create runtime with custom TalkConfig.
fn create_test_runtime_with_config(config: &TalkConfig) -> Lua {
    let lua = Lua::new();
    let module = sakura_script::register(&lua, Some(config)).unwrap();

    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    lua
}

// ============================================
// Requirement 1: Lua Module Exposure
// ============================================

#[test]
fn test_require_pasta_sakura_script() {
    let lua = create_test_runtime();

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
    let lua = create_test_runtime();

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime();

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime();

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
    let lua = create_test_runtime();

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
    let lua = create_test_runtime_with_config(&config);

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
