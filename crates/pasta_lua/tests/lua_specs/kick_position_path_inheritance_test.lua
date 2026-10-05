-- kick_position_path_inheritance_test.lua
-- Lua-side BDD integration tests: 位置パス由来のシーン文字列が既存 kick 取次点
-- （同一 dispatch フック）経由で起動され kick 本体セマンティクスを継承することを検証する。
-- pasta-scene-kick-from-cursor Task 6.2 (Requirements: 4.2, 4.3, 8.1)
-- scene-identity-format Task 2.2 (Requirements: 5.1, 5.3, 8.4, 8.6)
--
-- 背景（既存カバレッジとの差分）:
--   - 4.1 の Rust wiring テスト（wiring_play_scene_at_tests.rs）は、位置解決器が
--     KickSink を介して global / local-composite（`:親:ローカル`）を取次ぐ往復を
--     固定する（トランスポート脚）。
--   - virtual_dispatcher_kick_hook_test.lua / _kick_force_test.lua は dispatch フックの
--     co_scene 設置・preempt・kick_force ワンショット突破を固定するが、投入名は任意の
--     global 風名（`kick_scene`）であり、**位置パスが実際に生成する文字列**ではない。
--     とりわけ local-composite（`:会話_1:挨拶_1`）は try_dispatch 単体でしか通っておらず、
--     dispatch フック越しの preempt は未固定。
--
-- 本スイートが閉じるギャップ（R8.1 = 位置ベースキックは pasta-scene-kick の
-- co_scene 設置・preempt-and-abort・ワンショット抑制突破を継承し変更しない）:
--   位置パス由来の **実文字列**（シーン表の登録名 global `会話_1` / local-composite
--   `:会話_1:挨拶_1`）を `KICK.install` 相当で設置し `dispatcher.dispatch` を駆動して、
--     (a) is_blocked（talking）を kick_force ワンショットで突破しキックシーン co を返す、
--     (b) 進行中 co_scene を preempt（前 co 継続を返さずキック co を優先）する、
--     (c) kick_force が突破直後に消費され、2 度目の talking tick は通常ブロックされる、
--   ことを、global と local-composite の双方の位置パス文字列で観測する。
--
-- 注意: dispatch / kick / scene / store は同一インスタンスにそろえて一括新規ロードする。
-- キックはシーン表（`STORE.scenes`）から完全一致で引くため、対象シーンは `SCENE.register` で登録する。
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
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

-- キックのラッパー（wrap_local_func）はシーン関数を包み、末尾で resumed_act:build() を呼ぶ。
-- シーン関数ではビートを coroutine.yield で吐き、build は nil を返すモックにする。
local function create_mock_act(req)
    local act = { req = req }
    function act.build(_self)
        return nil
    end
    return act
end

