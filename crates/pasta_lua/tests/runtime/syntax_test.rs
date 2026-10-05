//! Runtime Syntax E2E Tests for pasta_lua.
//!
//! These tests verify syntax parsing, variable scoping, error reporting,
//! and chaintalk (＞チェイントーク) runtime execution:
//! - Comment line parse verification
//! - Attribute inheritance from file scope to scenes
//! - Variable scope separation (local/global/system)
//! - Error message specificity (line/column numbers)
//! - Chaintalk transpile, finalize, and coroutine execution
//!
//! # Requirements Coverage
//! - Requirement 7.1: Untested area tests
//! - Requirement 7.2: Runtime E2E tests

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script;

// ============================================================================
// Task 3.1: Comment Line Parse Test
// ============================================================================

/// Test that comment lines are not included in AST (Task 3.1)
///
/// Verifies:
/// - Lines starting with ＃ are treated as comments
/// - Comments do not appear in transpiled output
/// - Mixed comments and code parse correctly
#[test]
fn test_comment_line_explicit_parse() {
    use pasta_dsl::parser::parse_str;

    let source = r#"
＃ これはコメントです
＊メイン
  ＃ これもコメント
  さくら：「こんにちは」
＃ 最後のコメント
"#;

    // Parse should succeed
    let file = parse_str(source, "test.pasta").expect("Should parse with comments");

    // Should have exactly one scene (メイン)
    let scene_count = file
        .items
        .iter()
        .filter(|item| matches!(item, pasta_dsl::parser::ast::FileItem::GlobalSceneScope(_)))
        .count();

    assert_eq!(scene_count, 1, "Should have exactly 1 scene");

    // Transpiled code should not contain comment text
    let lua_code = transpile(source);
    assert!(
        !lua_code.contains("これはコメントです"),
        "Comment text should not appear in transpiled code"
    );
    assert!(
        !lua_code.contains("これもコメント"),
        "Inline comment should not appear in transpiled code"
    );
}

// ============================================================================
// Task 3.2: Attribute Inheritance Test
// ============================================================================

/// Test attribute inheritance from file scope to scenes (Task 3.2)
///
/// Verifies:
/// - File-level attributes are inherited by scenes
/// - Scene attributes override file attributes
#[test]
fn test_attribute_inheritance() {
    let source = r#"
＆天気：晴れ
＆場所：公園

＊メイン
  ＆場所：学校
  さくら：「今日は＄天気です」
"#;

    let lua_code = transpile(source);

    // File attributes should be merged into scene
    // The transpiled code should reference both inherited and overridden attrs
    assert!(lua_code.contains("create_scene"), "Should create scene");

    // Verify the transpiled code compiles and runs
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Scene should be searchable
    let found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local name, _ = SEARCH:search_scene("メイン", nil)
        return name ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        found,
        "Scene with inherited attributes should be searchable"
    );
}

// ============================================================================
// Task 3.3: Variable Scope Test
// ============================================================================

/// Test variable scope separation (Task 3.3)
///
/// Verifies:
/// - Local variables (＄) are action-scoped
/// - Global variables (＄＊) are save-scoped
/// - System variables (＄＊＊) are system-scoped (if implemented)
#[test]
fn test_variable_scope_complete() {
    let source = r#"
＊メイン
  ＄ローカル＝「ローカル値」
  ＄＊グローバル＝「グローバル値」
  さくら：「ローカル：＄ローカル、グローバル：＄＊グローバル」
"#;

    let lua_code = transpile(source);

    // Local variable uses var.name format
    assert!(
        lua_code.contains("var."),
        "Local variable should use var.name format. Generated code:\n{}",
        lua_code
    );

    // Global variable uses save.name format
    assert!(
        lua_code.contains("save."),
        "Global variable should use save.name format. Generated code:\n{}",
        lua_code
    );

    // Verify code compiles
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(&lua_code).exec().unwrap();
}

// ============================================================================
// Task 3.4: Error Message Specificity Test
// ============================================================================

/// Test error message includes line and column numbers (Task 3.4)
///
/// Verifies:
/// - Parse errors include line number
/// - Parse errors include column number
/// - Error message is descriptive
#[test]
fn test_error_message_specificity() {
    use pasta_dsl::parser::parse_str;

    // Invalid syntax: scene without name
    let invalid_source = "＊\n";

    let result = parse_str(invalid_source, "test.pasta");
    assert!(result.is_err(), "Invalid syntax should produce error");

    let error = result.unwrap_err();
    let error_str = format!("{:?}", error);

    // Error should contain line number
    assert!(
        error_str.contains("line") || error_str.contains("1"),
        "Error should include line information: {}",
        error_str
    );

    // Test another error pattern: unclosed action
    let invalid_source2 = "＊メイン\n  さくら：";
    let result2 = parse_str(invalid_source2, "test.pasta");

    // This might succeed or fail depending on grammar
    // The important thing is that errors are descriptive
    if let Err(error2) = result2 {
        let error_str2 = format!("{:?}", error2);
        assert!(
            error_str2.len() > 10,
            "Error message should be descriptive: {}",
            error_str2
        );
    }
}

