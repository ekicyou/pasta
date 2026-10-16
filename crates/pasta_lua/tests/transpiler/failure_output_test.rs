//! 実行時の失敗の出力（failure-output-unification）の DSL → 実行テスト
//!
//! Requirements: 2.11, 2.12, 3.5, 4.1, 4.3, 4.4, 5.1, 5.2, 5.5, 6.1, 10.2, 10.5
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
    run_main_scene_with(lua_code, "pasta.act")
}

/// `run_main_scene` と同じだが、act を `act_module`（`pasta.act`・`pasta.shiori.act`）で作る
fn run_main_scene_with(lua_code: &str, act_module: &str) -> Lua {
    let lua = load_scenes(lua_code);
    lua.load(format!(
        r#"
        local SCENE = require "pasta.scene"
        local STORE = require "pasta.store"
        local ACT = require "{act_module}"
        RS_ACT = ACT.new(STORE.actors)
        SCENE.get_start("メイン_1")(RS_ACT)
        "#
    ))
    .exec()
    .expect("scene must run without a runtime error");
    lua
}

/// ログを記録する `@pasta_log` を差し込んだランタイムで生成コードを読み込み、シーンを確定する（実行はしない）
fn load_scenes(lua_code: &str) -> Lua {
    let lua = create_runtime_with_finalize().unwrap();
    // pasta.shiori.sakura_builder が使う（既定の会話設定。ウェイトは既定値で入る）
    let sakura = pasta_lua::sakura_script::register(&lua, None).unwrap();
    let loaded: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("package")
        .unwrap()
        .get("loaded")
        .unwrap();
    loaded.set("@pasta_sakura_script", sakura).unwrap();
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
/// 参照の位置に表記 1 つ・警告 1 行。名前にならないときは外側の単語・関数・Call の検索をせず、
/// `＞＠＄名前（）` も `call_key` が重ねて出さない（重ねない規則）
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
            "　＞＠＄未代入（）\n",
        ),
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【名前にならない値：＄未代入（値なし）】",
            "talk|さくら|。後",
            "raw_script|-|【名前にならない値：＄空（空文字列）】",
            "raw_script|-|【名前にならない値：＄真（真偽値）】",
            "raw_script|-|【名前にならない値：＄未代入（値なし）】",
            "raw_script|-|【名前にならない値：＄行き先（値なし）】",
            "raw_script|-|【名前にならない値：＄未代入（値なし）】",
        ],
        warns: &[
            "proxy:word - undefined variable: 'var.未代入'",
            "proxy:expr_fn - empty variable: 'var.空'",
            "act:word - unsupported value type: 'var.真' (boolean)",
            "act:expr_fn - undefined variable: 'var.未代入'",
            "act:call - undefined variable: 'var.行き先'",
            "act:expr_fn - undefined variable: 'var.未代入'",
        ],
        vars: &["d=nil", "e=nil"],
    });
}

