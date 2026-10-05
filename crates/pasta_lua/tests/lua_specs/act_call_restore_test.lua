-- act:call_restore / act:restore_scene / act:failure のテスト (call-execution-correctness 2.1)
-- 途中の Call から戻った後に実行中のシーンを呼び出し前へ戻す口と、失敗表記の唯一の出口
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んだ pasta.act を新規ロードして body を実行し、
--- 実行後に元のモジュールと STORE.last_global_scene を戻す（他スイートを汚さない）
--- @param body fun(ACT: table, warns: string[])
local function with_captured_act(body)
    local STORE = require("pasta.store")
    local saved_last = STORE.last_global_scene
    local saved_log = package.loaded["@pasta_log"]
    local saved_act = package.loaded["pasta.act"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.act"] = nil
    local ok, err = pcall(function()
        body(require("pasta.act"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.act"] = saved_act
    STORE.last_global_scene = saved_last
    if not ok then error(err, 0) end
end

--- 呼ばれた側の生成コードと同じく、別のグローバルシーンを init_scene するハンドラ
local function scene_handler(other, ...)
    local rets = table.pack(...)
    return function(act)
        act:init_scene(other)
        return table.unpack(rets, 1, rets.n)
    end
end

describe("act:restore_scene", function()
    test("実行中のシーンを指定の値に戻し、残りの戻り値（nil を含む）をそのまま返す", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            local A = {}
            act.current_scene = {}
            local n, a, b, c = (function(...) return select("#", ...), ... end)(
                act:restore_scene(A, 1, nil, "x"))
            expect(act.current_scene):toBe(A)
            expect(n):toBe(3)
            expect(a):toBe(1)
            expect(b):toBe(nil)
            expect(c):toBe("x")
        end)
    end)

    test("戻す先が nil なら実行中のシーンは nil、戻り値なし", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            act.current_scene = {}
            expect(select("#", act:restore_scene(nil))):toBe(0)
            expect(act.current_scene):toBe(nil)
        end)
    end)
end)

describe("act:call_restore", function()
    test("呼ばれたシーンが別のグローバルシーンを初期化しても、戻った後は呼び出し前のシーン", function()
        with_captured_act(function(ACT)
            local STORE = require("pasta.store")
            local act = ACT.new({})
            local B = { __global_name__ = "B" }
            local A = { __global_name__ = "A", callee = scene_handler(B, "ret") }
            act:init_scene(A)
            local r = act:call_restore(nil, "callee", nil)
            expect(r):toBe("ret")
            expect(act.current_scene):toBe(A)
            -- STORE.last_global_scene は戻さない（呼ばれた側の init_scene の記録のまま）
            expect(STORE.last_global_scene):toBe("B")
        end)
    end)

    test("戻り値（複数・途中の nil を含む）と引数がそのまま渡る", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            local A = {}
            local got
            A.callee = function(a, x, y)
                a:init_scene({})
                got = { x, y }
                return "r1", nil, "r3"
            end
            act.current_scene = A
            local n, r1, r2, r3 = (function(...) return select("#", ...), ... end)(
                act:call_restore(nil, "callee", nil, "p1", "p2"))
            expect(n):toBe(3)
            expect(r1):toBe("r1")
            expect(r2):toBe(nil)
            expect(r3):toBe("r3")
            expect(got[1]):toBe("p1")
            expect(got[2]):toBe("p2")
            expect(act.current_scene):toBe(A)
        end)
    end)

    test("呼んだものが Lua の関数（GLOBAL）で中から別シーンを初期化しても戻す", function()
        with_captured_act(function(ACT)
            local GLOBAL = require("pasta.global")
            local act = ACT.new({})
            local A = {}
            GLOBAL.__call_restore_test_fn = function(a)
                a:init_scene({ __global_name__ = "C" })
                return 42
            end
            act.current_scene = A
            local ok, r = pcall(act.call_restore, act, nil, "__call_restore_test_fn", nil)
            GLOBAL.__call_restore_test_fn = nil
            expect(ok):toBe(true)
            expect(r):toBe(42)
            expect(act.current_scene):toBe(A)
        end)
    end)

    test("見つからないときも呼び出し前のシーンのまま、戻り値 nil", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local A = {}
            act.current_scene = A
            local r = act:call_restore(nil, "__no_such_scene_for_call_restore__", nil)
            expect(r):toBe(nil)
            expect(act.current_scene):toBe(A)
            expect(#warns):toBe(1)
        end)
    end)

    test("入れ子: 内側の call_restore は自分の保存値、外側は外側の保存値に戻す", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            local A, B, C = {}, {}, {}
            local seen_in_b
            C.leaf = function(a) a:init_scene({}) end
            B.mid = function(a)
                a:init_scene(B)
                B.leaf = C.leaf
                a:call_restore(nil, "leaf", nil)
                seen_in_b = a.current_scene
            end
            A.mid = B.mid
            act.current_scene = A
            act:call_restore(nil, "mid", nil)
            expect(seen_in_b):toBe(B)
            expect(act.current_scene):toBe(A)
        end)
    end)

    test("呼ばれた側が中断（yield）しても、再開して戻った後に呼び出し前のシーンへ戻す", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            local A, B = {}, {}
            A.callee = function(a)
                a:init_scene(B)
                coroutine.yield("paused")
                return "done"
            end
            act.current_scene = A
            local co = coroutine.create(function()
                return act:call_restore(nil, "callee", nil)
            end)
            local _, y = coroutine.resume(co)
            expect(y):toBe("paused")
            expect(act.current_scene):toBe(B)
            local ok, r = coroutine.resume(co)
            expect(ok):toBe(true)
            expect(r):toBe("done")
            expect(act.current_scene):toBe(A)
        end)
    end)
