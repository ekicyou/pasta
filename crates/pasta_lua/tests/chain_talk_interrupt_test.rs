//! chain-talk-interrupt の結合テスト（task 3.4）。
//!
//! ゴーストを一時フォルダに作って `PastaLoader::load_with_config` で読み込み、
//! DSL の変換・名前の完全一致の検索・組み立てを本物のまま、`SHIORI.request` で確かめる。
//!
//! - 台詞を出さないシーンの後は、次の自然なおしゃべりで後半が出る（要件 1.1・7.1）
//! - 台詞を出すシーンの後は中断の台詞だけが 1 本で返り、後半も出ない（要件 3.1・3.6・3.11）
//! - `＊チェイントーク中断ごっこ` だけを書いたゴーストでは中断の台詞が付かない（要件 3.4）
//!
//! 次の自然なおしゃべりは、`scene_play_event_test.rs` と同じく、OnTalk の時機に
//! `STORE.co_scene` を返すハンドラ（`OnTestContinue`）で流す。
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// `＊チェイントーク中断` を書いたゴースト
const FIXTURE: &str = "\
＃ chain-talk-interrupt 3.4 結合フィクスチャ
％さくら
　＠笑顔：\\s[5]

＊OnTalk
　さくら：前半の声
　＞チェイントーク
　さくら：後半の声

＊チェイントーク中断
　さくら：中断の声

＊OnTestQuiet
　さくら：＠笑顔

＊OnTestSpeak
　さくら：反応の声
";

/// `＊チェイントーク中断ごっこ` だけを書いたゴースト（前方一致では使わない）
const FIXTURE_LOOKALIKE: &str = "\
＃ chain-talk-interrupt 3.4 結合フィクスチャ（ごっこ）
＊OnTalk
　さくら：前半の声
　＞チェイントーク
　さくら：後半の声

＊チェイントーク中断ごっこ
　さくら：中断の声

＊OnTestSpeak
　さくら：反応の声
";

/// 指定の辞書で一時ゴーストを作って読み込み、続きを流すハンドラを置く。
fn load_ghost(base: &Path, dic: &str) -> PastaLuaRuntime {
    let pasta_file = base.join("dic/test/chain_talk_interrupt.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, dic).unwrap();
    std::fs::write(base.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }

    let runtime =
        PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load");
    // 仮想ディスパッチャの OnTalk の時機と同じく、待っている続きを返す
    runtime
        .exec(
            r#"
            local REG = require("pasta.shiori.event.register")
            REG.OnTestContinue = function() return require("pasta.store").co_scene end
            "#,
        )
        .unwrap();
    runtime
}

/// `SHIORI.request` に GET のリクエスト（SecurityLevel: local）を渡し、応答の文字列を返す。
fn request(runtime: &PastaLuaRuntime, id: &str) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", "local").unwrap();
    req.set("dic", lua.create_table().unwrap()).unwrap();
    req.set("reference", lua.create_table().unwrap()).unwrap();

    let shiori: Table = lua
        .load(r#"return require("pasta.shiori.entry")"#)
        .eval()
        .unwrap();
    shiori
        .get::<Function>("request")
        .unwrap()
        .call::<String>(req)
        .expect("SHIORI.request must return a response string")
}

/// 200 OK であることを確かめ、Value の行の値を返す。
fn value_of_200(response: &str) -> String {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 200 OK"),
        "200 expected: {response:?}"
    );
    response
        .lines()
        .find_map(|l| l.strip_prefix("Value: "))
        .unwrap_or_else(|| panic!("Value line expected: {response:?}"))
        .to_string()
}

/// 204 No Content であることを確かめる。
fn assert_204(response: &str) {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 204 No Content"),
        "204 expected: {response:?}"
    );
}

/// OnTalk で前半だけを出し、後半を待たせる。
fn start_chain(runtime: &PastaLuaRuntime) {
    let value = value_of_200(&request(runtime, "OnTalk"));
    assert!(value.contains("前半の声"), "first half delivered: {value}");
    assert!(
        !value.contains("後半の声"),
        "second half must wait: {value}"
    );
}

/// 1.1・7.1: 台詞を出さないシーンの後も続きが残り、次の自然なおしゃべりで後半が出る。
#[test]
fn quiet_scene_keeps_continuation() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), FIXTURE);

    start_chain(&runtime);

    let value = value_of_200(&request(&runtime, "OnTestQuiet"));
    assert!(value.contains(r"\s[5]"), "expression delivered: {value}");
    assert!(
        !value.contains("中断の声"),
        "quiet scene must not run the interrupt scene: {value}"
    );

    let value = value_of_200(&request(&runtime, "OnTestContinue"));
    assert!(value.contains("後半の声"), "second half delivered: {value}");
    assert!(
        !value.contains("前半の声"),
        "first half must not repeat: {value}"
    );
}

/// 3.1・3.6・3.11: 台詞を出すシーンの反応の代わりに中断の台詞だけが 1 本で返り、後半は出ない。
#[test]
fn speaking_scene_returns_interrupt_line_and_drops_continuation() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), FIXTURE);

    start_chain(&runtime);

    let response = request(&runtime, "OnTestSpeak");
    let value = value_of_200(&response);
    assert!(
        value.contains("中断の声"),
        "interrupt line delivered: {value}"
    );
    assert!(
        !value.contains("反応の声"),
        "reaction must not be delivered: {value}"
    );
    assert_eq!(
        response
            .lines()
            .filter(|l| l.starts_with("Value: "))
            .count(),
        1,
        "one sakura script: {response:?}"
    );
    assert!(value.ends_with(r"\e"), "one closed script: {value}");
    assert_eq!(value.matches(r"\e").count(), 1, "single \\e: {value}");

    // 続きは捨てられ、次の自然なおしゃべりで後半は出ない
    assert_204(&request(&runtime, "OnTestContinue"));
}

/// 3.4: `＊チェイントーク中断ごっこ` だけのゴーストでは中断の台詞が付かず、反応がそのまま返る。
#[test]
fn lookalike_name_is_not_used_as_interrupt_scene() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path(), FIXTURE_LOOKALIKE);

    start_chain(&runtime);

    let value = value_of_200(&request(&runtime, "OnTestSpeak"));
    assert!(value.contains("反応の声"), "reaction delivered: {value}");
    assert!(
        !value.contains("中断の声"),
        "lookalike scene must not run: {value}"
    );

    // 中断の台詞が無くても、台詞を出した割り込みは今どおり黙って続きを捨てる
    assert_204(&request(&runtime, "OnTestContinue"));
}
