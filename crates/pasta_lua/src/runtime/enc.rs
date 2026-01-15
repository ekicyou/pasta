//! Encoding conversion module for Lua.
//!
//! Provides the `@enc` module with functions to convert between
//! UTF-8 and ANSI (system locale) encodings.
//!
//! # Example
//! ```lua
//! local enc = require "@enc"
//!
//! -- Convert UTF-8 to ANSI
//! local ansi, err = enc.to_ansi("日本語パス")
//! if ansi then
//!     -- Use ANSI string for file operations
//! end
//!
//! -- Convert ANSI to UTF-8
//! local utf8, err = enc.to_utf8(ansi)
//! if utf8 then
//!     print(utf8)  -- "日本語パス"
//! end
//! ```

use crate::encoding::{Encoder, Encoding};
use mlua::{Lua, Result as LuaResult, String as LuaString, Table, Value};

/// Module version.
const VERSION: &str = "0.1.0";

/// Module description.
const DESCRIPTION: &str = "Encoding conversion (UTF-8 <-> ANSI)";

/// Register the @enc module with the Lua state.
///
/// Creates a module table with:
/// - `_VERSION` - Module version string
/// - `_DESCRIPTION` - Module description
/// - `to_ansi(utf8_str)` - Convert UTF-8 to ANSI bytes
/// - `to_utf8(ansi_str)` - Convert ANSI bytes to UTF-8
///
/// # Arguments
/// * `lua` - The Lua state to register the module with
///
/// # Returns
/// * `Ok(Table)` - The module table
/// * `Err(e)` - Registration failed
pub fn register(lua: &Lua) -> LuaResult<Table> {
    let enc = lua.create_table()?;

    // Set module metadata
    enc.set("_VERSION", VERSION)?;
    enc.set("_DESCRIPTION", DESCRIPTION)?;

    // Register to_ansi function
    enc.set("to_ansi", lua.create_function(to_ansi_impl)?)?;

    // Register to_utf8 function
    enc.set("to_utf8", lua.create_function(to_utf8_impl)?)?;

    Ok(enc)
}

/// Implementation of `enc.to_ansi(utf8_str)`.
///
/// Converts a UTF-8 Lua string to ANSI encoding.
///
/// # Returns
/// - On success: `(ansi_string, nil)`
/// - On error: `(nil, error_message)`
fn to_ansi_impl(lua: &Lua, value: Value) -> LuaResult<(Option<LuaString>, Option<String>)> {
    // Type check: must be a string
    let utf8_str = match value {
        Value::String(s) => s,
        _ => {
            let err_msg = format!("expected string, got {}", value.type_name());
            tracing::warn!(error = %err_msg, "enc.to_ansi type error");
            return Ok((None, Some(err_msg)));
        }
    };

    // Get the string as bytes first, then interpret as UTF-8
    let utf8_bytes = utf8_str.as_bytes();
    let utf8_str = match std::str::from_utf8(&utf8_bytes) {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("invalid UTF-8 input: {}", e);
            tracing::warn!(error = %err_msg, "enc.to_ansi encoding error");
            return Ok((None, Some(err_msg)));
        }
    };

    // Convert UTF-8 to ANSI
    match Encoding::ANSI.to_bytes(utf8_str) {
        Ok(ansi_bytes) => {
            let lua_string = lua.create_string(&ansi_bytes)?;
            Ok((Some(lua_string), None))
        }
        Err(e) => {
            let err_msg = format!("ANSI conversion failed: {}", e);
            tracing::warn!(error = %err_msg, "enc.to_ansi conversion error");
            Ok((None, Some(err_msg)))
        }
    }
}

/// Implementation of `enc.to_utf8(ansi_str)`.
///
/// Converts an ANSI Lua string to UTF-8 encoding.
///
/// # Returns
/// - On success: `(utf8_string, nil)`
/// - On error: `(nil, error_message)`
fn to_utf8_impl(lua: &Lua, value: Value) -> LuaResult<(Option<LuaString>, Option<String>)> {
    // Type check: must be a string
    let ansi_str = match value {
        Value::String(s) => s,
        _ => {
            let err_msg = format!("expected string, got {}", value.type_name());
            tracing::warn!(error = %err_msg, "enc.to_utf8 type error");
            return Ok((None, Some(err_msg)));
        }
    };

    // Get the raw bytes from Lua string
    let ansi_bytes = ansi_str.as_bytes();

    // Convert ANSI to UTF-8
    match Encoding::ANSI.to_string(&ansi_bytes) {
        Ok(utf8_string) => {
            let lua_string = lua.create_string(&utf8_string)?;
            Ok((Some(lua_string), None))
        }
        Err(e) => {
            let err_msg = format!("UTF-8 conversion failed: {}", e);
            tracing::warn!(error = %err_msg, "enc.to_utf8 conversion error");
            Ok((None, Some(err_msg)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::StdLib;

    fn create_test_lua() -> Lua {
        unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }
    }

    #[test]
    fn test_register_creates_table() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        // Check _VERSION exists
        let version: String = enc.get("_VERSION").unwrap();
        assert_eq!(version, VERSION);

        // Check _DESCRIPTION exists
        let desc: String = enc.get("_DESCRIPTION").unwrap();
        assert_eq!(desc, DESCRIPTION);
    }

    #[test]
    fn test_to_ansi_ascii() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        // Register module
        let globals = lua.globals();
        globals.set("enc", enc).unwrap();

        // Test ASCII string
        let result: (Option<LuaString>, Option<String>) =
            lua.load(r#"return enc.to_ansi("hello")"#).eval().unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_none());
        assert_eq!(result.0.unwrap().as_bytes(), b"hello");
    }

    #[test]
    fn test_to_ansi_type_error() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        let globals = lua.globals();
        globals.set("enc", enc).unwrap();

        // Test with number instead of string
        let result: (Option<LuaString>, Option<String>) =
            lua.load(r#"return enc.to_ansi(123)"#).eval().unwrap();

        assert!(result.0.is_none());
        assert!(result.1.is_some());
        assert!(result.1.unwrap().contains("expected string"));
    }

    #[test]
    fn test_to_utf8_ascii() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        let globals = lua.globals();
        globals.set("enc", enc).unwrap();

        // Test ASCII string
        let result: (Option<LuaString>, Option<String>) =
            lua.load(r#"return enc.to_utf8("hello")"#).eval().unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_none());
        assert_eq!(result.0.unwrap().as_bytes(), b"hello");
    }

    #[test]
    fn test_to_utf8_type_error() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        let globals = lua.globals();
        globals.set("enc", enc).unwrap();

        // Test with nil
        let result: (Option<LuaString>, Option<String>) =
            lua.load(r#"return enc.to_utf8(nil)"#).eval().unwrap();

        assert!(result.0.is_none());
        assert!(result.1.is_some());
        assert!(result.1.unwrap().contains("expected string"));
    }

    #[cfg(windows)]
    #[test]
    fn test_roundtrip_japanese() {
        let lua = create_test_lua();
        let enc = register(&lua).unwrap();

        let globals = lua.globals();
        globals.set("enc", enc).unwrap();

        // Test roundtrip: UTF-8 -> ANSI -> UTF-8
        let script = r#"
            local original = "日本語テスト"
            local ansi, err1 = enc.to_ansi(original)
            if not ansi then
                return nil, err1
            end
            local utf8, err2 = enc.to_utf8(ansi)
            if not utf8 then
                return nil, err2
            end
            return utf8, original
        "#;

        let result: (Option<String>, Option<String>) = lua.load(script).eval().unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_some());
        assert_eq!(result.0.unwrap(), result.1.unwrap());
    }
}
