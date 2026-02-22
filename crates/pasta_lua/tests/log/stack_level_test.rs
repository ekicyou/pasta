//! Stack level verification prototype for @pasta_log module.
//!
//! This test verifies the correct `inspect_stack` level to use
//! when capturing Lua caller information from within Rust closures.

use mlua::{Lua, Result as LuaResult, StdLib, Value};

/// Create a minimal Lua VM for testing.
fn create_test_lua() -> Lua {
    unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) }
}

/// Test inspect_stack levels to find the correct one for capturing
/// Lua caller source/line information from a Rust closure.
#[test]
fn test_inspect_stack_level_verification() -> LuaResult<()> {
    let lua = create_test_lua();

    // Create a Rust function that tests different stack levels
    let test_fn = lua.create_function(|lua_ctx, _: Value| {
        let mut results = Vec::new();

        // Try levels 0..=3
        for level in 0..=3 {
            let info = lua_ctx.inspect_stack(level, |debug| {
                let source_info = debug.source();
                let source = source_info
                    .short_src
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "<no source>".to_string());
                let line = debug.current_line();
                let names = debug.names();
                let fn_name = names
                    .name
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "<no name>".to_string());
                (source, line, fn_name)
            });
            match info {
                Some((source, line, fn_name)) => {
                    results.push(format!(
                        "level={}: source={}, line={:?}, fn={}",
                        level, source, line, fn_name
                    ));
                }
                None => {
                    results.push(format!("level={}: <no stack frame>", level));
                }
            }
        }

        Ok(results.join("\n"))
    })?;

    lua.globals().set("test_stack", test_fn)?;

    // Test 1: Direct call from main chunk
    let result: String = lua
        .load(
            r#"
        return test_stack("hello")
    "#,
        )
        .eval()?;
    println!("=== Direct call ===\n{}", result);

    // Test 2: Call from a named function
    let result2: String = lua
        .load(
            r#"
        local function my_func()
            return test_stack("from function")
        end
        return my_func()
    "#,
        )
        .eval()?;
    println!("\n=== Function call ===\n{}", result2);

    // Test 3: Call from nested function
    let result3: String = lua
        .load(
            r#"
        local function outer()
            local function inner()
                return test_stack("nested")
            end
            return inner()
        end
        return outer()
    "#,
        )
        .eval()?;
    println!("\n=== Nested call ===\n{}", result3);

    Ok(())
}

/// Verify that the correct level gives us the Lua caller's information
/// when called from within a create_function closure.
#[test]
fn test_stack_level_gives_lua_caller() -> LuaResult<()> {
    let lua = create_test_lua();

    // Create a function that captures caller info at different levels
    let capture_fn = lua.create_function(|lua_ctx, _: Value| {
        let info_l1 = lua_ctx.inspect_stack(1, |debug| {
            let source_info = debug.source();
            let source = source_info
                .short_src
                .map(|s| s.to_string())
                .unwrap_or_default();
            let line = debug.current_line();
            let names = debug.names();
            let fn_name = names.name.map(|s| s.to_string()).unwrap_or_default();
            (source, line, fn_name)
        });

        match info_l1 {
            Some((source, line, fn_name)) => Ok((source, line, fn_name)),
            None => Ok(("".to_string(), None, "".to_string())),
        }
    })?;

    lua.globals().set("capture_info", capture_fn)?;

    // Call from a named Lua function at a known line
    let result: (String, Option<usize>, String) = lua
        .load(
            r#"
        local function test_caller()
            return capture_info("test")
        end
        return test_caller()
    "#,
        )
        .set_name("test_script")
        .eval()?;

    println!(
        "Level 1: source={}, line={:?}, fn={}",
        result.0, result.1, result.2
    );

    // Verify that we got meaningful information
    assert!(
        !result.0.is_empty(),
        "Source should not be empty, got: '{}'",
        result.0
    );
    // Line should be Some with a positive value
    assert!(
        result.1.is_some(),
        "Line should be Some, got: {:?}",
        result.1
    );
    assert!(
        result.1.unwrap() > 0,
        "Line should be positive, got: {:?}",
        result.1
    );

    Ok(())
}
