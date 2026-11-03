//! scene_cond の単体テスト（要件 1.1–1.11・7.1・7.3）

use super::CondOutcome::*;
use super::*;
use mlua::Lua;
use pasta_core::registry::{CmpOp, SceneAttr, SceneAttrValue, SceneId, SceneInfo, SceneScope};

fn int(v: i64) -> SceneAttrValue {
    SceneAttrValue::Integer(v)
}

fn text(s: &str) -> SceneAttrValue {
    SceneAttrValue::String(s.into())
}

fn cond(key: &str, op: CmpOp, value: SceneAttrValue) -> SceneAttr {
    SceneAttr::Cond {
        key: key.into(),
        op,
        value,
    }
}

fn flag(key: &str) -> SceneAttr {
    SceneAttr::Flag { key: key.into() }
}

fn num(v: f64) -> Operand {
    Operand::Number(v)
}

fn txt(s: &str) -> Operand {
    Operand::Text(s.into())
}

fn eval(op: CmpOp, left: &Operand, right: SceneAttrValue) -> CondOutcome {
    evaluate(&cond("＄x", op, right), left)
}

fn scene(attributes: Vec<SceneAttr>) -> SceneInfo {
    SceneInfo {
        id: SceneId(0),
        name: "告白".into(),
        scope: SceneScope::Global,
        attributes,
        fn_name: "告白1::__start__".into(),
        parent: None,
    }
}

// --- parse_decimal（1.10） ---

#[test]
fn parse_decimal_reads_half_width_decimal_only() {
    assert_eq!(parse_decimal("500"), Some(500.0));
    assert_eq!(parse_decimal("-3"), Some(-3.0));
    assert_eq!(parse_decimal("1.5"), Some(1.5));
    assert_eq!(parse_decimal("0"), Some(0.0));
    for s in [
        "", "-", "1.", ".5", "+1", "1e3", " 1", "1 ", "３", "0x10", "1.2.3", "inf", "nan",
    ] {
        assert_eq!(parse_decimal(s), None, "{s:?} は数として読まない");
    }
}

// --- 6 種の演算子 × 数（1.6） ---

#[test]
fn numbers_compare_by_value_for_all_operators() {
    let cases = [
        (CmpOp::Ge, 500.0, Pass),
        (CmpOp::Ge, 499.0, Fail),
        (CmpOp::Le, 500.0, Pass),
        (CmpOp::Le, 501.0, Fail),
        (CmpOp::Gt, 501.0, Pass),
        (CmpOp::Gt, 500.0, Fail),
        (CmpOp::Lt, 499.0, Pass),
        (CmpOp::Lt, 500.0, Fail),
        (CmpOp::Eq, 500.0, Pass),
        (CmpOp::Eq, 501.0, Fail),
        (CmpOp::Ne, 501.0, Pass),
        (CmpOp::Ne, 500.0, Fail),
    ];
    for (op, left, want) in cases {
        assert_eq!(eval(op, &num(left), int(500)), want, "{left} {op:?} 500");
    }
}

#[test]
fn number_value_not_notation() {
    assert_eq!(eval(CmpOp::Eq, &num(1.0), int(1)), Pass);
    assert_eq!(eval(CmpOp::Eq, &num(1.0), SceneAttrValue::Float(1.0)), Pass);
    assert_eq!(eval(CmpOp::Lt, &num(1.0), SceneAttrValue::Float(1.5)), Pass);
}

// --- 半角十進の文字列は両辺で数（1.10） ---

#[test]
fn decimal_strings_are_numbers_on_both_sides() {
    // Reference の "500"（Lua の文字列は from_text で読む）と右辺 500
    assert_eq!(
        eval(CmpOp::Eq, &Operand::from_text("500".into()), int(500)),
        Pass
    );
    assert_eq!(
        eval(CmpOp::Ge, &Operand::from_text("500".into()), int(500)),
        Pass
    );
    // 変数の 500 と右辺の 「500」
    assert_eq!(eval(CmpOp::Eq, &num(500.0), text("500")), Pass);
    assert_eq!(eval(CmpOp::Gt, &num(501.0), text("500")), Pass);
    // "1.0" と 1 は値で等しい
    assert_eq!(
        eval(CmpOp::Eq, &Operand::from_text("1.0".into()), int(1)),
        Pass
    );
}

#[test]
fn full_width_digits_are_text() {
    // 全角の 「３」 は文字列なので 3 と等しくない・大小は数でない
    assert_eq!(eval(CmpOp::Eq, &txt("３"), int(3)), Fail);
    assert_eq!(eval(CmpOp::Ne, &txt("３"), int(3)), Pass);
    assert_eq!(eval(CmpOp::Eq, &txt("３"), text("３")), Pass);
    assert_eq!(eval(CmpOp::Ge, &txt("３"), int(3)), NotNumber);
}