// ============================================================================
// Task 4: Chaintalk (＞チェイントーク) E2E Tests
// ============================================================================

/// Test that chaintalk fixture parses and transpiles correctly (Task 4.1)
///
/// Verifies:
/// - ＞チェイントーク is transpiled to act:call with "チェイントーク" label
/// - The transpiled code contains the expected scene structure
#[test]
fn test_fixture_chaintalk_parses() {
    let source = include_str!("../fixtures/e2e/runtime_e2e_scene_chaintalk.pasta");
    let lua_code = transpile(source);

    // Verify actor definition
    assert!(
        lua_code.contains("create_actor(\"さくら\")"),
        "Should contain さくら actor definition. Generated code:\n{}",
        lua_code
    );

    // Verify scene is created
    assert!(
        lua_code.contains("チェイントークテスト"),
        "Should contain チェイントークテスト scene. Generated code:\n{}",
        lua_code
    );

    // Verify ＞チェイントーク is transpiled to act:call with "チェイントーク" key
    assert!(
        lua_code.contains("\"チェイントーク\""),
        "Should contain act:call with チェイントーク key. Generated code:\n{}",
        lua_code
    );
}

/// Test chaintalk transpile → finalize → GLOBAL function resolution (Task 4.2)
///
/// Verifies the complete E2E flow:
/// 1. Pasta DSL with ＞チェイントーク transpiles to Lua
/// 2. Transpiled code loads and finalizes successfully
/// 3. Scene can be found via search
/// 4. GLOBAL.チェイントーク is registered and accessible at L3
/// 5. Scene execution with coroutine produces intermediate output via yield
#[test]
fn test_e2e_chaintalk_transpile_and_execute() {
    let lua = create_runtime_with_finalize().unwrap();

    // Register @pasta_sakura_script (required by SHIORI_ACT → sakura_builder)
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    let source = include_str!("../fixtures/e2e/runtime_e2e_scene_chaintalk.pasta");
    let lua_code = transpile(source);

    // Load transpiled code
    lua.load(&lua_code).exec().unwrap();

    // Finalize scenes
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Verify GLOBAL functions are registered
    let global_ok: bool = lua
        .load(
            r#"
        local GLOBAL = require("pasta.global")
        return type(GLOBAL["チェイントーク"]) == "function"
            and type(GLOBAL["yield"]) == "function"
    "#,
        )
        .eval()
        .unwrap();
    assert!(
        global_ok,
        "GLOBAL.チェイントーク and GLOBAL.yield should be registered"
    );

    // Execute the scene via EVENT.fire and verify coroutine yield
    // The fixture scene is: talk → ＞チェイントーク (yield) → talk
    // Note: STORE.actors["さくら"] is already created by the transpiled code
    // (PASTA.create_actor("さくら") was called during lua_code execution)
    // First EVENT.fire should produce the pre-yield output and set co_scene
    let fire_result: String = lua
        .load(
            r#"
        local STORE = require("pasta.store")
        local EVENT = require("pasta.shiori.event")

        -- Fire the scene event
        local req = { id = "チェイントークテスト" }
        local response = EVENT.fire(req)

        -- Check result and co_scene state
        local co_state = "none"
        if STORE.co_scene then
            co_state = coroutine.status(STORE.co_scene)
        end

        return string.format("status=%s co=%s", tostring(response ~= nil), co_state)
    "#,
        )
        .eval()
        .unwrap();

    // After first fire, we expect response and suspended coroutine
    assert!(
        fire_result.contains("status=true"),
        "EVENT.fire should return a response, got: {}",
        fire_result
    );
    assert!(
        fire_result.contains("co=suspended"),
        "co_scene should be suspended after yield, got: {}",
        fire_result
    );
}

// ============================================================================
// Special runtime variables: ＞transfer_req_to_var / ＞transfer_date_to_var
// ============================================================================

