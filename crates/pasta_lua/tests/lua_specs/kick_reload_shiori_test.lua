-- kick_reload_shiori_test.lua
-- Lua-side BDD tests for KICK.try_dispatch の SHIORI リロード sentinel 分岐
-- pasta-scene-kick-from-cursor Task 4.3 (Requirements: 9.2)
--
-- 検証契約（sentinel 方式・debug 限定）:
--   KICK.try_dispatch(act):
--     - scene_name == RELOAD_SENTINEL（完全一致）→ reload 分岐:
--         act:raw_script("\\![reload,shiori]") → act:build() を行うコルーチンを返す。
--         resume すると build() の結果（`\![reload,shiori]` を含む）が返る。
--     - RELOAD_SENTINEL は `:` local-composite 分岐や global のシーン表引き当てより前で完全一致判定。
--     - sentinel 値は Rust 側 RELOAD_SENTINEL とバイト一致（@@pasta/reloadShiori@@）。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 依存閉包を整合した同一インスタンス集合として一括新規ロードする。
local RELOAD = {
    "pasta.shiori.event.kick",
    "pasta.scene",
    "pasta.word",
    "pasta.store",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local KICK = require("pasta.shiori.event.kick")
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

-- Rust 側 crate::debug::dap::RELOAD_SENTINEL とバイト一致させること（同期必須）。
local RELOAD_SENTINEL = "@@pasta/reloadShiori@@"

--- 最小モック act。
--- reload 分岐は act:raw_script(...) を呼び、act:build() で蓄積を文字列化して返す。
--- @return table mock_act
local function make_act()
    local act = { tokens = {} }
    function act.raw_script(self, text)
        table.insert(self.tokens, text)
        return self
    end
    function act.build(self)
        return table.concat(self.tokens, "")
    end
    return act
end

describe("KICK.try_dispatch - SHIORI リロード sentinel 分岐", function()
    test("RELOAD_SENTINEL は reload さくらスクリプトを出すコルーチンを返す", function()
        STORE.reset()

        -- local/global 分岐が誤って呼ばれていないことを観測するため、シーン表の引き当てをトラップ
        local original_get = SCENE.get
        local original_get_start = SCENE.get_start
        local other_branch_called = false
        SCENE.get = function()
            other_branch_called = true
            return nil
        end
        SCENE.get_start = function()
            other_branch_called = true
            return nil
        end

        local ok_all, err = pcall(function()
            STORE.kick_pending = RELOAD_SENTINEL
            local act = make_act()

            local co = KICK.try_dispatch(act)

            -- コルーチンが返り、フラグ消費
            expect(type(co)):toBe("thread")
            expect(STORE.kick_pending):toBeNil()
            -- local-composite/global 分岐は通っていない
            expect(other_branch_called):toBe(false)

            -- resume すると build() の結果が返り、`\![reload,shiori]` を含む
            local ok, value = coroutine.resume(co, act)
            expect(ok):toBe(true)
            expect(type(value)):toBe("string")
            -- `\![reload,shiori]` を含む（plain 部分一致・パターン無効化のため find の 4th 引数 true）
            local found = type(value) == "string"
                and string.find(value, "\\![reload,shiori]", 1, true) ~= nil
            expect(found):toBe(true)
        end)

        SCENE.get = original_get
        SCENE.get_start = original_get_start
        if not ok_all then error(err, 0) end
    end)

    test("RELOAD_SENTINEL 以外は reload 分岐を通らない（シーン表の global へ素通り）", function()
        STORE.reset()
        local ok, err = pcall(function()
            -- シーン表に登録した global シーンが再生されることで、reload 分岐の誤発火が無いことを観測
            local intro_ran = false
            SCENE.register("intro", "__start__", function() intro_ran = true end)

            STORE.kick_pending = "intro"
            local act = make_act()

            local co = KICK.try_dispatch(act)

            -- 通常 global 分岐: シーン表の intro が再生され、reload は出ない
            expect(type(co)):toBe("thread")
            local resumed, value = coroutine.resume(co, act)
            expect(resumed):toBe(true)
            expect(intro_ran):toBe(true)
            expect(value):toBe("")
        end)
        -- 成否にかかわらずシーン表・保留フラグを残さない
        STORE.reset()
        if not ok then error(err, 0) end
    end)
end)
