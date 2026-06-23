-- kick_install_test.lua
-- Lua-side BDD tests for KICK.install / SHIORI.kick kick-pending flag installation
-- pasta-scene-kick Task 4.1 (Requirements: 3.1, 5.4)
--
-- 検証契約:
--   SHIORI.kick("intro") 呼び出し後に
--     - STORE.kick_pending == "intro"
--     - STORE.kick_force == true
--     - STORE.co_scene が未変更（resume / レンダリングしていない）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 依存閉包を整合した同一インスタンス集合として一括新規ロードする。
-- entry を最初に require し、後続 require が同一インスタンスを返すことを保証する。
local RELOAD = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.shiori.event.kick",
    "pasta.shiori.res",
    "pasta.shiori.act",
    "pasta.shiori.sakura_builder",
    "pasta.act",
    "pasta.actor",
    "pasta.scene",
    "pasta.word",
    "pasta.global",
    "pasta.store",
    "pasta.config",
    "pasta.save",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local ENTRY = require("pasta.shiori.entry")
local KICK = require("pasta.shiori.event.kick")
local STORE = require("pasta.store")

describe("KICK.install - 保留フラグ設置（非ブロッキング）", function()
    test("KICK.install(scene) で kick_pending / kick_force が設置される", function()
        STORE.reset()
        KICK.install("intro")
        expect(STORE.kick_pending):toBe("intro")
        expect(STORE.kick_force):toBe(true)
    end)

    test("KICK.install は co_scene を変更しない（resume しない）", function()
        STORE.reset()
        local marker = coroutine.create(function() coroutine.yield() end)
        STORE.co_scene = marker
        KICK.install("intro")
        -- フラグ設置のみで進行中シーン状態は不変
        expect(STORE.co_scene):toBe(marker)
    end)

    test("KICK.install は nil を返す（フラグ設置のみ・値を返さない）", function()
        STORE.reset()
        local results = table.pack(KICK.install("intro"))
        expect(results.n):toBe(0)
    end)

    test("連続 KICK.install は kick_pending を上書きする（最後が有効）", function()
        STORE.reset()
        KICK.install("first")
        KICK.install("second")
        expect(STORE.kick_pending):toBe("second")
        expect(STORE.kick_force):toBe(true)
    end)
end)

describe("SHIORI.kick - 公開入口（KICK.install へ委譲）", function()
    test("SHIORI.kick は関数として公開されている", function()
        expect(type(ENTRY.kick)):toBe("function")
        expect(_G.SHIORI.kick):toBe(ENTRY.kick)
    end)

    test("SHIORI.kick(scene) 後に保留フラグが立ち co_scene は未変更", function()
        STORE.reset()
        local marker = coroutine.create(function() coroutine.yield() end)
        STORE.co_scene = marker

        ENTRY.kick("intro")

        expect(STORE.kick_pending):toBe("intro")
        expect(STORE.kick_force):toBe(true)
        expect(STORE.co_scene):toBe(marker)
    end)
end)
