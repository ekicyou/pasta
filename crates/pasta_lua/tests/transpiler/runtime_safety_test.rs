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

/// ログを記録する `@pasta_log` を差し込んだランタイムで生成コードを読み込み、`メイン_1` を実行する。
///
/// `@pasta_log` の表そのものの関数を差し替えるので、先にロード済みのモジュール（pasta.word など）の
/// ログも集まる。実行後の act は Lua のグローバル `RS_ACT`、警告は `RS_WARNS`、
/// 全レベル（trace〜error）のログは `RS_LOGS` に `レベル|文言` で置く。値の表記には `RS_FMT` を使う。
fn run_main_scene(lua_code: &str) -> Lua {
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(
        r#"
        RS_WARNS = {}
        RS_LOGS = {}
        local log = package.loaded["@pasta_log"]
        for _, level in ipairs({ "trace", "debug", "info", "warn", "error" }) do
            log[level] = function(msg)
                table.insert(RS_LOGS, level .. "|" .. tostring(msg))
                if level == "warn" then table.insert(RS_WARNS, msg) end
            end
        end

        -- 値の表記: nil は nil、非数は nan、文字列は '…'、数値は tostring、それ以外は (型)
        function RS_FMT(v)
            local t = type(v)
            if t == "nil" then return "nil" end
            if t == "number" then return v ~= v and "nan" or tostring(v) end
            if t == "string" then return "'" .. v .. "'" end
            return "(" .. t .. ")"
        end
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
        SCENE.get_start("メイン_1")(RS_ACT)
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

/// 全レベルのログを 1 行ずつ（`レベル|文言`）返す
fn all_logs(lua: &Lua) -> Vec<String> {
    lua.load("return RS_LOGS").eval::<Vec<String>>().unwrap()
}

/// `RS_ACT.var` の各変数を `名前=値` で空白区切りにして返す（値は `RS_FMT` の表記）
fn vars(lua: &Lua, names: &[&str]) -> String {
    names
        .iter()
        .map(|n| {
            format!(
                "{n}={}",
                eval_str(lua, &format!("return RS_FMT(RS_ACT.var[\"{n}\"])"))
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        // U22: 未代入の変数は算術の文脈で 0 とみなす（expr-nil-coercion 1.1）
        "talk|さくら|true|1",
        // U08: \\ は 2 文字のまま 1 トークン
        "talk|さくら|true|C：",
        r"talk|さくら|true|\\",
        "talk|さくら|true|new",
        "talk|さくら|true|終わり",
        r"talk|さくら|true|\\",
    ]
    .join("\n");
    assert_eq!(tokens, expected_tokens);

    // 変数状態: 値なしの代入は未代入（U18 の式）。U22 の算術は nil を 0、「abc」を警告＋0 とみなす
    // （expr-nil-coercion 1.1・4.1）
    let vars = eval_str(
        &lua,
        r#"
        local v = RS_ACT.var
        return string.format("u18=%s x=%s y=%s", tostring(v.u18), tostring(v.x), tostring(v.y))
        "#,
    );
    assert_eq!(vars, "u18=nil x=1 y=0");

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
        "act:arith - operand is not a number: op='*', value='abc' (string)",
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
// 連結の nil: 空文字列とみなし、連結も算術も値なしを作らない（expr-nil-coercion 2.1・2.2・4.1）
// ========================================================================

/// 連結の被演算子の nil は空文字列、括弧内の算術の nil は 0。内側の連結の結果（`x`）を算術に
/// 使うと変換できない値の警告 1 行と 0。見つからない関数は呼び出しの時点の警告だけ
#[test]
fn test_concat_with_nil_operands_never_yields_nil() {
    let source = r#"
＊メイン
　＄r1＝「合計」＆＄未代入＆「個」
　＄r2＝「a」＆（＄未代入＋1）
　＄r3＝（＄未代入＆「x」）＋1
　＄r4＝「a」＆＠＊未定義関数（）
"#;
    let lua = run_main_scene(&transpile(source));

    assert_eq!(
        vars(&lua, &["r1", "r2", "r3", "r4"]),
        "r1='合計個' r2='a1' r3=1 r4='a'"
    );
    assert_eq!(
        all_logs(&lua),
        [
            "warn|act:arith - operand is not a number: op='+', value='x' (string)",
            "warn|act:global_fn - function not found: key='未定義関数'",
        ]
    );
}

// ========================================================================
// 式文: 関数呼び出しでない式も Lua の文として読み込め、副作用は 1 回（expr-nil-coercion 1.4・2.4）
// ========================================================================

#[test]
fn test_non_call_expression_statements_load_and_run_once() {
    let source = r#"
＊メイン
```lua
function SCENE.f(act)
    RS_F_COUNT = (RS_F_COUNT or 0) + 1
end
```
　＄x＝２
　＄＝１
　＄＝＄x
　＄＝（＠f（））
　＄＝＄未代入＆「x」
　＄後＝「続行」
"#;
    let lua = run_main_scene(&transpile(source));

    let result = eval_str(
        &lua,
        r#"return string.format("f=%s 後=%s", tostring(RS_F_COUNT), tostring(RS_ACT.var["後"]))"#,
    );
    assert_eq!(result, "f=1 後=続行");
}

// ========================================================================
// 式の中の nil: 算術なら 0、連結なら空文字列、ログなし（expr-nil-coercion 1.x・2.x・3.x）
// ========================================================================

/// nil の出どころごとに、算術（`＋１`）と連結（`「a」＆`）の結果が 0・空文字列になり、
/// 呼び出しの時点の既存の警告のほかにログが出ない（1.1・1.6・2.1・3.1・3.3・7.1）
#[test]
fn test_nil_sources_coerce_to_zero_and_empty_without_logs() {
    let source = r#"
＊メイン
```lua
function SCENE.無返却(act)
end
```
　＄値なし＝＠無返却（）
　＄a1＝＄未代入＋１
　＄a2＝＄＊未代入nil＋１
　＄a3＝＄ｒ０＋１
　＄a4＝＠無返却（）＋１
　＄a5＝＠＊未定義（）＋１
　＄a6＝＠＄未代入（）＋１
　＄a7＝＄値なし＋１
　＄c1＝「a」＆＄未代入
　＄c2＝「a」＆＄＊未代入nil
　＄c3＝「a」＆＄ｒ０
　＄c4＝「a」＆＠無返却（）
　＄c5＝「a」＆＠＊未定義（）
　＄c6＝「a」＆＠＄未代入（）
　＄c7＝「a」＆＄値なし
　＞引数なし

　・引数なし
　＄a8＝＄０＋１
　＄c8＝「a」＆＄０
"#;
    let lua = run_main_scene(&transpile(source));

    assert_eq!(
        vars(
            &lua,
            &["値なし", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8"]
        ),
        "値なし=nil a1=1 a2=1 a3=1 a4=1 a5=1 a6=1 a7=1 a8=1"
    );
    assert_eq!(
        vars(&lua, &["c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8"]),
        "c1='a' c2='a' c3='a' c4='a' c5='a' c6='a' c7='a' c8='a'"
    );
    // 見つからない関数・未代入の変数での動的呼び出しは、呼び出しの時点の既存の警告だけ（3.3）
    assert_eq!(
        all_logs(&lua),
        [
            "warn|act:global_fn - function not found: key='未定義'",
            "warn|act:expr_fn - undefined variable: 'var.未代入'",
            "warn|act:global_fn - function not found: key='未定義'",
            "warn|act:expr_fn - undefined variable: 'var.未代入'",
        ]
    );
}

/// 5 つの算術演算子の左右どちらの nil も 0。除数の nil は 0 による除算・剰余と同じ（1.2・1.3）
#[test]
fn test_nil_operand_on_either_side_of_each_arith_operator() {
    let source = r#"
＊メイン
　＄o1＝１＋＄未代入
　＄o2＝＄未代入＋１
　＄o3＝１－＄未代入
　＄o4＝＄未代入－１
　＄o5＝２＊＄未代入
　＄o6＝＄未代入＊２
　＄o7＝１／＄未代入
　＄o8＝＄未代入／２
　＄o9＝１％＄未代入
　＄o10＝＄未代入％３
"#;
    let lua = run_main_scene(&transpile(source));

    assert_eq!(
        vars(
            &lua,
            &["o1", "o2", "o3", "o4", "o5", "o6", "o7", "o8", "o9", "o10"]
        ),
        "o1=1 o2=1 o3=1 o4=-1 o5=0 o6=0 o7=inf o8=0 o9=nan o10=0"
    );
    assert_eq!(all_logs(&lua), Vec::<String>::new());
}

/// 変数代入・式文・関数の引数・Call の引数・動的コールのターゲットで同じ結果（1.4・2.4）
#[test]
fn test_nil_coercion_is_the_same_in_every_expression_position() {
    let source = r#"
＊メイン
```lua
function SCENE.記録(act, v)
    RS_REC = RS_REC or {}
    table.insert(RS_REC, RS_FMT(v))
end
```
　＄p1＝＄未代入＋１
　＄p2＝「a」＆＄未代入
　＄＝＄未代入＋１
　＄＝「a」＆＄未代入
　＄＝＠記録（＄未代入＋１）
　＄＝＠記録（「a」＆＄未代入）
　＞受け（＄未代入＋１、「a」＆＄未代入）
　＞「数の行き先」＆（＄未代入＋１）
　＞＄未代入＆「文字の行き先」
　＄後＝「続行」

　・受け
　＄q1＝＄０
　＄q2＝＄１

＊数の行き先1
　＄来た数＝１

＊文字の行き先
　＄来た文字＝１
"#;
    let lua = run_main_scene(&transpile(source));

    assert_eq!(
        vars(&lua, &["p1", "p2", "q1", "q2", "来た数", "来た文字", "後"]),
        "p1=1 p2='a' q1=1 q2='a' 来た数=1 来た文字=1 後='続行'"
    );
    assert_eq!(
        eval_str(&lua, r#"return table.concat(RS_REC or {}, " ")"#),
        "1 'a'"
    );
    // 失敗表記を積まない
    assert_eq!(eval_str(&lua, "return tostring(#RS_ACT.token)"), "0");
    assert_eq!(all_logs(&lua), Vec::<String>::new());
}

/// 入れ子と境界: 内側の演算の結果を外側が使い、警告は変換できない値の 1 行だけ。
/// 空文字列は nil ではない。表を返す関数の演算でメタメソッドが呼ばれない（1.5・2.2・2.3・4.1・4.2・4.4・4.5）
#[test]
fn test_nested_operations_and_non_convertible_values() {
    let source = r#"
＊メイン
```lua
function SCENE.真(act)
    return true
end
function SCENE.表(act)
    local hit = function() RS_META = "called" return 1 end
    return setmetatable({}, { __add = hit, __concat = hit, __tostring = hit })
end
```
　＄n1＝（「abc」＊２）＋１
　＄n2＝（＄未代入＆「x」）＋1
　＄n3＝「合計」＆（＄未代入＋１）
　＄n4＝「a」＆（＠真（）＆「b」）
　＄空＝「」
　＄n5＝＄空＋１
　＄n6＝＠表（）＋１
　＄n7＝「a」＆＠表（）
"#;
    let lua = run_main_scene(&transpile(source));

    assert_eq!(
        vars(&lua, &["n1", "n2", "n3", "n4", "n5", "n6", "n7"]),
        "n1=1 n2=1 n3='合計1' n4='ab' n5=1 n6=1 n7='a'"
    );
    assert_eq!(eval_str(&lua, "return tostring(RS_META)"), "nil");
    assert_eq!(
        all_logs(&lua),
        [
            "warn|act:arith - operand is not a number: op='*', value='abc' (string)",
            "warn|act:arith - operand is not a number: op='+', value='x' (string)",
            "warn|act:concat - operand is not a string or number: op='&', operand='@真()', value=true (boolean)",
            "warn|act:arith - operand is not a number: op='+', operand='var.空', value='' (string)",
            "warn|act:arith - operand is not a number: op='+', operand='@表()', value=(table)",
            "warn|act:concat - operand is not a string or number: op='&', operand='@表()', value=(table)",
        ]
    );
}

/// 変えない規則: 台詞の未代入の変数は空文字＋既存の警告、関数の引数に nil を 1 つ渡すとそのまま nil が届く（5.1・5.5）
#[test]
fn test_nil_outside_operators_is_unchanged() {
    let source = r##"
％さくら
　＠通常：\s[0]

＊メイン
```lua
function SCENE.受取(act, ...)
    RS_ARGS = string.format("n=%d v=%s", select("#", ...), RS_FMT((...)))
end
```
　さくら：前＄未代入　後
　＄＝＠受取（＄未代入）
"##;
    let lua = run_main_scene(&transpile(source));

    let texts = eval_str(
        &lua,
        r#"
        local out = {}
        for _, t in ipairs(RS_ACT.token) do table.insert(out, tostring(t.text)) end
        return table.concat(out, "|")
        "#,
    );
    assert_eq!(texts, "前|後");
    assert_eq!(eval_str(&lua, "return RS_ARGS"), "n=1 v=nil");
    assert_eq!(
        all_logs(&lua),
        ["warn|act:talk - undefined variable: 'var.未代入'"]
    );
}
