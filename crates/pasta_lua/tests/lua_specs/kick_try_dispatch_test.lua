-- kick_try_dispatch_test.lua
-- Lua-side BDD tests for KICK.try_dispatch (保留キックシーン解決 → co 返却)
-- pasta-scene-kick Task 4.2 (Requirements: 3.2, 3.3, 3.5)
--
-- 検証契約:
--   KICK.try_dispatch(act):
--     - STORE.kick_pending == nil          -> nil（何もしない）
--     - kick_pending 設定済み + 解決可能    -> co（thread）を返し kick_pending クリア
--     - kick_pending 設定済み + 解決不能    -> nil + 診断ログ + kick_pending クリア
--   ctx 合成は当該 act を流用（SCENE.co_exec(act, scene) 経由・キック専用 ctx を作らない）。
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
local STORE = require("pasta.store")

--- 最小モック act を構築する。
--- SCENE.co_exec は act:find_scene(name, ...) のみを使うため、それだけを満たす。
--- @param resolvable boolean true なら find_scene が関数を返す（解決可能）
--- @return table mock_act, table calls（find_scene に渡された name を記録）
local function make_act(resolvable)
    local calls = { names = {} }
    local act = {}
    function act.find_scene(_self, name, _global_scene_name, _attrs)
        table.insert(calls.names, name)
        if resolvable then
            -- 解決可能: 何もしないシーン関数を返す
            return function(_resumed_act) end
        end
        return nil
    end
    return act, calls
end

describe("KICK.try_dispatch - 保留キックシーン解決", function()
    test("kick_pending=nil のとき nil を返し何もしない", function()
        STORE.reset()
        local act = make_act(true)
        local co = KICK.try_dispatch(act)
        expect(co):toBeNil()
        expect(STORE.kick_pending):toBeNil()
    end)

    test("kick_pending 設定済み + 解決可能 -> co(thread) を返し kick_pending クリア", function()
        STORE.reset()
        STORE.kick_pending = "intro"
        local act, calls = make_act(true)

        local co = KICK.try_dispatch(act)

        expect(type(co)):toBe("thread")
        -- 当該 act を流用してシーン名を解決した
        expect(calls.names):toContain("intro")
        -- フラグ消費（再発火防止）
        expect(STORE.kick_pending):toBeNil()
    end)

    test("kick_pending 設定済み + 解決不能 -> nil を返し kick_pending クリア", function()
        STORE.reset()
        STORE.kick_pending = "missing"
        local act, calls = make_act(false)

        local co = KICK.try_dispatch(act)

        expect(co):toBeNil()
        expect(calls.names):toContain("missing")
        -- 解決不能でもフラグは消費（再発火しない・前会話は保持）
        expect(STORE.kick_pending):toBeNil()
    end)
end)
