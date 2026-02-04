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
}

/// Register the `@pasta_sakura_script` module to Lua.
///
/// # Arguments
/// * `lua` - Lua state
/// * `config` - TalkConfig (uses hardcoded defaults if None)
///
/// # Returns
/// Module table containing `talk_to_script` function
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
    });

    // Create module table
    let module = lua.create_table()?;

    // Add metadata
    module.set("_VERSION", VERSION)?;
    module.set("_DESCRIPTION", DESCRIPTION)?;

    // Create talk_to_script function
    let state_clone = Arc::clone(&state);
    let talk_to_script =
        lua.create_function(move |lua, (actor, talk): (Value, Option<String>)| {
            talk_to_script_impl(lua, &state_clone, actor, talk)
        })?;

    module.set("talk_to_script", talk_to_script)?;

    Ok(module)
}

/// Implementation of talk_to_script function.
fn talk_to_script_impl(
    lua: &Lua,
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
    let wait_values = resolve_wait_values(lua, &actor, &state.default_wait_values)?;

    // Tokenize the input
    let tokens = state.tokenizer.tokenize(&talk);

    // Insert waits and return result
    let result = wait_inserter::insert_waits(&tokens, &wait_values);

    Ok(result)
}

/// Resolve wait values from actor table with fallback to defaults.
///
/// 3-level fallback: actor -> config -> hardcoded
fn resolve_wait_values(_lua: &Lua, actor: &Value, defaults: &WaitValues) -> LuaResult<WaitValues> {
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
