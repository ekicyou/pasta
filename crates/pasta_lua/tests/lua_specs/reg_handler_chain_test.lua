-- ============================================================================
-- REG:register で重ねたハンドラ（鎖）のテスト (reg-handler-chain 1.1, 1.2, 2.1, 2.2, 2.3)
-- Requirements: 1.1-1.7, 2.1-2.9, 3.1-3.6, 4.1-4.5, 5.1, 5.2, 6.1-6.5, 7.1, 7.3
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 登録表を読み直す（鎖も空になる）
local function reload()
    package.loaded["pasta.shiori.event.register"] = nil
    return require("pasta.shiori.event.register")
end

--- 鎖の写し（新しい順）
local function handlers(REG, id)
    return getmetatable(REG).handlers(id)
end

describe("REG:register - 登録と鎖", function()
    test("登録は外す関数を返し、写しは新しい順に並ぶ", function()
        local REG = reload()
        local a = function() end
        local b = function() end
        local c = function() end
        expect(type(REG:register("OnTest", a))):toBe("function")
        REG:register("OnTest", b)
        REG:register("OnTest", c)

        local list = handlers(REG, "OnTest")
        expect(#list):toBe(3)
        expect(list[1]):toBe(c)
        expect(list[2]):toBe(b)
        expect(list[3]):toBe(a)
    end)

    test("写しは毎回新しい配列で、写しを変えても鎖は変わらない", function()
        local REG = reload()
        local a = function() end
        REG:register("OnTest", a)

        local first = handlers(REG, "OnTest")
        first[1] = nil
        local second = handlers(REG, "OnTest")
        expect(first ~= second):toBe(true)
        expect(second[1]):toBe(a)
    end)

    test("登録の無い ID と nil の ID の写しは空の配列", function()
        local REG = reload()
        expect(#handlers(REG, "OnNothing")):toBe(0)
        expect(#handlers(REG, nil)):toBe(0)
    end)

    test("外す関数は自分の登録だけを外し、ほかの順を変えない", function()
        local REG = reload()
        local a = function() end
        local b = function() end
        local c = function() end
        REG:register("OnTest", a)
        local off_b = REG:register("OnTest", b)
        REG:register("OnTest", c)

        off_b()
        local list = handlers(REG, "OnTest")
        expect(#list):toBe(2)
        expect(list[1]):toBe(c)
        expect(list[2]):toBe(a)
    end)

    test("外す関数の 2 回目以降は何もしない", function()
        local REG = reload()
        local a = function() end
        local b = function() end
        REG:register("OnTest", a)
        local off_b = REG:register("OnTest", b)

        off_b()
        off_b()
        local list = handlers(REG, "OnTest")
        expect(#list):toBe(1)
        expect(list[1]):toBe(a)
    end)

    test("同じ関数の 2 回の登録は別々に外れる", function()
        local REG = reload()
        local f = function() end
        local off1 = REG:register("OnTest", f)
        local off2 = REG:register("OnTest", f)
        expect(#handlers(REG, "OnTest")):toBe(2)

        off1()
        expect(#handlers(REG, "OnTest")):toBe(1)
        off1()
        expect(#handlers(REG, "OnTest")):toBe(1)
        off2()
        expect(#handlers(REG, "OnTest")):toBe(0)
    end)

    test("ID ごとに鎖が分かれ、リソースの ID にも登録できる", function()
        local REG = reload()
        local a = function() end
        local b = function() end
        REG:register("OnTest", a)
        REG:register("sakura.recommendsites", b)
        expect(handlers(REG, "OnTest")[1]):toBe(a)
        expect(#handlers(REG, "OnTest")):toBe(1)
        expect(handlers(REG, "sakura.recommendsites")[1]):toBe(b)
    end)

    test("読み直せば鎖は空になる", function()
        local REG = reload()
        REG:register("OnTest", function() end)
        REG = reload()
        expect(#handlers(REG, "OnTest")):toBe(0)
    end)
end)

describe("REG:register - 表の見え方", function()
    test("pairs と next に鎖も登録の口も出ない", function()
        local REG = reload()
        REG:register("OnTest", function() end)
        expect(next(REG)):toBeNil()

        REG.OnBoot = function() end
        local keys = {}
        for k in pairs(REG) do
            keys[#keys + 1] = k
        end
        expect(#keys):toBe(1)
        expect(keys[1]):toBe("OnBoot")
    end)

    test("REG.register だけが関数で、ほかの未登録のキーと handlers は nil", function()
        local REG = reload()
        expect(type(REG.register)):toBe("function")
        expect(REG.Unknown):toBeNil()
        expect(REG.handlers):toBeNil()
        expect(REG.OnTest):toBeNil()
    end)

    test("代入は鎖に触れず、登録は表の値を変えない", function()
        local REG = reload()
        local chained = function() end
        local assigned = function() end
        REG:register("OnTest", chained)
        expect(REG.OnTest):toBeNil()

        REG.OnTest = assigned
        expect(REG.OnTest):toBe(assigned)
        expect(handlers(REG, "OnTest")[1]):toBe(chained)
    end)
end)

describe("REG:register - 引数の誤り", function()
    test("文字列でない ID はエラーで、何も登録しない", function()
        local REG = reload()
        local ok, err = pcall(function() REG:register(1, function() end) end)
        expect(ok):toBe(false)
        expect(err:find("REG:register: id must be a string (call as REG:register(id, fn))", 1, true) ~= nil):toBe(true)
        expect(#handlers(REG, 1)):toBe(0)
    end)

    test("関数でない値はエラーで、何も登録しない", function()
        local REG = reload()
        local ok, err = pcall(function() REG:register("OnTest", "x") end)
        expect(ok):toBe(false)
        expect(err:find("REG:register: fn must be a function", 1, true) ~= nil):toBe(true)
        expect(#handlers(REG, "OnTest")):toBe(0)
    end)

    test(": を付けない呼び方は ID の誤りになり、何も登録しない", function()
        local REG = reload()
        local f = function() end
        local ok, err = pcall(function() REG.register("OnTest", f) end)
        expect(ok):toBe(false)
        expect(err:find("id must be a string", 1, true) ~= nil):toBe(true)
        expect(#handlers(REG, "OnTest")):toBe(0)
    end)

    test("エラーの位置は呼んだ所を指す", function()
        local REG = reload()
        local ok, err = pcall(function() REG:register("OnTest", nil) end)
        expect(ok):toBe(false)
        expect(err:find("reg_handler_chain_test", 1, true) ~= nil):toBe(true)
    end)
end)

--- 振り分けの関連モジュールを読み直し、REG を空にする（鎖も空になる）
local function reload_event()
    package.loaded["pasta.shiori.event"] = nil
    package.loaded["pasta.shiori.event.callback"] = nil
    package.loaded["pasta.shiori.event.register"] = nil
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.shiori.event.boot"] = nil
    package.loaded["pasta.shiori.event.second_change"] = nil

    local REG = require("pasta.shiori.event.register")
    require("pasta.store").reset()
    for k in pairs(REG) do
        REG[k] = nil
    end
    local EVENT = require("pasta.shiori.event")
    return EVENT, REG
end

--- 200 の応答で Value が value か
local function is_value(res, value)
    return res:find("^SHIORI/3.0 200") ~= nil and res:find("Value: " .. value, 1, true) ~= nil
end

describe("EVENT.fire - 鎖の振り分け (1.2)", function()
    test("鎖を新しい順に呼び、nil と false は見送り、真の値で止める", function()
        local EVENT, REG = reload_event()
        local calls = {}
        REG:register("OnTest", function() calls[#calls + 1] = "oldest"; return "never" end)
        REG:register("OnTest", function() calls[#calls + 1] = "stop"; return "stopped" end)
        REG:register("OnTest", function() calls[#calls + 1] = "false"; return false end)
        REG:register("OnTest", function() calls[#calls + 1] = "nil" end)
        local bottom = false
        REG.OnTest = function() bottom = true; return "bottom" end

        local res = EVENT.fire({ id = "OnTest" })
        expect(is_value(res, "stopped")):toBe(true)
        expect(table.concat(calls, ",")):toBe("nil,false,stop")
        expect(bottom):toBe(false)
    end)

    test("true で止めたら 204", function()
        local EVENT, REG = reload_event()
        REG:register("OnTest", function() return true end)
        REG.OnTest = function() return "bottom" end

        expect(EVENT.fire({ id = "OnTest" }):find("^SHIORI/3.0 204") ~= nil):toBe(true)
    end)

    test("全部見送れば底の REG[id] を呼び、その戻り値で応答する", function()
        local EVENT, REG = reload_event()
        REG:register("OnTest", function() end)
        REG:register("OnTest", function() return false end)
        REG.OnTest = function() return "bottom" end

        expect(is_value(EVENT.fire({ id = "OnTest" }), "bottom")):toBe(true)
    end)

    test("register という ID でも底は生の値を引く", function()
        local EVENT, REG = reload_event()
        expect(EVENT.fire({ id = "register" }):find("^SHIORI/3.0 204") ~= nil):toBe(true)
    end)

    test("ハンドラと底は呼ぶたびに別の act を受け取る", function()
        local EVENT, REG = reload_event()
        local acts = {}
        REG:register("OnTest", function(act) acts[#acts + 1] = act end)
        REG:register("OnTest", function(act) acts[#acts + 1] = act end)
        REG.OnTest = function(act) acts[#acts + 1] = act end

        EVENT.fire({ id = "OnTest" })
        expect(#acts):toBe(3)
        expect(acts[1] ~= acts[2]):toBe(true)
        expect(acts[2] ~= acts[3]):toBe(true)
        expect(acts[1] ~= acts[3]):toBe(true)
        expect(acts[3].req.id):toBe("OnTest")
    end)

    test("止まったときの表記の控えは止まったハンドラの act を指す", function()
        local EVENT, REG = reload_event()
        local STOP = require("pasta.shiori.stop")
        local failed_act
        REG:register("OnTest", function() error("older must not run") end)
        REG:register("OnTest", function(act) failed_act = act; error("boom") end)
        REG:register("OnTest", function() end)

        local ok, err = pcall(EVENT.fire, { id = "OnTest" })
        expect(ok):toBe(false)
        expect(tostring(err):find("boom", 1, true) ~= nil):toBe(true)
        expect(STOP.take_handler()):toBe(failed_act)
    end)

    test("鎖が空なら今と同じく底へ回る", function()
        local EVENT, REG = reload_event()
        REG.OnTest = function() return "bottom" end
        expect(is_value(EVENT.fire({ id = "OnTest" }), "bottom")):toBe(true)
        expect(EVENT.fire({ id = "OnUnknownForChain" }):find("^SHIORI/3.0 204") ~= nil):toBe(true)
    end)
end)

--- SCENE.search を差し替えて fn を走らせ、落ちても元に戻す
--- @param search function 差し替える SCENE.search
--- @param fn function 走らせるテストの本体
local function with_scene_search(search, fn)
    local SCENE = require("pasta.scene")
    local original = SCENE.search
    SCENE.search = search
    local ok, err = pcall(fn)
    SCENE.search = original
    if not ok then
        error(err, 0)
    end
end

--- 名前が name のときだけ、text を話すシーンを返す SCENE.search
local function scene_search(name, text)
    return function(event_name)
        if event_name ~= name then
            return nil
        end
        return {
            func = function(act) act.sakura:talk(text) end,
            global_name = name,
            local_name = "__start__",
        }
    end
end

--- 振り分けを読み直し、台詞を話せるアクターを置く
--- act と scene も読み直し、act の引く SCENE と差し替える SCENE を同じ表に揃える
local function reload_event_with_actor()
    package.loaded["pasta.scene"] = nil
    package.loaded["pasta.shiori.act"] = nil
    package.loaded["pasta.act"] = nil
    local EVENT, REG = reload_event()
    require("pasta.store").actors.sakura = { name = "さくら", spot = 0 }
    return EVENT, REG
end

describe("EVENT.fire - 止まり方・底・互換・act の分離 (2.1)", function()
    test("真の値で止めると、古い方と底を呼ばず、値の種類ごとの応答になる", function()
        local raw = "SHIORI/3.0 200 OK\r\nValue: raw\r\n\r\n"
        local cases = {
            { value = "x", check = function(res) return is_value(res, "x") end },
            { value = raw, check = function(res) return res == raw end },
            { value = true },
            { value = "" },
            { value = 1 },
            { value = 0 },
            { value = {} },
        }
        for _, case in ipairs(cases) do
            local EVENT, REG = reload_event()
            local older, bottom = false, false
            REG:register("OnTest", function() older = true; return "older" end)
            REG:register("OnTest", function() return case.value end)
            REG.OnTest = function() bottom = true; return "bottom" end

            local res = EVENT.fire({ id = "OnTest" })
            local check = case.check or function(r) return r:find("^SHIORI/3.0 204") ~= nil end
            expect(check(res)):toBe(true)
            expect(older):toBe(false)
            expect(bottom):toBe(false)
        end
    end)

    test("REG の値が無ければ、全部見送った後に同じ名前のシーンを呼ぶ", function()
        local EVENT, REG = reload_event_with_actor()
        with_scene_search(scene_search("OnTest", "シーン"), function()
            REG:register("OnTest", function() end)
            expect(EVENT.fire({ id = "OnTest" }):find("シーン", 1, true) ~= nil):toBe(true)
        end)
    end)

    test("止めた回は同じ名前のシーンも呼ばない", function()
        local EVENT, REG = reload_event_with_actor()
        with_scene_search(scene_search("OnTest", "シーン"), function()
            REG:register("OnTest", function() return "stopped" end)
            local res = EVENT.fire({ id = "OnTest" })
            expect(is_value(res, "stopped")):toBe(true)
            expect(res:find("シーン", 1, true)):toBeNil()
        end)
    end)

    test("鎖が空なら、今と同じく同じ名前のシーンを呼ぶ", function()
        local EVENT = reload_event_with_actor()
        with_scene_search(scene_search("OnTest", "シーン"), function()
            expect(EVENT.fire({ id = "OnTest" }):find("シーン", 1, true) ~= nil):toBe(true)
        end)
    end)

    test("登録の後に REG に代入しても、鎖は先に呼ばれる", function()
        local EVENT, REG = reload_event()
        REG:register("OnTest", function() return "chained" end)
        REG.OnTest = function() return "assigned" end
        expect(is_value(EVENT.fire({ id = "OnTest" }), "chained")):toBe(true)
    end)

    test("包み合いは鎖が見送れば今どおり動き、鎖が止めれば呼ばれない", function()
        local EVENT, REG = reload_event()
        REG.OnTest = function() return "inner" end
        local prev = REG.OnTest
        REG.OnTest = function(act) return "outer(" .. prev(act) .. ")" end

        local stop = false
        REG:register("OnTest", function() if stop then return "chained" end end)
        expect(is_value(EVENT.fire({ id = "OnTest" }), "outer(inner)")):toBe(true)

        stop = true
        expect(is_value(EVENT.fire({ id = "OnTest" }), "chained")):toBe(true)
    end)

    test("見送ったハンドラの台詞は、後のハンドラの出力に混ざらない", function()
        local EVENT, REG = reload_event_with_actor()
        REG:register("OnTest", function(act)
            act.sakura:talk("止めた")
            return act:build()
        end)
        REG:register("OnTest", function(act) act.sakura:talk("見送り") end)

        local res = EVENT.fire({ id = "OnTest" })
        expect(res:find("止めた", 1, true) ~= nil):toBe(true)
        expect(res:find("見送り", 1, true)):toBeNil()
    end)

    test("見送ったハンドラの台詞は、底とシーンの出力に混ざらない", function()
        local EVENT, REG = reload_event_with_actor()
        with_scene_search(scene_search("OnScene", "シーン"), function()
            for _, id in ipairs({ "OnTest", "OnScene" }) do
                REG:register(id, function(act) act.sakura:talk("見送り") end)
            end
            REG.OnTest = function(act)
                act.sakura:talk("底")
                return act:build()
            end

            local res = EVENT.fire({ id = "OnTest" })
            expect(res:find("底", 1, true) ~= nil):toBe(true)
            expect(res:find("見送り", 1, true)):toBeNil()

            res = EVENT.fire({ id = "OnScene" })
            expect(res:find("シーン", 1, true) ~= nil):toBe(true)
            expect(res:find("見送り", 1, true)):toBeNil()
        end)
    end)

    test("止めたハンドラが返したコルーチンは、そのハンドラの act で再開する", function()
        local EVENT, REG = reload_event()
        local stopped_act, resumed_act
        REG:register("OnTest", function(act)
            stopped_act = act
            return coroutine.create(function(a)
                resumed_act = a
                return "resumed"
            end)
        end)
        REG:register("OnTest", function() end)

        expect(is_value(EVENT.fire({ id = "OnTest" }), "resumed")):toBe(true)
        expect(resumed_act):toBe(stopped_act)
    end)
end)

describe("EVENT.fire - 外す・ID (2.2)", function()
    test("外した登録は次の回から呼ばれず、ほかの順は変わらず、2 回目の外しは何もしない", function()
        local EVENT, REG = reload_event()
        local calls = {}
        REG:register("OnTest", function() calls[#calls + 1] = "a" end)
        local off_b = REG:register("OnTest", function() calls[#calls + 1] = "b" end)
        REG:register("OnTest", function() calls[#calls + 1] = "c" end)
        REG.OnTest = function() calls[#calls + 1] = "bottom" end

        EVENT.fire({ id = "OnTest" })
        expect(table.concat(calls, ",")):toBe("c,b,a,bottom")

        off_b()
        expect(pcall(off_b)):toBe(true)
        calls = {}
        EVENT.fire({ id = "OnTest" })
        expect(table.concat(calls, ",")):toBe("c,a,bottom")
    end)

    test("全部外せば、鎖を足す前と同じ応答になる", function()
        local EVENT, REG = reload_event()
        REG.OnTest = function() return "bottom" end
        local before_bottom = EVENT.fire({ id = "OnTest" })
        local before_none = EVENT.fire({ id = "OnNothing" })

        local offs = {}
        for _, id in ipairs({ "OnTest", "OnNothing" }) do
            offs[#offs + 1] = REG:register(id, function() return "chained" end)
            offs[#offs + 1] = REG:register(id, function() return true end)
        end
        expect(EVENT.fire({ id = "OnTest" }) ~= before_bottom):toBe(true)
        for _, off in ipairs(offs) do
            off()
        end

        expect(EVENT.fire({ id = "OnTest" })):toBe(before_bottom)
        expect(EVENT.fire({ id = "OnNothing" })):toBe(before_none)
    end)

    test("同じ関数の 2 回の登録はどちらも呼ばれ、片方を外せば 1 回になる", function()
        local EVENT, REG = reload_event()
        local count = 0
        local f = function() count = count + 1 end
        local off1 = REG:register("OnTest", f)
        REG:register("OnTest", f)

        EVENT.fire({ id = "OnTest" })
        expect(count):toBe(2)

        off1()
        count = 0
        EVENT.fire({ id = "OnTest" })
        expect(count):toBe(1)
    end)

    test("ループの途中の登録は、その回には呼ばれず次の回から呼ばれる", function()
        local EVENT, REG = reload_event()
        local calls = {}
        local added = false
        REG:register("OnTest", function() calls[#calls + 1] = "old" end)
        REG:register("OnTest", function()
            calls[#calls + 1] = "adder"
            if not added then
                added = true
                REG:register("OnTest", function() calls[#calls + 1] = "new" end)
            end
        end)

        EVENT.fire({ id = "OnTest" })
        expect(table.concat(calls, ",")):toBe("adder,old")

        calls = {}
        EVENT.fire({ id = "OnTest" })
        expect(table.concat(calls, ",")):toBe("new,adder,old")
    end)

    test("ループの途中の外しは、その回のまだ呼ばれていないハンドラを飛ばさず、次の回から効く", function()
        local EVENT, REG = reload_event()
        local calls = {}
        local off_old = REG:register("OnTest", function() calls[#calls + 1] = "old" end)
        local off_self
        off_self = REG:register("OnTest", function()
            calls[#calls + 1] = "remover"
            off_old()
            off_self()
        end)

        EVENT.fire({ id = "OnTest" })
        expect(table.concat(calls, ",")):toBe("remover,old")

        calls = {}
        EVENT.fire({ id = "OnTest" })
        expect(#calls):toBe(0)
    end)

    test("リソースの ID にも同じ鎖の決まりが当たる", function()
        local EVENT, REG = reload_event()
        local id = "sakura.recommendsites"
        local stop = false
        REG:register(id, function() if stop then return "chained" end end)
        REG[id] = function() return "bottom" end

        expect(is_value(EVENT.fire({ id = id }), "bottom")):toBe(true)
        stop = true
        expect(is_value(EVENT.fire({ id = id }), "chained")):toBe(true)
    end)

    test("register という ID の鎖を呼び、登録の口そのものは呼ばない", function()
        local EVENT, REG = reload_event()
        local called = false
        local off = REG:register("register", function() called = true; return "chained" end)

        expect(is_value(EVENT.fire({ id = "register" }), "chained")):toBe(true)
        expect(called):toBe(true)
        expect(type(REG.register)):toBe("function")

        off()
        expect(EVENT.fire({ id = "register" }):find("^SHIORI/3.0 204") ~= nil):toBe(true)
    end)

    test("待っているコールバックへの応答の回は、鎖を呼ばない", function()
        local EVENT, REG = reload_event()
        local CALLBACK = require("pasta.shiori.event.callback")
        local called = false
        REG:register("OnPastaCallBack1", function() called = true; return "chained" end)

        local co = coroutine.create(function()
            coroutine.yield()
            coroutine.yield("callback")
        end)
        coroutine.resume(co)
        CALLBACK.pending["OnPastaCallBack1"] = { co = co, act = {}, timeout_at = 999999 }

        local res = EVENT.fire({ id = "OnPastaCallBack1", reference = {} })
        expect(is_value(res, "callback")):toBe(true)
        expect(called):toBe(false)
        CALLBACK.reset()
    end)

    test("時計の返事（OnPastaClock）の回は、鎖を呼ばない", function()
        local EVENT, REG = reload_event()
        local CLOCK = require("pasta.shiori.clock")
        CLOCK._reset()
        local called = false
        REG:register("OnPastaClock", function() called = true; return "chained" end)

        local res = EVENT.fire({
            id = "OnPastaClock",
            date = { unix = os.time(), ns = 0 },
            reference = { [0] = "2026", "10", "18", "13", "0", "0" },
        })
        expect(res:find("^SHIORI/3.0 204") ~= nil):toBe(true)
        expect(called):toBe(false)
        CLOCK._reset()
    end)
end)

-- ============================================================================
-- SHIORI.request を通す結合のテスト (2.3)
-- ============================================================================

--- entry の依存閉包（既定のハンドラを置くモジュールを含む）を同一インスタンス集合として読み直す
--- （shiori_entry_test と同じ方針。新しい REG は鎖が空で、振り分けと既定のハンドラが同じ表を見る）
local ENTRY_CLOSURE = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.mouse",
    "pasta.shiori.event.scene_play",
    "pasta.shiori.event.external_control",
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

--- @return table ENTRY, table REG, table STORE, table dispatcher
local function reload_entry()
    local mocks = require("lua_test.mocks")
    mocks.reset()
    mocks.install()
    for _, name in ipairs(ENTRY_CLOSURE) do
        package.loaded[name] = nil
    end
    local ENTRY = require("pasta.shiori.entry")
    local STORE = require("pasta.store")
    STORE.reset()
    require("pasta.shiori.event.callback").reset()
    require("pasta.shiori.stop").clear()
    return ENTRY, require("pasta.shiori.event.register"), STORE, require("pasta.shiori.event.virtual_dispatcher")
end

--- 応答のヘッダ行の値（無ければ nil）
local function header_value(res, name)
    local prefix = name .. ": "
    for line in res:gmatch("(.-)\r\n") do
        if line:sub(1, #prefix) == prefix then
            return line:sub(#prefix + 1)
        end
    end
    return nil
end

--- 応答のステータス行
local function status_of(res)
    return res:match("^(.-)\r\n")
end

--- [logging] balloon_warning を value にした pasta.config の代役で body を走らせ、元へ戻す
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

--- 表記を出さない設定と出す設定で、読み直した entry に scenario を 1 回ずつ走らせ、応答を返す
--- @param scenario fun(ENTRY: table, REG: table, STORE: table): string
--- @return string off, string on
local function run_both(scenario)
    local off, on
    for _, setting in ipairs({ false, true }) do
        local ENTRY, REG, STORE = reload_entry()
        with_balloon_warning(setting, function()
            local res = scenario(ENTRY, REG, STORE)
            if setting then on = res else off = res end
        end)
    end
    return off, on
end

local function request(ENTRY, id)
    return ENTRY.request({ id = id, method = "get", version = 30 })
end

describe("SHIORI.request - 鎖を通す結合 (2.3)", function()
    test("鎖のハンドラが返したコルーチンは、出力があれば 200 で、続きは今どおり残る", function()
        local ENTRY, REG, STORE = reload_entry()
        local co
        REG:register("OnChainCo", function()
            co = coroutine.create(function(act)
                act:raw_script("前半")
                act:yield()
                act:raw_script("後半")
                return act:build()
            end)
            return co
        end)
        REG:register("OnChainCo", function() end)
        REG.OnChainContinue = function() return STORE.co_scene end

        local res = request(ENTRY, "OnChainCo")
        expect(status_of(res)):toBe("SHIORI/3.0 200 OK")
        expect(header_value(res, "Value")):toBe("前半\\e")
        expect(STORE.co_scene):toBe(co)

        res = request(ENTRY, "OnChainContinue")
        expect(header_value(res, "Value")):toBe("後半\\e")
        expect(STORE.co_scene):toBeNil()
    end)

    test("鎖のハンドラが返したコルーチンに出力が無ければ 204", function()
        local ENTRY, REG, STORE = reload_entry()
        REG:register("OnChainCoEmpty", function()
            return coroutine.create(function() end)
        end)

        expect(status_of(request(ENTRY, "OnChainCoEmpty"))):toBe("SHIORI/3.0 204 No Content")
        expect(STORE.co_scene):toBeNil()
    end)

    test("鎖のハンドラのエラーは、止まったハンドラの台詞の後に表記が付き、底を呼ばず、次の回も両方呼ぶ", function()
        local ID = "OnChainError"
        local err_line
        local calls, bottom
        local function scenario(ENTRY, REG)
            calls, bottom = {}, false
            REG:register(ID, function(act)
                calls[#calls + 1] = "old"
                act:raw_script("止めた台詞")
                -- debug ライブラリが無いので、空のエラー文から位置を読み、その 2 行後で止める
                local _, here = pcall(function() error("") end)
                err_line = tonumber(here:match(":(%d+):%s*$")) + 2
                error("chain boom")
            end)
            REG:register(ID, function(act)
                calls[#calls + 1] = "new"
                act:raw_script("見送った台詞")
            end)
            REG[ID] = function() bottom = true; return "bottom" end
            return request(ENTRY, ID)
        end
        local off, on = run_both(scenario)

        -- 出さない設定では今どおりの 500、出す設定では同じ理由の行の 200
        expect(status_of(off)):toBe("SHIORI/3.0 500 Internal Server Error")
        expect(header_value(off, "Value")):toBeNil()
        local reason = header_value(off, "X-Error-Reason")
        expect(reason:find("reg_handler_chain_test.lua:" .. err_line .. ": chain boom", 1, true) ~= nil):toBe(true)

        expect(status_of(on)):toBe("SHIORI/3.0 200 OK")
        expect(header_value(on, "X-Error-Reason")):toBe(reason)
        expect(header_value(on, "Value")):toBe("止めた台詞【スクリプトのエラー：" .. ID .. "（chain boom）】\\e")
        expect(table.concat(calls, ",")):toBe("new,old")
        expect(bottom):toBe(false)

        -- エラーで登録は外れず、次のイベントでも両方とも呼ばれる（出す設定の表のまま）
        with_balloon_warning(true, function()
            calls = {}
            request(require("pasta.shiori.entry"), ID)
        end)
        expect(table.concat(calls, ",")):toBe("new,old")
        expect(bottom):toBe(false)
    end)

    test("鎖のハンドラが返したコルーチンのエラーは、REG のハンドラのコルーチンと同じ応答になる", function()
        local ID = "OnChainCoError"
        local function scene(act)
            act:raw_script("途中")
            error("co boom")
        end
        local off_reg, on_reg = run_both(function(ENTRY, REG)
            REG[ID] = function() return coroutine.create(scene) end
            return request(ENTRY, ID)
        end)
        local off_chain, on_chain = run_both(function(ENTRY, REG)
            REG:register(ID, function() return coroutine.create(scene) end)
            REG:register(ID, function() end)
            return request(ENTRY, ID)
        end)

        expect(off_chain):toBe(off_reg)
        expect(on_chain):toBe(on_reg)
        expect(status_of(on_chain)):toBe("SHIORI/3.0 200 OK")
        expect(header_value(on_chain, "Value")):toBe("途中【スクリプトのエラー：" .. ID .. "（co boom）】\\e")
    end)

    test("鎖が全部見送った秒は底の既定の OnSecondChange を呼び、止めた秒は呼ばない", function()
        local ENTRY, REG, _, dispatcher = reload_entry()
        local count = 0
        local original = dispatcher.dispatch
        dispatcher.dispatch = function()
            count = count + 1
            return nil
        end
        local stop = false
        local ok, err = pcall(function()
            REG:register("OnSecondChange", function() end)
            REG:register("OnSecondChange", function() if stop then return true end end)

            expect(status_of(request(ENTRY, "OnSecondChange"))):toBe("SHIORI/3.0 204 No Content")
            expect(count):toBe(1)

            stop = true
            request(ENTRY, "OnSecondChange")
            expect(count):toBe(1)
        end)
        dispatcher.dispatch = original
        if not ok then error(err, 0) end
    end)
end)

describe("REG:register - 後片付け", function()
    test("最後に entry の閉包を読み直し、鎖の無い登録表を後のスイートへ渡す", function()
        local _, REG = reload_entry()
        for _, id in ipairs({ "OnTest", "OnSecondChange", "OnChainError", "register" }) do
            expect(#handlers(REG, id)):toBe(0)
        end
        -- 振り分けと既定のハンドラが同じ新しい表を見ている
        expect(require("pasta.shiori.event.register")):toBe(REG)
        expect(type(rawget(REG, "OnSecondChange"))):toBe("function")
    end)
end)