/// #8 `act:actor_proxy`: 未登録アクター。話者がその名前に切り替わるたびに目印と警告を 1 回。
/// 目印はアクター付きの `raw_script`。直前の話者の遡りは目印を発言と同じに数え、
/// アクターの無い失敗表記では止まらない（目印と失敗表記だけの行の次の継続行でも 1 回だけ）
#[test]
fn site_08_unregistered_actor() {
    run_case(&Case {
        name: "#8 act:actor_proxy",
        lua: false,
        body: concat!(
            "　ぱすたー：こんにちは\n",
            "　ぱすたー：続き\n",
            "　さくら：やあ\n",
            "　ぱすたー：＠無い単語\n",
            "　ぱすたー：また\n",
        ),
        tokens: &[
            "raw_script|ぱすたー|【未登録アクター：ぱすたー】",
            "talk|ぱすたー|こんにちは",
            "talk|ぱすたー|続き",
            "talk|さくら|やあ",
            "raw_script|ぱすたー|【未登録アクター：ぱすたー】",
            "raw_script|-|【単語が見つからない：＠無い単語】",
            "talk|ぱすたー|また",
        ],
        warns: &[
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
            "proxy:word - handler not found: key='無い単語', mode='word', via=proxy(ぱすたー)",
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
        tokens: &[
            "raw_script|-|【数にできない値：＄x（\"abc\"）】",
            "raw_script|-|【数にできない値：\"xy\"】",
        ],
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
        tokens: &[
            "raw_script|-|【文字列にできない値：＄t（true）】",
            "raw_script|-|【文字列にできない値：＠真偽（）（true）】",
        ],
        warns: &[
            "act:concat - operand is not a string or number: op='&', operand='var.t', value=true (boolean)",
            "act:concat - operand is not a string or number: op='&', operand='@真偽()', value=true (boolean)",
        ],
        vars: &["c='後'", "c2='前'"],
    });
}

/// #9・#10 の位置: act を受け取らない `PASTA.num`・`PASTA.str` の失敗が、実行中の act を引いて
/// 起きた時点の出力の位置に出る（代入行・Call 行の引数・式の中）
#[test]
fn site_09_10_position_in_output() {
    run_case(&Case {
        name: "#9/#10 position",
        lua: true,
        body: concat!(
            "　＄x＝「abc」\n",
            "　＄t＝＠真偽（）\n",
            "　さくら：前\n",
            "　＄y＝＄x＋１\n",
            "　さくら：中\n",
            "　＞サブ（＄x＊２）\n",
            "　さくら：後\n",
            "　＄w＝（＄t＆「a」）＋１\n",
            "　さくら：終わり\n",
            "\n＊サブ\n",
            "　さくら：＄０\n",
        ),
        tokens: &[
            "talk|さくら|前",
            "raw_script|-|【数にできない値：＄x（\"abc\"）】",
            "talk|さくら|中",
            "raw_script|-|【数にできない値：＄x（\"abc\"）】",
            "talk|さくら|0",
            "talk|さくら|後",
            "raw_script|-|【文字列にできない値：＄t（true）】",
            "raw_script|-|【数にできない値：\"a\"】",
            "talk|さくら|終わり",
        ],
        warns: &[
            "act:arith - operand is not a number: op='+', operand='var.x', value='abc' (string)",
            "act:arith - operand is not a number: op='*', operand='var.x', value='abc' (string)",
            "act:concat - operand is not a string or number: op='&', operand='var.t', value=true (boolean)",
            "act:arith - operand is not a number: op='+', value='a' (string)",
        ],
        vars: &["y=1", "w=1"],
    });
}

/// #9 の位置（チェイントークの再開後）: 再開までの間に別の act がメインスレッドで
/// シーンを始めても、再開したシーンのコルーチンの act に、再開後の出力の位置で出る
#[test]
fn site_09_position_after_chain_talk_resume() {
    let source = format!(
        "{SAKURA}\n＊メイン\n　＄x＝「abc」\n　さくら：前半\n　＞チェイントーク\n　さくら：後半\n　＄y＝＄x＋１\n　さくら：終わり\n"
    );
    let lua = load_scenes(&transpile(&source));
    // シーンのコルーチンで実行し、チェイントークで止まった後に別のリクエストを挟んで再開する
    let got = lua
        .load(
            r#"
            local SCENE = require "pasta.scene"
            local STORE = require "pasta.store"
            local ACT = require "pasta.act"
            local function dump(act)
                local lines = {}
                for _, t in ipairs(act.token) do
                    local name = t.actor and t.actor.name or "-"
                    table.insert(lines, string.format("%s|%s|%s", t.type, name, tostring(t.text)))
                end
                return table.concat(lines, "\n")
            end
            local act = ACT.new(STORE.actors)
            local co = coroutine.create(SCENE.get_start("メイン_1"))
            assert(coroutine.resume(co, act))
            assert(coroutine.status(co) == "suspended")
            -- 次のリクエスト: 別の act がメインスレッドでシーンを始める
            local other = ACT.new(STORE.actors)
            other:init_scene({})
            assert(coroutine.resume(co))
            assert(coroutine.status(co) == "dead")
            return dump(act) .. "\n--\n" .. dump(other)
            "#,
        )
        .eval::<String>()
        .unwrap();
    assert_eq!(
        got,
        concat!(
            "talk|さくら|後半\n",
            "raw_script|-|【数にできない値：＄x（\"abc\"）】\n",
            "talk|さくら|終わり\n",
            "--\n",
        )
    );
    assert_eq!(
        warns(&lua),
        ["act:arith - operand is not a number: op='+', operand='var.x', value='abc' (string)"]
    );
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
// 重ねない規則（5.1・5.2・5.5）
//
// 表の 6 行のうち、1 行目（単語・関数・グローバル関数 → talk）は #3・#5・#6、2 行目（動的参照
// → 検索）と 3 行目（動的コールの変数 → Call）は #7、4 行目（動的コールの関数 → call_key）は
// #13、6 行目（未登録アクター → 立ち位置の警告）は unregistered_actor_mark_in_sakura_output が
// 確かめる。ここでは 5 行目と、回ごとに報告すること（5.5）を確かめる。
// ========================================================================

/// 5 行目: 被演算子の関数が見つからないとき、関数だけが報告し `ACT.num`・`ACT.str` は黙る
#[test]
fn no_overlap_operand_fn_not_found() {
    run_case(&Case {
        name: "no-overlap: operand fn",
        lua: false,
        body: "　＄a＝＠無い関数（）＋１\n　＄b＝＠無い関数（）＆「x」\n　＄c＝＠＊無い関数（）＊２\n",
        tokens: &[
            "raw_script|-|【関数が見つからない：＠無い関数（）】",
            "raw_script|-|【関数が見つからない：＠無い関数（）】",
            "raw_script|-|【関数が見つからない：＠＊無い関数（）】",
        ],
        warns: &[
            "act:expr_fn - handler not found: key='無い関数', mode='expr', via=act",
            "act:expr_fn - handler not found: key='無い関数', mode='expr', via=act",
            "act:global_fn - function not found: key='無い関数'",
        ],
        vars: &["a=1", "b='x'", "c=0"],
    });
}

/// 5.5: 同じ書き間違いを 2 回通ると 2 回出る（同じシーンを 2 回呼ぶ）
#[test]
fn reported_each_time_passed() {
    run_case(&Case {
        name: "each time",
        lua: false,
        body: "　＞サブ\n　＞サブ\n\n＊サブ\n　さくら：＠挨拶\n",
        tokens: &[
            "raw_script|-|【単語が見つからない：＠挨拶】",
            "raw_script|-|【単語が見つからない：＠挨拶】",
        ],
        warns: &[
            "proxy:word - handler not found: key='挨拶', mode='word', via=proxy(さくら)",
            "proxy:word - handler not found: key='挨拶', mode='word', via=proxy(さくら)",
        ],
        vars: &[],
    });
}

/// 値なしを代入した変数を後で展開すると、代入の行と展開の行で別々に報告する（design の注記）
#[test]
fn nil_assigned_then_expanded_reported_separately() {
    run_case(&Case {
        name: "assigned nil then expanded",
        lua: false,
        body: "　＄x＝＠挨拶\n　さくら：＄x\n",
        tokens: &[
            "raw_script|-|【単語が見つからない：＠挨拶】",
            "raw_script|-|【値の無い変数：＄x】",
        ],
        warns: &[
            "act:word - handler not found: key='挨拶', mode='word', via=act",
            "act:talk - undefined variable: 'var.x'",
        ],
        vars: &["x=nil"],
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

// ========================================================================
// さくらスクリプトの出力の位置（sakura_builder を通した後）
// ========================================================================

/// 未登録アクター ぱすたー の目印
const PASTA_MARK: &str = "【未登録アクター：ぱすたー】";

/// `pasta.shiori.act` で `＊メイン` を実行し、さくらスクリプトの出力とログの警告を返す。
/// 話者は さくら（立ち位置 0）・うにゅう（立ち位置 1）。`actors` は追加のアクター定義。
/// `％` の登録だけでは立ち位置が決まらないので、`STORE.actor_spots` へ直接置く
fn run_sakura(actors: &str, body: &str) -> (String, Vec<String>) {
    let source = format!("{SAKURA}％うにゅう\n　＠通常：\\s[10]\n{actors}\n＊メイン\n{body}");
    let lua = run_main_scene_with(&transpile(&source), "pasta.shiori.act");
    let script = lua
        .load(
            r#"
            local STORE = require "pasta.store"
            STORE.actor_spots["さくら"] = 0
            STORE.actor_spots["うにゅう"] = 1
            if STORE.actors["ぱすたー"] then STORE.actor_spots["ぱすたー"] = 0 end
            return RS_ACT:build()
            "#,
        )
        .eval::<String>()
        .unwrap();
    (script, warns(&lua))
}

/// 目印はそのアクターの `\p[…]` の直後に出て、続く台詞の段落区切り・ウェイトは目印の無い場合
/// （同じ名前を登録して立ち位置 0 に置いた場合）と同じ。継続行では出ず、ログは 1 原因 1 行
/// （立ち位置の警告 `actor_spots fallback` を重ねない）
#[test]
fn unregistered_actor_mark_in_sakura_output() {
    let body = concat!(
        "　さくら：やあ、元気？\n",
        "　ぱすたー：こんにちは、元気です。\n",
        "　ぱすたー：続きです。\n",
        "　うにゅう：ふむ\n",
        "　ぱすたー：また、ね\n",
    );
    let (unregistered, unregistered_warns) = run_sakura("", body);
    let (registered, registered_warns) = run_sakura("％ぱすたー\n　＠口癖：だよ\n", body);

    assert_eq!(registered_warns, Vec::<String>::new(), "baseline warns");
    assert_eq!(unregistered.replace(PASTA_MARK, ""), registered);
    assert_eq!(
        unregistered.matches(PASTA_MARK).count(),
        2,
        "{unregistered}"
    );
    assert_eq!(
        unregistered.matches(&format!(r"\p[0]{PASTA_MARK}")).count(),
        2,
        "{unregistered}"
    );
    assert_eq!(
        unregistered_warns,
        [
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
        ]
    );
}

/// 目印以外の失敗表記は直前の話者のバルーンに続けて出し、出力の先頭ではスコープを切り替えない。
/// 未登録アクターの行の先頭の失敗は目印の後に出る
#[test]
fn failure_notation_position_in_sakura_output() {
    let (script, _) = run_sakura(
        "",
        "　さくら：＠無い単語、です\n　うにゅう：＄未定義、だね\n",
    );
    assert_eq!(
        script,
        concat!(
            r"【単語が見つからない：＠無い単語】\p[0]、\_w[450]です",
            r"【値の無い変数：＄未定義】\p[1]、\_w[450]だね\e",
        )
    );

    let (script, warns) = run_sakura("", "　ぱすたー：＠無い単語、だよ\n");
    assert_eq!(
        script,
        format!(r"\p[0]{PASTA_MARK}【単語が見つからない：＠無い単語】、\_w[450]だよ\e")
    );
    assert_eq!(
        warns,
        [
            "act:actor_proxy - unregistered actor: name='ぱすたー'",
            "proxy:word - handler not found: key='無い単語', mode='word', via=proxy(ぱすたー)",
        ]
    );
}
