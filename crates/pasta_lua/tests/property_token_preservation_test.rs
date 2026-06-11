//! Task 9: Lua API トークン保全テスト
//!
//! `act:get_property()` がトークンバッファを退避・復元することを検証する。
//! Requirements: 3.5, 4.4

mod common;

use common::create_runtime_with_pasta_path;

/// get_property yield 時に先行トークンが混入しないことを検証する。
///
/// flow: talk("BEFORE") → get_property("p") → yield → resume → talk("AFTER") → build()
/// - yield されたスクリプトには "BEFORE" が含まれないこと
/// - 最終ビルドには "BEFORE" と "AFTER" が両方含まれること
#[test]
fn test_get_property_does_not_contaminate_yielded_script() {
    let runtime = create_runtime_with_pasta_path();

    // Setup: reset modules
    runtime
        .exec(
            r#"
            package.loaded["pasta.shiori.act"] = nil
            package.loaded["pasta.shiori.event.callback"] = nil
            "#,
        )
        .unwrap();

    // Create coroutine and first resume (runs until get_property yields)
    let result = runtime
        .exec(
            r#"
            local SHIORI_ACT = require("pasta.shiori.act")
            local actors = { ["さくら"] = { name = "さくら", spot = 0 } }

            local co = coroutine.create(function()
                local act = SHIORI_ACT.new(actors)
                act.さくら:talk("BEFORE_TOKEN")
                act:get_property("testprop")
                -- if we reach here, get_property resumed successfully
                act.さくら:talk("AFTER_TOKEN")
                return act:build()
            end)

            -- First resume: runs until get_property yields
            local ok, yielded = coroutine.resume(co)
            if not ok then error("first resume failed: " .. tostring(yielded)) end

            -- The yielded script should contain the get tag but NOT "BEFORE_TOKEN"
            local yielded_str = tostring(yielded or "")
            local has_before = yielded_str:find("BEFORE_TOKEN") ~= nil
            local has_get_tag = yielded_str:find("get,property") ~= nil

            return tostring(has_before) .. "|" .. tostring(has_get_tag)
            "#,
        )
        .unwrap();

    let s = result.as_string().unwrap().to_string_lossy();
    // has_before should be false, has_get_tag should be true
    assert_eq!(s, "false|true", "yield script must NOT contain prior talk tokens, but MUST contain get tag");
}

/// resume 後のビルド結果に退避トークンが正しく復元されていることを検証する。
#[test]
fn test_get_property_restores_tokens_after_resume() {
    let runtime = create_runtime_with_pasta_path();

    runtime
        .exec(
            r#"
            package.loaded["pasta.shiori.act"] = nil
            package.loaded["pasta.shiori.event.callback"] = nil
            "#,
        )
        .unwrap();

    let result = runtime
        .exec(
            r#"
            local SHIORI_ACT = require("pasta.shiori.act")
            local actors = { ["さくら"] = { name = "さくら", spot = 0 } }

            local co = coroutine.create(function()
                local act = SHIORI_ACT.new(actors)
                act.さくら:talk("ALPHA")
                act:get_property("prop")
                act.さくら:talk("BETA")
                return act:build()
            end)

            -- First resume → yields at get_property
            local ok1, _ = coroutine.resume(co)
            if not ok1 then error("first resume failed") end

            -- Second resume → provide property value, coroutine finishes
            local ok2, final_output = coroutine.resume(co, { [1] = "dummy" }, nil)
            if not ok2 then error("second resume failed: " .. tostring(final_output)) end

            local output_str = tostring(final_output or "")
            local has_alpha = output_str:find("ALPHA") ~= nil
            local has_beta = output_str:find("BETA") ~= nil

            return tostring(has_alpha) .. "|" .. tostring(has_beta)
            "#,
        )
        .unwrap();

    let s = result.as_string().unwrap().to_string_lossy();
    assert_eq!(s, "true|true", "final build must contain BOTH pre- and post-get_property talk tokens");
}

/// get_property の戻り値が呼び出し側に正しく届くことを検証する。
#[test]
fn test_get_property_returns_value_to_caller() {
    let runtime = create_runtime_with_pasta_path();

    runtime
        .exec(
            r#"
            package.loaded["pasta.shiori.act"] = nil
            package.loaded["pasta.shiori.event.callback"] = nil
            "#,
        )
        .unwrap();

    let result = runtime
        .exec(
            r#"
            local SHIORI_ACT = require("pasta.shiori.act")
            local actors = { ["さくら"] = { name = "さくら", spot = 0 } }

            local returned_value = nil

            local co = coroutine.create(function()
                local act = SHIORI_ACT.new(actors)
                local val = act:get_property("ghost.name")
                returned_value = val
            end)

            -- First resume → yields at get_property
            local ok1, _ = coroutine.resume(co)
            if not ok1 then error("first resume failed") end

            -- Second resume → provide property value
            local ok2 = coroutine.resume(co, { [1] = "テストゴースト" }, nil)
            if not ok2 then error("second resume failed") end

            return tostring(returned_value)
            "#,
        )
        .unwrap();

    let s = result.as_string().unwrap().to_string_lossy();
    assert_eq!(s, "テストゴースト", "get_property must return the value provided on resume");
}
