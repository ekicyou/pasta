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

use mlua::{Lua, LuaSerdeExt, Result as LuaResult, Table, Value};

/// Module version.
const VERSION: &str = "0.1.0";

/// Module description.
const DESCRIPTION: &str = "Lua logging bridge to Rust tracing";

/// Maximum table element count for JSON conversion.
const MAX_TABLE_ELEMENTS: usize = 1000;

/// Maximum nesting depth for JSON conversion.
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
/// Stack level 1 was verified in `log_stack_level_test.rs`:
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
/// - `Table` → JSON if ≤1000 elements and ≤10 nesting depth, otherwise fallback
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
/// Tries JSON conversion first (with size and depth limits),
/// falls back to tostring() on failure.
fn table_to_string(lua: &Lua, table: &Table, original_value: &Value) -> String {
    // Check table size (count all key-value pairs)
    let element_count = count_table_elements(table);
    if element_count > MAX_TABLE_ELEMENTS {
        return format!("<table: {} elements>", element_count);
    }

    // Depth gate BEFORE the serde conversion (cell 3.17 / G3 hardening).
    // `from_value_with` walks nested tables with Rust recursion, so a deeply
    // nested table built in Lua (cheap: `for _ = 1, N do t = { t } end`) could
    // overflow the host stack — an abort reachable across the SHIORI boundary
    // (R3.7). The old post-conversion `json_depth` check ran too late to
    // prevent that. This iterative pre-check bails out to the same observable
    // tostring() fallback the post-check produced, and also fires on cyclic
    // tables (their traversal depth is unbounded), matching the previous
    // `deny_recursive_tables` fallback behavior.
    if table_depth_exceeds(table, MAX_NESTING_DEPTH) {
        return lua_tostring(lua, original_value);
    }

    // Try JSON conversion (depth already bounded above)
    let deserialize_options = mlua::DeserializeOptions::default().deny_recursive_tables(true);
    match lua.from_value_with::<serde_json::Value>(original_value.clone(), deserialize_options) {
        Ok(json_value) => match serde_json::to_string(&json_value) {
            Ok(s) => s,
            Err(_) => lua_tostring(lua, original_value),
        },
        Err(_) => lua_tostring(lua, original_value),
    }
}

/// Count the number of key-value pairs in a Lua table.
fn count_table_elements(table: &Table) -> usize {
    table.pairs::<Value, Value>().count()
}

/// Whether `table` nests deeper than `limit` levels (the table itself is depth 1).
///
/// Uses an explicit work stack instead of Rust recursion so that arbitrarily
/// deep Lua input can NEVER overflow the host stack (never panics/aborts).
/// Traversal stops descending as soon as `limit` is exceeded, which bounds the
/// cost for adversarial inputs and guarantees termination even for cyclic
/// tables (a cycle keeps increasing the path depth until it trips the limit).
/// Table-typed keys are inspected too: they reach the serde conversion as well.
fn table_depth_exceeds(table: &Table, limit: usize) -> bool {
    let mut stack: Vec<(Table, usize)> = vec![(table.clone(), 1)];
    while let Some((t, depth)) = stack.pop() {
        if depth > limit {
            return true;
        }
        for pair in t.pairs::<Value, Value>() {
            // Iteration errors are ignored here; the conversion itself will
            // surface them and fall back to tostring().
            let Ok((key, value)) = pair else { continue };
            if let Value::Table(kt) = key {
                stack.push((kt, depth + 1));
            }
            if let Value::Table(vt) = value {
                stack.push((vt, depth + 1));
            }
        }
    }
    false
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
