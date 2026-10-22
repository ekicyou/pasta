-- scene-play-event: シーンを名前・登録名の組から再生する口の単体テスト
-- pasta.scene の co_exec_exact（2 つ目の戻り値・ローカル）と co_exec_registered を確かめる。
--
-- STORE.scenes はスイート間で共有されるため、衝突しない一意プレフィックス "spe_" を使う。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil
package.loaded["pasta.scene"] = nil

local SCENE = require("pasta.scene")

--- @pasta_search を差し替えて fn を走らせ、必ず元に戻す
local function with_search(stub, fn)
    local saved = package.loaded["@pasta_search"]
    package.loaded["@pasta_search"] = stub
    local ok, err = pcall(fn)
    package.loaded["@pasta_search"] = saved
    if not ok then error(err, 0) end
end

--- build だけを持つ最小の act
local function make_act(build_result)
    return { build = function() return build_result end }
end

--- コルーチンを act で進め、(ok, 戻り値) を返す
local function run(co, act)
    return coroutine.resume(co, act)
end

describe("SCENE.co_exec_exact - 大域だけ", function()
    test("見つかればコルーチンを返し、最後に act:build() を返す", function()
        local ran = false
        SCENE.register("spe_g_1", "__start__", function() ran = true end)
        with_search({
            search_scene_exact = function(_, name, scope)
                expect(name):toBe("spe_g")
                expect(scope):toBeNil()
                return "spe_g_1", "__start__"
            end,
        }, function()
            local co, miss = SCENE.co_exec_exact("spe_g")
            expect(type(co)):toBe("thread")
            expect(miss):toBeNil()
            local ok, result = run(co, make_act("BUILT"))
            expect(ok):toBe(true)
            expect(result):toBe("BUILT")
            expect(ran):toBe(true)
        end)
    end)

    test("大域が無ければ nil と \"global\" を返す", function()
        with_search({
            search_scene_exact = function() return nil end,
        }, function()
            local co, miss = SCENE.co_exec_exact("spe_none")
            expect(co):toBeNil()
            expect(miss):toBe("global")
        end)
    end)
end)

describe("SCENE.co_exec_exact - ローカルを渡す", function()
    test("search_local_scene_exact で引き、ローカルのコルーチンを返す", function()
        local ran = false
        SCENE.register("spe_l_1", "挨拶_1", function() ran = true end)
        with_search({
            search_scene_exact = function() error("must not be called") end,
            search_local_scene_exact = function(_, g, l)
                expect(g):toBe("spe_l")
                expect(l):toBe("挨拶")
                return "spe_l_1", "挨拶_1"
            end,
        }, function()
            local co, miss = SCENE.co_exec_exact("spe_l", "挨拶")
            expect(type(co)):toBe("thread")
            expect(miss):toBeNil()
            local ok, result = run(co, make_act("LOCAL_BUILT"))
            expect(ok):toBe(true)
            expect(result):toBe("LOCAL_BUILT")
            expect(ran):toBe(true)
        end)
    end)

    test("大域が無ければ nil と \"global\"", function()
        with_search({
            search_local_scene_exact = function() return nil, "global" end,
        }, function()
            local co, miss = SCENE.co_exec_exact("spe_none", "挨拶")
            expect(co):toBeNil()
            expect(miss):toBe("global")
        end)
    end)

    test("ローカルが無ければ nil と \"local\"", function()
        with_search({
            search_local_scene_exact = function() return nil, "local" end,
        }, function()
            local co, miss = SCENE.co_exec_exact("spe_l", "不在")
            expect(co):toBeNil()
            expect(miss):toBe("local")
        end)
    end)
end)

