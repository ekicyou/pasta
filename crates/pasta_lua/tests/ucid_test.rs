//! Test for Unicode identifier (ucid) support in Lua
//!
//! This test verifies that Lua accepts Japanese identifiers when built with ucid feature.

use mlua::{Lua, Result};

/// Test that Japanese variable names work in Lua
#[test]
fn test_japanese_variable_name() -> Result<()> {
    let lua = Lua::new();

    // Try to create a variable with Japanese name
    let result = lua
        .load(
            r#"
        local 変数 = 42
        return 変数
    "#,
        )
        .eval::<i32>();

    match result {
        Ok(value) => {
            assert_eq!(value, 42);
            println!("SUCCESS: Japanese variable '変数' works! Value = {}", value);
            Ok(())
        }
        Err(e) => {
            println!("FAILED: Japanese variable not supported. Error: {}", e);
            panic!("ucid feature is NOT working: {}", e);
        }
    }
}

/// Test that Japanese function names work in Lua
#[test]
fn test_japanese_function_name() -> Result<()> {
    let lua = Lua::new();

    let result = lua
        .load(
            r#"
        function 挨拶(名前)
            return "こんにちは、" .. 名前 .. "さん！"
        end
        return 挨拶("太郎")
    "#,
        )
        .eval::<String>();

    match result {
        Ok(value) => {
            assert_eq!(value, "こんにちは、太郎さん！");
            println!(
                "SUCCESS: Japanese function '挨拶' works! Result = {}",
                value
            );
            Ok(())
        }
        Err(e) => {
            println!("FAILED: Japanese function not supported. Error: {}", e);
            panic!("ucid feature is NOT working: {}", e);
        }
    }
}

/// Test that Japanese table field names work in Lua
#[test]
fn test_japanese_table_field() -> Result<()> {
    let lua = Lua::new();

    let result = lua
        .load(
            r#"
        local テーブル = {
            名前 = "田中",
            年齢 = 30
        }
        return テーブル.名前 .. "(" .. テーブル.年齢 .. "歳)"
    "#,
        )
        .eval::<String>();

    match result {
        Ok(value) => {
            assert_eq!(value, "田中(30歳)");
            println!("SUCCESS: Japanese table fields work! Result = {}", value);
            Ok(())
        }
        Err(e) => {
            println!("FAILED: Japanese table fields not supported. Error: {}", e);
            panic!("ucid feature is NOT working: {}", e);
        }
    }
}
