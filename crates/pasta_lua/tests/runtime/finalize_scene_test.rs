//! E2E integration tests for the finalize_scene() function.

use crate::common;
use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use std::path::PathBuf;

// ============================================================================
// Scene Dictionary Collection E2E Tests
// ============================================================================

/// Basic scene collection and search using existing fixture
#[test]
fn test_scene_collection_basic() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("メイン", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.starts_with("メイン"),
        "Global name should start with 'メイン', got: {}",
        result.0
    );
    assert_eq!(result.1, "__start__", "Local name should be '__start__'");
}

/// Multiple global scenes with same name (counter test)
#[test]
fn test_scene_collection_multiple_global_scenes() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_scene("会話分岐", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("会話分岐"),
        "Should find scene with name containing '会話分岐', got: {}",
        result.0
    );
}

/// Local scenes under global scene
#[test]
fn test_scene_collection_local_scenes() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let search_exists: bool = lua
        .load(
            r#"
        local ok, SEARCH = pcall(require, "@pasta_search")
        return ok and SEARCH ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(search_exists, "@pasta_search module should be available");
}

/// Local scene search via finalize path
#[test]
fn test_scene_collection_local_scene_search() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Note: PASTA.create_scene("メイン") generates global_name "メイン_1"
    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local SCENE = require "pasta.scene"
        local gn = SCENE.get_global_table("メイン_1").__global_name__
        return SEARCH:search_scene("グローバル単語呼び出し", gn)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("メイン"),
        "Global name should contain 'メイン', got: {}",
        result.0
    );
    assert!(
        result.1.contains("グローバル単語呼び出し"),
        "Local name should contain 'グローバル単語呼び出し', got: {}",
        result.1
    );
}

/// Local scene prefix search via finalize path
#[test]
fn test_scene_collection_local_scene_prefix_search() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // Note: PASTA.create_scene("メイン") generates global_name "メイン_1"
    let result: (String, String) = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local SCENE = require "pasta.scene"
        local gn = SCENE.get_global_table("メイン_1").__global_name__
        return SEARCH:search_scene("会話分岐", gn)
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result.0.contains("メイン"),
        "Global name should contain 'メイン', got: {}",
        result.0
    );
    assert!(
        result.1.contains("会話分岐"),
        "Local name should contain '会話分岐', got: {}",
        result.1
    );
}

// ============================================================================
// Word Dictionary Collection E2E Tests
// ============================================================================

/// Global word collection
#[test]
fn test_word_collection_global() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: mlua::Value = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        return SEARCH:search_word("挨拶", nil)
    "#,
        )
        .eval()
        .unwrap();

    assert!(result.is_string(), "Word search should return a string");
    let word = result.as_string().map(|s| s.to_string_lossy()).unwrap();
    assert!(
        word == "こんにちは" || word == "やあ" || word == "ハロー",
        "Word should be one of the registered values, got: {}",
        word
    );
}

/// Local word collection
#[test]
fn test_word_collection_local() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let search_available: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        search_available,
        "@pasta_search should be available after finalize_scene"
    );
}

/// Builder pattern API (method chaining)
#[test]
fn test_word_builder_pattern() {
    let lua = create_runtime_with_finalize().unwrap();

    let result: bool = lua
        .load(
            r#"
        local PASTA = require("pasta")
        
        -- Create global word using builder pattern
        PASTA.create_word("テスト"):entry("値1", "値2", "値3")
        
        -- Call finalize_scene
        PASTA.finalize_scene()
        
        -- Verify word is searchable
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("テスト", nil)
        return word == "値1" or word == "値2" or word == "値3"
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Builder pattern should register word correctly");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Empty registry finalize (warning log + empty SearchContext)
#[test]
fn test_empty_registry_finalize() {
    let lua = create_runtime_with_finalize().unwrap();

    let result: bool = lua
        .load(
            r#"
        local PASTA = require("pasta")
        return PASTA.finalize_scene()
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        result,
        "finalize_scene should return true even with empty registry"
    );

    let search_available: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        search_available,
        "@pasta_search should be available even with empty registry"
    );
}

/// Scene not found returns nil
#[test]
fn test_scene_not_found() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local global_name = SEARCH:search_scene("存在しないシーン", nil)
        return global_name == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Non-existent scene search should return nil");
}

/// Word not found returns nil
#[test]
fn test_word_not_found() {
    let lua = create_runtime_with_finalize().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let result: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("存在しない単語", nil)
        return word == nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(result, "Non-existent word search should return nil");
}

// ============================================================================
// Initialization Timing Control Tests (Optional)
// ============================================================================

