//! 実行時の失敗の出力（failure-output-unification）の DSL → 実行テスト
//!
//! Requirements: 2.11, 3.5, 6.1, 10.2, 10.5
//!
//! 失敗の対応表（design.md「報告箇所」）#1〜#13 と `act:failure` の各箇所を、`.pasta` から
//! トランスパイルして実ランタイム（pasta_scripts の act.lua）で実行し、積んだトークン・
//! ログの警告・変数の値（失敗した参照の戻り値）を確かめる。
//!
//! 1 つの失敗の種類を 1 つの `Case` にしてあるので、種類ごとに期待値だけを書き換えられる。
//! 今の期待値は変更前の挙動の固定（特性化）であり、以降のタスクでは載せ替えた種類の期待値
//! だけを書き換える。ログの警告の文言と変数の値（3.5・6.1）は、重ねて出していた警告を
//! 落とす場合（#13 の関数が見つからないとき）を除いて変えない。
//! 失敗の無い辞書の出力は `sample.generated.lua` を使う既存のスナップショットが固定する（10.5）。

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use mlua::Lua;

/// 1 つの失敗の種類の DSL と、実行後の期待値
struct Case {
    /// 失敗の対応表の番号と箇所（失敗したときのメッセージに出す）
    name: &'static str,
    /// `＊メイン` の本文（全角空白の字下げ込み）
    body: &'static str,
    /// `＊メイン` の先頭に `LUA_HELPERS` を置くか
    lua: bool,
    /// 積んだトークン（`種類|アクター名|テキスト`。アクターが無ければ `-`）
    tokens: &'static [&'static str],
    /// ログの警告（現行の文言のまま）
    warns: &'static [&'static str],
    /// 実行後の `var` の値（`名前=値`。値の表記は `RS_FMT`）
    vars: &'static [&'static str],
}

/// 登録済みアクター さくら（以降のケースの話者）
const SAKURA: &str = "％さくら\n　＠通常：\\s[0]\n";

/// 失敗を起こすための Lua ブロック（`＊メイン` のシーン関数として登録される）
const LUA_HELPERS: &str = r#"```lua
function SCENE.真偽(act)
    return true
end
function SCENE.値なし(act)
    return nil
end
function SCENE.直接Call(act)
    act:call(nil, nil, nil)
end
function SCENE.直接失敗(act)
    act:failure("Lua失敗：テスト", "test - direct failure")
end
function SCENE.単語参照(act)
    return act:word("無い単語")
