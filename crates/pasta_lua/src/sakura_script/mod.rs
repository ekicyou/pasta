//! Sakura Script Module - Wait insertion and budoux line breaking for talk text.
//!
//! This module provides the `@pasta_sakura_script` Lua module for inserting
//! sakura script wait tags (`\_w[ms]`) into conversation text and, when the
//! actor table has a `budoux` width array, line break tags (`\n`) at budoux
//! word boundaries.
//!
//! # Usage from Lua
//!
//! ```lua
//! local SAKURA = require "@pasta_sakura_script"
//!
//! -- Actor table with script_wait_normal = 100 (effective wait = value - 50)
//! local script = SAKURA.talk_to_script(CONFIG.actor.sakura, "こんにちは。")
//! -- → "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]。\_w[950]"
//!
//! -- With nil actor (uses [talk] values or built-in defaults, no line breaking)
//! local script = SAKURA.talk_to_script(nil, "こんにちは。")
//! -- → "こんにちは。\_w[950]" (default script_wait_normal = 50 inserts no per-char wait)
//!
//! -- 3 つ目の引数に列の状態の表 { line, width, jump } を渡すと、始めの状態として読み、
//! -- 成功した後に終わりの状態を書き戻す（無いフィールドは 1・0・偽）。
//! -- budoux の無いアクター・nil でも、列の状態は進める（改行は入れない）。
//! local col = {}
//! SAKURA.talk_to_script(CONFIG.actor.sakura, "こんにちは。", col)
//! ```
//!
//! アクターの `budoux` が空でない表のときだけ、`budoux_newlines`（改行の幅の倍率。
//! `\n[N]` の N は 100 倍して切り捨て、100 なら `\n`）と `budoux_jump`（`\_l[…]` の後の幅の並び）を読む。

pub mod line_breaker;
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
    let talk_to_script = lua.create_function(
        move |_lua, (actor, talk, column): (Value, Option<String>, Option<Table>)| {
            talk_to_script_impl(&state_clone, actor, talk, column)
        },
    )?;

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
    column: Option<Table>,
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

    // budoux の改行（列の状態を受け取ったら、budoux が無くても状態は進める）
    let ab = read_actor_break(&actor)?;
    if ab.widths.is_empty() && column.is_none() {
        return Ok(result);
    }
    let mut col = match &column {
        Some(t) => read_column(t)?,
        None => line_breaker::Column::default(),
    };
    let rule = line_breaker::BreakRule {
        widths: &ab.widths,
        jump_widths: &ab.jump_widths,
        newline: &ab.newline,
    };
    let result = line_breaker::break_lines_impl(
        &result,
        &rule,
        &mut col,
        state.tokenizer.tag_regex(),
        &state.budoux_parser,
    );
    // 成功した後にだけ書き戻す
    if let Some(t) = &column {
        write_column(t, &col)?;
    }
    Ok(result)
}

/// アクターの表から読んだ改行の規則（`budoux`・`budoux_jump`・`budoux_newlines`）。
struct ActorBreak {
    widths: Vec<usize>,
    jump_widths: Vec<usize>,
    newline: String,
}

/// アクターの表から ActorBreak を作る。
///
/// `budoux` が空でない表のときだけ `budoux_newlines`・`budoux_jump` を読む。
/// 表でない actor・`budoux` の無いアクターは、空の並び・`\n`。
fn read_actor_break(actor: &Value) -> LuaResult<ActorBreak> {
    let mut ab = ActorBreak {
        widths: Vec::new(),
        jump_widths: Vec::new(),
        newline: newline_for_ratio(None),
    };
    let Value::Table(t) = actor else {
        return Ok(ab);
    };
    if let Value::Table(w) = t.get::<Value>("budoux")? {
        ab.widths = table_to_widths(&w)?;
    }
    if ab.widths.is_empty() {
        return Ok(ab);
    }
    let ratio = t.get::<Option<f64>>("budoux_newlines").map_err(|e| {
        mlua::Error::external(format!("budoux_newlines は数値で書いてください: {e}"))
    })?;
    ab.newline = newline_for_ratio(ratio);
    if let Value::Table(j) = t.get::<Value>("budoux_jump")? {
        ab.jump_widths = table_to_widths(&j)?;
    }
    Ok(ab)
}

/// 倍率から改行の文字列を作る。None・N が 100 なら `\n`、それ以外は `\n[N]`。
/// N は Lua の `math.floor(ratio * 100)` と同じ値。
fn newline_for_ratio(ratio: Option<f64>) -> String {
    match ratio.map(|r| (r * 100.0).floor() as i64) {
        None | Some(100) => r"\n".to_string(),
        Some(n) => format!(r"\n[{n}]"),
    }
}

/// Lua の表から列の状態を読む。無いフィールドは line = 1・width = 0・jump = false。
fn read_column(t: &Table) -> LuaResult<line_breaker::Column> {
    Ok(line_breaker::Column {
        line: t.get::<Option<i64>>("line")?.unwrap_or(1).max(1) as usize,
        width: t.get::<Option<i64>>("width")?.unwrap_or(0).max(0) as usize,
        jump: t.get::<bool>("jump")?,
    })
}

/// 列の状態を Lua の表に書く。
fn write_column(t: &Table, c: &line_breaker::Column) -> LuaResult<()> {
    t.set("line", c.line)?;
    t.set("width", c.width)?;
    t.set("jump", c.jump)
}

/// 新しい列の状態・`\n`・飛んだ後の並び無しで budoux の改行を入れる（`break_lines`）。
fn break_fresh(state: &SakuraScriptState, text: &str, widths: &[usize]) -> String {
    let rule = line_breaker::BreakRule {
        widths,
        jump_widths: &[],
        newline: r"\n",
    };
    line_breaker::break_lines_impl(
        text,
        &rule,
        &mut line_breaker::Column::default(),
        state.tokenizer.tag_regex(),
        &state.budoux_parser,
    )
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

    Ok(break_fresh(state, &text, &widths_vec))
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