/// Multiple finalize_scene calls (SearchContext rebuild)
#[test]
fn test_multiple_finalize_calls() {
    let lua = create_runtime_with_finalize().unwrap();

    lua.load(
        r#"
        local PASTA = require("pasta")
        PASTA.create_word("単語A"):entry("値A1", "値A2")
        PASTA.finalize_scene()
    "#,
    )
    .exec()
    .unwrap();

    let word_a_found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local word = SEARCH:search_word("単語A", nil)
        return word ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(word_a_found, "単語A should be found after first finalize");

    lua.load(
        r#"
        local PASTA = require("pasta")
        PASTA.create_word("単語B"):entry("値B1", "値B2")
        PASTA.finalize_scene()
    "#,
    )
    .exec()
    .unwrap();

    let both_found: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        local a = SEARCH:search_word("単語A", nil)
        local b = SEARCH:search_word("単語B", nil)
        return a ~= nil and b ~= nil
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        both_found,
        "Both words should be found after second finalize"
    );
}

/// @pasta_search not available before finalize_scene
#[test]
fn test_search_unavailable_before_finalize() {
    use mlua::{Lua, StdLib};

    // Create fresh Lua without finalize_scene registration
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, mlua::LuaOptions::default()) };

    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");

    lua.load(format!(
        r#"
        package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path
        "#
    ))
    .exec()
    .unwrap();

    let result: bool = lua
        .load(
            r#"
        local ok = pcall(require, "@pasta_search")
        return ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(
        !result,
        "@pasta_search should NOT be available before finalize_scene"
    );
}

/// Full E2E flow with transpiled pasta
#[test]
fn test_full_e2e_flow() {
    let lua = create_runtime_with_finalize().unwrap();

    let pasta_source = include_str!("../fixtures/sample.pasta");
    let lua_code = transpile(pasta_source);

    lua.load(&lua_code).exec().unwrap();

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    let all_ok: bool = lua
        .load(
            r#"
        local SEARCH = require "@pasta_search"
        
        -- Check scenes
        local main_ok = SEARCH:search_scene("メイン", nil) ~= nil
        local talk_ok = SEARCH:search_scene("会話分岐", nil) ~= nil
        
        -- Check words
        local greet_ok = SEARCH:search_word("挨拶", nil) ~= nil
        
        return main_ok and talk_ok and greet_ok
    "#,
        )
        .eval()
        .unwrap();

    assert!(all_ok, "All scenes and words should be searchable");
}

// ============================================================================
// 別名表がランタイムの 2 つの登録点へ渡る
// ============================================================================

/// `SEARCH:search_scene(name)`（範囲指定なし）が返すグローバルの登録名。見つからなければ `None`。
fn search_global(lua: &mlua::Lua, name: &str) -> Option<String> {
    lua.load(
        r#"
        local SEARCH = require "@pasta_search"
        local g = SEARCH:search_scene(...)
        return g
    "#,
    )
    .call(name)
    .unwrap()
}

/// `＊OnTalk` だけの辞書から、`aliases` を持つ `RuntimeConfig` でランタイムを作り、
/// 初回登録の `@pasta_search` と、`finalize_scene` が再登録した `@pasta_search` の
/// それぞれで `search_scene("会話")` を引く。戻りは (確定前, 確定後)。
fn search_kaiwa_before_and_after_finalize(
    aliases: pasta_core::registry::SceneAliasTable,
) -> (Option<String>, Option<String>) {
    use pasta_lua::runtime::{PastaLuaRuntime, RuntimeConfig};

    let file = pasta_dsl::parser::parse_str("＊OnTalk\n  さくら：やあ\n", "alias_runtime.pasta")
        .expect("source must parse");
    let mut out = Vec::new();
    let ctx = pasta_lua::LuaTranspiler::default()
        .transpile(&file, &mut out)
        .expect("source must transpile");

    let runtime = PastaLuaRuntime::with_config(
        ctx,
        RuntimeConfig::new().with_scene_aliases(aliases.clone()),
    )
    .expect("runtime must build");
    let lua = runtime.lua();

    // 初回登録（with_config_and_source_map）の @pasta_search
    let before = search_global(lua, "会話");

    // finalize_scene を動かすための最小環境（create_runtime_with_finalize と同じ）
    let scripts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pasta_scripts")
        .to_string_lossy()
        .replace('\\', "/");
    lua.load(format!(
        r#"package.path = "{scripts_dir}/?.lua;{scripts_dir}/?/init.lua;" .. package.path"#
    ))
    .exec()
    .unwrap();
    let persistence = pasta_lua::runtime::persistence::register(
        lua,
        &pasta_lua::loader::PersistenceConfig::default(),
        &std::env::temp_dir(),
    )
    .unwrap();
    let loaded: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set("@pasta_persistence", persistence).unwrap();
    pasta_lua::runtime::finalize::register_finalize_scene(lua, aliases).unwrap();

    lua.load(String::from_utf8(out).unwrap()).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    // finalize_scene が再登録した @pasta_search
    let after = search_global(lua, "会話");
    (before, after)
}

