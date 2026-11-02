//! R5: String Buffer の在ることの実機検証（本番同一経路）
//!
//! `pasta.shiori.sakura_builder` は LuaJIT の String Buffer Library（`string.buffer`）を
//! 代わりの実装なしで直接使う。本テストはそれが対象ランタイムで使えることを
//! **本番と同一の構築経路**で守る availability gate である。
//!
//! - 本番同一経路: `pasta_lua::RuntimeConfig::default().to_stdlib()` で StdLib を得て、
//!   `unsafe { mlua::Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) }` で構築する。
//!   これは `crates/pasta_lua/src/runtime/mod.rs` の本番 `with_config` と同一パターンであり、
//!   素の `ALL_SAFE` ハードコードを避けることで構成ドリフトを防ぐ。
//! - 検証内容: `require("string.buffer")` が成功し、`new()` のバッファが `put`/`tostring` で機能すること。
//!   満たされなければ失敗させ、エラーメッセージに `_VERSION` と `jit.version` を併記する。

use mlua::prelude::*;

#[test]
fn string_buffer_adopted_on_target_runtime() -> LuaResult<()> {
    let std_lib = pasta_lua::RuntimeConfig::default()
        .to_stdlib()
        .expect("RuntimeConfig::default().to_stdlib() should succeed");
    let lua = unsafe { mlua::Lua::unsafe_new_with(std_lib, mlua::LuaOptions::default()) };

    let result: LuaTable = lua
        .load(
            r#"
            local runtime = _VERSION .. " / " .. tostring(jit and jit.version)
            local sb_ok, sb = pcall(require, "string.buffer")

            local actual = nil
            if sb_ok then
                local ok, res = pcall(function()
                    return sb.new():put("Hello"):put(" "):put("World"):tostring()
                end)
                if ok then actual = res end
            end

            return { string_buffer_ok = sb_ok, actual = actual, runtime = runtime }
        "#,
        )
        .eval()?;

    let string_buffer_ok: bool = result.get("string_buffer_ok")?;
    let actual: Option<String> = result.get("actual")?;
    let runtime: String = result.get("runtime")?;

    assert!(
        string_buffer_ok,
        "R5 finding: require(\"string.buffer\") が失敗しました（ランタイム {runtime}）。\
         sakura_builder は String Buffer Library を直接使うため、対象ランタイムで使える必要があります。"
    );
    assert_eq!(
        actual.as_deref(),
        Some("Hello World"),
        "R5 finding: string.buffer の put/tostring が想定結果になりません（ランタイム {runtime}）。"
    );

    Ok(())
}