end
```
"#;

/// ログを記録する `@pasta_log` を差し込んだランタイムで生成コードを読み込み、`メイン_1` を実行する。
///
/// `@pasta_log` の表そのものの関数を差し替えるので、先にロード済みのモジュール（pasta.word など）の
/// ログも集まる。実行後の act は Lua のグローバル `RS_ACT`、警告は `RS_WARNS` に置く。
fn run_main_scene(lua_code: &str) -> Lua {
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(
        r#"
        RS_WARNS = {}
        local log = package.loaded["@pasta_log"]
        for _, level in ipairs({ "trace", "debug", "info", "warn", "error" }) do
            log[level] = function(msg)
                if level == "warn" then table.insert(RS_WARNS, tostring(msg)) end
            end
        end

        -- 値の表記: nil は nil、文字列は '…'、数値は tostring、それ以外は (型)
        function RS_FMT(v)
            local t = type(v)
            if t == "nil" then return "nil" end
            if t == "number" then return tostring(v) end
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

/// 積んだトークンを `種類|アクター名|テキスト` で 1 行ずつ返す
fn tokens(lua: &Lua) -> Vec<String> {
    lua.load(
        r#"
        local lines = {}
        for _, t in ipairs(RS_ACT.token) do
            local name = t.actor and t.actor.name or "-"
            table.insert(lines, string.format("%s|%s|%s", t.type, name, tostring(t.text)))
        end
        return lines
        "#,
    )
    .eval::<Vec<String>>()
    .unwrap()
}

/// ログの警告を 1 行ずつ返す
fn warns(lua: &Lua) -> Vec<String> {
    lua.load("return RS_WARNS").eval::<Vec<String>>().unwrap()
}

/// 期待値の `名前=値` の名前について、実行後の `var` の値を同じ形で返す
fn vars(lua: &Lua, expected: &[&str]) -> Vec<String> {
    expected
        .iter()
        .map(|e| {
            let name = e.split('=').next().unwrap();
            let v = lua
                .load(format!("return RS_FMT(RS_ACT.var[\"{name}\"])"))
                .eval::<String>()
                .unwrap();
            format!("{name}={v}")
        })
        .collect()
}

fn run_case(case: &Case) {
    let lua_block = if case.lua { LUA_HELPERS } else { "" };
    let source = format!("{SAKURA}\n＊メイン\n{lua_block}{}", case.body);
    let lua = run_main_scene(&transpile(&source));
    assert_eq!(tokens(&lua), case.tokens, "[{}] tokens", case.name);
    assert_eq!(warns(&lua), case.warns, "[{}] warns", case.name);
    assert_eq!(vars(&lua, case.vars), case.vars, "[{}] vars", case.name);
}

// ========================================================================
// 失敗の対応表 #1〜#13・act:failure（変更前の挙動の固定）
// ========================================================================

/// #1 `act:talk`: アクション行で値の無い変数を展開する（未代入・値なしを代入・渡されていない引数）
#[test]
fn site_01_talk_undefined_variable() {
    run_case(&Case {
        name: "#1 act:talk",
        lua: true,
        body: "　さくら：今日は＄話題。の話\n　＄n＝＠値なし（）\n　さくら：＄n\n　さくら：＄０だよ\n",
        tokens: &[
            "talk|さくら|今日は",
            "raw_script|-|【値の無い変数：＄話題】",
            "talk|さくら|。の話",
            "raw_script|-|【値の無い変数：＄n】",
            "raw_script|-|【値の無い変数：＄０】",
            "talk|さくら|だよ",
        ],
        warns: &[
            "act:talk - undefined variable: 'var.話題'",
            "act:talk - undefined variable: 'var.n'",
            "act:talk - undefined variable: 'args[1]'",
        ],
        vars: &["n=nil"],
    });
}

/// #2 `act:word`: アクターの無い単語参照（代入行の右辺）で単語が見つからない
#[test]
fn site_02_act_word_not_found() {
    run_case(&Case {
        name: "#2 act:word",
        lua: false,
        body: "　＄w＝＠挨拶\n",
        tokens: &["raw_script|-|【単語が見つからない：＠挨拶】"],
        warns: &["act:word - handler not found: key='挨拶', mode='word', via=act"],
        vars: &["w=nil"],
    });
}

/// #3 `proxy:word`: アクター付きの行の単語参照で単語が見つからない
#[test]
fn site_03_proxy_word_not_found() {
    run_case(&Case {
        name: "#3 proxy:word",
        lua: false,
        body: "　さくら：前＠挨拶。後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【単語が見つからない：＠挨拶】",
            "talk|さくら|。後",
        ],
        warns: &["proxy:word - handler not found: key='挨拶', mode='word', via=proxy(さくら)"],
        vars: &[],
    });
}

/// #4 act の `call_expr`: アクターの無い関数呼び出し（代入行の右辺）で関数が見つからない
#[test]
fn site_04_act_expr_fn_not_found() {
    run_case(&Case {
        name: "#4 act:expr_fn",
        lua: false,
        body: "　＄f＝＠計算（１）\n",
        tokens: &["raw_script|-|【関数が見つからない：＠計算（）】"],
        warns: &["act:expr_fn - handler not found: key='計算', mode='expr', via=act"],
        vars: &["f=nil"],
    });
}

/// #5 proxy の `call_expr`: アクター付きの行の関数呼び出しで関数が見つからない
#[test]
fn site_05_proxy_expr_fn_not_found() {
    run_case(&Case {
        name: "#5 proxy:expr_fn",
        lua: false,
        body: "　さくら：前＠計算（１）後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【関数が見つからない：＠計算（）】",
            "talk|さくら|後",
        ],
        warns: &["proxy:expr_fn - handler not found: key='計算', mode='expr', via=proxy(さくら)"],
        vars: &[],
    });
}

/// #6 `act:global_fn`: グローバル関数が見つからない（アクション行・代入行）
#[test]
fn site_06_global_fn_not_found() {
    run_case(&Case {
        name: "#6 act:global_fn",
        lua: false,
        body: "　さくら：前＠＊計算（１）後\n　＄g＝＠＊計算（）\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【関数が見つからない：＠＊計算（）】",
            "talk|さくら|後",
            "raw_script|-|【関数が見つからない：＠＊計算（）】",
        ],
        warns: &[
            "act:global_fn - function not found: key='計算'",
            "act:global_fn - function not found: key='計算'",
        ],
        vars: &["g=nil"],
    });
}

/// #7 `WORD.dynamic_key`: 動的参照の値が名前にならない（値なし・空文字列・型）。
/// 名前にならないときは外側の単語・関数・Call の検索をしない（見つからない警告を重ねない）
#[test]
fn site_07_dynamic_key_not_a_name() {
    run_case(&Case {
        name: "#7 WORD.dynamic_key",
        lua: true,
        body: concat!(
            "　＄空＝「」\n",
            "　＄真＝＠真偽（）\n",
            "　さくら：前＠＄未代入。後\n",
            "　さくら：＠＄空（）\n",
            "　＄d＝＠＄真\n",
            "　＄e＝＠＄未代入（）\n",
            "　＞＄行き先\n",
        ),
        tokens: &[
            "talk|さくら|前",
            "talk|さくら|。後",
            "raw_script|-|【名前にならない値：＄行き先（値なし）】",
        ],
        warns: &[
            "proxy:word - undefined variable: 'var.未代入'",
            "proxy:expr_fn - empty variable: 'var.空'",
            "act:word - unsupported value type: 'var.真' (boolean)",
            "act:expr_fn - undefined variable: 'var.未代入'",
            "act:call - undefined variable: 'var.行き先'",
        ],
        vars: &["d=nil", "e=nil"],
    });
}

/// #8 `act:actor_proxy`: 未登録アクター。話者がその名前に切り替わるたびに目印と警告を 1 回
#[test]
fn site_08_unregistered_actor() {
    run_case(&Case {
        name: "#8 act:actor_proxy",
        lua: false,
        body: "　ぱすたー：こんにちは\n　ぱすたー：続き\n　さくら：やあ\n　ぱすたー：また\n",
        tokens: &[
            "talk|ぱすたー|【未登録アクター：ぱすたー】",
            "talk|ぱすたー|こんにちは",
            "talk|ぱすたー|続き",
            "talk|さくら|やあ",
            "talk|ぱすたー|【未登録アクター：ぱすたー】",
            "talk|ぱすたー|また",
        ],
        warns: &[
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
        ],
        vars: &[],
    });
}

/// #9 `ACT.num`: 算術の被演算子が数にできない（値なしを除く）。0 として計算する
#[test]
fn site_09_num_not_a_number() {
    run_case(&Case {
        name: "#9 ACT.num",
        lua: false,
        body: "　＄x＝「abc」\n　＄y＝＄x＋１\n　＄z＝「xy」＊２\n",
        tokens: &[],
        warns: &[
            "act:arith - operand is not a number: op='+', operand='var.x', value='abc' (string)",
            "act:arith - operand is not a number: op='*', value='xy' (string)",
        ],
        vars: &["y=1", "z=0"],
    });
}

/// #10 `ACT.str`: 連結の被演算子が文字列にできない（値なしを除く）。空文字列として連結する
#[test]
fn site_10_str_not_a_string() {
    run_case(&Case {
        name: "#10 ACT.str",
        lua: true,
        body: "　＄t＝＠真偽（）\n　＄c＝＄t＆「後」\n　＄c2＝「前」＆＠真偽（）\n",
        tokens: &[],
        warns: &[
            "act:concat - operand is not a string or number: op='&', operand='var.t', value=true (boolean)",
            "act:concat - operand is not a string or number: op='&', operand='@真偽()', value=true (boolean)",
        ],
        vars: &["c='後'", "c2='前'"],
    });
}

/// #11 `act:call`: Lua から値の無いキーで直接呼ぶ（Lua ブロックから呼んだ act のメソッド）
#[test]
fn site_11_call_nil_key() {
    run_case(&Case {
        name: "#11 act:call(nil)",
        lua: true,
        body: "　さくら：前＠直接Call（）後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【名前にならない値：値なし】",
            "talk|さくら|後",
        ],
        warns: &["act:call - nil key (undefined variable?), skipping scene search"],
        vars: &[],
    });
}

/// #12 `act:call`: Call のターゲットのシーンが見つからない
#[test]
fn site_12_call_scene_not_found() {
    run_case(&Case {
        name: "#12 act:call not found",
        lua: false,
        body: "　さくら：前\n　＞会話２\n　さくら：後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【シーンが見つからない：＞会話２】",
            "talk|さくら|後",
        ],
        warns: &["act:call - handler not found: key='会話２', mode='scene', via=act"],
        vars: &[],
    });
}

/// #13 `act:call_key`: 変数パスの無い動的コールの式の値が名前にならない。
/// 式が関数 1 つでその関数が見つからないときは、関数だけが報告し Call は黙る（重ねない規則）
#[test]
fn site_13_call_key_not_a_name() {
    run_case(&Case {
        name: "#13 act:call_key",
        lua: true,
        body: "　＞＠値なし（）\n　＞＠真偽（）\n　＞＠無い関数（）\n",
        tokens: &[
            "raw_script|-|【名前にならない値：＠値なし（）（値なし）】",
            "raw_script|-|【名前にならない値：＠真偽（）（真偽値）】",
            "raw_script|-|【関数が見つからない：＠無い関数（）】",
        ],
        warns: &[
            "act:call - key is not a string or number: operand='@値なし()', value=nil",
            "act:call - key is not a string or number: operand='@真偽()', value=true (boolean)",
            "act:expr_fn - handler not found: key='無い関数', mode='expr', via=act",
        ],
        vars: &[],
    });
}

/// 生成コードの評価順の固定: 動的コールの式が関数 1 つのとき、関数の参照は `call_key` の
/// 引数として `call_key` の直前に評価される（「報告済み」の印はこの順序を前提にする）。
/// `＞＠無い関数（）` は表記 1 つ・警告 1 行になる
#[test]
fn dynamic_call_fn_evaluated_immediately_before_call_key() {
    let lua_code = transpile(&format!("{SAKURA}\n＊メイン\n　＞＠無い関数（）\n"));
    assert!(
        lua_code.contains(r#"act:call_key(act:expr_fn("無い関数"), nil, "@無い関数()")"#),
        "generated code must pass the function call directly to call_key:\n{lua_code}"
    );
    let lua = run_main_scene(&lua_code);
    assert_eq!(
        tokens(&lua),
        ["raw_script|-|【関数が見つからない：＠無い関数（）】"]
    );
    assert_eq!(
        warns(&lua),
        ["act:expr_fn - handler not found: key='無い関数', mode='expr', via=act"]
    );
}

/// `act:failure`: Lua ブロックから直接呼ぶ（text をエスケープして【】で囲んで積み、警告を出す）
#[test]
fn site_14_act_failure_direct() {
    run_case(&Case {
        name: "act:failure",
        lua: true,
        body: "　さくら：前＠直接失敗（）後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【Lua失敗：テスト】",
            "talk|さくら|後",
        ],
        warns: &["test - direct failure"],
        vars: &[],
    });
}

/// Lua ブロックから呼んだ `act:word` が見つからない（#2 の別経路。2.12 の位置）
#[test]
fn site_02_act_word_from_lua_block() {
    run_case(&Case {
        name: "#2 act:word from Lua block",
        lua: true,
        body: "　さくら：前＠単語参照（）後\n",
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【単語が見つからない：＠無い単語】",
            "talk|さくら|後",
        ],
        warns: &["act:word - handler not found: key='無い単語', mode='word', via=act"],
        vars: &[],
    });
}

// ========================================================================
// 今は警告を出さない状況（2.11）: 警告も表記も出さないまま保つ
// ========================================================================

/// 式の中の値なし（算術・連結の被演算子・代入）、見つかった関数が値なしを返す、は黙る
#[test]
fn silent_nil_in_expressions() {
    run_case(&Case {
        name: "silent: nil in expressions",
        lua: true,
        body: concat!(
            "　＄a＝＄未代入＋１\n",
            "　＄b＝＄未代入＆「x」\n",
            "　＄c＝＄未代入\n",
            "　＄d＝＠値なし（）＋１\n",
            "　さくら：前＠値なし（）後\n",
        ),
        tokens: &["talk|さくら|前", "talk|さくら|後"],
        warns: &[],
        vars: &["a=1", "b='x'", "c=nil", "d=1"],
    });
}

/// 静的な単語参照の空の名前（Lua から `act:word("")`・`act:word(nil)`）は黙って値なし
#[test]
fn silent_empty_static_word_name() {
    let lua = run_main_scene(&transpile("＊メイン\n"));
    let got = lua
        .load(
            r#"
            local r1 = RS_ACT:word("")
            local r2 = RS_ACT:word(nil)
            return RS_FMT(r1) .. " " .. RS_FMT(r2) .. " " .. #RS_ACT.token
            "#,
        )
        .eval::<String>()
        .unwrap();
    assert_eq!(got, "nil nil 0");
    assert_eq!(warns(&lua), Vec::<String>::new());
}