/// 既定表を持つ設定のランタイムでは、確定の前後どちらでも「会話」が OnTalk に解決される。
#[test]
fn test_scene_alias_table_reaches_both_search_registrations() {
    let (before, after) = search_kaiwa_before_and_after_finalize(
        pasta_core::registry::SceneAliasTable::builtin_default(),
    );
    assert_eq!(before.as_deref(), Some("OnTalk_1"), "before finalize_scene");
    assert_eq!(after.as_deref(), Some("OnTalk_1"), "after finalize_scene");
}

/// 空の表では従来どおり「会話」は OnTalk に当たらない。
#[test]
fn test_empty_scene_alias_table_keeps_search_unchanged() {
    let (before, after) =
        search_kaiwa_before_and_after_finalize(pasta_core::registry::SceneAliasTable::empty());
    assert_eq!(before, None, "before finalize_scene");
    assert_eq!(after, None, "after finalize_scene");
}

// ============================================================================
// 辞書確定で Lua の倉から Rust のシーン表へ属性を運ぶ
// ============================================================================

const ATTR_SOURCE: &str = "＆＄＊経験値≧500＆背景：教室
＊会話＆率：1.0＆倍率：1.5
　＆題：「」
　Alice：やあ
　・選択肢＆優先：3＆＄済み
　Alice：うん
＊会話＆作者：Bob
　Alice：別
％さくら
　＆x：9
";

/// ATTR_SOURCE から属性だけを除いた同じ形の辞書
const PLAIN_SOURCE: &str = "＊会話
　Alice：やあ
　・選択肢
　Alice：うん
＊会話
　Alice：別
％さくら
";

/// 辞書を読み込んで辞書確定まで済ませた VM
fn finalized(source: &str) -> mlua::Lua {
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(transpile(source)).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    lua
}

/// 辞書確定と同じ道で Rust のシーン表（登録）を作る
fn rust_registry(lua: &mlua::Lua) -> pasta_core::registry::SceneRegistry {
    use pasta_lua::runtime::finalize::{build_scene_registry, collect_scene_attrs, collect_scenes};
    let scenes = collect_scenes(lua).unwrap();
    let attrs = collect_scene_attrs(lua).unwrap();
    build_scene_registry(&scenes, &attrs)
}

/// Lua の倉を「グローバル|ローカル|項目,…」の行で（行は並べ替え、項目は書いた順のまま）
fn lua_store_dump(lua: &mlua::Lua) -> String {
    lua.load(
        r#"
        local STORE = require "pasta.store"
        local function fmt(it)
            local s = it.key
            if it.op ~= nil then s = s .. " " .. it.op end
            if it.value ~= nil then
                s = s .. " " .. type(it.value):sub(1, 1) .. ":" .. tostring(it.value)
            end
            return s
        end
        local out = {}
        for g, by_local in pairs(STORE.scene_attrs) do
            for l, items in pairs(by_local) do
                local parts = {}
                for _, it in ipairs(items) do parts[#parts + 1] = fmt(it) end
                out[#out + 1] = g .. "|" .. l .. "|" .. table.concat(parts, ",")
            end
        end
        table.sort(out)
        return table.concat(out, "\n")
    "#,
    )
    .eval()
    .unwrap()
}

/// Rust の表を lua_store_dump と同じ形で（属性のあるシーンだけ）
fn rust_dump(registry: &pasta_core::registry::SceneRegistry) -> String {
    use pasta_core::registry::{SceneAttr, SceneAttrValue};
    let val = |v: &SceneAttrValue| match v {
        SceneAttrValue::String(s) => format!("s:{s}"),
        n => format!("n:{n}"),
    };
    let mut out: Vec<String> = registry
        .all_scenes()
        .into_iter()
        .filter(|e| !e.attributes.is_empty())
        .map(|e| {
            let (g, l) = match &e.parent {
                Some(p) => (p.as_str(), e.name.as_str()),
                None => (e.name.as_str(), "__start__"),
            };
            let items: Vec<String> = e
                .attributes
                .iter()
                .map(|a| match a {
                    SceneAttr::Decl { key, value } => format!("{key} {}", val(value)),
                    SceneAttr::Cond { key, op, value } => {
                        format!("{key} {} {}", op.as_str(), val(value))
                    }
                    SceneAttr::Flag { key } => key.clone(),
                })
                .collect();
            format!("{g}|{l}|{}", items.join(","))
        })
        .collect();
    out.sort();
    out.join("\n")
}

fn attrs_of<'a>(
    registry: &'a pasta_core::registry::SceneRegistry,
    name: &str,
    parent: Option<&str>,
) -> &'a [pasta_core::registry::SceneAttr] {
    &registry
        .all_scenes()
        .into_iter()
        .find(|e| e.name == name && e.parent.as_deref() == parent)
        .unwrap_or_else(|| panic!("scene {name} / {parent:?} not registered"))
        .attributes
}

