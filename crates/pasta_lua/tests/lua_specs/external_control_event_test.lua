-- external-control-events: 外からの操作のイベントの既定のハンドラの単体テスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

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
    REG.OnPastaSetSaveVariable = nil
    package.loaded[MODULE] = nil
    require(MODULE)
    return REG
end

--- 既定のハンドラを 1 回呼ぶ（act は本物の ShioriAct。find_handler・build だけ差し替える）
--- opts: find（find_handler の代役）、build（build の代役）、balloon、security_level、save（pasta.save の代役）
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
        ["pasta.save"] = opts.save or {},
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

-- ============================================================================
-- 2.2: 既定のハンドラ REG.OnPastaSetSaveVariable
-- pasta.save は代役の表。Reference2 の型に従って Reference1 を読む。
-- ============================================================================

local EVENT_SET = "OnPastaSetSaveVariable"

--- 代入を 1 回送る。kind は Reference2（nil なら送らない）。save は前の値を入れた代役の表
local function set(name, text, kind, opts)
    opts = opts or {}
    opts.save = opts.save or { ["会員番号"] = "前の値" }
    local result, warns = call(EVENT_SET, { [0] = name, [1] = text, [2] = kind }, opts)
    return result, warns, opts.save
end

describe("OnPastaSetSaveVariable - 門と作者の優先", function()
    test("SecurityLevel が external なら変えずに nil を返し、warn を 1 行出す", function()
        local result, warns, save = set("会員番号", "60", "number", { security_level = "external" })
        expect(result):toBeNil()
        expect(save["会員番号"]):toBe("前の値")
        expect(#warns):toBe(1)
        expect(warns[1]):toBe(EVENT_SET .. ": SecurityLevel external is refused; nothing done")
    end)

    test("作者が先に定義したハンドラは残る", function()
        local REG = require("pasta.shiori.event.register")
        local saved = REG.OnPastaSetSaveVariable
        local mine = function() end
        REG.OnPastaSetSaveVariable = mine
        package.loaded[MODULE] = nil
        require(MODULE)
        local kept = REG.OnPastaSetSaveVariable
        REG.OnPastaSetSaveVariable = saved
        expect(kept):toBe(mine)
    end)
end)

--- 代入して nil（204）を返し、警告を出さず、値が want（型も）であることを確かめる
local function expect_assigned(name, text, kind, want)
    local result, warns, save = set(name, text, kind)
    expect(result):toBeNil()
    expect(#warns):toBe(0)
    expect(type(save[name])):toBe(type(want))
    expect(save[name]):toBe(want)
end

describe("OnPastaSetSaveVariable - 代入", function()
    test("型を省く・空・string なら届いた文字列をそのまま代入する（何の書式としても読まない）", function()
        for _, kind in ipairs({ false, "", "string" }) do
            for _, text in ipairs({ "01", '"01"', "60", "null", "" }) do
                expect_assigned("会員番号", text, kind or nil, text)
            end
        end
    end)

    test("number なら Lua の標準の数の読み方で数値を代入する", function()
        local cases = { { "60", 60 }, { "-1.5", -1.5 }, { " 60 ", 60 }, { "0x10", 16 }, { "1e3", 1000 } }
        for _, c in ipairs(cases) do
            expect_assigned("会員番号", c[1], "number", c[2])
        end
    end)

    test("boolean なら true・false を真偽値で代入する。false も消えない", function()
        expect_assigned("会員番号", "true", "boolean", true)
        expect_assigned("会員番号", "false", "boolean", false)
    end)

    test("delete なら Reference1 を見ずにキーを消し、nil（204）を返す", function()
        for _, text in ipairs({ false, "", "60", "abc" }) do
            local result, warns, save = set("会員番号", text or nil, "delete")
            expect(result):toBeNil()
            expect(#warns):toBe(0)
            expect(save["会員番号"]):toBeNil()
        end
    end)

    test("名前を区切らない（a.b はそのままのキー）", function()
        local _, _, save = set("a.b", "1", "number")
        expect(save["a.b"]):toBe(1)
        expect(save.a):toBeNil()
    end)

    test("予約変数にも代入できる", function()
        local result, _, save = set("pasta_talk_interval_min", "60", "number")
        expect(result):toBeNil()
        expect(save.pasta_talk_interval_min):toBe(60)
    end)
end)

--- 失敗して表記・警告 1 行を出し、前の値が残ることを確かめる
local function expect_refused(text, kind, notation, warning)
    local result, warns, save = set("会員番号", text, kind)
    expect(contains(result, notation)):toBe(true)
    expect(#warns):toBe(1)
    expect(warns[1]):toBe(warning)
    expect(save["会員番号"]):toBe("前の値")
end

describe("OnPastaSetSaveVariable - 失敗", function()
    test("Reference0 が無い・空なら 名前にならない値 の表記と警告", function()
        local result, warns = set(nil, "60")
        expect(contains(result, "【名前にならない値：OnPastaSetSaveVariable Reference0（値なし）】")):toBe(true)
        expect(warns[1]):toBe("OnPastaSetSaveVariable: bad argument: Reference0=nil")
        result, warns = set("", "60")
        expect(contains(result, "【名前にならない値：OnPastaSetSaveVariable Reference0（空文字列）】")):toBe(true)
        expect(warns[1]):toBe("OnPastaSetSaveVariable: bad argument: Reference0=''")
    end)

    test("number で数にならない・無限大・非数なら 数にできない値（値は引用符なし）で、前の値が残る", function()
        local cases = {
            { "abc", "abc", "'abc'" },
            { "", "空文字列", "''" },
            { nil, "値なし", "nil" },
            { "inf", "inf", "'inf'" },
            { "nan", "nan", "'nan'" },
            { "1e999", "1e999", "'1e999'" },
            { "-1e999", "-1e999", "'-1e999'" },
        }
        for i = 1, #cases do
            local c = cases[i]
            expect_refused(c[1], "number",
                "【数にできない値：OnPastaSetSaveVariable ＄＊会員番号（" .. c[2] .. "）】",
                "OnPastaSetSaveVariable: not a number: name='会員番号', Reference1=" .. c[3])
        end
    end)

    test("boolean で true・false でないなら 代入できない値 で、前の値が残る", function()
        local cases = {
            { "TRUE", "TRUE", "'TRUE'" },
            { "1", "1", "'1'" },
            { "", "空文字列", "''" },
            { nil, "値なし", "nil" },
        }
        for i = 1, #cases do
            local c = cases[i]
            expect_refused(c[1], "boolean",
                "【代入できない値：OnPastaSetSaveVariable ＄＊会員番号（" .. c[2] .. "）】",
                "OnPastaSetSaveVariable: not a boolean: name='会員番号', Reference1=" .. c[3])
        end
    end)

    test("知らない型の名前（Number・数値）なら 型の名前にならない値 で、Reference1 が無くても型の誤りを出す", function()
        for _, kind in ipairs({ "Number", "数値" }) do
            for _, text in ipairs({ "60", false }) do
                expect_refused(text or nil, kind,
                    "【型の名前にならない値：OnPastaSetSaveVariable ＄＊会員番号（" .. kind .. "）】",
                    "OnPastaSetSaveVariable: unknown type: name='会員番号', Reference2='" .. kind .. "'")
            end
        end
    end)

    test("型を省く・空・string で Reference1 が無いなら 代入できない値（値なし）で、前の値が残る", function()
        for _, kind in ipairs({ false, "", "string" }) do
            expect_refused(nil, kind or nil,
                "【代入できない値：OnPastaSetSaveVariable ＄＊会員番号（値なし）】",
                "OnPastaSetSaveVariable: no value: name='会員番号', Reference1=nil")
        end
    end)

    test("balloon_warning = false なら表記を出さず nil、ログは出し、前の値が残る", function()
        local result, warns, save = set("会員番号", "abc", "number", { balloon = false })
        expect(result):toBeNil()
        expect(#warns):toBe(1)
        expect(save["会員番号"]):toBe("前の値")
    end)
end)
