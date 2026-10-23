-- external-control-events: 外からの操作のイベントの既定のハンドラの単体テスト
-- まずは単体テストの土台が本物の @json を出していることを確かめる。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("単体テストの土台の @json", function()
    test("null は userdata、false は真偽値で返る", function()
        local json = require("@json")
        expect(type(json.decode("null"))):toBe("userdata")
        expect(json.decode("false")):toBe(false)
    end)
end)

-- ============================================================================
-- 2.1: 既定のハンドラ REG.OnPastaCallFunction
-- @pasta_log・pasta.config を差し替え、act は本物の ShioriAct の find_handler（と build）を差し替えて確かめる。
-- ============================================================================

local MODULE = "pasta.shiori.event.external_control"

--- package.loaded の複数のキーを差し替えて fn を走らせ、必ず元に戻す（値が false なら外す）
local function with_loaded(stubs, fn)
    local saved = {}
    for name, stub in pairs(stubs) do
        saved[name] = package.loaded[name]
        package.loaded[name] = stub or nil
    end
    local ok, err = pcall(fn)
    for name in pairs(stubs) do
        package.loaded[name] = saved[name]
    end
    if not ok then error(err, 0) end
end

--- 警告を集めるログ
local function make_log(warns)
    local noop = function() end
    return {
        trace = noop, debug = noop, info = noop, error = noop,
        warn = function(msg) warns[#warns + 1] = msg end,
    }
end

--- [logging] balloon_warning を value にした pasta.config の代役
local function make_config(value)
    local real = require("pasta.config")
    return {
        get = function(section, key, default)
            if section == "logging" and key == "balloon_warning" then return value end
            return real.get(section, key, default)
        end,
    }
end

--- ハンドラを読み直す（作者のハンドラが無い状態で既定のハンドラを置く）
local function load_handlers()
    local REG = require("pasta.shiori.event.register")
    REG.OnPastaCallFunction = nil
    package.loaded[MODULE] = nil
    require(MODULE)
    return REG
end

--- 既定のハンドラを 1 回呼ぶ（act は本物の ShioriAct。find_handler・build だけ差し替える）
--- opts: find（find_handler の代役）、build（build の代役）、balloon、security_level
---@return any result, string[] warns, table act
local function call(event, refs, opts)
    opts = opts or {}
    local warns = {}
    local result, act
    local balloon = opts.balloon
    if balloon == nil then balloon = true end
    with_loaded({
        ["@pasta_log"] = make_log(warns),
        ["pasta.config"] = make_config(balloon),
    }, function()
        local REG = load_handlers()
        act = require("pasta.shiori.act").new({}, {
            id = event,
            method = "get",
            reference = refs,
            security_level = opts.security_level or "local",
        })
        act.find_handler = opts.find or function() return nil end
        if opts.build then act.build = opts.build end
        result = REG[event](act)
    end)
    return result, warns, act
end

local function contains(s, part)
    return type(s) == "string" and s:find(part, 1, true) ~= nil
end

local EVENT_CALL = "OnPastaCallFunction"

describe("OnPastaCallFunction - 門", function()
    test("SecurityLevel が external なら引かずに nil を返し、warn を 1 行出す（デバッグの代役は無い）", function()
        local looked = false
        local result, warns = call(EVENT_CALL, { [0] = "x" }, {
            security_level = "external",
            find = function() looked = true end,
        })
        expect(result):toBeNil()
        expect(looked):toBe(false)
        expect(#warns):toBe(1)
        expect(warns[1]):toBe(EVENT_CALL .. ": SecurityLevel external is refused; nothing done")
    end)

    test("作者が先に定義したハンドラは残る", function()
        local REG = require("pasta.shiori.event.register")
        local saved = REG.OnPastaCallFunction
        local mine = function() end
        REG.OnPastaCallFunction = mine
        package.loaded[MODULE] = nil
        require(MODULE)
        local kept = REG.OnPastaCallFunction
        REG.OnPastaCallFunction = saved
        expect(kept):toBe(mine)
    end)
end)

describe("OnPastaCallFunction - Reference0 の誤り", function()
    test("無い・空なら 名前にならない値 の表記と警告", function()
        local result, warns = call(EVENT_CALL, {})
        expect(contains(result, "【名前にならない値：OnPastaCallFunction Reference0（値なし）】")):toBe(true)
        expect(warns[1]):toBe("OnPastaCallFunction: bad argument: Reference0=nil")
        result, warns = call(EVENT_CALL, { [0] = "" })
        expect(contains(result, "【名前にならない値：OnPastaCallFunction Reference0（空文字列）】")):toBe(true)
        expect(warns[1]):toBe("OnPastaCallFunction: bad argument: Reference0=''")
    end)

    test("balloon_warning = false なら表記を出さず nil、ログは出す", function()
        local result, warns = call(EVENT_CALL, {}, { balloon = false })
        expect(result):toBeNil()
        expect(#warns):toBe(1)
    end)
end)

describe("OnPastaCallFunction - 見つからない", function()
    test("nil・関数でない値なら シーンが見つからない の表記と警告で、コルーチンを返さない", function()
        for _, found in ipairs({ false, "文字列" }) do
            local result, warns = call(EVENT_CALL, { [0] = "自動トーク停止" }, {
                find = function() return found or nil end,
            })
            expect(type(result)):toBe("string")
            expect(contains(result, "【シーンが見つからない：OnPastaCallFunction ＞自動トーク停止】")):toBe(true)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("OnPastaCallFunction: handler not found: name='自動トーク停止'")
        end
    end)
end)

describe("OnPastaCallFunction - 呼び出し", function()
    test("＞名前 と同じ探し方で引く（scene・名前のまま・skip_methods を渡さない）", function()
        local got
        call(EVENT_CALL, { [0] = "好感度" }, {
            find = function(_, mode, key, ...)
                got = { mode = mode, key = key, extra = select("#", ...) }
                return function() end
            end,
        })
        expect(got.mode):toBe("scene")
        expect(got.key):toBe("好感度")
        expect(got.extra):toBe(0)
    end)

    test("引数は文字列のまま、空も渡し、値の無い最初の Reference で止まる。戻り値は載せず build を返す", function()
        local args
        local co, warns, act = call(EVENT_CALL, { [0] = "f", [1] = "10", [2] = "", [4] = "b" }, {
            find = function()
                return function(a, ...)
                    args = { a = a, n = select("#", ...), ... }
                    return "RETURNED"
                end
            end,
            build = function() return "BUILT" end,
        })
        expect(type(co)):toBe("thread")
        expect(args):toBeNil()
        local ok, result = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(result):toBe("BUILT")
        expect(args.a):toBe(act)
        expect(args.n):toBe(2)
        expect(args[1]):toBe("10")
        expect(args[2]):toBe("")
        expect(#warns):toBe(0)
    end)

    test("引数が無ければ act だけを渡す", function()
        local n
        local co, _, act = call(EVENT_CALL, { [0] = "f" }, {
            find = function() return function(_, ...) n = select("#", ...) end end,
        })
        coroutine.resume(co, act)
        expect(n):toBe(0)
    end)

    test("呼び先が区切っても、再開で続きが進み、最後に build を返す", function()
        local after = false
        local co, _, act = call(EVENT_CALL, { [0] = "f" }, {
            find = function()
                return function()
                    coroutine.yield("BEAT1")
                    after = true
                end
            end,
            build = function() return "BUILT" end,
        })
        local ok, beat = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(beat):toBe("BEAT1")
        expect(after):toBe(false)
        local ok2, result = coroutine.resume(co)
        expect(ok2):toBe(true)
        expect(after):toBe(true)
        expect(result):toBe("BUILT")
        expect(coroutine.status(co)):toBe("dead")
    end)

    test("act のメソッドを引いても除かずに呼ぶ", function()
        local co, _, act = call(EVENT_CALL, { [0] = "build" }, {
            find = function(self, _, key) return self[key] end,
            build = function() return "BUILT" end,
        })
        expect(type(co)):toBe("thread")
        local ok, result = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(result):toBe("BUILT")
    end)
end)
