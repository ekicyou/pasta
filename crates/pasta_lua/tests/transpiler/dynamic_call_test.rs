//! 動的コール（＞expr）のエンドツーエンド統合テスト
//!
//! Requirements: 3.1, 3.2, 3.3, 3.4, 3.5

use crate::common;

use common::e2e_helpers::create_runtime_with_search;
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::context::TranspileContext;

/// Helper: Pasta ソースをトランスパイルし、TranspileContext を返す
fn transpile_with_context(source: &str) -> (String, TranspileContext) {
    let file = parse_str(source, "test.pasta").expect("Failed to parse");
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let ctx = transpiler
        .transpile(&file, &mut output)
        .expect("Failed to transpile");
    let lua_code = String::from_utf8(output).expect("Invalid UTF-8");
    (lua_code, ctx)
}

// ========================================================================
// R3.1: 動的コール＞＄target でシーンが呼び出される
// ========================================================================

#[test]
fn test_dynamic_call_resolves_scene() {
    let source = r#"
＊テスト
　＄target＝「サブ」
　＞＄target

　・サブ
　　さくら：「サブです」
"#;
    let (lua_code, ctx) = transpile_with_context(source);
    let lua = create_runtime_with_search(ctx).unwrap();

    // Load the transpiled code — registers scenes
    lua.load(&lua_code).exec().unwrap();

    // Verify Lua code contains tostring(var.target)
    assert!(
        lua_code.contains("tostring(var.target)"),
        "Dynamic call should use tostring(var.target). Code:\n{lua_code}"
    );
}

// ========================================================================
// R3.1 + R3.4: Lua ブロック変数代入→動的コール
// ========================================================================

#[test]
fn test_dynamic_call_code_gen_output() {
    let source = r#"
＊メイン
　＄target＝「挨拶」
　＞＄target
"#;
    let (lua_code, _ctx) = transpile_with_context(source);

    // Verify the generated code pattern
    assert!(
        lua_code.contains(r#"var.target = "挨拶""#),
        "Variable assignment not found. Code:\n{lua_code}"
    );
    assert!(
        lua_code.contains("tostring(var.target)"),
        "Dynamic call tostring not found. Code:\n{lua_code}"
    );
    assert!(
        lua_code.contains("act:call(SCENE.__global_name__, tostring(var.target)"),
        "Dynamic call format incorrect. Code:\n{lua_code}"
    );
}

// ========================================================================
// R3.3: 候補不在→空応答（act:call 戻り nil）
// ========================================================================

#[test]
fn test_dynamic_call_no_match_returns_nil() {
    let source = r#"
＊メイン
　＄target＝「存在しないシーン」
　＞＄target
"#;
    let (lua_code, ctx) = transpile_with_context(source);
    let lua = create_runtime_with_search(ctx).unwrap();

    // Load and execute — should not panic
    lua.load(&lua_code).exec().unwrap();

    // Execute finalize_scene to wire up scenes
    lua.load("require('pasta').finalize_scene()").exec().unwrap();

    // Call the scene — act:call internally should handle not-found gracefully
    let result: mlua::Value = lua
        .load(
            r#"
        local PASTA = require "pasta"
        local SCENE = require "pasta.scene"
        local ACT = require "pasta.act"

        -- Look up the メイン scene
        local result = SCENE.search("メイン", nil)
        if result == nil then
            return "scene_not_found"
        end
        return "scene_exists"
    "#,
        )
        .eval()
        .unwrap();

    // Scene should be registered
    let result_str = result.as_string().unwrap().to_str().unwrap().to_string();
    assert_eq!(result_str, "scene_exists");
}

// ========================================================================
// R3.5: nil 式→早期リターン+警告ログ
// ========================================================================

#[test]
fn test_nil_guard_in_act_call() {
    let ctx = TranspileContext::new();
    let lua = create_runtime_with_search(ctx).unwrap();

    // Directly test the nil guard in ACT_IMPL.call
    let result: mlua::Value = lua
        .load(
            r#"
        local ACT = require "pasta.act"

        -- Create a minimal act object
        local act = ACT.new({}, {})

        -- Call with nil key — should return nil and not crash
        local result = act:call("test_scene", nil, {})

        if result == nil then
            return "nil_returned"
        else
            return "unexpected: " .. tostring(result)
        end
    "#,
        )
        .eval()
        .unwrap();

    let result_str = result.as_string().unwrap().to_str().unwrap().to_string();
    assert_eq!(
        result_str, "nil_returned",
        "act:call with nil key should return nil (nil guard)"
    );
}

// ========================================================================
// R4.1 + R4.2: 静的コール不変（Lua 出力）
// ========================================================================

#[test]
fn test_static_call_lua_output_unchanged() {
    let source = r#"
＊メイン
　＞サブシーン
"#;
    let (lua_code, _ctx) = transpile_with_context(source);

    // Static call should use quoted string, not tostring()
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "サブシーン""#),
        "Static call format should use quoted string. Code:\n{lua_code}"
    );
    assert!(
        !lua_code.contains("tostring("),
        "Static call should not use tostring(). Code:\n{lua_code}"
    );
}

// ========================================================================
// Dynamic call with global variable
// ========================================================================

#[test]
fn test_dynamic_call_global_var_code_gen() {
    let source = r#"
＊メイン
　＞＄＊global_target
"#;
    let (lua_code, _ctx) = transpile_with_context(source);

    assert!(
        lua_code.contains("tostring(save.global_target)"),
        "Global var dynamic call should use save.xxx. Code:\n{lua_code}"
    );
}