// --- 数でなければ ＝・≠ は文字列、大小は NotNumber（1.7・1.8） ---

#[test]
fn non_numbers_compare_as_strings_for_eq_ne() {
    assert_eq!(eval(CmpOp::Eq, &txt("春"), text("春")), Pass);
    assert_eq!(eval(CmpOp::Eq, &txt("春"), text("夏")), Fail);
    assert_eq!(eval(CmpOp::Ne, &txt("春"), text("夏")), Pass);
    assert_eq!(eval(CmpOp::Ne, &txt("春"), text("春")), Fail);
    // 片方だけ数: 数は tostring と同じ表記で比べる
    assert_eq!(eval(CmpOp::Eq, &num(1.5), text("春")), Fail);
    assert_eq!(eval(CmpOp::Ne, &num(1.5), text("春")), Pass);
    assert_eq!(eval(CmpOp::Eq, &txt("春"), int(1)), Fail);
}

#[test]
fn ordering_with_non_number_is_not_number() {
    for op in [CmpOp::Ge, CmpOp::Le, CmpOp::Gt, CmpOp::Lt] {
        assert_eq!(eval(op, &txt("春"), int(1)), NotNumber, "{op:?}");
        assert_eq!(eval(op, &num(1.0), text("春")), NotNumber, "{op:?}");
        assert_eq!(eval(op, &txt("春"), text("夏")), NotNumber, "{op:?}");
        assert_eq!(eval(op, &Operand::Other, int(1)), NotNumber, "{op:?}");
        assert_eq!(eval(op, &Operand::Nil, text("春")), NotNumber, "{op:?}");
    }
}

#[test]
fn other_values_equal_no_string() {
    assert_eq!(eval(CmpOp::Eq, &Operand::Other, text("")), Fail);
    assert_eq!(eval(CmpOp::Eq, &Operand::Other, int(1)), Fail);
    assert_eq!(eval(CmpOp::Ne, &Operand::Other, text("x")), Pass);
}

// --- 値の無い変数は 0・空文字列（1.9・1.11） ---

#[test]
fn nil_is_zero_or_empty_string() {
    assert_eq!(eval(CmpOp::Eq, &Operand::Nil, int(0)), Pass);
    assert_eq!(eval(CmpOp::Lt, &Operand::Nil, int(1)), Pass);
    assert_eq!(eval(CmpOp::Ge, &Operand::Nil, int(500)), Fail);
    assert_eq!(eval(CmpOp::Ne, &Operand::Nil, int(0)), Fail);
    assert_eq!(eval(CmpOp::Eq, &Operand::Nil, text("")), Pass);
    assert_eq!(eval(CmpOp::Ne, &Operand::Nil, text("春")), Pass);
    assert_eq!(eval(CmpOp::Eq, &Operand::Nil, text("春")), Fail);
}

// --- フラグ（1.4・1.5・1.10） ---

#[test]
fn flag_rules() {
    let f = flag("＄＊初回待ち");
    assert_eq!(evaluate(&f, &Operand::Nil), Fail);
    assert_eq!(evaluate(&f, &num(0.0)), Fail);
    assert_eq!(evaluate(&f, &num(2.0)), Pass);
    assert_eq!(evaluate(&f, &txt("")), Fail);
    assert_eq!(evaluate(&f, &txt("春")), Pass);
    assert_eq!(evaluate(&f, &Operand::Other), Pass);
}

// --- Lua の値の読み方（1.5・1.9・1.10・10.5） ---

#[test]
fn operand_from_lua_values() {
    let lua = Lua::new();
    let v = |src: &str| Operand::from_lua(lua.load(src).eval().unwrap());
    assert_eq!(v("return nil"), Operand::Nil);
    assert_eq!(v("return true"), num(1.0));
    assert_eq!(v("return false"), num(0.0));
    assert_eq!(v("return 500"), num(500.0));
    assert_eq!(v("return 2147483647"), num(2147483647.0));
    assert_eq!(v("return -2147483648"), num(-2147483648.0));
    assert_eq!(v("return 1.5"), num(1.5));
    assert_eq!(v("return 2^40"), num(1099511627776.0));
    assert_eq!(v("return '5'"), num(5.0));
    // 文字列の "0" は数の 0
    assert_eq!(v("return '0'"), num(0.0));
    assert_eq!(v("return '３'"), txt("３"));
    assert_eq!(v("return ''"), txt(""));
    assert_eq!(v("return {}"), Operand::Other);
    assert_eq!(v("return print"), Operand::Other);
}

#[test]
fn booleans_are_one_and_zero() {
    assert_eq!(eval(CmpOp::Eq, &num(1.0), int(1)), Pass);
    // false は ＝０ を満たす（打ち消しの書き方。1.11）
    let lua = Lua::new();
    let f = Operand::from_lua(lua.load("return false").eval().unwrap());
    assert_eq!(eval(CmpOp::Eq, &f, int(0)), Pass);
    assert_eq!(evaluate(&flag("＄x"), &f), Fail);
}

