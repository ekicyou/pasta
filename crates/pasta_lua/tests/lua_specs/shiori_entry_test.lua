-- pasta.shiori.entry（SHIORI load/request/unload・GLOBAL.close_ghost）と
-- EVENT.fire のコールバックルーティング統合・OnSecondChange sweep 分岐のテスト
-- review-improvement-loop cell 3.48 (G1): SHIORI プロトコル境界の公開挙動を固定する
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect
local mocks = require("lua_test.mocks")

-- 先行スイートが個別リロードで package.loaded を分裂させているため、
-- entry の依存閉包を整合した同一インスタンス集合として一括新規ロードする。
-- entry を最初に require し、後続 require が同一インスタンスを返すことを保証する。
mocks.reset()
mocks.install()

local RELOAD = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.virtual_dispatcher",
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
local EVENT = require("pasta.shiori.event")
local REG = require("pasta.shiori.event.register")
local CALLBACK = require("pasta.shiori.event.callback")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local GLOBAL = require("pasta.global")
local STORE = require("pasta.store")
local SHIORI_ACT = require("pasta.shiori.act")

--- fire 系テストの共通状態リセット
local function reset_state()
    STORE.reset()
    CALLBACK.reset()
end

-- ============================================================================
-- SHIORI.load / SHIORI.unload / グローバルテーブル
-- ============================================================================
describe("SHIORI.entry - load / unload / グローバルテーブル", function()
    test("require はグローバル SHIORI テーブルと同一のモジュールを返す", function()
        expect(_G.SHIORI):toBe(ENTRY)
        expect(type(ENTRY.load)):toBe("function")
        expect(type(ENTRY.request)):toBe("function")
        expect(type(ENTRY.unload)):toBe("function")
    end)

    test("SHIORI.load は true を返す", function()
        expect(ENTRY.load(0, "C:/ghost/master/")):toBe(true)
    end)

    test("SHIORI.unload はエラーなく完了し値を返さない", function()
        local results = table.pack(pcall(ENTRY.unload))
        expect(results[1]):toBe(true)
        expect(results.n):toBe(1) -- pcall の ok のみ（unload 自体は無返却）
    end)
end)