/// Rust の表の属性は Lua の倉と同じ。整数の値になる小数は整数、項目は 3 種に写る。
/// 辞書確定は倉を書き換えない（2 回確定しても倉は同じ）。
#[test]
fn test_scene_attrs_reach_rust_table_same_as_lua_store() {
    use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue};
    let lua = finalized(ATTR_SOURCE);
    let store_before = lua_store_dump(&lua);
    let registry = rust_registry(&lua);

    assert!(!store_before.is_empty());
    assert_eq!(rust_dump(&registry), store_before);

    let decl = |k: &str, v: SceneAttrValue| SceneAttr::Decl {
        key: k.into(),
        value: v,
    };
    let cond = SceneAttr::Cond {
        key: "＄＊経験値".into(),
        op: CmpOp::Ge,
        value: SceneAttrValue::Integer(500),
    };
    assert_eq!(
        attrs_of(&registry, "会話_1", None),
        &[
            cond.clone(),
            decl("背景", SceneAttrValue::String("教室".into())),
            decl("率", SceneAttrValue::Integer(1)),
            decl("倍率", SceneAttrValue::Float(1.5)),
            decl("題", SceneAttrValue::String(String::new())),
        ]
    );
    assert_eq!(
        attrs_of(&registry, "選択肢_1", Some("会話_1")),
        &[
            decl("優先", SceneAttrValue::Integer(3)),
            SceneAttr::Flag {
                key: "＄済み".into()
            },
        ]
    );

    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    assert_eq!(
        lua_store_dump(&lua),
        store_before,
        "finalize must not rewrite the store"
    );
}

/// 同じ宣言名の 2 つの宣言の属性は混ざらない
#[test]
fn test_scene_attrs_same_name_declarations_do_not_mix() {
    use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue};
    let registry = rust_registry(&finalized(ATTR_SOURCE));
    assert_eq!(
        attrs_of(&registry, "会話_2", None),
        &[
            SceneAttr::Cond {
                key: "＄＊経験値".into(),
                op: CmpOp::Ge,
                value: SceneAttrValue::Integer(500),
            },
            SceneAttr::Decl {
                key: "背景".into(),
                value: SceneAttrValue::String("教室".into()),
            },
            SceneAttr::Decl {
                key: "作者".into(),
                value: SceneAttrValue::String("Bob".into()),
            },
        ]
    );
    assert!(
        !attrs_of(&registry, "会話_1", None)
            .iter()
            .any(|a| a.key() == "作者")
    );
}

/// 属性の有無でシーンの数（と並び）が変わらない
#[test]
fn test_scene_attrs_do_not_change_scene_count() {
    let order = |r: &pasta_core::registry::SceneRegistry| -> Vec<(String, Option<String>)> {
        r.all_scenes()
            .into_iter()
            .map(|e| (e.name.clone(), e.parent.clone()))
            .collect()
    };
    let with_attrs = rust_registry(&finalized(ATTR_SOURCE));
    let plain = rust_registry(&finalized(PLAIN_SOURCE));
    assert_eq!(order(&with_attrs), order(&plain));
    assert_eq!(with_attrs.all_scenes().len(), 3);
}

/// アクター辞書の配下の属性は倉にも Rust の表のどのシーンにも入らない
#[test]
fn test_actor_dictionary_attrs_go_nowhere() {
    let lua = finalized(ATTR_SOURCE);
    let registry = rust_registry(&lua);
    assert!(
        registry
            .all_scenes()
            .iter()
            .all(|e| e.attributes.iter().all(|a| a.key() != "x"))
    );
    assert!(!lua_store_dump(&lua).contains("x "));
}