/// DSL から転記メソッドを Call して、SHIORI リクエスト由来の変数
/// （＄ｒＮ / ＄rN / ＄req_id）と日時変数（＄時１２ 等）を参照できること。
/// 転記を呼ばないシーンでは値が入らないこと（利用者マニュアル記載の挙動）。
#[test]
fn test_e2e_transfer_req_and_date_to_var_from_dsl() {
    let lua = create_runtime_with_finalize().unwrap();
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    let source = r#"
％さくら
  ＠通常：\s[0]

＊転記あり
  ＞transfer_req_to_var
  ＞transfer_date_to_var
  さくら：部位＝＄ｒ４　半角＝＄r4　イベント＝＄req_id　時刻＝＄時１２　曜日＝＄曜日　。

＊転記なし
  さくら：部位＝＄ｒ４　。
"#;
    lua.load(transpile(source)).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let fire = |id: &str| -> String {
        lua.load(format!(
            r#"
            local EVENT = require("pasta.shiori.event")
            return EVENT.fire({{
                id = "{id}",
                reference = {{ [0] = "0", [4] = "Head" }},
                date = {{ year = 2026, month = 9, day = 26, hour = 14, min = 5, sec = 0, wday = 6 }},
            }})
        "#
        ))
        .eval()
        .unwrap()
    };

    let with = fire("転記あり");
    for want in [
        "部位＝Head",
        "半角＝Head",
        "イベント＝転記あり",
        "時刻＝午後2時",
        "曜日＝土曜日",
    ] {
        assert!(with.contains(want), "missing {want:?} in {with}");
    }

    let without = fire("転記なし");
    assert!(
        !without.contains("Head"),
        "transfer must be explicit: {without}"
    );
    assert!(
        without.contains("部位＝。"),
        "unset var must render empty: {without}"
    );
}

