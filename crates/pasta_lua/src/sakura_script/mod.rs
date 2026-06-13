//! Sakura Script Module - Wait insertion for talk text.
//!
//! This module provides the `@pasta_sakura_script` Lua module for inserting
//! sakura script wait tags (`\_w[ms]`) into conversation text.
//!
//! # Usage from Lua
//!
//! ```lua
//! local SAKURA = require "@pasta_sakura_script"
//!
//! -- Basic usage
//! local script = SAKURA.talk_to_script(CONFIG.actor.sakura, "こんにちは。")
//! -- → "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]。\_w[950]"
//!
//! -- With nil actor (uses pasta.toml defaults)
//! local script = SAKURA.talk_to_script(nil, "こんにちは。")
//! ```

pub mod tokenizer;
pub mod wait_inserter;
pub mod line_breaker;

use crate::loader::TalkConfig;
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::sync::Arc;
use tokenizer::Tokenizer;
use wait_inserter::WaitValues;

/// Module version.
const VERSION: &str = "1.0.0";

/// Module description.
const DESCRIPTION: &str = "Sakura Script wait insertion module for natural conversation tempo";

/// Internal state for the sakura script module.
struct SakuraScriptState {
    tokenizer: Tokenizer,
    default_wait_values: WaitValues,
    budoux_parser: budouy::Parser,
}

/// Register the `@pasta_sakura_script` module to Lua.
///
/// # Arguments
/// * `lua` - Lua state
/// * `config` - TalkConfig (uses hardcoded defaults if None)
///
/// # Returns
/// Module table containing `talk_to_script` and `break_lines` functions
pub fn register(lua: &Lua, config: Option<&TalkConfig>) -> LuaResult<Table> {
    let config = config.cloned().unwrap_or_default();

    // Initialize tokenizer with regex (compile once at registration time)
    let tokenizer = Tokenizer::new(&config).map_err(|e| {
        mlua::Error::external(format!("Failed to compile sakura script regex: {}", e))
    })?;

    let default_wait_values = WaitValues::from_config(&config);

    // Store state in Arc for sharing with closure
    let state = Arc::new(SakuraScriptState {
        tokenizer,
        default_wait_values,
        budoux_parser: budouy::model::load_default_japanese_parser(),
    });

    // Create module table
    let module = lua.create_table()?;

    // Add metadata
    module.set("_VERSION", VERSION)?;
    module.set("_DESCRIPTION", DESCRIPTION)?;

    // Create talk_to_script function
    let state_clone = Arc::clone(&state);
    let talk_to_script =
        lua.create_function(move |_lua, (actor, talk): (Value, Option<String>)| {
            talk_to_script_impl(&state_clone, actor, talk)
        })?;

    module.set("talk_to_script", talk_to_script)?;

    // Create break_lines function
    let state_clone = Arc::clone(&state);
    let break_lines =
        lua.create_function(move |_lua, (text, widths): (Option<String>, Value)| {
            break_lines_lua_impl(&state_clone, text, widths)
        })?;

    module.set("break_lines", break_lines)?;

    Ok(module)
}

/// Implementation of talk_to_script function.
fn talk_to_script_impl(
    state: &SakuraScriptState,
    actor: Value,
    talk: Option<String>,
) -> LuaResult<String> {
    // Requirement 6.1: Return empty string for nil or empty input
    let talk = match talk {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(String::new()),
    };

    // Resolve wait values from actor table or use defaults
    let wait_values = resolve_wait_values(&actor, &state.default_wait_values)?;

    // Tokenize the input
    let tokens = state.tokenizer.tokenize(&talk);

    // Insert waits
    let result = wait_inserter::insert_waits(&tokens, &wait_values);

    // Apply budoux line breaking if actor has budoux config
    let result = apply_budoux_if_configured(&actor, state, result)?;

    Ok(result)
}

/// Apply budoux line breaking if the actor table has a `budoux` field.
fn apply_budoux_if_configured(
    actor: &Value,
    state: &SakuraScriptState,
    text: String,
) -> LuaResult<String> {
    let actor_table = match actor {
        Value::Table(t) => t,
        _ => return Ok(text),
    };

    let widths: Vec<usize> = match actor_table.get::<Value>("budoux")? {
        Value::Table(t) => table_to_widths(&t)?,
        _ => return Ok(text),
    };

    if widths.is_empty() {
        return Ok(text);
    }

    Ok(line_breaker::break_lines_impl(
        &text,
        &widths,
        state.tokenizer.tag_regex(),
        &state.budoux_parser,
    ))
}

/// Convert a Lua array table of widths to `Vec<usize>`.
///
/// Non-integer entries propagate as a Lua error.
fn table_to_widths(t: &Table) -> LuaResult<Vec<usize>> {
    let mut v = Vec::new();
    for i in 1..=t.raw_len() {
        let w: i64 = t.get(i)?;
        v.push(w as usize);
    }
    Ok(v)
}

/// Resolve wait values from actor table with fallback to defaults.
///
/// 3-level fallback: actor -> config -> hardcoded
fn resolve_wait_values(actor: &Value, defaults: &WaitValues) -> LuaResult<WaitValues> {
    // If actor is nil or not a table, use defaults
    let actor_table = match actor {
        Value::Table(t) => t,
        _ => return Ok(defaults.clone()),
    };

    // Helper to get i64 from actor table with fallback
    let get_wait =
        |key: &str, default: i64| -> i64 { actor_table.get::<i64>(key).ok().unwrap_or(default) };

    Ok(WaitValues {
        normal: get_wait("script_wait_normal", defaults.normal),
        period: get_wait("script_wait_period", defaults.period),
        comma: get_wait("script_wait_comma", defaults.comma),
        strong: get_wait("script_wait_strong", defaults.strong),
        leader: get_wait("script_wait_leader", defaults.leader),
    })
}

/// Implementation of break_lines Lua function.
fn break_lines_lua_impl(
    state: &SakuraScriptState,
    text: Option<String>,
    widths: Value,
) -> LuaResult<String> {
    let text = match text {
        Some(s) if !s.is_empty() => s,
        _ => return Ok(String::new()),
    };

    // Convert Lua table to Vec<usize>
    let widths_vec: Vec<usize> = match widths {
        Value::Table(t) => table_to_widths(&t)?,
        _ => return Ok(text),
    };

    if widths_vec.is_empty() {
        return Ok(text);
    }

    let result = line_breaker::break_lines_impl(
        &text,
        &widths_vec,
        state.tokenizer.tag_regex(),
        &state.budoux_parser,
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creates_module_table() {
        let lua = Lua::new();
        let config = TalkConfig::default();
        let module = register(&lua, Some(&config)).unwrap();

        assert!(module.contains_key("_VERSION").unwrap());
        assert!(module.contains_key("_DESCRIPTION").unwrap());
        assert!(module.contains_key("talk_to_script").unwrap());
        assert!(module.contains_key("break_lines").unwrap());
    }

    #[test]
    fn test_version_and_description() {
        let lua = Lua::new();
        let config = TalkConfig::default();
        let module = register(&lua, Some(&config)).unwrap();

        let version: String = module.get("_VERSION").unwrap();
        assert_eq!(version, "1.0.0");

        let desc: String = module.get("_DESCRIPTION").unwrap();
        assert!(!desc.is_empty());
    }
}
