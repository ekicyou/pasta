//! Log module for Lua - bridges Lua log calls to Rust tracing infrastructure.
//!
//! Provides the `@pasta_log` module with trace/debug/info/warn/error functions.
//! Each function accepts any Lua value, converts it to a string, captures caller
//! information from the Lua call stack, and emits a structured tracing event.
//!
//! # Example
//! ```lua
//! local log = require "@pasta_log"
//!
//! log.info("Hello from Lua!")
//! log.debug({key = "value"})
//! log.warn(42)
//! log.trace(nil) -- outputs empty string, no error
//! ```

use mlua::{Lua, Result as LuaResult, Table, Value};
use std::fmt::Write as _;

use super::persistence::lua_value_to_toml_strict;

/// Module version.
const VERSION: &str = "0.1.0";

/// Module description.
const DESCRIPTION: &str = "Lua logging bridge to Rust tracing";

/// Maximum table element count for TOML conversion.
const MAX_TABLE_ELEMENTS: usize = 1000;

/// Maximum nesting depth for TOML conversion (the root table is depth 1).
const MAX_NESTING_DEPTH: usize = 10;

/// Lua caller information extracted from the call stack.
#[derive(Debug, Clone, Default)]
struct LuaCallerInfo {
    /// Source file name (e.g., "@main.lua", "=stdin")
    source: String,
    /// Line number (0 if unavailable)
    line: usize,
    /// Function name (empty string if unavailable)
    fn_name: String,
}

/// Get caller information from the Lua call stack.
///
/// Uses `inspect_stack(1)` to capture the direct Lua caller's
/// source file, line number, and function name.
///
/// # Verification Result
/// Stack level 1 was verified in `tests/log/stack_level_test.rs`:
/// - level=0: Rust function ([C]) — not useful
/// - level=1: Direct Lua caller — correct level
///
/// Never panics or returns Lua errors.
fn get_caller_info(lua: &Lua) -> LuaCallerInfo {
    lua.inspect_stack(1, |debug| {
        let source_info = debug.source();
        let source = source_info
            .short_src
            .map(|s| s.to_string())
            .unwrap_or_default();
        let line = debug.current_line().unwrap_or(0);
        let names = debug.names();
        let fn_name = names.name.map(|s| s.to_string()).unwrap_or_default();

        LuaCallerInfo {
            source,
            line,
            fn_name,
        }
    })
    .unwrap_or_default()
}

/// Convert a Lua value to a human-readable string for logging.
///
/// Conversion rules (priority order):
/// - `String` → as-is
/// - `Integer`, `Number`, `Boolean` → tostring() equivalent
/// - `Table` → one-line TOML value if ≤1000 elements and convertible within 10 levels, otherwise fallback
/// - `Nil`, no argument → empty string `""`
/// - `Function`, `UserData`, `Thread` etc. → tostring() fallback
/// - Final fallback → `"<unconvertible value>"`
///
/// Never panics or returns errors.
fn value_to_string(lua: &Lua, value: Value) -> String {
    match value {
        Value::Nil => String::new(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string_lossy().to_string(),
        Value::Table(ref t) => table_to_string(lua, t, &value),
        // Function, UserData, Thread, etc.
        _ => lua_tostring(lua, &value),
    }
}

/// Convert a Lua table to a string representation.
///
/// Gates in order: >1000 elements → `<table: N elements>`; then the shared
/// strict conversion (depth-bounded recursion, so deep input cannot overflow the
/// host stack; cycles and unwritable values fail) → one-line TOML value;
/// on failure → tostring().
fn table_to_string(lua: &Lua, table: &Table, original_value: &Value) -> String {
    let element_count = count_table_elements(table);
    if element_count > MAX_TABLE_ELEMENTS {
        return format!("<table: {} elements>", element_count);
    }
    match lua_value_to_toml_strict(lua, original_value, MAX_NESTING_DEPTH) {
        Ok(v) => {
            let mut out = String::new();
            write_inline(&v, &mut out);
            out
        }
        Err(_) => lua_tostring(lua, original_value),
    }
}

/// Count the number of key-value pairs in a Lua table.
fn count_table_elements(table: &Table) -> usize {
    table.pairs::<Value, Value>().count()
}

/// Write a TOML value on one line (inline tables with ascending keys, arrays,
/// strings always as single-line basic strings).
fn write_inline(value: &toml::Value, out: &mut String) {
    match value {
        toml::Value::String(s) => write_basic_string(s, out),
        toml::Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_inline(item, out);
            }
            out.push(']');
        }
        toml::Value::Table(t) if t.is_empty() => out.push_str("{}"),
        toml::Value::Table(t) => {
            let mut entries: Vec<_> = t.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            out.push_str("{ ");
            for (i, (k, v)) in entries.into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let bare = !k.is_empty()
                    && k.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                if bare {
                    out.push_str(k);
                } else {
                    write_basic_string(k, out);
                }
                out.push_str(" = ");
                write_inline(v, out);
            }
            out.push_str(" }");
        }
        // Integer・Float（1.0・nan・inf）・Boolean・Datetime は toml の書き方のまま
        other => {
            let _ = write!(out, "{other}");
        }
    }
}