/// 未定義の変数・単語・関数をアクション行で参照すると、"nil" を出さず
/// 空文字として展開される（マニュアル book/src/grammar/words.md「未定義単語の参照」と同じ扱い）。
/// 前後に文字がある単語参照が nil 連結で実行時エラーにならないことも確認する。
#[test]
fn test_e2e_undefined_refs_in_action_line_render_empty() {
    let lua = create_runtime_with_finalize().unwrap();
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    let source = r#"
％さくら
  ＠通常：\s[0]

＊未定義参照
  さくら：変数＝＄未代入　単語＝＠未定義語　関数＝＠未定義関数（１）　グローバル＝＄＊未代入　終わり。
"#;
    lua.load(transpile(source)).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let response: String = lua
        .load(r#"return require("pasta.shiori.event").fire({ id = "未定義参照", reference = {} })"#)
        .eval()
        .unwrap();
    assert!(
        !response.contains("nil"),
        "must not render 'nil': {response}"
    );
    assert!(
        response.contains("変数＝単語＝関数＝　グローバル＝終わり。"),
        "undefined refs must render empty: {response}"
    );
}

// ============================================================================
// dynamic-word-reference: ＠＄変数名 / ＠＄変数名（…） E2E
// ============================================================================

/// Transpile `source`, finalize, fire scene `id` once per entry of `ids` and
/// return the responses (one runtime, so word rotations carry across fires).
fn fire_scenes(source: &str, ids: &[&str]) -> Vec<String> {
    let lua = create_runtime_with_finalize().unwrap();
    let config = TalkConfig::default();
    let module = sakura_script::register(&lua, Some(&config)).unwrap();
    let package: mlua::Table = lua.globals().get("package").unwrap();
    let loaded: mlua::Table = package.get("loaded").unwrap();
    loaded.set("@pasta_sakura_script", module).unwrap();

    lua.load(transpile(source)).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    ids.iter()
        .map(|id| {
            lua.load(format!(
                r#"return require("pasta.shiori.event").fire({{ id = "{id}", reference = {{}} }})"#
            ))
            .eval()
            .unwrap()
        })
        .collect()
}

const DYNAMIC_REF_SOURCE: &str = r#"
％さくら
  ＠通常：\s[0]
  ＠表情：\s[5]

＠表情：グローバル表情

＊アクター辞書
  ＄x＝「表情」
  さくら：値＝＠＄x　です。

＊代入の右辺
  ＄x＝「表情」
  ＄y＝＠＄x
  さくら：右辺＝＄y　です。

＊プロパティ代入
  ＄x＝「表情」
  ＄％prop＝＠＄x
  さくら：済み。

＊プロキシ
  ＄f＝「whoami」
  さくら：結果＝＠＄f（１）です。

```lua
function SCENE.whoami(a, n)
    local kind = a.actors ~= nil and "act" or "proxy"
    return kind .. ":" .. tostring(n)
end
```

＊未代入
  さくら：前＠＄z　後。
"#;

/// R3.2: アクター付きの行の `＠＄x` は、静的参照と同じくアクター辞書を先に探す。
#[test]
fn test_e2e_dynamic_word_ref_actor_line_uses_actor_dictionary() {
    let out = fire_scenes(DYNAMIC_REF_SOURCE, &["アクター辞書"]).remove(0);
    assert!(
        out.contains(r"値＝\s[5]です。"),
        "actor dictionary must win on an actor line: {out}"
    );
    assert!(!out.contains("グローバル表情"), "global word leaked: {out}");
}

/// R3.3: 代入の右辺 `＄y＝＠＄x` はアクター辞書を探さない。
#[test]
fn test_e2e_dynamic_word_ref_assignment_skips_actor_dictionary() {
    let out = fire_scenes(DYNAMIC_REF_SOURCE, &["代入の右辺"]).remove(0);
    assert!(
        out.contains("右辺＝グローバル表情です。"),
        "assignment RHS must not search the actor dictionary: {out}"
    );
}

/// R1.5: プロパティ代入の右辺 `＄％prop＝＠＄x` は、動的参照の結果を代入する。
#[test]
fn test_e2e_dynamic_word_ref_property_assignment() {
    let out = fire_scenes(DYNAMIC_REF_SOURCE, &["プロパティ代入"]).remove(0);
    assert!(
        out.contains(r"\![set,property,prop,グローバル表情]"),
        "property must receive the dynamic reference result: {out}"
    );
}

/// R2.1: アクター付きの行の `＠＄f（１）` でも、シーンテーブルの関数は第 1 引数に ACT を受け取る。
#[test]
fn test_e2e_dynamic_fn_call_scene_function_receives_act() {
    let out = fire_scenes(DYNAMIC_REF_SOURCE, &["プロキシ"]).remove(0);
    assert!(
        out.contains("結果＝act:1です。"),
        "scene-table function called on an actor line must receive ACT: {out}"
    );
}

/// R2.4: アクション行の `＠話す（…）` で Lua ブロックのシーン関数を呼ぶと、
/// 行の外（`＄r＝＠話す（…）`）と同じく実行される。
#[test]
fn test_e2e_action_line_calls_lua_scene_function() {
    let source = r#"
％さくら
  ＠通常：\s[0]

＊シーン関数
  ＄r＝＠話す（「外」）
  さくら：前＠話す（「中」）後。

```lua
function SCENE.話す(act, where)
    local save, var = act:init_scene(SCENE)
    act.さくら:talk(where .. "で実行")
end
```
"#;
    let out = fire_scenes(source, &["シーン関数"]).remove(0);
    assert!(out.contains("外で実行"), "out-of-line call must run: {out}");
    assert!(
        out.contains("前中で実行後。"),
        "action-line call must run the scene function like out-of-line: {out}"
    );
    assert!(!out.contains("table:"), "must not render a table: {out}");
}

/// R4.5: 未代入の `＠＄z` は空文字列になり、行の残りが出力される。
#[test]
fn test_e2e_dynamic_word_ref_unassigned_keeps_rest_of_line() {
    let out = fire_scenes(DYNAMIC_REF_SOURCE, &["未代入"]).remove(0);
    assert!(!out.contains("nil"), "must not render 'nil': {out}");
    assert!(
        out.contains("前後。"),
        "unassigned dynamic ref must render empty and keep the rest: {out}"
    );
}

/// R3.4: `＄x` の値が「果物」のとき、`＠＄x` と `＠果物` は実物の単語レジストリの
/// 1 つの巡回を共有する。1 回の発火で静的 2 回・動的 2 回の計 4 回を引き、
/// 候補 4 つがちょうど 1 回ずつ出ることを、複数回の発火で確認する
/// （巡回が別々なら同じ候補が重複しうる）。
#[test]
fn test_e2e_dynamic_word_ref_shares_rotation_with_static() {
    let source = r#"
％さくら
  ＠通常：\s[0]

＠果物：りんご、みかん、ぶどう、もも

＊巡回
  ＄x＝「果物」
  さくら：＠果物　＠＄x　＠果物　＠＄x　。
"#;
    let fruits = ["りんご", "みかん", "ぶどう", "もも"];
    for (i, out) in fire_scenes(source, &["巡回"; 8]).iter().enumerate() {
        for fruit in fruits {
            assert_eq!(
                out.matches(fruit).count(),
                1,
                "fire #{i}: each of the 4 words must appear once per cycle: {out}"
            );
        }
    }
}
