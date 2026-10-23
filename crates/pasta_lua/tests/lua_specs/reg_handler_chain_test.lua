-- ============================================================================
-- REG:register で重ねたハンドラ（鎖）のテスト (reg-handler-chain 1.1, 1.2)
-- Requirements: 1.1-1.7, 2.1-2.4, 2.7, 2.9, 3.1, 3.3, 3.4, 4.1-4.3, 5.2, 6.3, 6.5, 7.1
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

    test("最後に読み直し、鎖の無い振り分けを残す", function()
        local EVENT = reload_event()
        expect(type(EVENT.fire)):toBe("function")
    end)
end)

describe("REG:register - 後片付け", function()
    test("最後に読み直し、鎖の無い登録表を後のスイートへ渡す", function()
        local REG = reload()
        expect(#handlers(REG, "OnTest")):toBe(0)
    end)
end)
