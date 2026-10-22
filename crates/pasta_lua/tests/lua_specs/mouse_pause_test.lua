-- mouse_pause_test.lua
-- 止めている間のつつき・撫での既定ハンドラ（pasta.shiori.event.mouse）のテスト
-- ontalk-suppression Task 2.2 (Requirements: 2.9, 2.10, 2.11, 5.1)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local REG = require("pasta.shiori.event.register")
local CLOCK = require("pasta.shiori.clock")

--- 1〜8 段のうち このシーンだけがある
local SHORTCUT = { ["つっつき反応その他"] = true, ["さわり反応その他"] = true }

--- 既定のハンドラを持つ mouse を読み直して body(M) を回し、必ず元に戻す
--- シーンはスタブ: co_exec はイベント名を、co_exec_exact は "exact:" ＋名前を返す
---@param paused boolean 止めている印
---@param preset table|nil 先に置く自前のハンドラ { OnMouseDoubleClick = fn, OnMouseMove = fn }
local function with_mouse(paused, preset, body)
    local STORE = require("pasta.store")
    local saved_mod = package.loaded["pasta.shiori.event.mouse"]
    local saved_poke, saved_touch = REG.OnMouseDoubleClick, REG.OnMouseMove
    local saved_scene = package.loaded["pasta.scene"]
    local saved_paused = STORE.auto_talk_paused
    package.loaded["pasta.scene"] = {
        co_exec = function(_, event) return event end,
        co_exec_exact = function(name)
            if SHORTCUT[name] then return "exact:" .. name end
            return nil
        end,
    }
    package.loaded["pasta.shiori.event.mouse"] = nil
    REG.OnMouseDoubleClick = preset and preset.OnMouseDoubleClick
    REG.OnMouseMove = preset and preset.OnMouseMove
    STORE.auto_talk_paused = paused
    CLOCK._reset()
    local ok, err = pcall(function()
        local M = require("pasta.shiori.event.mouse")
        M._reset()
        body(M)
    end)
    STORE.auto_talk_paused = saved_paused
    REG.OnMouseDoubleClick, REG.OnMouseMove = saved_poke, saved_touch
    package.loaded["pasta.shiori.event.mouse"] = saved_mod
    package.loaded["pasta.scene"] = saved_scene
    CLOCK._reset()
    if not ok then error(err, 0) end
end

local function act(id, u)
    return { req = {
        id = id, status = "idle",
        reference = { [3] = "0", [4] = "Head" }, date = { unix = u or 1000, ns = 0 },
    } }
end

--- 同じ所を 0〜2.5 秒撫でて、最後の戻り値を返す
local function stroke()
    local last
    for _, dt in ipairs({ 0, 1, 2, 2.5 }) do
        last = REG.OnMouseMove(act("OnMouseMove", 1000 + dt))
    end
    return last
end

describe("つつき - 止めている間", function()
    test("止めている間は 1〜8 段があっても 9 段目のイベント名のシーンを返す", function()
        with_mouse(true, nil, function()
            expect(REG.OnMouseDoubleClick(act("OnMouseDoubleClick"))):toBe("OnMouseDoubleClick")
        end)
    end)

    test("止めていなければ今どおり 1〜8 段を返す", function()
        with_mouse(false, nil, function()
            expect(REG.OnMouseDoubleClick(act("OnMouseDoubleClick"))):toBe("exact:つっつき反応その他")
        end)
    end)
end)

describe("撫で - 止めている間", function()
    test("2 秒撫でても決めず、OnMouseMove のシーンを返し、判定の状態は Off", function()
        with_mouse(true, nil, function(M)
            local seen = {}
            for _, dt in ipairs({ 0, 1, 2, 2.5 }) do
                seen[#seen + 1] = REG.OnMouseMove(act("OnMouseMove", 1000 + dt))
            end
            expect(table.concat(seen, "|")):toBe("OnMouseMove|OnMouseMove|OnMouseMove|OnMouseMove")
            expect(M._get_state().target):toBeNil()
            expect(M._get_state().fired):toBe(false)
        end)
    end)

    test("止めていなければ今どおり 2 秒で 1〜8 段を返す", function()
        with_mouse(false, nil, function(M)
            local seen = {}
            for _, dt in ipairs({ 0, 1, 2 }) do
                seen[#seen + 1] = REG.OnMouseMove(act("OnMouseMove", 1000 + dt))
            end
            expect(table.concat(seen, "|")):toBe("OnMouseMove|OnMouseMove|exact:さわり反応その他")
            expect(M._get_state().fired):toBe(true)
        end)
    end)
end)

describe("自前のハンドラ - 止めている間", function()
    test("ゴーストが先に置いたハンドラは置き換えず、止めていても呼ばれる", function()
        local function own_poke() return "own_poke" end
        local function own_touch() return "own_touch" end
        with_mouse(true, { OnMouseDoubleClick = own_poke, OnMouseMove = own_touch }, function()
            expect(REG.OnMouseDoubleClick):toBe(own_poke)
            expect(REG.OnMouseMove):toBe(own_touch)
            expect(REG.OnMouseDoubleClick(act("OnMouseDoubleClick"))):toBe("own_poke")
            expect(stroke()):toBe("own_touch")
        end)
    end)
end)
