-- ACT_IMPL.find_scene 5段階フォールバック検索テスト
-- Task 4: act:find_scene() 単体テスト
-- Requirements: 1.1, 1.3, 2.3, 2.4
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")

-- Task 4.1: Level 1 — current_scene ローカル検索
describe("act:find_scene - Level 1 (current_scene)", function()
    test("current_scene[key] から関数を取得できる", function()
        local act = ACT.new({})
        local handler = function() return "L1" end
        act.current_scene = { my_scene = handler }

        local result = act:find_scene("my_scene")
        expect(result):toBe(handler)
    end)

    test("current_scene が nil のとき L1 をスキップする", function()
        local act = ACT.new({})
        act.current_scene = nil

        local result = act:find_scene("my_scene")
        expect(result):toBe(nil)
    end)

    test("current_scene[key] が nil のとき次レベルへ進む", function()
        local act = ACT.new({})
        act.current_scene = { other = function() end }

        local result = act:find_scene("my_scene")
        expect(result):toBe(nil)
    end)
end)

-- Task 4.2: Level 2 — SCENE.search スコープ付き検索
describe("act:find_scene - Level 2 (SCENE.search scoped)", function()
    local original_search

    test("SCENE.search(key, scope) で見つかった関数を返す", function()
        original_search = SCENE.search
        local handler = function() return "L2" end
        SCENE.search = function(key, scope, attrs)
            if key == "target" and scope == "my_scope" then
                return { func = handler, global_name = "my_scope", local_name = "target" }
            end
            return nil
        end

        local act = ACT.new({})
        act.current_scene = {} -- L1 miss

        local result = act:find_scene("target", "my_scope")
        expect(result):toBe(handler)

        SCENE.search = original_search
    end)

    test("SCENE.search が nil を返したとき次レベルへ進む", function()
        original_search = SCENE.search
        SCENE.search = function() return nil end

        local act = ACT.new({})
        act.current_scene = {}

        local result = act:find_scene("target", "scope")
        expect(result):toBe(nil)

        SCENE.search = original_search
    end)
end)

-- Task 4.3: Level 3 — GLOBAL テーブル（本仕様の主要修正点）
-- Note: global_chaintalk_integration_test が package.loaded["pasta.global"] をリセットする
-- ため、act.lua 内の GLOBAL 参照と一致させるにはモジュール再読込が必要
describe("act:find_scene - Level 3 (GLOBAL)", function()
    test("GLOBAL[key] から関数を取得できる", function()
        -- モジュール同期: act.lua が参照する GLOBAL と同一テーブルを取得
        package.loaded["pasta.act"] = nil
        local fresh_ACT = require("pasta.act")
        local fresh_GLOBAL = require("pasta.global")

        local handler = function() return "L3" end
        fresh_GLOBAL.L3_test_scene = handler

        local act = fresh_ACT.new({})
        act.current_scene = {} -- L1 miss

        local result = act:find_scene("L3_test_scene")
        expect(result):toBe(handler)

        fresh_GLOBAL.L3_test_scene = nil
    end)

    test("L1・L2 が nil で GLOBAL に関数があるとき L3 が返る", function()
        package.loaded["pasta.act"] = nil
        local fresh_ACT = require("pasta.act")
        local fresh_GLOBAL = require("pasta.global")

        local handler = function() return "global_handler" end
        fresh_GLOBAL.L3_fallback_test = handler

        local act = fresh_ACT.new({})

        local result = act:find_scene("L3_fallback_test")
        expect(result):toBe(handler)

        fresh_GLOBAL.L3_fallback_test = nil
    end)
end)

-- Task 4.4: Level 5 — スコープなし全体検索
describe("act:find_scene - Level 5 (SCENE.search scopeless)", function()
    local original_search

    test("L1〜L4 全ミスで SCENE.search(key, nil) が呼ばれる", function()
        original_search = SCENE.search
        local handler = function() return "L5" end
        local search_calls = {}
        SCENE.search = function(key, scope, attrs)
            table.insert(search_calls, { key = key, scope = scope })
            -- L2 はスコープ付きで nil、L5 はスコープなしで返す
            if scope == nil then
                return { func = handler, global_name = "any", local_name = key }
            end
            return nil
        end

        local act = ACT.new({})
        act.current_scene = {} -- L1 miss
        -- L3: GLOBAL に未登録
        -- L4: act メソッドなし

        local result = act:find_scene("fallback_key", "some_scope")
        expect(result):toBe(handler)

        -- L2 と L5 の 2 回呼ばれていることを確認
        expect(#search_calls):toBe(2)
        expect(search_calls[1].scope):toBe("some_scope") -- L2
        expect(search_calls[2].scope):toBe(nil)          -- L5

        SCENE.search = original_search
    end)
end)

-- Task 4.5: 全レベル未発見で nil
describe("act:find_scene - all levels miss", function()
    local original_search

    test("すべてのレベルで未発見のとき nil を返す", function()
        original_search = SCENE.search
        SCENE.search = function() return nil end

        local act = ACT.new({})
        act.current_scene = {}

        local result = act:find_scene("nonexistent_key_xyz", "scope")
        expect(result):toBe(nil)

        SCENE.search = original_search
    end)

    test("エラーが発生しない", function()
        original_search = SCENE.search
        SCENE.search = function() return nil end

        local act = ACT.new({})
        local ok, err = pcall(function()
            act:find_scene("no_such_key")
        end)
        expect(ok):toBe(true)

        SCENE.search = original_search
    end)
end)

-- Task 4.6: find_scene は関数を返すが実行しない
describe("act:find_scene - returns function without executing", function()
    test("ハンドラーが実行されていないことを確認する", function()
        local executed = false
        local handler = function()
            executed = true
            return "should not run"
        end

        local act = ACT.new({})
        act.current_scene = { check_exec = handler }

        local result = act:find_scene("check_exec")
        expect(result):toBe(handler)
        expect(executed):toBe(false)
    end)
end)

-- Task 4.7: 優先順位検証（L1 > L3）
describe("act:find_scene - Priority", function()
    test("L1 と L3 の両方に同じキーがある場合 L1 が優先", function()
        package.loaded["pasta.act"] = nil
        local fresh_ACT = require("pasta.act")
        local fresh_GLOBAL = require("pasta.global")

        local l1_handler = function() return "L1" end
        local l3_handler = function() return "L3" end
        fresh_GLOBAL.priority_find_test = l3_handler

        local act = fresh_ACT.new({})
        act.current_scene = { priority_find_test = l1_handler }

        local result = act:find_scene("priority_find_test")
        expect(result):toBe(l1_handler)

        fresh_GLOBAL.priority_find_test = nil
    end)

    test("L1 が nil で L3 がある場合 L3 が使われる", function()
        package.loaded["pasta.act"] = nil
        local fresh_ACT = require("pasta.act")
        local fresh_GLOBAL = require("pasta.global")

        local l3_handler = function() return "L3" end
        fresh_GLOBAL.priority_find_test2 = l3_handler

        local act = fresh_ACT.new({})
        act.current_scene = {} -- L1 miss

        local result = act:find_scene("priority_find_test2")
        expect(result):toBe(l3_handler)

        fresh_GLOBAL.priority_find_test2 = nil
    end)
end)
