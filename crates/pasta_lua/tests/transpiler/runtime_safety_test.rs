//! 書き間違い 5 件（U18・U19・U20・U22・U08）のトランスパイル→実行テスト
//!
//! Requirements: 3.1, 3.4, 5.5
//!
//! `.pasta` をトランスパイルして生成形をスナップショットで固定し、実ランタイム（pasta_scripts の
//! act.lua）で実行して、例外にならず期待したトークン・変数状態・警告になることを確かめる。
//! 算術は、優先順位を組み直した生成コードの値が「変更前の平らな Lua 式」の値と一致することを
//! 式の一覧で確かめる（パーサの式の組み方が変わったときの関門。design.md Revalidation Triggers）。

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use insta::assert_snapshot;
use mlua::Lua;

/// 警告を記録する `@pasta_log` を差し込んだランタイムで生成コードを読み込み、`メイン1` を実行する。
///
/// 差し込みは pasta.act のロード前に行う（act.lua はロード時に `@pasta_log` を取り込むため）。
/// 実行後の act は Lua のグローバル `RS_ACT`、警告は `RS_WARNS` に置く。
fn run_main_scene(lua_code: &str) -> Lua {
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(
        r#"
        assert(package.loaded["pasta.act"] == nil, "pasta.act must not be loaded before the log hook")
        RS_WARNS = {}
        local real = package.loaded["@pasta_log"]
        package.loaded["@pasta_log"] = setmetatable({
            warn = function(msg) table.insert(RS_WARNS, msg) end,
        }, { __index = real })
        "#,
    )
    .exec()
    .unwrap();
    lua.load(lua_code).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    lua.load(
        r#"
        local SCENE = require "pasta.scene"
        local STORE = require "pasta.store"
        local ACT = require "pasta.act"
        RS_ACT = ACT.new(STORE.actors)
        SCENE.get_start("メイン1")(RS_ACT)
        "#,
    )
    .exec()
    .expect("scene must run without a runtime error");
    lua
}

/// 実行後の Lua 式を評価して文字列で返す
fn eval_str(lua: &Lua, chunk: &str) -> String {
    lua.load(chunk).eval::<String>().unwrap()
}

// ========================================================================
// 5 件の書き間違い（5.5, 3.4）
// ========================================================================

const RUNTIME_SAFETY_SOURCE: &str = r#"
％さくら
　＠通常：\s[0]

％talk
　＠通常：\s[5]

％var
　＠通常：\s[6]

＊メイン
　さくら：開始
　さくら：＠＊未定義関数（）です
　＄u18＝＠＊未定義関数（１）
　未登録さん：こんにちは
　未登録さん：続きです
　talk：トークです
　var：変数です
　＄x＝＄未代入＋１
　＄y＝「abc」＊２
　さくら：＄x
　さくら：C：\\new
　さくら：終わり\\
"#;

#[test]
fn test_runtime_safety_generated_code_snapshot() {
    assert_snapshot!("runtime_safety", transpile(RUNTIME_SAFETY_SOURCE));
}

