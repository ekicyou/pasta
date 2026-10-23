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
    "pasta.shiori.stop",
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
local RES = require("pasta.shiori.res")

--- fire 系テストの共通状態リセット
local function reset_state()
    STORE.reset()
    CALLBACK.reset()
end

--- 応答のヘッダ行の値を返す（行全体が「name: 値」の形のものだけを拾う。無ければ nil）
--- @param res string SHIORI 応答
--- @param name string ヘッダ名
--- @return string|nil
local function header_value(res, name)
    local prefix = name .. ": "
    for line in res:gmatch("(.-)\r\n") do
        if line:sub(1, #prefix) == prefix then
            return line:sub(#prefix + 1)
        end
    end
    return nil
end

--- [logging] balloon_warning を value にした pasta.config の代役で body を実行し、元へ戻す
--- （FAILURE.enabled は呼んだ時点で pasta.config を引く）
--- @param value boolean
--- @param body fun()
local function with_balloon_warning(value, body)
    local saved = package.loaded["pasta.config"]
    package.loaded["pasta.config"] = {
        get = function(section, key, default)
            if section == "logging" and key == "balloon_warning" then return value end
            return saved.get(section, key, default)
        end,
    }
    local ok, err = pcall(body)
    package.loaded["pasta.config"] = saved
    if not ok then error(err, 0) end
end

--- 応答の形を検査する（6.2）: 先頭行がステータス行と完全に一致し、SHIORI/3.0 が 1 回だけ現れる
--- @param res string SHIORI 応答
--- @param status_line string 期待するステータス行
local function expect_status(res, status_line)
    expect(res:match("^(.-)\r\n")):toBe(status_line)
    local _, count = res:gsub("SHIORI/3%.0", "")
    expect(count):toBe(1)
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

        -- 表記を出さない設定では今のまま（6.5）
        local res
        with_balloon_warning(false, function()
            res = ENTRY.request({ id = "OnEntryRequestError", method = "get", version = 30 })
        end)

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

        local res
        with_balloon_warning(false, function()
            res = ENTRY.request({ id = "OnEntryRequestTableError", method = "get", version = 30 })
        end)

        expect(res:find("500 Internal Server Error", 1, true)).not_:toBe(nil)
        expect(res:find("X-Error-Reason: Unknown error", 1, true)).not_:toBe(nil)
    end)
end)

describe("SHIORI.request - 止まるエラーの表記（表記を出す設定）", function()
    test("ハンドラのエラーは 200 と理由の行と表記付きの Value になる", function()
        reset_state()
        REG.OnEntryNotationError = function(_act)
            error("entry notation boom\nsecond line detail")
        end

        local res
        with_balloon_warning(true, function()
            res = ENTRY.request({ id = "OnEntryNotationError", method = "get", version = 30 })
        end)

        expect_status(res, "SHIORI/3.0 200 OK")
        expect(header_value(res, "X-Error-Reason")).not_:toBe(nil)
        local value = header_value(res, "Value")
        expect(value).not_:toBe(nil)
        expect(value:find("スクリプトのエラー", 1, true)).not_:toBe(nil)
        expect(value:find("entry notation boom", 1, true)).not_:toBe(nil)
        expect(value:find("second line detail", 1, true)):toBe(nil)
    end)

    test("シーンの中のエラーも 200 と表記付きの Value になる", function()
        reset_state()
        REG.OnEntryNotationScene = function(_act)
            return coroutine.create(function(_a)
                error("entry scene boom")
            end)
        end

        local res
        with_balloon_warning(true, function()
            res = ENTRY.request({ id = "OnEntryNotationScene", method = "get", version = 30 })
        end)

        expect_status(res, "SHIORI/3.0 200 OK")
        local value = header_value(res, "Value")
        expect(value).not_:toBe(nil)
        expect(value:find("entry scene boom", 1, true)).not_:toBe(nil)
    end)

    test("On で始まらない ID の要求は表記を出さず今の 500 になる", function()
        reset_state()
        REG.entry_notation_plain = function(_act)
            error("entry plain boom")
        end

        local res
        with_balloon_warning(true, function()
            res = ENTRY.request({ id = "entry_notation_plain", method = "get", version = 30 })
        end)

        expect_status(res, "SHIORI/3.0 500 Internal Server Error")
        expect(header_value(res, "Value")):toBe(nil)
    end)
end)