end)

describe("act:failure", function()
    test("警告ありはログへ 1 件出し、【…】のアクター無し talk トークンを積んで nil を返す", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local r = act:failure("Call失敗：「X」が見つからない", "act:call - test warning")
            expect(r):toBe(nil)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:call - test warning")
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("talk")
            expect(act.token[1].actor):toBe(nil)
            expect(act.token[1].text):toBe("【Call失敗：「X」が見つからない】")
        end)
    end)

    test("警告 nil ならログに出さずトークンだけ積む", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            act:failure("X")
            expect(#warns):toBe(0)
            expect(act.token[1].text):toBe("【X】")
        end)
    end)

    test("実行中のシーンと STORE.last_global_scene を触らない", function()
        with_captured_act(function(ACT)
            local STORE = require("pasta.store")
            local act = ACT.new({})
            local A = {}
            act.current_scene = A
            STORE.last_global_scene = "G"
            act:failure("X", "w")
            expect(act.current_scene):toBe(A)
            expect(STORE.last_global_scene):toBe("G")
        end)
    end)
end)

describe("act:failure → group_by_actor → sakura_builder", function()
    local function count(s, pat)
        local _, n = s:gsub(pat, "")
        return n
    end

    test("発言の後: 切替タグが増えずに文字が続けて出る", function()
        with_captured_act(function(ACT)
            local BUILDER = require("pasta.shiori.sakura_builder")
            local sakura = { name = "さくら" }
            local act = ACT.new({ ["さくら"] = sakura })
            act:talk(sakura, "あ")
            act:failure("X")
            act:talk(sakura, "い")
            local out = BUILDER.build(act:build(), {}, { ["さくら"] = 0 })
            expect(count(out, "\\p%[")):toBe(1)
            expect(out:find("あ【X】い", 1, true)):toBeTruthy()
        end)
    end)

    test("出力の先頭: 切替タグなしで先頭に文字が出る", function()
        with_captured_act(function(ACT)
            local BUILDER = require("pasta.shiori.sakura_builder")
            local sakura = { name = "さくら" }
            local act = ACT.new({ ["さくら"] = sakura })
            act:failure("X")
            local only = BUILDER.build(act:build(), {}, {})
            expect(only):toBe("【X】\\e")

            act:failure("X")
            act:talk(sakura, "い")
            local out = BUILDER.build(act:build(), {}, { ["さくら"] = 0 })
            expect(out:sub(1, #"【X】")):toBe("【X】")
            expect(count(out, "\\p%[")):toBe(1)
        end)
    end)
end)
