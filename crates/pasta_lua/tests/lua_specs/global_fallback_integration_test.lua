-- ============================================================================
-- GLOBAL フォールバック統合テスト & DSL+GLOBAL 優先順位テスト
-- Task 5.1: GLOBAL フォールバック統合テスト
-- Task 5.2: DSL + GLOBAL 共存時の優先順位テスト
-- 検証Gap対応: OnTalk GLOBAL fallback 統合テスト (Req 2.2)
-- 検証Gap対応: act:build() 両経路明示テスト (Req 4.3)
-- Requirements: 1.1, 2.2, 2.3, 2.4, 4.3
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- Task 5.1: GLOBAL フォールバック統合テスト
-- DSL ラベル未定義の状態で GLOBAL 関数が EVENT.fire 経由で呼ばれることを確認
-- ============================================================================

describe("Integration - GLOBAL fallback via EVENT.fire", function()
    local EVENT
    local STORE
    local GLOBAL

    local function setup()
        -- 全モジュールリセット（GLOBAL 同期のため pasta.act も含む）
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.global"] = nil
        package.loaded["pasta.scene"] = nil

        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        GLOBAL = require("pasta.global")

        -- アクター設定
        STORE.actors = { sakura = { name = "さくら", spot = 0 } }
        STORE.co_scene = nil
    end

    test("GLOBAL.OnHourOther が EVENT.fire 経由で呼ばれる（DSL ラベルなし）", function()
        setup()

        local global_called = false
        GLOBAL.OnHourOther = function(act)
            global_called = true
            act:talk(act.actors.sakura, "正午です")
        end

        local response = EVENT.fire({ id = "OnHourOther", method = "get", version = 30 })

        expect(global_called):toBe(true)
        -- RES.ok が返されている（204 ではない）
        expect(response:find("SHIORI/3.0 200 OK")).not_:toBe(nil)
        expect(response:find("正午です")).not_:toBe(nil)

        GLOBAL.OnHourOther = nil
    end)

    test("GLOBAL に未登録のイベントは 204 No Content を返す", function()
        setup()

        local response = EVENT.fire({ id = "OnUnknownGlobalEvent", method = "get", version = 30 })

        -- 204 No Content
        expect(response:find("204 No Content")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Task 5.2: DSL ラベルと GLOBAL 共存時の優先順位テスト
-- SCENE.search (L2) が GLOBAL (L3) より先に解決されることを確認
-- ============================================================================

describe("Integration - DSL scene priority over GLOBAL", function()
    local EVENT
    local STORE
    local SCENE
    local GLOBAL

    local function setup()
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.global"] = nil
        package.loaded["pasta.scene"] = nil

        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        SCENE = require("pasta.scene")
        GLOBAL = require("pasta.global")

        STORE.actors = { sakura = { name = "さくら", spot = 0 } }
        STORE.co_scene = nil
    end

    test("DSL ラベルと GLOBAL の両方がある場合、DSL が優先される", function()
        setup()

        -- L3: GLOBAL に登録
        local global_called = false
        GLOBAL.OnPriorityTest = function(act)
            global_called = true
            act:talk(act.actors.sakura, "GLOBAL版")
        end

        -- L2: SCENE.search をモックして DSL シーンを返す
        local dsl_called = false
        local dsl_scene_fn = function(act)
            dsl_called = true
            act:talk(act.actors.sakura, "DSL版")
        end
        local original_search = SCENE.search
        SCENE.search = function(name, scope, attrs)
            if name == "OnPriorityTest" then
                return {
                    func = dsl_scene_fn,
                    global_name = "talk",
                    local_name = "OnPriorityTest",
                }
            end
            return nil
        end

        local response = EVENT.fire({ id = "OnPriorityTest", method = "get", version = 30 })

        -- DSL が呼ばれ、GLOBAL は呼ばれない
        expect(dsl_called):toBe(true)
        expect(global_called):toBe(false)
        expect(response:find("DSL版")).not_:toBe(nil)

        SCENE.search = original_search
        GLOBAL.OnPriorityTest = nil
    end)

    test("DSL が nil で GLOBAL がある場合、GLOBAL にフォールバックする", function()
        setup()

        -- L2: SCENE.search は nil を返す
        local original_search = SCENE.search
        SCENE.search = function() return nil end

        -- L3: GLOBAL に登録
        local global_called = false
        GLOBAL.OnFallbackTest = function(act)
            global_called = true
            act:talk(act.actors.sakura, "GLOBALフォールバック")
        end

        local response = EVENT.fire({ id = "OnFallbackTest", method = "get", version = 30 })

        expect(global_called):toBe(true)
        expect(response:find("GLOBALフォールバック")).not_:toBe(nil)

        SCENE.search = original_search
        GLOBAL.OnFallbackTest = nil
    end)
end)

-- ============================================================================
-- 検証Gap対応: OnTalk GLOBAL fallback 統合テスト (Req 2.2)
-- OnHour と同様に OnTalk も find_scene 経由で GLOBAL にフォールバックすることを確認
-- ============================================================================

describe("Integration - GLOBAL fallback for OnTalk (Req 2.2)", function()
    local EVENT
    local STORE
    local GLOBAL

    local function setup()
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.global"] = nil
        package.loaded["pasta.scene"] = nil

        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        GLOBAL = require("pasta.global")

        STORE.actors = { sakura = { name = "さくら", spot = 0 } }
        STORE.co_scene = nil
    end

    test("GLOBAL.OnTalk が EVENT.fire 経由で呼ばれる（DSL ラベルなし）", function()
        setup()

        local global_called = false
        GLOBAL.OnTalk = function(act)
            global_called = true
            act:talk(act.actors.sakura, "ランダムトーク")
        end

        local response = EVENT.fire({ id = "OnTalk", method = "get", version = 30 })

        expect(global_called):toBe(true)
        expect(response:find("SHIORI/3.0 200 OK")).not_:toBe(nil)
        expect(response:find("ランダムトーク")).not_:toBe(nil)

        GLOBAL.OnTalk = nil
    end)
end)

-- ============================================================================
-- 検証Gap対応: act:build() 両経路明示テスト (Req 4.3)
-- act:call() 経由と SCENE.co_exec() (EVENT.fire) 経由の両方で
-- act:build() がさくらスクリプトを正しく構築することを確認
-- ============================================================================

describe("Integration - act:build() works via both call and co_exec (Req 4.3)", function()
    local EVENT
    local STORE
    local GLOBAL
    local SHIORI_ACT

    local function setup()
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.global"] = nil
        package.loaded["pasta.scene"] = nil

        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        GLOBAL = require("pasta.global")
        SHIORI_ACT = require("pasta.shiori.act")

        STORE.actors = { sakura = { name = "さくら", spot = 0 } }
        STORE.co_scene = nil
    end

    test("EVENT.fire 経由（co_exec）で act:build() がさくらスクリプトを構築する", function()
        setup()

        GLOBAL.OnBuildTestCoExec = function(act)
            act:talk(act.actors.sakura, "co_exec経由ビルド")
        end

        local response = EVENT.fire({ id = "OnBuildTestCoExec", method = "get", version = 30 })

        -- act:build() がさくらスクリプトを構築していることを確認
        expect(response:find("200 OK")).not_:toBe(nil)
        expect(response:find("co_exec経由ビルド")).not_:toBe(nil)

        GLOBAL.OnBuildTestCoExec = nil
    end)

    test("act:call() 経由で act:build() がさくらスクリプトを構築する", function()
        setup()

        -- GLOBAL にハンドラを登録し act:call() 経由で呼び出す
        GLOBAL.OnBuildTestCall = function(act)
            act:talk(act.actors.sakura, "call経由ビルド")
        end

        -- SHIORI_ACT で生成（build() がさくらスクリプト文字列を返す）
        local act = SHIORI_ACT.new(STORE.actors)

        -- act:call() で GLOBAL ハンドラを呼び出す（find_scene L3）
        act:call(nil, "OnBuildTestCall", nil)

        -- act:build() でさくらスクリプト文字列を取得
        local result = act:build()

        expect(result).not_:toBe(nil)
        expect(result:find("call経由ビルド")).not_:toBe(nil)

        GLOBAL.OnBuildTestCall = nil
    end)
end)