-- ============================================================================
-- SHIORI.request - 止まるエラーの表記を場所ごとに確かめる（1.1, 1.2, 1.6, 1.8, 2.5, 3.3, 3.4, 3.6, 5.2, 6.1）
-- ============================================================================
describe("SHIORI.request - 止まるエラーの表記の場所ごとの確かめ", function()
    --- イベントを SHIORI.request で送る
    --- @param id string イベント名
    --- @param reference table|nil Reference（0 始まり）
    --- @return string
    local function request(id, reference)
        return ENTRY.request({ id = id, method = "get", version = 30, reference = reference })
    end

    --- シーン関数をコルーチンにして返すハンドラを登録する
    --- @param event_name string 登録イベント名
    --- @param scene function シーン関数（引数は EVENT.fire が渡す act）
    local function register_scene(event_name, scene)
        REG[event_name] = function(_act)
            return coroutine.create(scene)
        end
    end

    --- 次の OnTalk の機会の代わりに、保存された継続をそのまま返すハンドラ
    REG.OnEntryPlaceContinue = function(_act)
        return STORE.co_scene
    end

    --- 同じ手順を表記を出さない設定と出す設定で 1 回ずつ実行し、それぞれの最後の応答を返す
    --- @param scenario fun(): string 状態を空にした後に実行する手順（最後の応答を返す）
    --- @return string off 表記を出さない設定の応答
    --- @return string on 表記を出す設定の応答
    local function run_both(scenario)
        local off, on
        with_balloon_warning(false, function()
            reset_state()
            off = scenario()
        end)
        with_balloon_warning(true, function()
            reset_state()
            on = scenario()
        end)
        return off, on
    end

    --- 表記を出す応答を検査する: 200・理由の行は出さない設定の 500 と同じ中身・Value の完全一致
    --- @param off string 表記を出さない設定の応答
    --- @param on string 表記を出す設定の応答
    --- @param value string 期待する Value（act:build が末尾に付ける \e を除いた部分）
    --- @return string reason 理由の行
    local function expect_notation(off, on, value)
        expect_status(off, "SHIORI/3.0 500 Internal Server Error")
        expect(header_value(off, "Value")):toBe(nil)
        local reason = header_value(off, "X-Error-Reason")
        expect(reason).not_:toBe(nil)

        expect_status(on, "SHIORI/3.0 200 OK")
        expect(header_value(on, "X-Error-Reason")):toBe(reason)
        expect(header_value(on, "Value")):toBe(value .. "\\e")
        return reason
    end

    test("REG のハンドラの例外は、途中の台詞の後に要求の ID を対象にした表記が出る", function()
        local off, on = run_both(function()
            REG.OnEntryPlaceHandler = function(act)
                act:raw_script("途中の台詞")
                error("handler place boom\nsecond line")
            end
            return request("OnEntryPlaceHandler")
        end)

        local reason = expect_notation(off, on, "途中の台詞【スクリプトのエラー：OnEntryPlaceHandler（handler place boom）】")
        expect(reason:find("handler place boom", 1, true)).not_:toBe(nil)
    end)

    test("シーンの中の例外は、書いた名前を対象にし、投げ直しの場所を表記に入れない", function()
        local off, on = run_both(function()
            register_scene("OnEntryPlaceScene", function(act)
                act:init_scene({}, "天気", "雨の日")
                act:raw_script("途中の台詞")
                error("scene place boom")
            end)
            return request("OnEntryPlaceScene")
        end)

        local reason = expect_notation(off, on, "途中の台詞【スクリプトのエラー：＊天気・雨の日（scene place boom）】")
        -- 理由の行は今のまま投げ直しの場所を含む（表記にだけ入らない）
        expect(reason:find("init.lua:", 1, true)).not_:toBe(nil)
    end)

    test("チェイントークの再開の中の例外は、再開した後の途中の台詞の後に表記が出る", function()
        local first_off, first_on
        local off, on = run_both(function()
            register_scene("OnEntryPlaceChain", function(act)
                act:init_scene({}, "会話")
                act:raw_script("前半")
                act:yield()
                act:raw_script("後半の途中")
                error("chain place boom")
            end)
            local first = request("OnEntryPlaceChain")
            if first_off then first_on = first else first_off = first end
            return request("OnEntryPlaceContinue")
        end)

        -- 中断までの応答は設定にかかわらず普通のトーク
        expect(header_value(first_off, "Value")):toBe("前半\\e")
        expect(first_on):toBe(first_off)
        local reason = expect_notation(off, on, "後半の途中【スクリプトのエラー：＊会話（chain place boom）】")
        expect(reason:find("init.lua:", 1, true)).not_:toBe(nil)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("待ち合わせからの再開の中の例外は、コールバックの投げ直しの場所を表記に入れない", function()
        local off, on = run_both(function()
            register_scene("OnEntryPlaceWait", function(act)
                act:init_scene({}, "天気")
                local v = act:get_property("name")
                act:raw_script("届いた:" .. tostring(v))
                error("wait place boom")
            end)
            request("OnEntryPlaceWait")
            return request("OnPastaCallBack1", { [0] = "v1" })
        end)

        local reason = expect_notation(off, on, "届いた:v1【スクリプトのエラー：＊天気（wait place boom）】")
        expect(reason:find("callback.lua:", 1, true)).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
    end)

    test("On で始まらない ID は、表記を出す設定でも出さない設定と同じ 500 になる", function()
        local off, on = run_both(function()
            REG.entry_place_plain = function(act)
                act:raw_script("途中の台詞")
                error("plain place boom")
            end
            return request("entry_place_plain")
        end)

        expect_status(on, "SHIORI/3.0 500 Internal Server Error")
        expect(on):toBe(off)
        expect(header_value(on, "Value")):toBe(nil)
    end)

    test("作者が RES.err で返した応答は、表記を付けずにそのまま返る", function()
        local built = RES.err("author reason")
        local off, on = run_both(function()
            REG.OnEntryPlaceAuthorErr = function(act)
                act:raw_script("途中の台詞")
                return built
            end
            return request("OnEntryPlaceAuthorErr")
        end)

        expect(off):toBe(built)
        expect(on):toBe(built)
    end)

    test("表記を出した次の要求は普通に処理される", function()
        with_balloon_warning(true, function()
            reset_state()
            register_scene("OnEntryPlaceStop", function(act)
                act:init_scene({}, "会話")
                act:raw_script("途中の台詞")
                error("stop then next boom")
            end)
            REG.OnEntryPlaceNext = function(_act)
                return "次のトーク"
            end

            local stopped = request("OnEntryPlaceStop")
            expect(header_value(stopped, "Value")).not_:toBe(nil)

            local res = request("OnEntryPlaceNext")
            expect_status(res, "SHIORI/3.0 200 OK")
            expect(header_value(res, "X-Error-Reason")):toBe(nil)
            expect(header_value(res, "Value")):toBe("次のトーク")
            -- 止まったシーンの継続は残らない
            expect_status(request("OnEntryPlaceContinue"), "SHIORI/3.0 204 No Content")
        end)
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
-- SHIORI.request - コールバック・掃引の再開の回帰テスト（1.1–1.4, 1.7, 1.8, 2.2, 2.4, 2.6, 3.1, 3.2, 3.4）
-- ============================================================================
describe("SHIORI.request - コールバック・掃引の再開", function()
    --- シーン関数をコルーチンにして返すハンドラを登録する
    --- @param event_name string 登録イベント名
    --- @param scene function シーン関数（引数は EVENT.fire が渡す act）
    local function register_scene(event_name, scene)
        REG[event_name] = function(_act)
            return coroutine.create(scene)
        end
    end

    --- イベントを SHIORI.request で送る
    --- @param id string イベント名
    --- @param reference table|nil Reference（0 始まり）
    --- @return string
    local function request(id, reference)
        return ENTRY.request({ id = id, method = "get", version = 30, reference = reference })
    end

    --- 次の OnTalk の機会の代わりに、保存された継続をそのまま返すハンドラ
    REG.OnEntryContinue = function(_act)
        return STORE.co_scene
    end

    --- 200 OK と Value の完全一致を検査する
    --- @param res string SHIORI 応答
    --- @param script string 期待する Value（act:build が末尾に付ける \e を除いた部分）
    local function expect_ok(res, script)
        expect_status(res, "SHIORI/3.0 200 OK")
        expect(header_value(res, "Value")):toBe(script .. "\\e")
    end

    test("コールバックの後のチェイントークは応答に前半、次の OnTalk の機会に後半が出る", function()
        reset_state()
        local scene_co
        register_scene("OnEntryChainStart", function(act)
            scene_co = coroutine.running()
            local v = act:get_property("name")
            act:raw_script("前半:" .. tostring(v))
            act:yield()
            act:raw_script("後半")
            return act:build()
        end)

        expect_ok(request("OnEntryChainStart"), "\\![get,property,OnPastaCallBack1,name]")

        expect_ok(request("OnPastaCallBack1", { [0] = "v1" }), "前半:v1")
        expect(STORE.co_scene):toBe(scene_co)
        expect(STORE.co_callback):toBe(nil)

        expect_ok(request("OnEntryContinue"), "後半")
        expect(STORE.co_scene):toBe(nil)
    end)

    test("出力の無い最初の中断は同じイベントの中で進み、トークの 200 になる", function()
        reset_state()
        register_scene("OnEntryQuietYield", function(act)
            act:get_property("name")
            act:yield() -- トークを積まずに中断
            act:raw_script("後半")
            return act:build()
        end)

        request("OnEntryQuietYield")

        expect_ok(request("OnPastaCallBack1", { [0] = "v1" }), "後半")
        expect(STORE.co_scene):toBe(nil)
    end)

    test("コールバックの後の再度の get_property は新しい待機になり、継続には残らない", function()
        reset_state()
        local scene_co
        register_scene("OnEntryGetTwice", function(act)
            scene_co = coroutine.running()
            act:get_property("a")
            act:get_property("b")
        end)

        request("OnEntryGetTwice")

        expect_ok(request("OnPastaCallBack1", { [0] = "v1" }), "\\![get,property,OnPastaCallBack2,b]")
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack2"].co):toBe(scene_co)
        expect(STORE.co_scene):toBe(nil)
        expect(STORE.co_callback):toBe(nil)
    end)

    --- コールバック待ちの間に、別のシーンの継続を作る
    --- @return thread other_co 別のシーンのコルーチン
    local function start_other_continuation()
        local other_co
        register_scene("OnEntryOther", function(act)
            other_co = coroutine.running()
            act:raw_script("別1")
            act:yield()
            act:raw_script("別2")
            return act:build()
        end)
        expect_ok(request("OnEntryOther"), "別1")
        expect(STORE.co_scene):toBe(other_co)
        return other_co
    end

    test("コールバック側のシーンが中断すれば、既存の継続を置き換える", function()
        reset_state()
        local scene_co
        register_scene("OnEntryReplace", function(act)
            scene_co = coroutine.running()
            act:get_property("name")
            -- 台詞を出すので、戻った回に別のシーンの継続を置き換える (chain-talk-interrupt 2.5)
            act:talk(require("pasta.actor").get_or_create("さくら"), "前半")
            act:yield()
            act:raw_script("後半")
            return act:build()
        end)

        request("OnEntryReplace")
        local other_co = start_other_continuation()

        expect_ok(request("OnPastaCallBack1", { [0] = "v1" }), "\\p[0]前半")
        expect(STORE.co_scene):toBe(scene_co)
        expect(STORE.co_scene).not_:toBe(other_co)
    end)

    test("コールバック側のシーンが終われば、既存の継続を空にする", function()
        reset_state()
        register_scene("OnEntryFinish", function(act)
            act:get_property("name")
            -- 台詞を出すので、戻った回に別のシーンの継続を空にする (chain-talk-interrupt 2.5)
            act:talk(require("pasta.actor").get_or_create("さくら"), "終わり")
            return act:build()
        end)

        request("OnEntryFinish")
        start_other_continuation()

        expect_ok(request("OnPastaCallBack1", { [0] = "v1" }), "\\p[0]終わり")
        expect(STORE.co_scene):toBe(nil)
    end)

    --- 仮想ディスパッチャの呼び出しを数えながら OnSecondChange を SHIORI.request で送る
    --- @return string res 応答
    --- @return number dispatch_count 仮想ディスパッチャが呼ばれた回数
    local function request_second_change()
        local dispatch_count = 0
        local original_dispatch = dispatcher.dispatch
        dispatcher.dispatch = function(_act)
            dispatch_count = dispatch_count + 1
            return nil
        end
        local ok, res = pcall(request, "OnSecondChange")
        dispatcher.dispatch = original_dispatch
        assert(ok, res)
        return res, dispatch_count
    end

    test("理由付きの待機のタイムアウトは X-Error-Reason に理由を持つ 500 で、仮想イベントを出さない", function()
        with_balloon_warning(false, function()
            reset_state()
            dispatcher._reset()
            register_scene("OnEntryTimeout", function(act)
                act:get_property("name", -1, "entry sweep timeout") -- 期限は既に過ぎている
                act:raw_script("届かない")
                return act:build()
            end)
            request("OnEntryTimeout")

            local res, dispatch_count = request_second_change()

            expect_status(res, "SHIORI/3.0 500 Internal Server Error")
            expect(header_value(res, "X-Error-Reason")):toBe("entry sweep timeout")
            expect(header_value(res, "Value")):toBe(nil)
            expect(dispatch_count):toBe(0)
            expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
            expect(STORE.co_scene):toBe(nil)
        end)
    end)

    --- タイムアウトを捕まえて再度 get_property するシーンを掃引する
    --- 掃引の応答の検査は各テストの最後に回し、予約の漏れの検査を先に行う
    --- @return thread scene_co 掃引したシーンのコルーチン
    --- @return function expect_sweep_response 掃引の応答（再度の get_property の get タグの 200）を検査する
    local function sweep_retrying_scene()
        local scene_co
        register_scene("OnEntryRetry", function(act)
            scene_co = coroutine.running()
            pcall(act.get_property, act, "a", -1, "entry retry timeout") -- 期限は既に過ぎている
            act:get_property("b", 60)
        end)
        request("OnEntryRetry")
        local res, dispatch_count = request_second_change()
        return scene_co, function()
            expect_ok(res, "\\![get,property,OnPastaCallBack2,b]")
            expect(dispatch_count):toBe(0)
        end
    end

    test("掃引の後、コルーチンを返す別のイベントのシーンは古いイベント名で登録されない", function()
        reset_state()
        dispatcher._reset()
        local scene_co, expect_sweep_response = sweep_retrying_scene()
        register_scene("OnEntryAfterSweep", function(act)
            act:raw_script("別のシーン")
            return act:build()
        end)

        expect_ok(request("OnEntryAfterSweep"), "別のシーン")
        expect(CALLBACK.pending["OnPastaCallBack2"].co):toBe(scene_co)
        expect(STORE.co_callback):toBe(nil)
        expect_sweep_response()
    end)

    test("掃引の後、get_property する別のイベントは multiple staging にならない", function()
        reset_state()
        dispatcher._reset()
        local _, expect_sweep_response = sweep_retrying_scene()
        register_scene("OnEntryGetAfterSweep", function(act)
            act:get_property("c")
        end)

        expect_ok(request("OnEntryGetAfterSweep"), "\\![get,property,OnPastaCallBack3,c]")
        expect(CALLBACK.pending["OnPastaCallBack3"]).not_:toBe(nil)
        expect_sweep_response()
    end)

    -- ========================================================================
    -- runtime-error-balloon 5: 時間切れの出口の表記（1.3, 1.4, 1.5, 4.2, 5.6, 6.1, 6.2）
    -- ========================================================================

    --- body を実行する間の警告ログを集める
    --- @param body fun()
    --- @return string[] warns
    local function collect_warns(body)
        local log = package.loaded["@pasta_log"]
        local original = log.warn
        local warns = {}
        log.warn = function(msg) warns[#warns + 1] = msg end
        local ok, err = pcall(body)
        log.warn = original
        if not ok then error(err, 0) end
        return warns
    end

    --- 時間切れの手順を表記を出さない設定と出す設定で 1 回ずつ実行する
    --- @param scene function OnEntryTimeoutPlace のシーン関数
    --- @return table off { res, dispatch_count, warns }
    --- @return table on { res, dispatch_count, warns }
    local function run_timeout_both(scene)
        local results = {}
        for _, value in ipairs({ false, true }) do
            with_balloon_warning(value, function()
                reset_state()
                dispatcher._reset()
                register_scene("OnEntryTimeoutPlace", scene)
                request("OnEntryTimeoutPlace")
                local r = {}
                r.warns = collect_warns(function()
                    r.res, r.dispatch_count = request_second_change()
                end)
                results[#results + 1] = r
            end)
        end
        return results[1], results[2]
    end

    test("理由付きの時間切れは、途中の台詞の後に時間切れの表記の 200（理由の行は出さない設定の 500 と同じ）", function()
        local off, on = run_timeout_both(function(act)
            act:init_scene({}, "天気")
            act:raw_script("途中の台詞")
            act:get_property("name", -1) -- 期限は既に過ぎている・既定の timeout_message
            act:raw_script("届かない")
        end)

        expect_status(off.res, "SHIORI/3.0 500 Internal Server Error")
        expect(header_value(off.res, "X-Error-Reason")):toBe("callback timeout: get_property")
        expect(header_value(off.res, "Value")):toBe(nil)

        expect_status(on.res, "SHIORI/3.0 200 OK")
        expect(header_value(on.res, "X-Error-Reason")):toBe("callback timeout: get_property")
        expect(header_value(on.res, "Value")):toBe("途中の台詞【時間切れ：callback timeout: get_property】\\e")
        for _, r in ipairs({ off, on }) do
            expect(r.dispatch_count):toBe(0)
            -- 警告は再開の前の 1 行だけ（今のまま）
            expect(#r.warns):toBe(1)
            expect(r.warns[1]):toBe("OnPastaCallBack1: callback timeout: get_property")
        end
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("理由の無い時間切れの後のエラーは、スクリプトのエラーの表記の 200 を返し、整えた警告を 1 行出す", function()
        local off, on = run_timeout_both(function(act)
            act:init_scene({}, "天気")
            local v = act:get_property("name", -1, false) -- 期限は既に過ぎている・静かに再開
            act:raw_script("値:" .. tostring(v))
            error("silent place boom")
        end)

        -- 出さない設定は今と同じ: 掃引の応答は無く、仮想イベントへ進む
        expect_status(off.res, "SHIORI/3.0 204 No Content")
        expect(off.dispatch_count):toBe(1)

        expect_status(on.res, "SHIORI/3.0 200 OK")
        expect(on.dispatch_count):toBe(0)
        expect(header_value(on.res, "Value")):toBe("値:nil【スクリプトのエラー：＊天気（silent place boom）】\\e")
        local reason = header_value(on.res, "X-Error-Reason")
        expect(reason:find("silent place boom", 1, true)).not_:toBe(nil)

        -- 警告は両方の設定で、event_id を前に付けた整えた中身 1 行（変換した Lua の場所を含まない）
        for _, r in ipairs({ off, on }) do
            expect(#r.warns):toBe(1)
            expect(r.warns[1]):toBe("OnPastaCallBack1: スクリプトのエラー：＊天気（silent place boom）")
        end
    end)

    test("掃引で止まったシーンの控えは、同じ要求の後の止まるエラーに混ざらない", function()
        with_balloon_warning(false, function()
            reset_state()
            dispatcher._reset()
            register_scene("OnEntryTimeoutStale", function(act)
                act:init_scene({}, "天気")
                act:get_property("name", -1, false)
                error("stale place boom")
            end)
            request("OnEntryTimeoutStale")

            local original_dispatch = dispatcher.dispatch
            dispatcher.dispatch = function(_act)
                error("dispatch place boom")
            end
            local ok, res
            local warns = collect_warns(function()
                ok, res = pcall(request, "OnSecondChange")
            end)
            dispatcher.dispatch = original_dispatch
            assert(ok, res)

            expect_status(res, "SHIORI/3.0 500 Internal Server Error")
            expect(header_value(res, "X-Error-Reason"):find("dispatch place boom", 1, true)).not_:toBe(nil)
            expect(#warns):toBe(2)
            expect(warns[1]):toBe("OnPastaCallBack1: スクリプトのエラー：＊天気（stale place boom）")
            expect(warns[2]):toBe("スクリプトのエラー：OnSecondChange（dispatch place boom）")
        end)
    end)
end)

-- ============================================================================
-- SHIORI.request - ハンドラの戻り値の応答化（4.1, 4.2, 4.4, 4.5, 4.6）
-- ============================================================================
describe("SHIORI.request - ハンドラの戻り値の応答化", function()
    --- 戻り値を返すハンドラを登録して SHIORI.request を通した応答を返す
    --- @param value any ハンドラの戻り値
    --- @return string
    local function request_with(value)
        reset_state()
        REG.OnEntryReturnTest = function(_act)
            return value
        end
        return ENTRY.request({ id = "OnEntryReturnTest", method = "get", version = 30 })
    end

    local responses = {
        { "RES.ok", RES.ok("x") },
        { "RES.no_content", RES.no_content() },
        { "RES.warn", RES.warn("r") },
        { "RES.not_enough", RES.not_enough() },
        { "RES.advice", RES.advice() },
        { "RES.err", RES.err("r") },
    }
    for _, case in ipairs(responses) do
        local name, built = case[1], case[2]
        test(name .. " で作った応答は包まれずにそのまま返る", function()
            expect(request_with(built)):toBe(built)
        end)
    end

    test("普通の文字列は Value にした 200 OK になる", function()
        local res = request_with("plain")
        expect_status(res, "SHIORI/3.0 200 OK")
        expect(header_value(res, "Value")):toBe("plain")
    end)

    test("空文字列は Value の無い 204 No Content になる", function()
        local res = request_with("")
        expect_status(res, "SHIORI/3.0 204 No Content")
        expect(header_value(res, "Value")):toBe(nil)
    end)

    test("nil は Value の無い 204 No Content になる", function()
        local res = request_with(nil)
        expect_status(res, "SHIORI/3.0 204 No Content")
        expect(header_value(res, "Value")):toBe(nil)
    end)
end)