/// Write `s` as a TOML basic string `"…"` (never multi-line; control chars escaped).
fn write_basic_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if c <= '\u{1f}' || c == '\u{7f}' => {
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Convert any Lua value to string using Lua's tostring().
///
/// Falls back to `"<unconvertible value>"` if tostring() fails.
fn lua_tostring(lua: &Lua, value: &Value) -> String {
    let tostring: Result<mlua::Function, _> = lua.globals().get("tostring");
    match tostring {
        Ok(func) => match func.call::<String>(value.clone()) {
            Ok(s) => s,
            Err(_) => "<unconvertible value>".to_string(),
        },
        Err(_) => "<unconvertible value>".to_string(),
    }
}

/// Register the `@pasta_log` module with the Lua state.
///
/// Creates a module table with:
/// - `_VERSION` - Module version string
/// - `_DESCRIPTION` - Module description
/// - `trace(value)` - Log at TRACE level
/// - `debug(value)` - Log at DEBUG level
/// - `info(value)` - Log at INFO level
/// - `warn(value)` - Log at WARN level
/// - `error(value)` - Log at ERROR level
///
/// # Arguments
/// * `lua` - The Lua state to register the module with
///
/// # Returns
/// * `Ok(Table)` - The module table
/// * `Err(e)` - Registration failed
pub fn register(lua: &Lua) -> LuaResult<Table> {
    let module = lua.create_table()?;

    // Set module metadata
    module.set("_VERSION", VERSION)?;
    module.set("_DESCRIPTION", DESCRIPTION)?;

    // Register log level functions
    module.set("trace", lua.create_function(log_trace)?)?;
    module.set("debug", lua.create_function(log_debug)?)?;
    module.set("info", lua.create_function(log_info)?)?;
    module.set("warn", lua.create_function(log_warn)?)?;
    module.set("error", lua.create_function(log_error)?)?;

    Ok(module)
}

/// Generate one Lua-callable log function for a tracing level.
///
/// The five level functions are identical except for the `tracing` macro they
/// dispatch to (the level must be known at compile time), so they are stamped
/// out by this macro to keep them in sync.
macro_rules! log_fn {
    ($fn_name:ident => $level:ident) => {
        #[doc = concat!("Log at `", stringify!($level), "` level.")]
        fn $fn_name(lua: &Lua, value: Value) -> LuaResult<()> {
            let msg = value_to_string(lua, value);
            let caller = get_caller_info(lua);
            tracing::$level!(
                lua_source = %caller.source,
                lua_line = caller.line,
                lua_fn = %caller.fn_name,
                "{}",
                msg
            );
            Ok(())
        }
    };
}

log_fn!(log_trace => trace);
log_fn!(log_debug => debug);
log_fn!(log_info => info);
log_fn!(log_warn => warn);
log_fn!(log_error => error);

#[cfg(test)]
mod tests {
    use super::*;

    fn inline(v: toml::Value) -> String {
        let mut out = String::new();
        write_inline(&v, &mut out);
        out
    }

    #[test]
    fn empty_array_and_table() {
        assert_eq!(inline(toml::Value::Array(vec![])), "[]");
        assert_eq!(inline(toml::Value::Table(toml::Table::new())), "{}");
    }

    #[test]
    fn strings_escape_control_chars_on_one_line() {
        let s = "q\"b\\n\nt\tr\r\u{8}\u{c}\u{1}\u{7f}日";
        assert_eq!(
            inline(toml::Value::String(s.into())),
            r#""q\"b\\n\nt\tr\r\b\f\u0001\u007F日""#
        );
    }

    #[test]
    fn keys_quoted_unless_bare() {
        let mut t = toml::Table::new();
        t.insert("a-b_1".into(), toml::Value::Integer(1));
        t.insert("".into(), toml::Value::Integer(2));
        t.insert("a b".into(), toml::Value::Integer(3));
        assert_eq!(
            inline(toml::Value::Table(t)),
            r#"{ "" = 2, "a b" = 3, a-b_1 = 1 }"#
        );
    }

    #[test]
    fn scalars_in_toml_notation() {
        assert_eq!(inline(toml::Value::Float(1.0)), "1.0");
        assert_eq!(inline(toml::Value::Float(f64::NAN)), "nan");
        assert_eq!(inline(toml::Value::Float(f64::NEG_INFINITY)), "-inf");
        assert_eq!(inline(toml::Value::Integer(-5)), "-5");
        assert_eq!(inline(toml::Value::Boolean(true)), "true");
        let dt: toml::value::Datetime = "1979-05-27T07:32:00Z".parse().unwrap();
        assert_eq!(inline(toml::Value::Datetime(dt)), "1979-05-27T07:32:00Z");
    }

    /// 書いた 1 行を toml で読み戻すと元の値に戻る（エスケープ・引用キー・特殊な数の確かめ）
    #[test]
    fn output_parses_back_as_toml() {
        use toml::Value as V;
        let mut inner = toml::Table::new();
        inner.insert(
            "s".into(),
            V::String("q\"b\\\n\t\r\u{8}\u{c}\u{1}\u{1f}\u{7f}日本".into()),
        );
        inner.insert("".into(), V::Integer(0));
        inner.insert("a b".into(), V::Boolean(false));
        inner.insert("a.b".into(), V::Float(1.0));
        inner.insert(
            "キー".into(),
            V::Array(vec![V::Integer(-5), V::Table(toml::Table::new())]),
        );
        inner.insert(
            "dt".into(),
            V::Datetime("1979-05-27T07:32:00Z".parse().unwrap()),
        );
        let mut root = toml::Table::new();
        root.insert("nested".into(), V::Table(inner.clone()));
        root.insert("nan".into(), V::Float(f64::NAN));
        root.insert("ninf".into(), V::Float(f64::NEG_INFINITY));
        root.insert("nzero".into(), V::Float(-0.0));

        let out = inline(V::Table(root));
        let mut parsed = toml::from_str::<toml::Table>(&format!("x = {out}"))
            .unwrap_or_else(|e| panic!("not TOML: {out}\n{e}"));
        let V::Table(mut x) = parsed.remove("x").unwrap() else {
            panic!("x is not a table: {out}");
        };
        assert!(x.remove("nan").unwrap().as_float().unwrap().is_nan());
        assert_eq!(x.remove("ninf"), Some(V::Float(f64::NEG_INFINITY)));
        let nzero = x.remove("nzero").unwrap().as_float().unwrap();
        assert!(nzero == 0.0 && nzero.is_sign_negative(), "{out}");
        assert_eq!(x.remove("nested"), Some(V::Table(inner)));
        assert!(x.is_empty());
    }
}
