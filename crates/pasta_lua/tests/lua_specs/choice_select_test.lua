-- ============================================================================
-- Task 4.2: OnChoiceSelectEx 自動ルーティングハンドラテスト
-- Requirements: 3.1, 3.2, 3.3, 3.4, 3.5
-- ============================================================================

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("OnChoiceSelectEx auto-routing handler", function()
    local REG
    local SCENE
    local STORE
    local SHIORI_ACT
    local original_search

    local function setup()
        -- Reset modules
        package.loaded["pasta.shiori.event.register"] = nil
        package.loaded["pasta.shiori.event.choice_select"] = nil
        package.loaded["pasta.scene"] = nil
        package.loaded["pasta.store"] = nil
        package.loaded["pasta.shiori.act"] = nil
        package.loaded["pasta.act"] = nil

        STORE = require("pasta.store")
        SCENE = require("pasta.scene")
        SHIORI_ACT = require("pasta.shiori.act")
        REG = require("pasta.shiori.event.register")

        -- Save original before choice_select overwrites it
        original_search = SCENE.search

        -- Load choice_select handler (registers REG.OnChoiceSelectEx)
        require("pasta.shiori.event.choice_select")
    end

    local function teardown()
        if SCENE and original_search then
            SCENE.search = original_search
        end
    end

    local function make_actors()
        return { sakura = { name = "さくら", spot = 0 } }
    end

    local function make_req(choice_id, choice_label)
        return {
            id = "OnChoiceSelectEx",
            method = "get",
            version = 30,
            reference = {
                [0] = choice_id,
                [1] = choice_label or "label",
            },
        }
    end

    -- ================================================================
    -- 3.1: マッチ実行 — 選択IDがシーンにマッチしたら実行
    -- ================================================================
    test("returns thread when choice_id matches a scene (3.1)", function()
        setup()

        SCENE.search = function(name, global_scene_name, attrs)
            if name == "choice_greeting" then
                return {
                    func = function(act)
                        act.sakura:talk("こんにちは")
                    end,
                    global_name = "TestScene1",
                    local_name = "choice_greeting",
                }
            end
            return nil
        end

        local act = SHIORI_ACT.new(make_actors(), make_req("choice_greeting"))
        local result = REG.OnChoiceSelectEx(act)

        expect(type(result)):toBe("thread")
        expect(coroutine.status(result)):toBe("suspended")

        -- Resume to verify scene executes
        local ok, value = coroutine.resume(result, act)
        expect(ok):toBe(true)
        expect(type(value)):toBe("string")
        expect(value:find("こんにちは")):toBeTruthy()

        teardown()
    end)

    -- ================================================================
    -- 3.2: 複数マッチ → SCENE.search が正しく呼ばれている
    -- (シャッフル＆順次消費は SCENE.search/co_exec 内部処理)
    -- ================================================================
    test("delegates shuffle/consumption to SCENE.search (3.2)", function()
        setup()

        local search_called_with = {}
        SCENE.search = function(name, global_scene_name, attrs)
            table.insert(search_called_with, { name = name, global = global_scene_name })
            -- Explicit handler check should return nil
            if name == "OnChoiceSelectEx" then
                return nil
            end
            return {
                func = function(act)
                    act.sakura:talk("shuffled result")
                end,
                global_name = "TestScene1",
                local_name = "matched_scene",
            }
        end

        STORE.last_global_scene = "CurrentGlobal"
        local act = SHIORI_ACT.new(make_actors(), make_req("multi_match"))
        local result = REG.OnChoiceSelectEx(act)

        expect(type(result)):toBe("thread")

        -- Verify SCENE.search was called with choice_id and last_global_scene
        -- First call is for explicit handler check ("OnChoiceSelectEx", nil)
        -- Second call is for choice_id search
        expect(#search_called_with >= 1):toBe(true)
        local last_call = search_called_with[#search_called_with]
        expect(last_call.name):toBe("multi_match")
        expect(last_call.global):toBe("CurrentGlobal")

        teardown()
    end)

    -- ================================================================
    -- 3.3: 非マッチ時 → nil を返す（EVENT.fire が 204 に変換）
    -- ================================================================
    test("returns nil when no scene matches choice_id (3.3)", function()
        setup()

        SCENE.search = function(name, global_scene_name, attrs)
            return nil
        end

        local act = SHIORI_ACT.new(make_actors(), make_req("nonexistent_choice"))
        local result = REG.OnChoiceSelectEx(act)

        expect(result):toBe(nil)

        teardown()
    end)

    -- ================================================================
    -- 3.4: ローカル→グローバル検索 — STORE.last_global_scene をスコープに使用
    -- ================================================================
    test("passes STORE.last_global_scene to SCENE.search for scoped search (3.4)", function()
        setup()

        local captured_global_scope = nil
        SCENE.search = function(name, global_scene_name, attrs)
            -- Only capture for the choice_id search, not the explicit handler check
            if name ~= "OnChoiceSelectEx" then
                captured_global_scope = global_scene_name
            end
            return nil
        end

        STORE.last_global_scene = "MyGlobalScene1"
        local act = SHIORI_ACT.new(make_actors(), make_req("some_choice"))
        REG.OnChoiceSelectEx(act)

        expect(captured_global_scope):toBe("MyGlobalScene1")

        teardown()
    end)

    test("passes nil global scope when STORE.last_global_scene is nil (3.4)", function()
        setup()

        local captured_global_scope = "sentinel"
        SCENE.search = function(name, global_scene_name, attrs)
            if name ~= "OnChoiceSelectEx" then
                captured_global_scope = global_scene_name
            end
            return nil
        end

        STORE.last_global_scene = nil
        local act = SHIORI_ACT.new(make_actors(), make_req("some_choice"))
        REG.OnChoiceSelectEx(act)

        expect(captured_global_scope):toBe(nil)

        teardown()
    end)

    -- ================================================================
    -- 3.5: 明示的 OnChoiceSelectEx シーン優先
    -- ================================================================
    test("explicit OnChoiceSelectEx scene takes priority over auto-routing (3.5)", function()
        setup()

        local auto_route_searched = false
        SCENE.search = function(name, global_scene_name, attrs)
            if name == "OnChoiceSelectEx" then
                -- Explicit scene found
                return {
                    func = function(act)
                        act.sakura:talk("explicit handler")
                    end,
                    global_name = "ExplicitHandler1",
                    local_name = "__start__",
                }
            end
            -- This should NOT be reached if explicit handler is found
            auto_route_searched = true
            return nil
        end

        local act = SHIORI_ACT.new(make_actors(), make_req("choice_greeting"))
        local result = REG.OnChoiceSelectEx(act)

        expect(type(result)):toBe("thread")
        expect(auto_route_searched):toBe(false)

        -- Verify the explicit handler content
        local ok, value = coroutine.resume(result, act)
        expect(ok):toBe(true)
        expect(value:find("explicit handler")):toBeTruthy()

        teardown()
    end)

    test("falls through to auto-routing when no explicit scene exists (3.5)", function()
        setup()

        local choice_id_searched = false
        SCENE.search = function(name, global_scene_name, attrs)
            if name == "OnChoiceSelectEx" then
                return nil  -- No explicit scene
            end
            choice_id_searched = true
            return {
                func = function(act)
                    act.sakura:talk("auto-routed")
                end,
                global_name = "AutoRoute1",
                local_name = "choice_test",
            }
        end

        local act = SHIORI_ACT.new(make_actors(), make_req("choice_test"))
        local result = REG.OnChoiceSelectEx(act)

        expect(type(result)):toBe("thread")
        expect(choice_id_searched):toBe(true)

        teardown()
    end)

    -- ================================================================
    -- Edge cases: empty/nil choice_id → nil (204)
    -- ================================================================
    test("returns nil when choice_id is empty string", function()
        setup()

        SCENE.search = function(name, global_scene_name, attrs)
            return nil
        end

        local act = SHIORI_ACT.new(make_actors(), make_req(""))
        local result = REG.OnChoiceSelectEx(act)

        expect(result):toBe(nil)

        teardown()
    end)

    test("returns nil when choice_id is nil", function()
        setup()

        SCENE.search = function(name, global_scene_name, attrs)
            return nil
        end

        local act = SHIORI_ACT.new(make_actors(), make_req(nil))
        local result = REG.OnChoiceSelectEx(act)

        expect(result):toBe(nil)

        teardown()
    end)

    test("returns nil when reference table is missing", function()
        setup()

        SCENE.search = function(name, global_scene_name, attrs)
            return nil
        end

        local req = {
            id = "OnChoiceSelectEx",
            method = "get",
            version = 30,
            -- no reference table
        }
        local act = SHIORI_ACT.new(make_actors(), req)
        local result = REG.OnChoiceSelectEx(act)

        expect(result):toBe(nil)

        teardown()
    end)

    -- ================================================================
    -- Handler registration
    -- ================================================================
    test("handler is registered as REG.OnChoiceSelectEx", function()
        setup()

        expect(type(REG.OnChoiceSelectEx)):toBe("function")

        teardown()
    end)
end)