describe("位置パス由来文字列の kick セマンティクス継承（dispatch フック越し）", function()
    -- 内部タイマ初期化（idle・非ブロックの init dispatch で next_hour_unix を設定）。
    local function setup()
        STORE.reset()
        dispatcher._reset()
        local init_act = create_mock_act({
            id = "OnSecondChange", status = "idle", date = { unix = 1702648800 },
        })
        dispatcher.dispatch(init_act)
    end

    --- setup して body を実行し、成否にかかわらずシーン表・保留フラグ・dispatcher を初期化し直す
    --- @param body fun()
    local function run(body)
        setup()
        local ok, err = pcall(body)
        STORE.reset()
        dispatcher._reset()
        if not ok then error(err, 0) end
    end

    --------------------------------------------------------------------------
    -- global 位置パス文字列（`会話_1`）の継承
    --------------------------------------------------------------------------

    test("global 文字列 `会話_1`: talking でも kick_force 突破でキックシーン co を返す（R8.1 ワンショット突破）", function()
        run(function()
            SCENE.register("会話_1", "__start__", function(_resumed_act)
                coroutine.yield("会話_1_beat_1")
            end)
            -- 位置解決器が KickSink へ載せる global 形式（登録名）を install 相当で設置。
            KICK.install("会話_1")
            expect(STORE.kick_force):toBe(true)

            local act = create_mock_act(
                { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } })

            local result = dispatcher.dispatch(act)

            -- talking（is_blocked）でも force 突破でキックシーン co が返る
            expect(type(result)):toBe("thread")
            -- 返った co がキックシーンであることを resume で確認
            local ok, value = coroutine.resume(result, act)
            expect(ok):toBe(true)
            expect(value):toBe("会話_1_beat_1")
            -- kick_force はワンショット消費（突破直後 false 化）
            expect(STORE.kick_force):toBe(false)
            -- kick_pending も try_dispatch で消費
            expect(STORE.kick_pending):toBeNil()
        end)
    end)

    test("global 文字列 `会話_1`: 進行中 co_scene を preempt（前 co 継続を返さない・R8.1 preempt-and-abort）", function()
        run(function()
            SCENE.register("会話_1", "__start__", function() end)
            -- 進行中シーン（suspended）を co_scene に据える
            local prev_co = coroutine.create(function()
                coroutine.yield("prev_beat")
                return "prev_final"
            end)
            coroutine.resume(prev_co)
            STORE.co_scene = prev_co

            KICK.install("会話_1")
            local act = create_mock_act(
                { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } })

            local result = dispatcher.dispatch(act)

            -- dispatch はキック co を返し、前 co_scene 継続（check_talk）には落ちない。
            -- 実 preempt（set_co_scene 置換）は EVENT.fire 側だが、ここで「キック co 優先で
            -- 前 co 継続を返さない」ことが preempt の前提（pasta-scene-kick R5.2 と同一観測点）。
            expect(type(result)):toBe("thread")
            expect(result ~= prev_co):toBe(true)
        end)
    end)

    test("global 文字列: force 突破は 1 回限り（消費後の talking tick は通常ブロック）", function()
        run(function()
            SCENE.register("会話_1", "__start__", function() coroutine.yield("会話_1_beat_1") end)
            KICK.install("会話_1")

            local act1 = create_mock_act(
                { id = "OnSecondChange", status = "talking", date = { unix = 1702648900 } })
            local first = dispatcher.dispatch(act1)
            expect(type(first)):toBe("thread")
            expect(STORE.kick_force):toBe(false)

            -- 2 度目の talking tick: kick_pending/force 消費済み → 通常ブロックで nil
            local act2 = create_mock_act(
                { id = "OnSecondChange", status = "talking", date = { unix = 1702648950 } })
            local second = dispatcher.dispatch(act2)
            expect(second):toBe(nil)
        end)
    end)

    --------------------------------------------------------------------------
    -- local-composite 位置パス文字列（`:会話_1:挨拶_1`）の継承
    --   ※ dispatch フック越しに local 分岐が co_scene 設置・preempt を継承することを固定。
    --     既存 kick_local_composite_test は try_dispatch 単体までしか通していない。
    --------------------------------------------------------------------------

    test("local-composite `:会話_1:挨拶_1`: dispatch 越しに local 分岐で解決しキック co を返す（R8.1 継承）", function()
        run(function()
            -- 親 `会話_1` の中に `挨拶_1`・`挨拶_10` を登録（`挨拶_10` は再生されない）
            local ran = {}
            SCENE.register("会話_1", "挨拶_1", function(act)
                act.local_ran = true
                table.insert(ran, "挨拶_1")
                coroutine.yield("挨拶_1_beat")
            end)
            SCENE.register("会話_1", "挨拶_10", function()
                table.insert(ran, "挨拶_10")
            end)

            -- 位置解決器が KickSink へ載せる local-composite 形式を install 相当で設置。
            KICK.install(":会話_1:挨拶_1")

            local act = create_mock_act({
                id = "OnSecondChange", status = "talking", date = { unix = 1702648900 },
            })

            local result = dispatcher.dispatch(act)

            -- talking でも force 突破でキック co（thread）が返る
            expect(type(result)):toBe("thread")
            -- resume で local シーン本体が走る（local 同一性を保持・global の __start__ へ潰れない）
            local ok, value = coroutine.resume(result, act)
            expect(ok):toBe(true)
            expect(act.local_ran):toBe(true)
            expect(value):toBe("挨拶_1_beat")
            expect(#ran):toBe(1)
            expect(ran[1]):toBe("挨拶_1")
            -- ワンショット消費
            expect(STORE.kick_force):toBe(false)
            expect(STORE.kick_pending):toBeNil()
        end)
    end)

    test("local-composite `:会話_1:挨拶_1`: 進行中 co_scene を preempt（前 co 継続を返さない・R8.1）", function()
        run(function()
            SCENE.register("会話_1", "挨拶_1", function() end)

            -- 進行中シーン（suspended）を co_scene に据える
            local prev_co = coroutine.create(function()
                coroutine.yield("prev_beat")
                return "prev_final"
            end)
            coroutine.resume(prev_co)
            STORE.co_scene = prev_co

            KICK.install(":会話_1:挨拶_1")
            local act = create_mock_act({
                id = "OnSecondChange", status = "talking", date = { unix = 1702648900 },
            })

            local result = dispatcher.dispatch(act)

            -- local-composite キック co を返し、前 co_scene 継続には落ちない（preempt 前提）
            expect(type(result)):toBe("thread")
            expect(result ~= prev_co):toBe(true)
        end)
    end)

    test("不在 local-composite は素通り（前会話保持・R3.5 継承）", function()
        run(function()
            -- 親はあるが、その中に `存在しない_1` は無い（解決不能）
            SCENE.register("会話_1", "挨拶_1", function() end)

            -- 前 co_scene を据える（解決不能時に閉じないこと）
            local prev_co = coroutine.create(function()
                coroutine.yield("prev")
                return "prev_final"
            end)
            coroutine.resume(prev_co)
            STORE.co_scene = prev_co

            KICK.install(":会話_1:存在しない_1")
            local act = create_mock_act({
                id = "OnSecondChange", status = "talking", date = { unix = 1702648900 },
            })

            local result = dispatcher.dispatch(act)

            -- try_dispatch=nil → フック素通り。talking ブロックで nil（前会話は co_scene 保持）。
            expect(result):toBe(nil)
            expect(STORE.co_scene):toBe(prev_co)
            -- kick_pending は try_dispatch で消費済み
            expect(STORE.kick_pending):toBeNil()
        end)
    end)
end)
