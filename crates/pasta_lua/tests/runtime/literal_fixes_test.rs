//! dsl-literal-fixes のランタイムテスト。
//!
//! 空の候補（`＠w：「」`）が実行時に空文字列として返ることを確かめる。
//! 未定義単語の警告は `nil` を返す経路だけなので、空文字列が返れば警告の経路を通っていない。
//!
//! # Requirements Coverage
//! - Requirement 3.4: 空の候補は `nil` ではなく空文字列になる（act 経由・アクター単語の経路）
//! - Requirement 3.5: `＠w：「」、あ` を 2 回引くと空文字列と `あ` が 1 回ずつ出る
//! - Requirement 5.1: 改行を含む値の Lua リテラルを評価すると元の値に戻る
//! - Requirement 5.2, 8.2: 改行入りの値・別々の行の `""` を含む生成 Lua がロードできる

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use mlua::Lua;
use pasta_dsl::parser::{FileItem, parse_str};
use pasta_lua::{LuaTranspiler, StringLiteralizer};

/// 改行を含む値（LF・CR・CRLF、`\` と改行の同時、先頭が改行、`"`・`]` と改行の同時）。
const NEWLINE_VALUES: &[&str] = &["a\nb", "a\rb", "a\r\nb", "a\\\nb\\", "\nlead", "q\"\n]]=]"];

/// pasta ソースをトランスパイルしてロードし、finalize_scene まで通したランタイムを返す。
fn load(source: &str) -> Lua {
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(transpile(source)).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();
    lua
}

/// 3.4: グローバル単語 `＠w：「」` を act 経由で引くと空文字列になる。
#[test]
fn test_empty_candidate_via_act_returns_empty_string() {
    let lua = load("＠w：「」\n");

    let value: Option<String> = lua
        .load(
            r#"
        local ACT = require "pasta.act"
        return ACT.new():word("w")
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(value, Some(String::new()));
}

/// 3.4: アクター単語 `＠w：「」` をアクターのプロキシ経由で引くと空文字列になる。
#[test]
fn test_empty_candidate_via_actor_word_returns_empty_string() {
    let lua = load("％さくら\n  ＠w：「」\n");

    let value: Option<String> = lua
        .load(
            r#"
        local ACTOR = require "pasta.actor"
        local ACT = require "pasta.act"
        local proxy = ACTOR.create_proxy(ACTOR.get_or_create("さくら"), ACT.new())
        return proxy:word("w")
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(value, Some(String::new()));
}

/// 3.5: `＠w：「」、あ` を 2 回引くと、空文字列と `あ` が 1 回ずつ出る。
#[test]
fn test_empty_and_nonempty_candidates_each_drawn_once() {
    let lua = load("＠w：「」、あ\n");

    let (first, second): (Option<String>, Option<String>) = lua
        .load(
            r#"
        local ACT = require "pasta.act"
        local act = ACT.new()
        local first = act:word("w")
        return first, act:word("w")
    "#,
        )
        .eval()
        .unwrap();
    let mut values = vec![first, second];
    values.sort();

    assert_eq!(values, vec![Some(String::new()), Some("あ".to_string())]);
}

/// 5.1: 規則 0 の出力を Lua で評価すると、元の値とバイト単位で一致する。
#[test]
fn test_newline_literal_round_trips_in_lua() {
    let lua = Lua::new();
    for value in NEWLINE_VALUES {
        let literal = StringLiteralizer::literalize(value).unwrap();
        let back: mlua::String = lua.load(format!("return {literal}")).eval().unwrap();
        assert_eq!(
            back.as_bytes().as_ref(),
            value.as_bytes(),
            "literal: {literal}"
        );
    }
}

/// 8.2: 別々の行に `""` を複数書いた `.pasta` がロードと finalize_scene まで通る。
#[test]
fn test_separate_line_blank_strings_load() {
    load("＠a：\"\"\n＠b：\"\"\n＊s\n　＄x＝\"\"\n　＄y＝\"\"\n　さくら：「ok」\n");
}

/// 5.2, 8.2: AST の単語の候補を改行入りの値に差し替えても、ロードと finalize_scene が通り、
/// グローバル単語は元の値のまま読み戻せる。
#[test]
fn test_newline_word_values_load_and_read_back() {
    let globals: String = (0..NEWLINE_VALUES.len())
        .map(|i| format!("＠w{i}：x\n"))
        .collect();
    let source = format!("{globals}％さくら\n　＠a：x\n＊s\n　＠l：x\n　さくら：「ok」\n");
    let mut file = parse_str(&source, "test.pasta").unwrap();

    // グローバル単語は 1 候補ずつ（読み戻しのため）、シーン・アクター単語は全候補に差し替える。
    let mut global_index = 0;
    let mut replaced_scoped = 0;
    for item in &mut file.items {
        let words = match item {
            FileItem::GlobalWord(kw) => {
                kw.words = vec![NEWLINE_VALUES[global_index].to_string()];
                global_index += 1;
                continue;
            }
            FileItem::GlobalSceneScope(scene) => &mut scene.words,
            FileItem::ActorScope(actor) => &mut actor.words,
            _ => continue,
        };
        for kw in words {
            kw.words = NEWLINE_VALUES.iter().map(|v| v.to_string()).collect();
            replaced_scoped += 1;
        }
    }
    assert_eq!(global_index, NEWLINE_VALUES.len());
    assert_eq!(replaced_scoped, 2);

    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile(&file, &mut output)
        .unwrap();
    let lua = create_runtime_with_finalize().unwrap();
    lua.load(output).exec().unwrap();
    lua.load("require('pasta').finalize_scene()")
        .exec()
        .unwrap();

    for (i, value) in NEWLINE_VALUES.iter().enumerate() {
        let back: mlua::String = lua
            .load(format!("return require('pasta.act').new():word('w{i}')"))
            .eval()
            .unwrap();
        assert_eq!(back.as_bytes().as_ref(), value.as_bytes(), "word w{i}");
    }
}
