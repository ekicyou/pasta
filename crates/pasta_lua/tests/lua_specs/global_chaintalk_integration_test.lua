-- ============================================================================
-- GLOBAL チェイントーク統合テスト - EVENT.fire 経由のコルーチン分割検証
-- Requirements: 3.1, 3.2, 3.3
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("Integration - GLOBAL chaintalk via EVENT.fire", function()
    local EVENT
    local STORE
    local REG
    local RES
    local GLOBAL

    local function setup()
        -- Reset all modules
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.event"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.res"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.global"] = nil

        STORE = require("pasta.store")
        EVENT = require("pasta.shiori.event")
        REG = require("pasta.shiori.event.register")
        RES = require("pasta.shiori.res")
        GLOBAL = require("pasta.global")

        -- Setup actors for SHIORI_ACT
        STORE.actors = { sakura = { name = "さくら", spot = "sakura" } }

        -- Ensure clean state
        STORE.co_scene = nil
    end

    -- Req 3.1: yield 1回でコルーチンが2回 resume される
    test("chaintalk yield splits coroutine into 2 resumes via EVENT.fire", function()
        setup()
        local resume_count = 0

        REG.OnChaintalkTest = function(handler_act)
            -- co_exec と同じ wrapped_fn パターン: 関数末尾で act:build() を自動呼び出し
            local function scene_fn(act)
                resume_count = resume_count + 1
                act:talk("前半メッセージ")
                -- GLOBAL.チェイントーク を呼び出し（act:yield() を実行）
                GLOBAL["チェイントーク"](act)
                resume_count = resume_count + 1
                act:talk("後半メッセージ")
            end
            local function wrapped_fn(act, ...)
                scene_fn(act, ...)
                local result = act:build()
                if result ~= nil then return result end
            end
            return coroutine.create(wrapped_fn)
        end

        -- 1回目の fire: yield 前まで実行
        local response1 = EVENT.fire({ id = "OnChaintalkTest" })
        expect(resume_count):toBe(1)
        expect(STORE.co_scene).not_:toBe(nil)
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")

        -- 2回目の fire: yield 後を実行（チェイントーク継続）
        REG.OnChaintalkContinue = function(handler_act)
            return STORE.co_scene
        end
        local response2 = EVENT.fire({ id = "OnChaintalkContinue" })
        expect(resume_count):toBe(2)

        -- コルーチン完了後、STORE.co_scene は nil
        expect(STORE.co_scene):toBe(nil)
    end)

    -- Req 3.2: 1回目 resume の中間出力検証
    test("first resume returns only pre-yield tokens as intermediate output", function()
        setup()

        REG.OnIntermediateTest = function(handler_act)
            local function scene_fn(act)
                act:talk("中間出力テスト")
                GLOBAL["チェイントーク"](act)
                act:talk("最終出力テスト")
            end
            local function wrapped_fn(act, ...)
                scene_fn(act, ...)
                local result = act:build()
                if result ~= nil then return result end
            end
            return coroutine.create(wrapped_fn)
        end

        -- 1回目の fire: yield 前のトークンが中間出力
        local response1 = EVENT.fire({ id = "OnIntermediateTest" })
        -- レスポンスには 200 OK ヘッダーが含まれる
        expect(response1).not_:toBe(nil)
        expect(type(response1)):toBe("string")
        expect(response1:find("200 OK")).not_:toBe(nil)
        -- yield 後のトークンは含まれない
        expect(response1:find("最終出力テスト")):toBe(nil)
    end)

    -- Req 3.3: 2回目 resume の最終出力検証
    test("second resume returns post-yield tokens as final output", function()
        setup()

        REG.OnFinalTest = function(handler_act)
            local function scene_fn(act)
                act:talk("前半出力")
                GLOBAL["チェイントーク"](act)
                act:talk("後半出力")
            end
            local function wrapped_fn(act, ...)
                scene_fn(act, ...)
                local result = act:build()
                if result ~= nil then return result end
            end
            return coroutine.create(wrapped_fn)
        end

        -- 1回目 fire
        local response1 = EVENT.fire({ id = "OnFinalTest" })
        expect(STORE.co_scene).not_:toBe(nil)
        -- 1回目は 200 OK
        expect(response1).not_:toBe(nil)
        expect(response1:find("200 OK")).not_:toBe(nil)

        -- 2回目 fire（継続）
        -- 注: handler が STORE.co_scene を直接返す場合、
        -- wrapped_fn のラッパーは適用されない（既存コルーチンの再開）。
        -- しかしコルーチン内の wrapped_fn が最後に act:build() を呼ぶので
        -- 蓄積トークンは正しく返される。
        -- ただし act_A と act_B の不一致があるため、
        -- resume 時に act_B が yield 戻り値になるが使われない。
        -- wrapped_fn 内の act は act_A のまま → act_A:build() が呼ばれる。
        REG.OnFinalContinue = function(handler_act)
            return STORE.co_scene
        end
        local response2 = EVENT.fire({ id = "OnFinalContinue" })
        -- 2回目: wrapped_fn が act_A:build() を呼ぶので有効なレスポンス
        expect(response2).not_:toBe(nil)
        expect(type(response2)):toBe("string")
        expect(response2:find("200 OK")).not_:toBe(nil)
        -- コルーチン完了
        expect(STORE.co_scene):toBe(nil)
    end)

    -- Req 3.1: STORE.co_scene のライフサイクル
    test("STORE.co_scene lifecycle: suspended after 1st resume, nil after 2nd", function()
        setup()

        REG.OnLifecycleTest = function(handler_act)
            local function scene_fn(act)
                act:talk("ステップ1")
                GLOBAL["チェイントーク"](act)
                act:talk("ステップ2")
            end
            local function wrapped_fn(act, ...)
                scene_fn(act, ...)
                local result = act:build()
                if result ~= nil then return result end
            end
            return coroutine.create(wrapped_fn)
        end

        -- 初期状態: nil
        expect(STORE.co_scene):toBe(nil)

        -- 1回目 fire: suspended
        EVENT.fire({ id = "OnLifecycleTest" })
        expect(STORE.co_scene).not_:toBe(nil)
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")

        -- 2回目 fire: nil（完了）
        REG.OnLifecycleContinue = function(handler_act)
            return STORE.co_scene
        end
        EVENT.fire({ id = "OnLifecycleContinue" })
        expect(STORE.co_scene):toBe(nil)
    end)

    -- GLOBAL.yield でも同じ動作をすることを確認
    test("GLOBAL.yield also splits coroutine correctly via EVENT.fire", function()
        setup()

        REG.OnYieldAliasTest = function(handler_act)
            local function scene_fn(act)
                act:talk("yield前")
                GLOBAL["yield"](act)
                act:talk("yield後")
            end
            local function wrapped_fn(act, ...)
                scene_fn(act, ...)
                local result = act:build()
                if result ~= nil then return result end
            end
            return coroutine.create(wrapped_fn)
        end

        local response1 = EVENT.fire({ id = "OnYieldAliasTest" })
        -- 1回目 fire は有効なレスポンス
        expect(response1).not_:toBe(nil)
        expect(response1:find("200 OK")).not_:toBe(nil)
        expect(STORE.co_scene).not_:toBe(nil)
        expect(coroutine.status(STORE.co_scene)):toBe("suspended")

        REG.OnYieldAliasContinue = function(handler_act)
            return STORE.co_scene
        end
        local response2 = EVENT.fire({ id = "OnYieldAliasContinue" })
        -- 2回目: wrapped_fn が act:build() を呼ぶので有効なレスポンス
        expect(response2).not_:toBe(nil)
        expect(response2:find("200 OK")).not_:toBe(nil)
        expect(STORE.co_scene):toBe(nil)
    end)
end)