describe("SCENE.co_exec_registered", function()
    test("登録名の組（__start__）から大域のコルーチンを作る", function()
        local ran = false
        SCENE.register("spe_r_1", "__start__", function() ran = true end)
        local co = SCENE.co_exec_registered("spe_r_1", "__start__")
        expect(type(co)):toBe("thread")
        local ok, result = run(co, make_act("REG_BUILT"))
        expect(ok):toBe(true)
        expect(result):toBe("REG_BUILT")
        expect(ran):toBe(true)
    end)

    test("登録名の組からローカルのコルーチンを作り、act を渡す", function()
        local got
        SCENE.register("spe_r_1", "挨拶_1", function(a) got = a end)
        local act = make_act(nil)
        local co = SCENE.co_exec_registered("spe_r_1", "挨拶_1")
        local ok, result = run(co, act)
        expect(ok):toBe(true)
        expect(result):toBeNil()
        expect(got):toBe(act)
    end)

    test("引けなければ nil を返す", function()
        expect(SCENE.co_exec_registered("spe_r_none", "__start__")):toBeNil()
        expect(SCENE.co_exec_registered("spe_r_1", "不在_1")):toBeNil()
    end)
end)

-- ============================================================================
-- 3.2: 既定のハンドラ REG.OnPastaScene・REG.OnPastaSceneAt
-- @pasta_debug・@pasta_search・@pasta_log・pasta.config を差し替えて、門・引数・失敗・成功を確かめる。
-- ============================================================================

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

--- 有効・無効と位置の引き当てを持つ @pasta_debug の代役
local function make_debug(enabled, scene_at)
    return {
        enabled = function() return enabled end,
        scene_at = scene_at or function() return nil end,
    }
end

--- ハンドラを読み直す（作者のハンドラが無い状態で既定のハンドラを置く）
local function load_handlers()
    local REG = require("pasta.shiori.event.register")
    REG.OnPastaScene = nil
    REG.OnPastaSceneAt = nil
    package.loaded["pasta.shiori.event.scene_play"] = nil
    require("pasta.shiori.event.scene_play")
    return REG
end

--- 既定のハンドラを本物の ShioriAct で 1 回呼ぶ
--- opts: debug（@pasta_debug の代役。false なら無し）、search、balloon、security_level
---@return any result, string[] warns
local function call(event, refs, opts)
    opts = opts or {}
    local warns = {}
    local result
    local debug = opts.debug
    if debug == nil then debug = make_debug(true) end
    local balloon = opts.balloon
    if balloon == nil then balloon = true end
    with_loaded({
        ["@pasta_debug"] = debug,
        ["@pasta_search"] = opts.search or { search_scene_exact = function() return nil end },
        ["@pasta_log"] = make_log(warns),
        ["pasta.config"] = make_config(balloon),
    }, function()
        local REG = load_handlers()
        local act = require("pasta.shiori.act").new({}, {
            id = event,
            method = "get",
            reference = refs,
            security_level = opts.security_level or "local",
        })
        result = REG[event](act)
    end)
    return result, warns
end

local function contains(s, part)
    return type(s) == "string" and s:find(part, 1, true) ~= nil
end

describe("既定のハンドラ - 門", function()
    test("デバッグが無効なら nil を返し、warn を出す（引数が誤っていても表記しない）", function()
        for _, event in ipairs({ "OnPastaScene", "OnPastaSceneAt" }) do
            local result, warns = call(event, {}, { debug = make_debug(false) })
            expect(result):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(event .. ": debug is disabled; scene not played")
        end
    end)

    test("@pasta_debug が無ければ無効と同じ", function()
        local result, warns = call("OnPastaScene", { [0] = "会話" }, { debug = false })
        expect(result):toBeNil()
        expect(warns[1]):toBe("OnPastaScene: debug is disabled; scene not played")
    end)

    test("SecurityLevel が external なら nil を返し、warn を出す", function()
        for _, event in ipairs({ "OnPastaScene", "OnPastaSceneAt" }) do
            local result, warns = call(event, { [0] = "x", [1] = "1" }, { security_level = "external" })
            expect(result):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(event .. ": SecurityLevel external is refused; scene not played")
        end
    end)

    test("作者が先に定義したハンドラは残る", function()
        local REG = require("pasta.shiori.event.register")
        local saved_a, saved_b = REG.OnPastaScene, REG.OnPastaSceneAt
        local mine_a, mine_b = function() end, function() end
        REG.OnPastaScene, REG.OnPastaSceneAt = mine_a, mine_b
        package.loaded["pasta.shiori.event.scene_play"] = nil
        require("pasta.shiori.event.scene_play")
        local kept_a, kept_b = REG.OnPastaScene, REG.OnPastaSceneAt
        REG.OnPastaScene, REG.OnPastaSceneAt = saved_a, saved_b
        expect(kept_a):toBe(mine_a)
        expect(kept_b):toBe(mine_b)
    end)
end)

