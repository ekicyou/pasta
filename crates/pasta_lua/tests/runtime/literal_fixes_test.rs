//! dsl-literal-fixes のランタイムテスト。
//!
//! 空の候補（`＠w：「」`）が実行時に空文字列として返ることを確かめる。
//! 未定義単語の警告は `nil` を返す経路だけなので、空文字列が返れば警告の経路を通っていない。
//!
//! # Requirements Coverage
//! - Requirement 3.4: 空の候補は `nil` ではなく空文字列になる（act 経由・アクター単語の経路）
//! - Requirement 3.5: `＠w：「」、あ` を 2 回引くと空文字列と `あ` が 1 回ずつ出る

use crate::common;

use common::e2e_helpers::{create_runtime_with_finalize, transpile};
use mlua::Lua;

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
