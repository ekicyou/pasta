-- kick_local_composite_test.lua
-- Lua-side BDD tests for KICK.try_dispatch の local-composite (`:parent:local`) 分岐
-- pasta-scene-kick-from-cursor Task 3.1 (Requirements: 2.2, 2.4)
--
-- 検証契約（composite-string 方式・debug 限定）:
--   KICK.try_dispatch(act):
--     - scene_name が `:会話1:挨拶_1` 形式 → local 分岐:
--         SCENE.search("挨拶_1", "会話1") の **local 分岐**で func を取得しコルーチン化
--         （global の __start__ へ潰さず local 同一性を保持）。
--     - scene_name に先頭 `:` が無い（`intro` 等）→ global 分岐:
--         従来通り SCENE.co_exec(act, scene)（act:find_scene 流用・挙動不変）。
--     - local-composite が解決不能（SCENE.search が nil）→ warn + drop + nil。
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

--- 最小モック act。
--- global 分岐は act:find_scene を使う。local 分岐は SCENE.search を使い act:build を呼ぶ。
--- @param resolvable boolean global 分岐用: find_scene が関数を返すか
--- @return table mock_act, table calls（find_scene の name 記録）
local function make_act(resolvable)
    local calls = { names = {} }
    local act = {}
    function act.find_scene(_self, name, _global_scene_name, _attrs)
        table.insert(calls.names, name)
        if resolvable then
            return function(_resumed_act) end
        end
        return nil
    end
    -- local 分岐ラッパー（wrap_local_func）が co の中で resumed_act:build() を呼ぶ。
    function act.build(_self)
        return "<built>"
    end
    return act, calls
end

describe("KICK.try_dispatch - local-composite (`:parent:local`) 分岐", function()
    local original_search

    local function save_search()
        original_search = SCENE.search
    end
    local function restore_search()
        if original_search then
            SCENE.search = original_search
        end
    end

    test("`:会話1:挨拶_1` は SCENE.search(local, parent) の local 分岐で解決する", function()
        STORE.reset()
        save_search()

        local captured = {}
        SCENE.search = function(name, global_scene_name, _attrs)
            captured.name = name
            captured.parent = global_scene_name
            return {
                func = function(act)
                    -- local シーン本体が実行されたことを観測可能にする
                    act.local_ran = true
                end,
                global_name = "会話1",
                local_name = "挨拶_1",
            }
        end

        STORE.kick_pending = ":会話1:挨拶_1"
        local act = make_act(true)

        local co = KICK.try_dispatch(act)

        -- local 分岐: search に (local_name=挨拶_1, parent=会話1) が渡る
        expect(captured.name):toBe("挨拶_1")
        expect(captured.parent):toBe("会話1")
        -- コルーチンが返る
        expect(type(co)):toBe("thread")
        -- フラグ消費
        expect(STORE.kick_pending):toBeNil()

        -- resume で local func が走り、build() の結果が返る（local 同一性を保持）
        local ok, value = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(act.local_ran):toBe(true)
        expect(value):toBe("<built>")

        restore_search()
    end)

    test("local-composite が解決不能なら warn + drop + nil（kick_pending 消費）", function()
        STORE.reset()
        save_search()

        local searched = false
        SCENE.search = function(_name, _parent, _attrs)
            searched = true
            return nil
        end

        STORE.kick_pending = ":会話9:存在しない_1"
        local act = make_act(true)

        local co = KICK.try_dispatch(act)

        expect(searched):toBe(true)
        expect(co):toBeNil()
        -- 解決不能でもフラグ消費（再発火しない）
        expect(STORE.kick_pending):toBeNil()

        restore_search()
    end)

    test("先頭 `:` 無しは従来 global 分岐（co_exec→find_scene）で不変", function()
        STORE.reset()
        save_search()

        -- local 分岐が誤って呼ばれていないことを観測するため search をトラップ
        local local_branch_called = false
        SCENE.search = function(_name, _parent, _attrs)
            local_branch_called = true
            return nil
        end

        STORE.kick_pending = "intro"
        local act, calls = make_act(true)

        local co = KICK.try_dispatch(act)

        -- global 分岐: act:find_scene("intro") を経由（SCENE.search の local 分岐は不使用）
        expect(calls.names):toContain("intro")
        expect(local_branch_called):toBe(false)
        expect(type(co)):toBe("thread")
        expect(STORE.kick_pending):toBeNil()

        restore_search()
    end)
end)
