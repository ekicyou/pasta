//! `@pasta_search` の条件つき検索（scene-condition-attributes 2.3）。
//!
//! `search_scene(name, gsn, var, save)`・`search_scene_exact(...)` の 3 つの結果
//! （見つかった・名前が無い・条件漏れ）と、表を渡さないと条件も優先の段も見ないこと。

use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue};
use pasta_lua::{PastaLuaRuntime, TranspileContext};

/// `＆＄＊経験値≧500`
fn exp_ge_500() -> SceneAttr {
    SceneAttr::Cond {
        key: "＄＊経験値".to_string(),
        op: CmpOp::Ge,
        value: SceneAttrValue::Integer(500),
    }
}

/// `＆優先：1`
fn priority_1() -> SceneAttr {
    SceneAttr::Decl {
        key: "優先".to_string(),
        value: SceneAttrValue::Integer(1),
    }
}

/// 告白_1（＆＄＊経験値≧500）・挨拶_1（条件なし）・メイン_1 の下の
/// ローカル 選択肢_1（＆＄気分＝「高」）
fn create_context() -> TranspileContext {
    let mut ctx = TranspileContext::new();
    ctx.scene_registry
        .register_global("告白", vec![exp_ge_500()]);
    ctx.scene_registry.register_global("挨拶", Vec::new());
    let (_, counter) = ctx.scene_registry.register_global("メイン", Vec::new());
    ctx.scene_registry.register_local(
        "選択肢",
        "メイン",
        counter,
        1,
        vec![SceneAttr::Cond {
            key: "＄気分".to_string(),
            op: CmpOp::Eq,
            value: SceneAttrValue::String("高".to_string()),
        }],
    );
    ctx
}

/// Lua のかたまりを新しいランタイムで流し、文字列の結果を返す。
fn exec_string(ctx: TranspileContext, code: &str) -> String {
    let runtime = PastaLuaRuntime::new(ctx).unwrap();
    let value = runtime.exec(code).unwrap();
    value.as_string().unwrap().to_string_lossy()
}

/// `search_scene` を 1 回呼び、`(1 つ目, 2 つ目)` を `"a|b"` の形で返す Lua の前置き
const PRELUDE: &str = r#"
    local SEARCH = require "@pasta_search"
    local function show(a, b) return tostring(a) .. "|" .. tostring(b) end
"#;

fn run(ctx: TranspileContext, body: &str) -> String {
    exec_string(ctx, &format!("{PRELUDE}\n{body}"))
}

#[test]
fn test_found_when_condition_met() {
    let result = run(
        create_context(),
        r#"return show(SEARCH:search_scene("告白", nil, {}, { ["経験値"] = 500 }))"#,
    );
    assert_eq!(result, "告白_1|__start__");
}

#[test]
fn test_not_found_returns_single_nil() {
    let result = run(
        create_context(),
        r#"return show(SEARCH:search_scene("存在しない", nil, {}, {}))"#,
    );
    assert_eq!(result, "nil|nil");
}

#[test]
fn test_unmet_returns_nil_and_unmet() {
    let result = run(
        create_context(),
        r#"return show(SEARCH:search_scene("告白", nil, {}, { ["経験値"] = 10 }))"#,
    );
    assert_eq!(result, "nil|unmet");
}

#[test]
fn test_missing_table_is_treated_as_empty() {
    // var だけ渡すと save は空の表として扱う（経験値は nil → 0 で満たさない）
    let result = run(
        create_context(),
        r#"return show(SEARCH:search_scene("告白", nil, { ["経験値"] = 999 }))"#,
    );
    assert_eq!(result, "nil|unmet");
}

#[test]
fn test_passing_no_tables_ignores_conditions() {
    // 表を渡さなければ条件を見ない（名指しの引き。2.9）
    let result = run(
        create_context(),
        r#"
        return show(SEARCH:search_scene("告白", nil)) .. ","
            .. show(SEARCH:search_scene("告白", nil, nil, nil)) .. ","
            .. show(SEARCH:search_scene_exact("告白", nil))
    "#,
    );
    assert_eq!(result, "告白_1|__start__,告白_1|__start__,告白_1|__start__");
}

#[test]
fn test_passing_no_tables_ignores_priority_tier() {
    // 雑談_1 は ＆優先：1、雑談_2 は優先なし。
    // 表を渡せば段は 1 で雑談_1 しか出ない。渡さなければ段を見ず 1 巡で両方出る
    let ctx = || {
        let mut ctx = TranspileContext::new();
        ctx.scene_registry
            .register_global("雑談", vec![priority_1()]);
        ctx.scene_registry.register_global("雑談", Vec::new());
        ctx
    };
    let with_tables = run(
        ctx(),
        r#"
        local r = {}
        for i = 1, 4 do r[i] = SEARCH:search_scene("雑談", nil, {}, {}) end
        return table.concat(r, ",")
    "#,
    );
    assert_eq!(with_tables, "雑談_1,雑談_1,雑談_1,雑談_1");

    let without_tables = run(
        ctx(),
        r#"
        local seen = {}
        for _ = 1, 2 do seen[SEARCH:search_scene("雑談", nil)] = true end
        return tostring(seen["雑談_1"]) .. "," .. tostring(seen["雑談_2"])
    "#,
    );
    assert_eq!(without_tables, "true,true");
}

#[test]
fn test_local_search_uses_var_table() {
    let result = run(
        create_context(),
        r#"
        return show(SEARCH:search_scene("選択肢", "メイン_1", { ["気分"] = "高" }, {})) .. ","
            .. show(SEARCH:search_scene("選択肢", "メイン_1", { ["気分"] = "低" }, {})) .. ","
            .. show(SEARCH:search_scene("無い", "メイン_1", {}, {}))
    "#,
    );
    assert_eq!(result, "メイン_1|選択肢_1,nil|unmet,nil|nil");
}

#[test]
fn test_exact_search_three_results() {
    let result = run(
        create_context(),
        r#"
        return show(SEARCH:search_scene_exact("告白", nil, {}, { ["経験値"] = 600 })) .. ","
            .. show(SEARCH:search_scene_exact("告白", nil, {}, {})) .. ","
            .. show(SEARCH:search_scene_exact("告", nil, {}, { ["経験値"] = 600 }))
    "#,
    );
    assert_eq!(result, "告白_1|__start__,nil|unmet,nil|nil");
}
