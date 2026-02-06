-- ============================================================================
-- GLOBAL チェイントーク関数の登録と call 解決テスト
-- Requirements: 2.1, 2.2, 2.3, 2.4
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local GLOBAL = require("pasta.global")

-- モック用の CTX オブジェクトを作成
local function create_mock_ctx()
    return {
        actors = {},
        save = {},
        yield = function() end,
        end_action = function() end,
    }
end

-- ============================================================================
-- Task 2.1: GLOBAL 関数登録の検証テスト
-- ============================================================================

describe("GLOBAL chaintalk function registration", function()
    test("GLOBAL.チェイントーク が非 nil の関数として登録されている", function()
        expect(GLOBAL["チェイントーク"]).not_:toBe(nil)
        expect(type(GLOBAL["チェイントーク"])):toBe("function")
    end)

    test("GLOBAL.yield が非 nil の関数として登録されている", function()
        expect(GLOBAL["yield"]).not_:toBe(nil)
        expect(type(GLOBAL["yield"])):toBe("function")
    end)
end)

-- ============================================================================
-- Task 2.2: L3 解決と yield 動作の検証
-- ============================================================================

describe("GLOBAL chaintalk L3 resolution and yield", function()
    test("act:call で GLOBAL.チェイントーク が L3 解決される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        -- current_scene は nil → L1 スキップ
        -- SCENE.search もなし → L2 スキップ
        -- L3: GLOBAL.チェイントーク が見つかる

        -- コルーチン内で実行（act:yield は coroutine.yield を呼ぶため）
        local co = coroutine.create(function()
            act:call("global", "チェイントーク", {})
        end)
        local ok, result = coroutine.resume(co)
        expect(ok):toBe(true)
        -- yield したのでコルーチンは suspended
        expect(coroutine.status(co)):toBe("suspended")
    end)

    test("act:call で GLOBAL.yield が L3 解決される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        local co = coroutine.create(function()
            act:call("global", "yield", {})
        end)
        local ok, result = coroutine.resume(co)
        expect(ok):toBe(true)
        expect(coroutine.status(co)):toBe("suspended")
    end)

    test("コルーチン内で act:yield() が正しく動作する（yield/resume サイクル）", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        local resumed_after_yield = false

        local co = coroutine.create(function()
            -- チェイントーク呼び出し → 内部で act:yield() → coroutine.yield()
            act:call("global", "チェイントーク", {})
            -- resume 後にここに来る
            resumed_after_yield = true
        end)

        -- 1回目 resume: yield まで実行
        local ok1, result1 = coroutine.resume(co)
        expect(ok1):toBe(true)
        expect(resumed_after_yield):toBe(false)
        expect(coroutine.status(co)):toBe("suspended")

        -- 2回目 resume: yield 後の処理を継続
        local ok2, result2 = coroutine.resume(co)
        expect(ok2):toBe(true)
        expect(resumed_after_yield):toBe(true)
        expect(coroutine.status(co)):toBe("dead")
    end)

    test("yield 前のトークンが蓄積出力として返る", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        local co = coroutine.create(function()
            -- トークンを蓄積
            act:talk("テスト出力")
            -- チェイントークで yield → build() → coroutine.yield(result)
            act:call("global", "チェイントーク", {})
            -- resume 後に追加トークン
            act:talk("後続出力")
        end)

        -- 1回目 resume: yield で中断、蓄積トークンが返る
        local ok, result = coroutine.resume(co)
        expect(ok):toBe(true)
        -- result は act:build() の結果（蓄積トークンのグループ化結果）
        expect(result).not_:toBe(nil)
    end)
end)
