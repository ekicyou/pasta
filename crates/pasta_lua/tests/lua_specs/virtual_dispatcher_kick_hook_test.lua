-- virtual_dispatcher_kick_hook_test.lua
-- Lua-side BDD integration tests for KickDispatchHook / KickPreempt
-- pasta-scene-kick Task 4.4
-- Requirements: 3.4, 4.1, 4.2, 4.3, 4.5, 5.1, 5.2, 5.3, 6.2, 6.3
--
-- 検証契約（virtual_dispatcher.lua dispatch 前段のキック起動フック）:
--   - kick_pending 設定 + status=talking
--       -> dispatch がキックシーンの co（thread）を返す（割り込み起動・R5.1）。
--       前 STORE.co_scene は EVENT.fire の set_co_scene 置換で閉じる（preempt・R5.2）。
--   - kick_pending=nil -> フックが完全素通り、既存 dispatch と同一結果（バイト不変・R6.3）。
--   - マルチ yield キックシーン -> 初回ビートのみ今 tick・残りは次 tick の
--       check_talk 継続（STORE.co_scene）が配信（高々1ビート/GET・R3.4/R4.1）。
--
-- 注意: dispatch / kick / store は同一インスタンスにそろえて一括新規ロードする。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local RELOAD = {
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.shiori.event.kick",
    "pasta.scene",
    "pasta.word",
    "pasta.store",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local KICK = require("pasta.shiori.event.kick")
local STORE = require("pasta.store")

local function create_mock_act(req)
    return { req = req }
end

-- KICK.try_dispatch は SCENE.co_exec(act, name) → act:find_scene(name) を使う。
-- SCENE.co_exec は scene_fn を wrapped_fn でラップし、末尾で resumed_act:build() を呼ぶ。
-- そのため scene_fn 内ではビートを coroutine.yield で吐き、build は nil を返すモックにする。
--- @param scene_fn function|nil 解決シーン関数（nil なら解決不能）
local function make_act_with_scene(req, scene_fn)
    local act = create_mock_act(req)
    function act.find_scene(_self, _name, _global, _attrs)
        return scene_fn
    end
    -- co_exec の wrapped_fn 末尾で呼ばれる build。nil を返し追加ビートを生まない。
    function act.build(_self)
        return nil
    end
    return act
end

describe("KickDispatchHook - dispatch 前段キック起動と preempt", function()
    -- 内部タイマ初期化（idle・非ブロックの init dispatch で next_hour_unix を設定）
    local function setup()
        STORE.reset()
        dispatcher._reset()
        local init_act = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702648800 },
        })
        dispatcher.dispatch(init_act)
    end

    test("kick_pending + status=talking で dispatch がキックシーン co を返す（R5.1）", function()
        setup()
        -- KICK.install 相当: 保留シーン名 + force を設置
        KICK.install("kick_scene")

        local act = make_act_with_scene(
            { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } },
            function(_resumed_act)
                -- ビートは yield で吐く（wrapped_fn が末尾で build を呼ぶため）
                coroutine.yield("kick_beat_1")
            end)

        local result = dispatcher.dispatch(act)

        -- talking でもキックシーンの co（thread）が割り込み起動として返る
        expect(type(result)):toBe("thread")
        -- resume すると初回ビートが出る（キックシーンの出力であることを確認）
        local ok, value = coroutine.resume(result, act)
        expect(ok):toBe(true)
        expect(value):toBe("kick_beat_1")
    end)

    test("preempt: 前 co_scene がキック起動後に閉じる（R5.2 / 自動復帰なし R5.3）", function()
        setup()
        -- 前進行中シーン（suspended）を co_scene に据える
        local prev_co = coroutine.create(function()
            coroutine.yield("prev_beat")
            return "prev_final"
        end)
        coroutine.resume(prev_co)
        STORE.co_scene = prev_co

        KICK.install("kick_scene")
        local act = make_act_with_scene(
            { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } },
            function() return "kick_beat_1" end)

        -- dispatch はキック co を返す。preempt は EVENT.fire の set_co_scene で起こるため、
        -- ここでは「dispatch がキック co を返し、前 co_scene 継続を返さない」ことを確認する。
        local result = dispatcher.dispatch(act)

        expect(type(result)):toBe("thread")
        -- 前 co_scene 継続ではない（キック側 co が優先・preempt 対象が後で閉じられる）
        expect(result ~= prev_co):toBe(true)
    end)

    test("kick_pending=nil でフックが完全素通り（既存 dispatch と同一・R6.3）", function()
        setup()
        -- キック未設置。talking は通常どおりブロックされ nil。
        expect(STORE.kick_pending):toBeNil()
        local act = create_mock_act({
            id = "OnSecondChange", status = "talking", date = { unix = 1702648900 },
        })

        local result = dispatcher.dispatch(act)

        -- 素通り＝既存 dispatch のまま（talking ブロックで nil）
        expect(result):toBe(nil)
    end)

    test("kick_pending=nil + idle で既存 dispatch と同一結果（バイト不変・R6.2/R6.3）", function()
        setup()
        -- 既存挙動: 初回 idle dispatch 後の同一 tick は talk タイマ未到達で nil
        expect(STORE.kick_pending):toBeNil()
        local act = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702648800 },
        })

        local result = dispatcher.dispatch(act)

        expect(result):toBe(nil)
    end)

    test("マルチ yield キックシーンは初回ビートのみ今 tick・残りは次 tick 継続（R3.4/R4.1）", function()
        setup()
        KICK.install("multi_scene")
        local act = make_act_with_scene(
            { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } },
            function(_resumed_act)
                coroutine.yield("beat_1")
                coroutine.yield("beat_2")
                return "beat_3"
            end)

        local kick_co = dispatcher.dispatch(act)
        expect(type(kick_co)):toBe("thread")

        -- 初回ビート（今 tick 相当）
        local ok1, v1 = coroutine.resume(kick_co, act)
        expect(ok1):toBe(true)
        expect(v1):toBe("beat_1")

        -- 残りビートはまだ co に保持され suspended（次 tick の継続が配信する）
        expect(coroutine.status(kick_co)):toBe("suspended")

        -- 次 tick 相当の継続 resume で 2 ビート目が出る
        local ok2, v2 = coroutine.resume(kick_co)
        expect(ok2):toBe(true)
        expect(v2):toBe("beat_2")
    end)

    test("解決不能キックシーンは素通り（前会話保持・R3.5）", function()
        setup()
        -- 前 co_scene を据える（解決不能時に閉じないこと）
        local prev_co = coroutine.create(function()
            coroutine.yield("prev")
            return "prev_final"
        end)
        coroutine.resume(prev_co)
        STORE.co_scene = prev_co

        KICK.install("missing_scene")
        -- find_scene が nil を返す（解決不能）
        local act = make_act_with_scene(
            { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } },
            nil)

        local result = dispatcher.dispatch(act)

        -- try_dispatch=nil → フック素通り。talking ブロックで nil（前会話は co_scene に保持）。
        expect(result):toBe(nil)
        expect(STORE.co_scene):toBe(prev_co)
        -- kick_pending は try_dispatch で消費済み
        expect(STORE.kick_pending):toBeNil()
    end)
end)
