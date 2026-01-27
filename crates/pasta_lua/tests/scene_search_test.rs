//! Integration tests for SCENE.search() function.
//!
//! These tests verify that the SCENE.search() Lua function works correctly
//! with @pasta_search module integration.

mod common;

use common::e2e_helpers::create_runtime_with_search;
use pasta_lua::context::TranspileContext;
use std::collections::HashMap;

/// Helper to create a TranspileContext with test data for SCENE.search() testing.
fn create_scene_search_context() -> TranspileContext {
    let mut ctx = TranspileContext::new();

    // Register global scenes that match Lua STORE.scenes structure
    // Format: "グローバル名_カウンタ" for fn_name, which becomes "グローバル名_カウンタ" in global_name

    // Register "メイン" as global scene (will become メイン_1)
    let (_, counter) = ctx.scene_registry.register_global("メイン", HashMap::new());

    // Register local scene under メイン
    ctx.scene_registry
        .register_local("選択肢", "メイン", counter, 1, HashMap::new());

    // Register another global scene for testing
    ctx.scene_registry.register_global("OnBoot", HashMap::new());

    ctx
}

/// Helper function to extract string result
fn get_string_result(value: mlua::Value) -> String {
    value.as_string().unwrap().to_str().unwrap().to_string()
}

// =============================================================================
// Task 2.1: 正常系テスト（ローカル・グローバル検索）
// =============================================================================

#[test]
fn test_scene_search_returns_result_with_metadata() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    // Register a scene in Lua side first
    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Register a scene
        SCENE.register("メイン_1", "__start__", function(act)
            return "hello"
        end)

        -- Search for it
        local result = SCENE.search("メイン", nil)

        -- Verify result has expected fields
        if result == nil then
            return "result is nil"
        end

        if result.global_name == nil then
            return "global_name is nil"
        end

        if result.local_name == nil then
            return "local_name is nil"
        end

        if result.func == nil then
            return "func is nil"
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

#[test]
fn test_scene_search_global_returns_start_scene() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Register a global scene with __start__
        SCENE.register("メイン_1", "__start__", function(act)
            return "started"
        end)

        -- Global search (global_scene_name = nil)
        local result = SCENE.search("メイン", nil)

        if result == nil then
            return "result is nil"
        end

        -- Global search should return __start__
        if result.local_name ~= "__start__" then
            return "expected __start__, got: " .. tostring(result.local_name)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

#[test]
fn test_scene_search_local_search() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Register scenes
        SCENE.register("メイン_1", "__start__", function() return "start" end)
        SCENE.register("メイン_1", "__選択肢_1__", function() return "choice" end)

        -- Local search (within メイン_1)
        local result = SCENE.search("選択肢", "メイン_1")

        if result == nil then
            return "result is nil"
        end

        if result.global_name ~= "メイン_1" then
            return "expected global_name メイン_1, got: " .. tostring(result.global_name)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

// =============================================================================
// Task 2.2: エラー系・境界値テスト
// =============================================================================

#[test]
fn test_scene_search_nil_name_returns_nil() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"
        local result = SCENE.search(nil, nil)
        return result == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result);
}

#[test]
fn test_scene_search_non_string_name_returns_nil() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Test with number
        local result1 = SCENE.search(123, nil)
        if result1 ~= nil then return "number should return nil" end

        -- Test with table
        local result2 = SCENE.search({}, nil)
        if result2 ~= nil then return "table should return nil" end

        -- Test with boolean
        local result3 = SCENE.search(true, nil)
        if result3 ~= nil then return "boolean should return nil" end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

#[test]
fn test_scene_search_not_found_returns_nil() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Search for non-existent scene
        local result = SCENE.search("存在しないシーン", nil)
        return result == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result);
}

#[test]
fn test_scene_search_scene_not_registered_in_lua_returns_nil() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    // The scene exists in Rust's SceneRegistry but not registered in Lua's STORE.scenes
    let result: bool = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- メイン is in Rust registry, but not registered in Lua
        -- So even if @pasta_search finds it, SCENE.get() will return nil
        local result = SCENE.search("メイン", nil)

        -- Should return nil because scene func is not registered in Lua
        return result == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result);
}

// =============================================================================
// Task 2.3: __call メタメソッドの動作検証
// =============================================================================

#[test]
fn test_scene_search_result_is_callable() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Register a scene that returns a value
        SCENE.register("メイン_1", "__start__", function(act, arg1)
            return "called with: " .. tostring(arg1)
        end)

        -- Search and call
        local result = SCENE.search("メイン", nil)

        if result == nil then
            return "result is nil"
        end

        -- Call the result directly (using __call metatable)
        local output = result(nil, "test_arg")

        if output ~= "called with: test_arg" then
            return "unexpected output: " .. tostring(output)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

#[test]
fn test_scene_search_result_func_field_is_callable() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        SCENE.register("メイン_1", "__start__", function(act)
            return "direct call"
        end)

        local result = SCENE.search("メイン", nil)

        if result == nil then
            return "result is nil"
        end

        -- Call via func field
        local output = result.func(nil)

        if output ~= "direct call" then
            return "unexpected output: " .. tostring(output)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

#[test]
fn test_scene_search_result_metadata_access() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        SCENE.register("メイン_1", "__start__", function() end)

        local result = SCENE.search("メイン", nil)

        if result == nil then
            return "result is nil"
        end

        -- Access metadata
        local gn = result.global_name
        local ln = result.local_name

        if type(gn) ~= "string" then
            return "global_name is not string: " .. type(gn)
        end

        if type(ln) ~= "string" then
            return "local_name is not string: " .. type(ln)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

// =============================================================================
// Task 3.3: 既存 API との互換性検証
// =============================================================================

#[test]
fn test_existing_scene_api_still_works() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Test SCENE.register()
        SCENE.register("テスト_1", "__start__", function() return "test" end)

        -- Test SCENE.get()
        local func = SCENE.get("テスト_1", "__start__")
        if func == nil then
            return "SCENE.get() failed"
        end

        -- Test SCENE.create_scene()
        local scene_table = SCENE.create_scene("新規", "__start__", function() end)
        if scene_table == nil then
            return "SCENE.create_scene() failed"
        end

        -- Test SCENE.get_global_table()
        local gt = SCENE.get_global_table("テスト_1")
        if gt == nil then
            return "SCENE.get_global_table() failed"
        end

        -- Test SCENE.get_start()
        local start = SCENE.get_start("テスト_1")
        if start == nil then
            return "SCENE.get_start() failed"
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}

// =============================================================================
// Task 4.1: イベントハンドラからのシーン呼び出しテスト
// =============================================================================

#[test]
fn test_scene_search_from_event_handler_pattern() {
    let ctx = create_scene_search_context();
    let lua = create_runtime_with_search(ctx).unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SCENE = require "pasta.scene"

        -- Register an OnBoot scene
        SCENE.register("OnBoot_1", "__start__", function(act)
            return "booted"
        end)

        -- Simulate event handler pattern
        local function handle_event(event_name)
            local result = SCENE.search(event_name, nil)
            if result then
                return result(nil)  -- Call the scene
            else
                return "default"  -- Default behavior
            end
        end

        -- Test with existing scene
        local output1 = handle_event("OnBoot")
        if output1 ~= "booted" then
            return "OnBoot should return 'booted', got: " .. tostring(output1)
        end

        -- Test with non-existing scene (should fall back to default)
        local output2 = handle_event("OnClose")
        if output2 ~= "default" then
            return "OnClose should return 'default', got: " .. tostring(output2)
        end

        return "success"
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(get_string_result(result), "success");
}