// --- 述語（1.1・1.2・1.3・7.1・7.3） ---

#[test]
fn predicate_without_conditions_is_true_without_tables() {
    let mut pred = cond_predicate("告白", None, None);
    assert!(pred(&scene(vec![])));
    assert!(pred(&scene(vec![SceneAttr::Decl {
        key: "季節".into(),
        value: text("春"),
    }])));
}

#[test]
fn predicate_reads_save_for_star_and_var_for_plain() {
    let lua = Lua::new();
    let var = lua.create_table().unwrap();
    let save = lua.create_table().unwrap();
    var.raw_set("経験値", 10).unwrap();
    save.raw_set("経験値", 600).unwrap();
    let mut pred = cond_predicate("告白", Some(var), Some(save));
    assert!(pred(&scene(vec![cond("＄＊経験値", CmpOp::Ge, int(500))])));
    assert!(!pred(&scene(vec![cond("＄経験値", CmpOp::Ge, int(500))])));
    assert!(pred(&scene(vec![cond("＄経験値", CmpOp::Eq, int(10))])));
}

#[test]
fn predicate_missing_table_means_no_value() {
    let mut pred = cond_predicate("告白", None, None);
    assert!(!pred(&scene(vec![flag("＄＊初回待ち")])));
    assert!(pred(&scene(vec![cond("＄＊段2済み", CmpOp::Eq, int(0))])));
}

#[test]
fn predicate_does_not_call_metamethods() {
    let lua = Lua::new();
    let var: mlua::Table = lua
        .load("return setmetatable({}, { __index = function() return 999 end })")
        .eval()
        .unwrap();
    let mut pred = cond_predicate("告白", Some(var), None);
    assert!(!pred(&scene(vec![cond("＄x", CmpOp::Ge, int(1))])));
}

#[test]
fn predicate_is_conjunction_of_all_conditions() {
    let lua = Lua::new();
    let save = lua.create_table().unwrap();
    save.raw_set("経験値", 600).unwrap();
    save.raw_set("初回待ち", true).unwrap();
    let mut pred = cond_predicate("告白", None, Some(save.clone()));
    let attrs = vec![
        cond("＄＊経験値", CmpOp::Ge, int(500)),
        flag("＄＊初回待ち"),
    ];
    assert!(pred(&scene(attrs.clone())));
    save.raw_set("初回待ち", false).unwrap();
    assert!(!pred(&scene(attrs)));
}

#[test]
fn predicate_evaluates_at_each_call() {
    let lua = Lua::new();
    let save = lua.create_table().unwrap();
    let mut pred = cond_predicate("告白", None, Some(save.clone()));
    let s = scene(vec![flag("＄＊初回待ち")]);
    assert!(!pred(&s));
    save.raw_set("初回待ち", 1).unwrap();
    assert!(pred(&s));
}

#[tracing_test::traced_test]
#[test]
fn predicate_traces_excluded_candidate() {
    let lua = Lua::new();
    let save = lua.create_table().unwrap();
    save.raw_set("経験値", 120).unwrap();
    let mut pred = cond_predicate("告白", None, Some(save));
    assert!(!pred(&scene(vec![cond("＄＊経験値", CmpOp::Ge, int(500))])));
    assert!(logs_contain("Scene candidate excluded by condition"));
    assert!(logs_contain("告白1::__start__"));
    assert!(logs_contain("＄＊経験値 >= 500"));
    assert!(logs_contain("value=120"));
    assert!(!logs_contain("non-number"));
}

#[tracing_test::traced_test]
#[test]
fn predicate_warns_on_non_number_ordering() {
    let lua = Lua::new();
    let var = lua.create_table().unwrap();
    var.raw_set("季節", "春").unwrap();
    let mut pred = cond_predicate("告白", Some(var), None);
    assert!(!pred(&scene(vec![cond("＄季節", CmpOp::Ge, int(1))])));
    assert!(logs_contain(
        "Condition compares a non-number with an ordering operator; treated as unmet"
    ));
    assert!(logs_contain("＄季節 >= 1"));
    assert!(logs_contain("value=春"));
}

#[tracing_test::traced_test]
#[test]
fn predicate_stops_at_first_failing_condition() {
    let lua = Lua::new();
    let var = lua.create_table().unwrap();
    var.raw_set("季節", "春").unwrap();
    let mut pred = cond_predicate("告白", Some(var), None);
    // 1 つ目で外れるので、2 つ目（数でない大小）は見ず warn も出ない
    assert!(!pred(&scene(vec![
        flag("＄初回待ち"),
        cond("＄季節", CmpOp::Ge, int(1)),
    ])));
    assert!(logs_contain("Scene candidate excluded by condition"));
    assert!(!logs_contain("non-number"));
}
