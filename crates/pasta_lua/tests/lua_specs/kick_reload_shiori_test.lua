-- kick_reload_shiori_test.lua
-- Lua-side BDD tests for KICK.try_dispatch の SHIORI リロード sentinel 分岐
-- pasta-scene-kick-from-cursor Task 4.3 (Requirements: 9.2)
--
-- 検証契約（sentinel 方式・debug 限定）:
--   KICK.try_dispatch(act):
--     - scene_name == RELOAD_SENTINEL（完全一致）→ reload 分岐:
--         act:raw_script("\\![reload,shiori]") → act:build() を行うコルーチンを返す。
--         resume すると build() の結果（`\![reload,shiori]` を含む）が返る。
--     - RELOAD_SENTINEL は `:` local-composite 分岐や global co_exec より前で完全一致判定。
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

        -- local/global 分岐が誤って呼ばれていないことを観測するため search をトラップ
        local original_search = SCENE.search
        local other_branch_called = false
        SCENE.search = function(_name, _parent, _attrs)
            other_branch_called = true
            return nil
        end

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

        SCENE.search = original_search
    end)

    test("RELOAD_SENTINEL 以外は reload 分岐を通らない（global へ素通り）", function()
        STORE.reset()

        -- global co_exec をトラップして reload 分岐が誤発火しないことを観測
        local original_co_exec = SCENE.co_exec
        local co_exec_called_with
        SCENE.co_exec = function(_act, scene)
            co_exec_called_with = scene
            return coroutine.create(function() end)
        end

        STORE.kick_pending = "intro"
        local act = make_act()

        local co = KICK.try_dispatch(act)

        -- 通常 global 分岐: co_exec(act, "intro") が呼ばれる（reload 分岐ではない）
        expect(co_exec_called_with):toBe("intro")
        expect(type(co)):toBe("thread")

        SCENE.co_exec = original_co_exec
    end)
end)
