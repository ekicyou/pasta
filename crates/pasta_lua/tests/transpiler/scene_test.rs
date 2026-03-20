//! Scene-related integration tests for pasta_lua transpiler.
//!
//! - set_spot tests (scene-actors-lua-codegen)
//! - Tail Call Optimization (TCO) tests (Requirement 4)

use pasta_dsl::parse_str;
use pasta_lua::LuaTranspiler;

// ============================================================================
// set_spot Tests (scene-actors-lua-codegen)
// ============================================================================

/// Test 3.1: 複数アクター生成テスト
#[test]
fn test_set_spot_multiple_actors() {
    let source = r#"
＊シーン
　％さくら、うにゅう＝２
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify set_spot calls are generated with new format (Requirement 3.1, 3.2)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing さくら set_spot call. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("うにゅう", 2)"#),
        "Missing うにゅう set_spot call. Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let sakura_pos = lua_code
        .find(r#"act:set_spot("さくら""#)
        .expect("さくら set_spot not found");
    let unyu_pos = lua_code
        .find(r#"act:set_spot("うにゅう""#)
        .expect("うにゅう set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < sakura_pos,
        "clear_spot should come before さくら set_spot"
    );
    assert!(
        sakura_pos < unyu_pos,
        "Actor order not preserved: さくら should come before うにゅう"
    );
}

/// Test 3.2: 単一アクター生成テスト
#[test]
fn test_set_spot_single_actor() {
    let source = r#"
＊シーン
　％さくら
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify single actor set_spot is generated with new format (Requirement 3.1, 3.2)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing single actor set_spot call. Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let set_spot_pos = lua_code
        .find(r#"act:set_spot("さくら""#)
        .expect("set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < set_spot_pos,
        "clear_spot should come before set_spot"
    );
}

/// Test 3.3: アクター未定義テスト
#[test]
fn test_set_spot_empty_actors() {
    let source = r#"
＊シーン
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify no set_spot calls are generated (Requirement 2.2)
    assert!(
        !lua_code.contains("set_spot"),
        "set_spot should not be generated when no actors defined. Generated code:\n{lua_code}"
    );

    // Verify no clear_spot call is generated (Requirement 2.2)
    assert!(
        !lua_code.contains("act:clear_spot()"),
        "clear_spot should not be generated when no actors defined. Generated code:\n{lua_code}"
    );
}

/// Test 3.4: 明示的番号付きアクターテスト
#[test]
fn test_set_spot_with_explicit_number() {
    let source = r#"
＊シーン
　％さくら、うにゅう＝２、まりか
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify explicit numbers are respected with new format (C# enum rule)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing さくら set_spot(0). Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("うにゅう", 2)"#),
        "Missing うにゅう set_spot(2). Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("まりか", 3)"#),
        "Missing まりか set_spot(3). Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> all set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let marika_pos = lua_code
        .find(r#"act:set_spot("まりか""#)
        .expect("まりか set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < marika_pos,
        "clear_spot should come before last set_spot"
    );
}

// ============================================================================
// Tail Call Optimization (TCO) Tests - Requirement 4
// ============================================================================

/// Test 4.1: 単一の act:call() のみを含むシーン関数から return act:call(...) が生成される
#[test]
fn test_single_call_scene_gets_return() {
    let source = r#"
＊メイン
　　＞シーン2
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify that single call gets return prefix (Requirement 4.1)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "Single call should have 'return' prefix for TCO. Generated code:\n{lua_code}"
    );
}

/// Test 4.2: 複数の act:call() を含むシーン関数から、最後の呼び出しのみに return が付与される
#[test]
fn test_multiple_call_scenes_only_last_gets_return() {
    let source = r#"
＊メイン
　　＞シーン1
　　＞シーン2
　　＞シーン3
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify first and second calls do NOT have return (Requirement 4.2)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン1""#),
        "First call should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン2""#),
        "Second call should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );

    // Verify the last call HAS return
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン3""#),
        "Last call should have 'return' prefix for TCO. Generated code:\n{lua_code}"
    );

    // Verify first calls do not have return prefix
    // Check that "return act:call" appears exactly once for シーン3
    let return_calls: Vec<_> = lua_code.match_indices("return act:call").collect();
    assert_eq!(
        return_calls.len(),
        1,
        "Only the last call should have 'return'. Found {} return calls. Generated code:\n{lua_code}",
        return_calls.len()
    );
}

/// Test 4.3: act:call() の後に ActionLine が続く場合、return が生成されない
#[test]
fn test_call_scene_followed_by_action_no_return() {
    let source = r#"
％さくら
　＠通常：\s[0]

＊メイン
　　＞シーン2
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify that call does NOT have return when followed by action (Requirement 4.3)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    // The call should appear without return prefix
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン2""#),
        "Call should be present. Generated code:\n{lua_code}"
    );

    // Verify return is NOT before this call (it's not the tail)
    assert!(
        !lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "Call followed by action should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );

    // Verify the talk action comes after the call
    let call_pos = lua_code
        .find(r#"act:call(SCENE.__global_name__, "シーン2""#)
        .expect("Call not found");
    let talk_pos = lua_code
        .find(r#"act.さくら:talk("#)
        .expect("Talk not found");
    assert!(call_pos < talk_pos, "Call should come before talk action");
}

/// Test 4.4: シーン関数に act:call() が含まれない場合、return が生成されない
#[test]
fn test_no_call_scene_no_return() {
    let source = r#"
％さくら
　＠通常：\s[0]

＊メイン
　　さくら：こんにちは
　　さくら：さようなら
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify no return statements for calls (only function return, not call return)
    // The scene should have talk actions but no return act:call
    assert!(
        !lua_code.contains("return act:call"),
        "No call scene means no 'return act:call' should be generated. Generated code:\n{lua_code}"
    );

    // Verify talk actions are generated correctly
    assert!(
        lua_code.contains(r#"act.さくら:talk("こんにちは")"#),
        "Talk action should be present. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act.さくら:talk("さようなら")"#),
        "Second talk action should be present. Generated code:\n{lua_code}"
    );
}

/// Test 4.5: テストフィクスチャを使用した末尾再帰最適化の E2E 検証
#[test]
fn test_tail_call_optimization_fixture() {
    let source = include_str!("../fixtures/tail_call_optimization.pasta");
    let file = parse_str(source, "tail_call_optimization.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Pattern 1: 単一呼び出し - return が付く
    // Counter now assigned by Lua runtime, so uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "単一呼び出し scene should have return. Generated code:\n{lua_code}"
    );

    // Pattern 2: 複数呼び出し - 最後の呼び出しのみ return
    // シーン1, シーン2 には return なし
    let multiple_call_section = lua_code
        .find("SCENE.複数呼び出し_")
        .expect("複数呼び出し function not found");
    let next_function = lua_code[multiple_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| multiple_call_section + pos)
        .unwrap_or(lua_code.len());

    let multiple_call_code = &lua_code[multiple_call_section..next_function];

    // シーン3 のみ return
    assert!(
        multiple_call_code.contains(r#"return act:call(SCENE.__global_name__, "シーン3""#),
        "Last call in 複数呼び出し should have return. Section:\n{multiple_call_code}"
    );

    // シーン1, シーン2 は return なし（カウント確認）
    let return_count = multiple_call_code.matches("return act:call").count();
    assert_eq!(
        return_count, 1,
        "Only シーン3 should have return in 複数呼び出し. Found {} return calls. Section:\n{multiple_call_code}",
        return_count
    );

    // Pattern 3: 呼び出し後にアクション - return なし
    let action_after_call_section = lua_code
        .find("SCENE.呼び出し後にアクション_")
        .expect("呼び出し後にアクション function not found");
    let next_function2 = lua_code[action_after_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| action_after_call_section + pos)
        .unwrap_or(lua_code.len());

    let action_after_call_code = &lua_code[action_after_call_section..next_function2];

    assert!(
        !action_after_call_code.contains("return act:call"),
        "Call followed by action should NOT have return. Section:\n{action_after_call_code}"
    );

    // Pattern 4: 呼び出しなし - return act:call なし
    let no_call_section = lua_code
        .find("SCENE.呼び出しなし_")
        .expect("呼び出しなし function not found");
    let next_function3 = lua_code[no_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| no_call_section + pos)
        .unwrap_or(lua_code.len());

    let no_call_code = &lua_code[no_call_section..next_function3];

    assert!(
        !no_call_code.contains("return act:call"),
        "No call scene should have no return act:call. Section:\n{no_call_code}"
    );
}
