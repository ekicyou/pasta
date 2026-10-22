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
