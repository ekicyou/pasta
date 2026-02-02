-- store_coroutine_test.lua
-- Lua-side BDD tests for STORE.co_scene coroutine management
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- Task 1.1: co_scene フィールドと初期化処理
-- Requirements: 7.1, 7.3
-- ============================================================================

describe("STORE.co_scene - フィールド定義", function()
    local STORE

    local function setup()
        -- パッケージキャッシュをクリアして新鮮なモジュールをロード
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
    end

    test("co_scene フィールドが存在する", function()
        setup()
        -- co_sceneフィールドが存在すること（nilで初期化）
        expect(STORE.co_scene):toBe(nil)
    end)

    test("co_scene はthreadまたはnilを保持できる", function()
        setup()
        -- threadを設定
        local co = coroutine.create(function() end)
        STORE.co_scene = co
        expect(type(STORE.co_scene)):toBe("thread")

        -- nilに戻せること
        STORE.co_scene = nil
        expect(STORE.co_scene):toBe(nil)
    end)
end)

-- ============================================================================
-- Task 1.2: STORE.reset() クリーンアップ処理
-- Requirements: 7.2
-- ============================================================================

describe("STORE.reset() - co_sceneクリーンアップ", function()
    local STORE

    local function setup()
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
    end

    test("reset()後にco_sceneがnilになる", function()
        setup()
        -- suspendedコルーチンを設定
        local co = coroutine.create(function()
            coroutine.yield("test")
        end)
        -- resume once to make it suspended
        coroutine.resume(co)
        STORE.co_scene = co

        -- reset()を呼び出し
        STORE.reset()

        -- co_sceneがnilであること
        expect(STORE.co_scene):toBe(nil)
    end)

    test("reset()がsuspendedコルーチンをclose()する", function()
        setup()
        -- suspendedコルーチンを作成
        local co = coroutine.create(function()
            coroutine.yield("step1")
            coroutine.yield("step2")
        end)
        coroutine.resume(co) -- suspended状態に
        STORE.co_scene = co

        -- reset()前はsuspended
        expect(coroutine.status(co)):toBe("suspended")

        -- reset()を呼び出し
        STORE.reset()

        -- coroutine.close()後はdead状態
        expect(coroutine.status(co)):toBe("dead")
    end)

    test("reset()がdead状態のco_sceneでも安全に動作する", function()
        setup()
        -- deadコルーチンを作成
        local co = coroutine.create(function() end)
        coroutine.resume(co) -- dead状態に
        STORE.co_scene = co

        -- reset()を呼び出し（エラーが発生しないこと）
        local ok, err = pcall(function()
            STORE.reset()
        end)

        expect(ok):toBe(true)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("reset()がnilのco_sceneでも安全に動作する", function()
        setup()
        STORE.co_scene = nil

        -- reset()を呼び出し（エラーが発生しないこと）
        local ok, err = pcall(function()
            STORE.reset()
        end)

        expect(ok):toBe(true)
        expect(STORE.co_scene):toBe(nil)
    end)

    test("reset()が他のフィールドも正常にリセットする", function()
        setup()
        -- 他のフィールドを設定
        STORE.actors = { test = "actor" }
        STORE.scenes = { test = "scene" }

        -- reset()を呼び出し
        STORE.reset()

        -- 他のフィールドもリセットされていること
        expect(next(STORE.actors)):toBe(nil)
        expect(next(STORE.scenes)):toBe(nil)
    end)
end)

return true
