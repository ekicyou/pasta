//! R5: String Buffer 採用の実機検証（本番同一経路）
//!
//! 本テストは「LuaJIT String Buffer Library（`string.buffer`）が対象ランタイムで
//! 実際に採用されること」を**本番と同一の構築経路**で検証する availability gate である。
//!
//! - 本番同一経路: `pasta_lua::RuntimeConfig::default().to_stdlib()` で StdLib を得て、
//!   `unsafe { mlua::Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) }` で構築する。
//!   これは `crates/pasta_lua/src/runtime/mod.rs` の本番 `with_config` と同一パターンであり、
//!   素の `ALL_SAFE` ハードコードを避けることで構成ドリフトを防ぐ。
//! - 検証内容（5.1）: `require("string.buffer")` が成功し、`pasta.buf.new()` が
//!   String Buffer 実体（`backend == "luajit"`）を採用し、`put`/`tostring` で機能すること。
//! - 不在時 finding（5.2, 5.3）: 上記が満たされない場合はテストを**失敗**させ、
//!   エラーメッセージに `pasta.lua_version.get()` の数値（実行ランタイム識別）を併記する。
//!   フォールバックを黙って合格にしない検証ゲートそのものである。

use mlua::prelude::*;
use std::path::PathBuf;

#[test]
fn string_buffer_adopted_on_target_runtime() -> LuaResult<()> {
    // 本番同一経路でランタイムを構築する（runtime/mod.rs:108 と同一パターン）。
    // 素の StdLib をハードコードせず、本番 RuntimeConfig::default() の to_stdlib() を経由する
    // ことで、本番構成からのドリフトを防止する。
    let std_lib = pasta_lua::RuntimeConfig::default()
        .to_stdlib()
        .expect("RuntimeConfig::default().to_stdlib() should succeed");
    let lua = unsafe { mlua::Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) };

    // pasta_scripts を package.path に追加し、`pasta.*` を require 可能にする
    // （lua_unittest_runner.rs のパターンを流用）。
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let scripts_path = workspace_root.join("crates/pasta_lua/pasta_scripts");

    let package_path = format!(
        "{}/?.lua;{}/?/init.lua",
        scripts_path.display(),
        scripts_path.display()
    );

    lua.load(format!(
        r#"package.path = "{};;" .. package.path"#,
        package_path.replace('\\', "\\\\")
    ))
    .exec()?;

    // 検証結果を Lua 側でまとめて 1 テーブルに集約して返す。
    // - string_buffer_ok: pcall(require, "string.buffer") の成否
    // - backend:          pasta.buf.backend（"luajit" なら native 採用）
    // - put_tostring_ok:  pasta.buf.new() の put/tostring が想定どおり機能するか
    // - lua_version:      pasta.lua_version.get()（finding に併記する実行ランタイム識別子）
    let result: LuaTable = lua
        .load(
            r#"
            local lua_version = require("pasta.lua_version")
            local version_num = lua_version.get()

            local sb_ok = pcall(require, "string.buffer")

            local buf = require("pasta.buf")
            local backend = buf.backend

            -- put/tostring の機能検証（追記順保持・非破壊連結）
            local put_tostring_ok = false
            local actual = nil
            do
                local ok, res = pcall(function()
                    local b = buf.new()
                    b:put("Hello"):put(" "):put("World")
                    return b:tostring()
                end)
                if ok then
                    actual = res
                    put_tostring_ok = (res == "Hello World")
                end
            end

            return {
                string_buffer_ok = sb_ok,
                backend = backend,
                put_tostring_ok = put_tostring_ok,
                actual = actual,
                lua_version = version_num,
            }
        "#,
        )
        .eval()?;

    let string_buffer_ok: bool = result.get("string_buffer_ok")?;
    let backend: String = result.get("backend")?;
    let put_tostring_ok: bool = result.get("put_tostring_ok")?;
    let actual: Option<String> = result.get("actual")?;
    let lua_version: i64 = result.get("lua_version")?;

    // 5.1 / 5.2 / 5.3:
    // require("string.buffer") 成功・backend=="luajit"・put/tostring 機能の全てを要求する。
    // いずれかが満たされなければ finding として失敗させ、lua_version を併記する
    // （フォールバックを黙って合格にしない）。
    assert!(
        string_buffer_ok,
        "R5 finding: require(\"string.buffer\") が失敗しました。\
         対象ランタイム（mlua luajit52/vendored）で String Buffer Library が利用不可です。\
         実行ランタイム識別 pasta.lua_version.get() = {lua_version}（>=200 なら LuaJIT）。\
         最小実装フォールバックを黙って合格としないため、本テストは失敗で finding 化します。"
    );

    assert_eq!(
        backend, "luajit",
        "R5 finding: pasta.buf.backend が \"luajit\" ではなく \"{backend}\" でした。\
         String Buffer 実体が採用されていません（フォールバック経路）。\
         実行ランタイム識別 pasta.lua_version.get() = {lua_version}（>=200 なら LuaJIT）。"
    );

    assert!(
        put_tostring_ok,
        "R5 finding: pasta.buf.new() の put/tostring が想定結果になりません \
         (expected \"Hello World\", actual {actual:?})。backend={backend}。\
         実行ランタイム識別 pasta.lua_version.get() = {lua_version}（>=200 なら LuaJIT）。"
    );

    // 採用検証が通った場合、対象ランタイムは LuaJIT（>=200）であることを併せて確認する。
    assert!(
        lua_version >= 200,
        "R5 finding: backend=\"luajit\" を採用しつつ lua_version.get() = {lua_version} (<200) は不整合です。"
    );

    Ok(())
}