describe("既定のハンドラ - 引数の誤り", function()
    test("Reference0 が無い・空なら 名前にならない値", function()
        local result, warns = call("OnPastaScene", {})
        expect(contains(result, "【名前にならない値：OnPastaScene Reference0（値なし）】")):toBe(true)
        expect(warns[1]):toBe("OnPastaScene: bad argument: Reference0=nil")
        result = call("OnPastaSceneAt", { [0] = "", [1] = "1" })
        expect(contains(result, "【名前にならない値：OnPastaSceneAt Reference0（空文字列）】")):toBe(true)
    end)

    test("行が半角数字だけの 1 以上の整数でなければ 数にできない値", function()
        local called = false
        local debug = make_debug(true, function() called = true end)
        for _, line in ipairs({ "４２", " 42", "0", "1.5", "abc", "-1", "" }) do
            local result, warns = call("OnPastaSceneAt", { [0] = "dic/a.pasta", [1] = line }, { debug = debug })
            expect(contains(result, "【数にできない値：OnPastaSceneAt Reference1（\"" .. line .. "\"）】")):toBe(true)
            expect(warns[1]):toBe("OnPastaSceneAt: bad argument: Reference1='" .. line .. "'")
        end
        local result = call("OnPastaSceneAt", { [0] = "dic/a.pasta" }, { debug = debug })
        expect(contains(result, "【数にできない値：OnPastaSceneAt Reference1（値なし）】")):toBe(true)
        expect(called):toBe(false)
    end)

    test("balloon_warning = false なら表記を出さず nil、ログは出す", function()
        local result, warns = call("OnPastaScene", {}, { balloon = false })
        expect(result):toBeNil()
        expect(#warns):toBe(1)
    end)
end)

describe("既定のハンドラ - 見つからない", function()
    test("大域が無い", function()
        local result, warns = call("OnPastaScene", { [0] = "会話" })
        expect(contains(result, "【シーンが見つからない：OnPastaScene ＊会話】")):toBe(true)
        expect(warns[1]):toBe("OnPastaScene: scene not found: global='会話'")
    end)

    test("ローカルが無い", function()
        local search = { search_local_scene_exact = function() return nil, "local" end }
        local result, warns = call("OnPastaScene", { [0] = "会話", [1] = "挨拶" }, { search = search })
        expect(contains(result, "【シーンが見つからない：OnPastaScene ＊会話 ・挨拶】")):toBe(true)
        expect(warns[1]):toBe("OnPastaScene: scene not found: global='会話', local='挨拶'")
    end)

    test("位置に無い（渡された文字列のまま。\\ はエスケープされる）", function()
        local got
        local debug = make_debug(true, function(file, line) got = { file, line } end)
        local result, warns = call("OnPastaSceneAt", { [0] = "dic\\boot.pasta", [1] = "42" }, { debug = debug })
        expect(got[1]):toBe("dic\\boot.pasta")
        expect(got[2]):toBe(42)
        expect(contains(result, "【シーンが見つからない：OnPastaSceneAt dic\\\\boot.pasta:42】")):toBe(true)
        expect(warns[1]):toBe("OnPastaSceneAt: scene not found: file='dic\\boot.pasta', line=42")
    end)

    test("位置に無い（u32 を超える行は、ログにも渡された文字列のまま出す）", function()
        local big = "99999999999999999999"
        local result, warns = call("OnPastaSceneAt", { [0] = "dic/a.pasta", [1] = big })
        expect(contains(result, "【シーンが見つからない：OnPastaSceneAt dic/a.pasta:" .. big .. "】")):toBe(true)
        expect(warns[1]):toBe("OnPastaSceneAt: scene not found: file='dic/a.pasta', line=" .. big)
    end)
end)

describe("既定のハンドラ - 再生", function()
    test("名前: 空の Reference1 は省略と同じで、大域のコルーチンを返し 1 回目の再開で台本が返る", function()
        SCENE.register("spe_h_1", "__start__", function() end)
        local search = {
            search_scene_exact = function(_, name)
                if name == "spe_h" then return "spe_h_1", "__start__" end
            end,
            search_local_scene_exact = function() error("must not be called") end,
        }
        local co = call("OnPastaScene", { [0] = "spe_h", [1] = "" }, { search = search })
        expect(type(co)):toBe("thread")
        local ok, result = coroutine.resume(co, make_act("SCRIPT"))
        expect(ok):toBe(true)
        expect(result):toBe("SCRIPT")
    end)

    test("名前: ローカルを指名する", function()
        SCENE.register("spe_h_1", "挨拶_1", function() end)
        local search = {
            search_local_scene_exact = function(_, g, l)
                if g == "spe_h" and l == "挨拶" then return "spe_h_1", "挨拶_1" end
            end,
        }
        local co = call("OnPastaScene", { [0] = "spe_h", [1] = "挨拶" }, { search = search })
        expect(type(co)):toBe("thread")
    end)

    test("位置: scene_at の登録名の組からコルーチンを返す", function()
        SCENE.register("spe_h_1", "挨拶_1", function() end)
        local debug = make_debug(true, function() return "spe_h_1", "挨拶_1" end)
        local co = call("OnPastaSceneAt", { [0] = "dic/a.pasta", [1] = "007" }, { debug = debug })
        expect(type(co)):toBe("thread")
    end)
end)

describe("既定のハンドラ - EVENT.fire を通す", function()
    --- イベントのモジュールを読み直し、EVENT・STORE を返す
    local function reload()
        for _, name in ipairs({
            "pasta.shiori.event", "pasta.shiori.event.callback", "pasta.shiori.event.register",
            "pasta.store", "pasta.shiori.event.boot", "pasta.shiori.event.second_change",
            "pasta.shiori.event.scene_play",
        }) do
            package.loaded[name] = nil
        end
        local STORE = require("pasta.store")
        STORE.reset()
        return require("pasta.shiori.event"), STORE
    end

    --- @pasta_debug を有効にし、spe_f を見つける検索で fn(EVENT, STORE) を走らせる
    local function with_fire(fn)
        SCENE.register("spe_f_1", "__start__", function()
            coroutine.yield("BEAT1")
            coroutine.yield("BEAT2")
        end)
        with_loaded({
            ["@pasta_debug"] = make_debug(true),
            ["@pasta_search"] = {
                search_scene_exact = function(_, name)
                    if name == "spe_f" then return "spe_f_1", "__start__" end
                end,
            },
            ["@pasta_log"] = make_log({}),
        }, function()
            fn(reload())
        end)
    end

    --- 止まっている場面の続き（suspended のコルーチン）
    local function pending_scene()
        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)
        return co
    end

    test("成功: 1 ビート目を 200 で返し、続きを co_scene に置く。選択肢の表示中でも再生する", function()
        with_fire(function(EVENT, STORE)
            local old = pending_scene()
            STORE.co_scene = old
            local res = EVENT.fire({
                id = "OnPastaScene", method = "get", reference = { [0] = "spe_f" },
                status = "choosing", security_level = "local",
            })
            expect(contains(res, "200 OK")):toBe(true)
            expect(contains(res, "BEAT1")):toBe(true)
            expect(contains(res, "BEAT2")):toBe(false)
            expect(STORE.co_scene ~= old):toBe(true)
            expect(coroutine.status(STORE.co_scene)):toBe("suspended")
        end)
    end)

    test("失敗: 場面の続きを変えない", function()
        with_fire(function(EVENT, STORE)
            local old = pending_scene()
            STORE.co_scene = old
            local res = EVENT.fire({
                id = "OnPastaScene", method = "get", reference = { [0] = "none" },
                security_level = "local",
            })
            expect(contains(res, "シーンが見つからない")):toBe(true)
            expect(STORE.co_scene):toBe(old)
        end)
    end)
end)