#[test]
fn test_runtime_safety_tokens_vars_and_warnings() {
    let lua = run_main_scene(&transpile(RUNTIME_SAFETY_SOURCE));

    // トークン列: 種類|アクター名|登録済みアクターか|テキスト
    let tokens = eval_str(
        &lua,
        r#"
        local STORE = require "pasta.store"
        local lines = {}
        for _, t in ipairs(RS_ACT.token) do
            local name = t.actor and t.actor.name or "-"
            local registered = t.actor ~= nil and STORE.actors[name] == t.actor
            table.insert(lines, string.format("%s|%s|%s|%s", t.type, name, tostring(registered), tostring(t.text)))
        end
        return table.concat(lines, "\n")
        "#,
    );
    let expected_tokens = [
        "talk|さくら|true|開始",
        // U18（アクション行）: 未定義のグローバル関数は何も出さない
        "talk|さくら|true|です",
        // U19: 未登録アクターは目印を 1 回だけ積み、続く行も同じアクターで話す
        "talk|未登録さん|false|【未登録アクター：未登録さん】",
        "talk|未登録さん|false|こんにちは",
        "talk|未登録さん|false|続きです",
        // U20: act のメンバー名と同名の登録済みアクターが話せる
        "talk|talk|true|トークです",
        "talk|var|true|変数です",
        // 3.4: 値なしの代入の後の参照は空文字（トークンを積まない）
        // U08: \\ は 2 文字のまま 1 トークン
        "talk|さくら|true|C：",
        r"talk|さくら|true|\\",
        "talk|さくら|true|new",
        "talk|さくら|true|終わり",
        r"talk|さくら|true|\\",
    ]
    .join("\n");
    assert_eq!(tokens, expected_tokens);

    // 変数状態: 値なしの代入は未代入（U18 の式・U22 の算術）
    let vars = eval_str(
        &lua,
        r#"
        local v = RS_ACT.var
        return string.format("u18=%s x=%s y=%s", tostring(v.u18), tostring(v.x), tostring(v.y))
        "#,
    );
    assert_eq!(vars, "u18=nil x=nil y=nil");

    // 未登録アクターは登録されない・act のメンバーは上書きされない
    let state = eval_str(
        &lua,
        r#"
        local STORE = require "pasta.store"
        return string.format("store=%s talk_is_method=%s",
            tostring(STORE.actors["未登録さん"]), tostring(type(RS_ACT.talk) == "function"))
        "#,
    );
    assert_eq!(state, "store=nil talk_is_method=true");

    let warns = eval_str(&lua, r#"return table.concat(RS_WARNS, "\n")"#);
    let expected_warns = [
        "act:global_fn - function not found: key='未定義関数'",
        "act:global_fn - function not found: key='未定義関数'",
        "act:actor_proxy - unregistered actor: name='未登録さん'",
        "act:arith - operand is not a number: op='+', operand='var.未代入', value=nil",
        "act:arith - operand is not a number: op='*', value='abc' (string)",
        "act:talk - undefined variable: 'var.x'",
    ]
    .join("\n");
    assert_eq!(warns, expected_warns);
}

// ========================================================================
// 算術: 組み直した生成コード ＝ 変更前の平らな Lua 式（3.1）
// ========================================================================

/// (DSL の右辺, 変更前の生成器が出していた平らな Lua 式)
///
/// 変数は `＄a＝５`・`＄b＝「3」`（数値文字列）・`＄c＝２.５` を先に代入しておく。
const ARITH_CASES: &[(&str, &str)] = &[
    ("1＋2＊3", "1 + 2 * 3"),
    ("1－2－3", "1 - 2 - 3"),
    ("（1＋2）＊3", "(1 + 2) * 3"),
    ("1－2＊3－4", "1 - 2 * 3 - 4"),
    ("8／2／2", "8 / 2 / 2"),
    ("1＋2＊3＋4＊5", "1 + 2 * 3 + 4 * 5"),
    ("2＊（3＋4）％5", "2 * (3 + 4) % 5"),
    ("10％3＊2", "10 % 3 * 2"),
    ("1－（2－3）", "1 - (2 - 3)"),
    ("10－－3", "10 - -3"),
    ("－7％3", "-7 % 3"),
    ("2.5＊4－1.5", "2.5 * 4 - 1.5"),
    ("7／2", "7 / 2"),
    ("1＋2＊3－4／2％3", "1 + 2 * 3 - 4 / 2 % 3"),
    ("（1＋2）＊（3－4）／5", "(1 + 2) * (3 - 4) / 5"),
    ("＄a＋1", "var.a + 1"),
    ("＄a＊＄b－＄c", "var.a * var.b - var.c"),
    ("＄b＋2＊＄a", "var.b + 2 * var.a"),
    ("＄a－＄b－1", "var.a - var.b - 1"),
    ("＄a％＄b＊＄c", "var.a % var.b * var.c"),
    ("（＄a＋＄b）＊＄c／2", "(var.a + var.b) * var.c / 2"),
    ("「1」＋2", "\"1\" + 2"),
    ("＄b／＄a＋「4」＊＄b", "var.b / var.a + \"4\" * var.b"),
];

#[test]
fn test_regrouped_arith_matches_flat_lua() {
    assert_matches_flat_lua(ARITH_CASES, "arith");
}

/// (DSL の右辺, 同じ値になる平らな Lua 式)。連結は算術より低い段（`..` と同じ）。
const CONCAT_CASES: &[(&str, &str)] = &[
    ("「x」＆1＋2＊3", "\"x\" .. 1 + 2 * 3"),
    ("1＋2＆3＊4", "1 + 2 .. 3 * 4"),
    ("（「1」＆「2」）＋1", "(\"1\" .. \"2\") + 1"),
    ("「a」＆「b」＆「c」", "\"a\" .. \"b\" .. \"c\""),
    ("「r」＆7／2", "\"r\" .. 7 / 2"),
    ("＄a＆＄b＆＄c", "var.a .. var.b .. var.c"),
    ("「n」＆－7％3＋＄a", "\"n\" .. -7 % 3 + var.a"),
    ("（＄a＆1）＊2", "(var.a .. 1) * 2"),
];

#[test]
fn test_regrouped_concat_matches_flat_lua() {
    assert_matches_flat_lua(CONCAT_CASES, "concat");
}

/// 各式を `＄r{i}` に代入して実行し、平らな Lua 式と型も値も同じであること、警告が無いことを確かめる。
fn assert_matches_flat_lua(cases: &[(&str, &str)], what: &str) {
    let mut source = String::from("＊メイン\n　＄a＝５\n　＄b＝「3」\n　＄c＝２.５\n");
    for (i, (dsl, _)) in cases.iter().enumerate() {
        source.push_str(&format!("　＄r{i}＝{dsl}\n"));
    }
    let lua = run_main_scene(&transpile(&source));

    let flat = cases
        .iter()
        .map(|(_, flat)| format!("({flat})"))
        .collect::<Vec<_>>()
        .join(",\n");
    let chunk = format!(
        r#"
        local var = RS_ACT.var
        local flat = {{ {flat} }}
        local out = {{}}
        for i = 1, {n} do
            local got = var["r" .. (i - 1)]
            local want = flat[i]
            local ok = got ~= nil and type(got) == type(want) and got == want
            table.insert(out, string.format("%d|%s|%s|%s", i - 1, tostring(ok), tostring(got), tostring(want)))
        end
        return table.concat(out, "\n")
        "#,
        n = cases.len()
    );
    let result = eval_str(&lua, &chunk);

    let mismatches: Vec<String> = result
        .lines()
        .filter(|l| !l.split('|').nth(1).is_some_and(|ok| ok == "true"))
        .map(|l| {
            let i: usize = l.split('|').next().unwrap().parse().unwrap();
            format!("{} (flat: {}) => {l}", cases[i].0, cases[i].1)
        })
        .collect();
    assert_eq!(result.lines().count(), cases.len());
    assert!(
        mismatches.is_empty(),
        "regrouped {what} differs from flat Lua:\n{}",
        mismatches.join("\n")
    );

    // 正しい式では警告を出さない
    let warns = eval_str(&lua, r#"return table.concat(RS_WARNS, "\n")"#);
    assert_eq!(warns, "");
}

// ========================================================================
// 連結の失敗: 警告＋値なし、内側の失敗は外側で警告を足さない（3.3・3.5）
// ========================================================================

#[test]
fn test_concat_failures_warn_and_yield_nil() {
    let source = r#"
＊メイン
　＄r1＝「合計」＆＄未代入＆「個」
　＄r2＝「a」＆（＄未代入＋1）
　＄r3＝（＄未代入＆「x」）＋1
　＄r4＝「a」＆＠＊未定義関数（）
"#;
    let lua = run_main_scene(&transpile(source));

    let vars = eval_str(
        &lua,
        r#"
        local v = RS_ACT.var
        return string.format("r1=%s r2=%s r3=%s r4=%s",
            tostring(v.r1), tostring(v.r2), tostring(v.r3), tostring(v.r4))
        "#,
    );
    assert_eq!(vars, "r1=nil r2=nil r3=nil r4=nil");

    let warns = eval_str(&lua, r#"return table.concat(RS_WARNS, "\n")"#);
    let expected_warns = [
        "act:concat - operand is not a string or number: op='&', operand='var.未代入', value=nil",
        "act:arith - operand is not a number: op='+', operand='var.未代入', value=nil",
        "act:concat - operand is not a string or number: op='&', operand='var.未代入', value=nil",
        "act:global_fn - function not found: key='未定義関数'",
        "act:concat - operand is not a string or number: op='&', operand='@*未定義関数()', value=nil",
    ]
    .join("\n");
    assert_eq!(warns, expected_warns);
}