-- ============================================================================
-- SHIORI.request - xpcall 境界（正常系・エラー系）
-- ============================================================================
describe("SHIORI.request - xpcall 境界", function()
    test("登録ハンドラが文字列を返すと 200 OK + Value で応答する", function()
        reset_state()
        REG.OnEntryRequestTest = function(_act)
            return "hello from entry"
        end

        local res = ENTRY.request({ id = "OnEntryRequestTest", method = "get", version = 30 })

        expect(res:find("SHIORI/3.0 200 OK", 1, true)).not_:toBe(nil)
        expect(res:find("Value: hello from entry", 1, true)).not_:toBe(nil)
    end)

    test("未登録イベントはシーン不在なら 204 No Content で応答する", function()
        reset_state()
        local res = ENTRY.request({ id = "OnEntryNoSuchEvent", method = "get", version = 30 })

        expect(res:find("SHIORI/3.0 204 No Content", 1, true)).not_:toBe(nil)
    end)

    test("ハンドラのエラーは 500 となり X-Error-Reason は先頭行のみを含む", function()
        reset_state()
        REG.OnEntryRequestError = function(_act)
            error("entry boom\nsecond line detail")
        end

        local res = ENTRY.request({ id = "OnEntryRequestError", method = "get", version = 30 })

        expect(res:find("500 Internal Server Error", 1, true)).not_:toBe(nil)
        expect(res:find("X%-Error%-Reason:")).not_:toBe(nil)
        expect(res:find("entry boom", 1, true)).not_:toBe(nil)
        -- error_handler が先頭行のみ抽出するため 2 行目はレスポンス全体に現れない
        expect(res:find("second line detail", 1, true)):toBe(nil)
    end)

    test("文字列以外のエラーオブジェクトは Unknown error として 500 になる", function()
        reset_state()
        REG.OnEntryRequestTableError = function(_act)
            error({ code = 42 })
        end

        local res = ENTRY.request({ id = "OnEntryRequestTableError", method = "get", version = 30 })

        expect(res:find("500 Internal Server Error", 1, true)).not_:toBe(nil)
        expect(res:find("X-Error-Reason: Unknown error", 1, true)).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- GLOBAL.close_ghost / GLOBAL.ゴースト終了
-- ============================================================================
describe("GLOBAL.close_ghost - ゴースト終了スクリプト", function()
    --- wait / raw_script 呼び出しを記録するスパイ act を作成
    local function make_act_spy()
        local calls = {}
        local act = {
            wait = function(_self, ms)
                calls[#calls + 1] = "wait:" .. tostring(ms)
            end,
            raw_script = function(_self, script)
                calls[#calls + 1] = "raw:" .. script
            end,
        }
        return act, calls
    end

    test("ms >= 1 のとき wait(ms) の後に \\- を出力する", function()
        local act, calls = make_act_spy()
        GLOBAL.close_ghost(act, 250)

        expect(#calls):toBe(2)
        expect(calls[1]):toBe("wait:250")
        expect(calls[2]):toBe("raw:\\-")
    end)

    test("ms 省略時は wait せず \\- のみ出力する", function()
        local act, calls = make_act_spy()
        GLOBAL.close_ghost(act)

        expect(#calls):toBe(1)
        expect(calls[1]):toBe("raw:\\-")
    end)

    test("ms = 0 は待機条件（>= 1）を満たさず wait しない", function()
        local act, calls = make_act_spy()
        GLOBAL.close_ghost(act, 0)

        expect(#calls):toBe(1)
        expect(calls[1]):toBe("raw:\\-")
    end)

    test("ms が数値以外（文字列）の場合も wait しない", function()
        local act, calls = make_act_spy()
        GLOBAL.close_ghost(act, "500")

        expect(#calls):toBe(1)
        expect(calls[1]):toBe("raw:\\-")
    end)

    test("GLOBAL.ゴースト終了 は close_ghost と同一関数", function()
        expect(GLOBAL["ゴースト終了"]):toBe(GLOBAL.close_ghost)
    end)
end)

-- ============================================================================
-- EVENT.fire × CALLBACK - コールバック待ちコルーチンの登録とルーティング
-- ============================================================================
describe("EVENT.fire - コールバックルーティング統合", function()
    --- stage_pending → yield でコールバック待ちに入るハンドラを登録する
    --- @param event_name string 登録イベント名
    --- @return nil
    local function register_callback_handler(event_name)
        REG[event_name] = function(_act)
            return coroutine.create(function(_a)
                local cb_id = CALLBACK.next_event_id()
                CALLBACK.stage_pending(cb_id, os.time() + 60, nil)
                local refs = coroutine.yield("get-tag-script")
                coroutine.yield("received: " .. tostring(refs and refs[1]))
            end)
        end
    end

    test("コールバック待ちコルーチンは pending に登録され co_scene には保持されない", function()
        reset_state()
        register_callback_handler("OnEntryCallbackStage")

        local res = EVENT.fire({ id = "OnEntryCallbackStage" })

        -- yield 値（get タグ）が 200 OK で返る
        expect(res:find("200 OK", 1, true)).not_:toBe(nil)
        expect(res:find("get-tag-script", 1, true)).not_:toBe(nil)
        -- consume_staged により pending 登録、set_co_scene は co_scene へ保持しない
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
        expect(STORE.co_scene):toBe(nil)
        expect(STORE.co_callback):toBe(nil)
    end)

    test("到着したコールバックイベントは try_route 経由で待機コルーチンへ届く", function()
        reset_state()
        register_callback_handler("OnEntryCallbackRoute")

        EVENT.fire({ id = "OnEntryCallbackRoute" })
        local res = EVENT.fire({
            id = "OnPastaCallBack1",
            reference = { [0] = "value123" },
        })

        -- Reference0 が 1-based 配列としてコルーチンに渡り、続きの yield 値が返る
        expect(res:find("200 OK", 1, true)).not_:toBe(nil)
        expect(res:find("received: value123", 1, true)).not_:toBe(nil)
        -- ルーティング済みエントリは pending から除去される
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
    end)
end)

-- ============================================================================
-- OnSecondChange - コールバックタイムアウト sweep 分岐
-- ============================================================================
describe("OnSecondChange - sweep タイムアウト分岐", function()
    test("タイムアウト済みペンディングがあれば dispatcher を呼ばず 500 を返す", function()
        reset_state()
        dispatcher._reset()

        local co = coroutine.create(function()
            coroutine.yield()
        end)
        coroutine.resume(co)
        CALLBACK.pending["OnPastaCallBackEntryTimeout"] = {
            co = co,
            act = {},
            timeout_at = 0, -- 必ず現在時刻超過
            on_timeout = "entry sweep timeout",
        }

        local dispatch_called = false
        local original_dispatch = dispatcher.dispatch
        dispatcher.dispatch = function(...)
            dispatch_called = true
            return nil
        end

        local act = SHIORI_ACT.new(STORE.actors, {
            id = "OnSecondChange",
            status = "idle",
            date = { unix = os.time(), year = 2026, month = 6, day = 11, hour = 12, min = 0, sec = 0, wday = 4 },
        })
        local result = REG.OnSecondChange(act)

        dispatcher.dispatch = original_dispatch

        expect(type(result)):toBe("string")
        expect(result:find("500 Internal Server Error", 1, true)).not_:toBe(nil)
        expect(result:find("entry sweep timeout", 1, true)).not_:toBe(nil)
        -- タイムアウト応答時は仮想イベントディスパッチへ進まない
        expect(dispatch_called):toBe(false)
        -- 掃引済みエントリは pending から除去される
        expect(CALLBACK.pending["OnPastaCallBackEntryTimeout"]):toBe(nil)
    end)
end)
