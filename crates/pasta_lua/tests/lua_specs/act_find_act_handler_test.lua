-- ACT_IMPL.find_act_handler / find_handler 単体テスト
-- Task 1.3: 各フォールバックレベル・mode切り替えを検証する
-- Requirements: 1.4, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10, 2.11
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local SCENE = require("pasta.scene")

-- pasta.act を再読込して新鮮なモジュールテーブルを返す
-- （act.lua が参照する GLOBAL / @pasta_search との同一性を保つための再読込規約）
local function reload_fresh_act()
    package.loaded["pasta.act"] = nil
    return require("pasta.act")
end

-- @pasta_search を mock（nil 指定で不在固定）へ差し替えたうえで pasta.act を再読込する
-- （package.loaded への差し替えは pasta.act 再読込より先に行う必要がある）
local function reload_act_with_search(mock_search)
    package.loaded["@pasta_search"] = mock_search
    return reload_fresh_act()
end

-- SCENE.search 側をモックする scene/expr モードテスト用の @pasta_search ダミー
-- （pcall require を成功させるためだけに存在し、検索自体は SCENE.search モックが担う）
local function dummy_search_module()
    return { search_scene = function() end, search_word = function() return nil end }
end

-- ============================================================================
-- L1: current_scene[key] 完全一致（全モード共通）
-- ============================================================================
describe("find_act_handler - L1 (current_scene 完全一致)", function()
    test("scene[key] に関数が設定されていれば返す（scene モード）", function()
        local act = ACT.new({})
        local handler = function() return "L1-scene" end
        act.current_scene = { my_fn = handler }

        local result = act:find_act_handler("scene", "my_fn")
        expect(result):toBe(handler)
    end)

    test("scene[key] に関数が設定されていれば返す（word モード）", function()
        local act = ACT.new({})
        local handler = function() return "L1-word" end
        act.current_scene = { greet = handler }

        local result = act:find_act_handler("word", "greet")
        expect(result):toBe(handler)
    end)

    test("scene[key] に関数が設定されていれば返す（expr モード）", function()
        local act = ACT.new({})
        local handler = function() return "L1-expr" end
        act.current_scene = { calc = handler }

        local result = act:find_act_handler("expr", "calc")
        expect(result):toBe(handler)
    end)

    test("scene[key] に文字列値が設定されていれば返す", function()
        local act = ACT.new({})
        act.current_scene = { my_val = "hello" }

        local result = act:find_act_handler("word", "my_val")
        expect(result):toBe("hello")
    end)

    test("current_scene が nil のときスキップして nil を返す", function()
        local act = ACT.new({})
        act.current_scene = nil

        local result = act:find_act_handler("scene", "any")
        expect(result):toBe(nil)
    end)

    test("current_scene[key] が nil のとき次レベルへ進む", function()
        local act = ACT.new({})
        act.current_scene = { other = function() end }

        local result = act:find_act_handler("scene", "absent")
        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- L2: ローカル辞書前方一致（word モード: search_word）
-- ============================================================================
describe("find_act_handler - L2 word モード (ローカル単語辞書前方一致)", function()
    test("@pasta_search があれば search_word(key, scene_name) の結果を返す", function()
        -- @pasta_search モック（package.loadedに設定でpcall require成功）
        local mock_search = {
            search_word = function(self, key, scope)
                if key == "greet" and scope == "テストシーン_1" then
                    return "こんにちは"
                end
                return nil
            end
        }
        local fresh_ACT = reload_act_with_search(mock_search)

        local act = fresh_ACT.new({})
        act.current_scene = { __global_name__ = "テストシーン_1" }

        local result = act:find_act_handler("word", "greet")
        expect(result):toBe("こんにちは")

        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)

    test("@pasta_search がなければ L2 をスキップして nil を返す", function()
        local fresh_ACT = reload_act_with_search(nil)

        local act = fresh_ACT.new({})
        act.current_scene = { __global_name__ = "テストシーン_1" }

        local result = act:find_act_handler("word", "missing")
        expect(result):toBe(nil)

        package.loaded["pasta.act"] = nil
    end)

    test("current_scene.__global_name__ がなければ L2 をスキップする（L5グローバルも返らない）", function()
        -- scope が nil でない（ローカルスコープ）呼び出しのみ値を返すモック
        -- → L2 スキップ検証: __global_name__ なしのとき scope 付き呼び出しされないことを確認
        -- L5（scope=nil）も返さないよう、すべて nil を返す
        local mock_search = {
            search_word = function(self, key, scope)
                -- ローカルスコープ呼び出しがあれば "called_local" を返す（L2 スキップの検証）
                if scope ~= nil then
                    return "should_not_be_called_with_local_scope"
                end
                return nil -- L5 グローバルも nil → 最終 nil を確認
            end
        }
        local fresh_ACT = reload_act_with_search(mock_search)

        local act = fresh_ACT.new({})
        act.current_scene = {} -- __global_name__ なし

        local result = act:find_act_handler("word", "greet")
        expect(result):toBe(nil)

        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)
end)

-- ============================================================================
-- L2: ローカル辞書前方一致（scene/expr モード: SCENE.search）
-- ============================================================================
describe("find_act_handler - L2 scene モード (ローカルシーン辞書前方一致)", function()
    local original_search

    test("@pasta_search があり SCENE.search がマッチすれば func を返す", function()
        original_search = SCENE.search
        local handler = function() return "L2-scene" end
        SCENE.search = function(key, scope, attrs)
            if key == "target" and scope == "テストシーン_1" then
                return { func = handler }
            end
            return nil
        end

        -- @pasta_search モックをpackage.loadedに設定することでpcall成功させる
        local fresh_ACT = reload_act_with_search(dummy_search_module())

        local act = fresh_ACT.new({})
        act.current_scene = { __global_name__ = "テストシーン_1" } -- L1 miss (no "target" key)

        local result = act:find_act_handler("scene", "target")
        expect(result):toBe(handler)

        SCENE.search = original_search
        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)

    test("expr モードでも L2 はシーン辞書前方一致を使う", function()
        original_search = SCENE.search
        local handler = function() return "L2-expr" end
        SCENE.search = function(key, scope, attrs)
            if key == "target" and scope == "テストシーン_1" then
                return { func = handler }
            end
            return nil
        end
        local fresh_ACT = reload_act_with_search(dummy_search_module())

        local act = fresh_ACT.new({})
        act.current_scene = { __global_name__ = "テストシーン_1" }

        local result = act:find_act_handler("expr", "target")
        expect(result):toBe(handler)

        SCENE.search = original_search
        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)
end)

-- ============================================================================
-- L3: ACT_IMPL メソッドフォールバック（function型のみ）
-- ============================================================================
describe("find_act_handler - L3 (act.XX メソッドフォールバック)", function()
    test("ACT.IMPL にある function が返る", function()
        local act = ACT.new({})
        act.current_scene = {} -- L1 miss

        local handler = function() return "L3_method" end
        ACT.IMPL._test_l3_handler = handler

        local result = act:find_act_handler("scene", "_test_l3_handler")
        expect(result):toBe(handler)

        ACT.IMPL._test_l3_handler = nil
    end)

    test("非 function 値（テーブル等）は L3 対象外", function()
        local act = ACT.new({})
        act.current_scene = {}

        ACT.IMPL._test_l3_table = { not_a_function = true }

        local result = act:find_act_handler("word", "_test_l3_table")
        expect(result):toBe(nil)

        ACT.IMPL._test_l3_table = nil
    end)

    test("word モードでも L3 は function 型を返す", function()
        local act = ACT.new({})
        act.current_scene = {}

        local handler = function() return "L3_word" end
        ACT.IMPL._test_l3_word = handler

        local result = act:find_act_handler("word", "_test_l3_word")
        expect(result):toBe(handler)

        ACT.IMPL._test_l3_word = nil
    end)
end)

-- ============================================================================
-- L4: GLOBAL[key] 完全一致（全モード共通）
-- ============================================================================
describe("find_act_handler - L4 (GLOBAL 完全一致)", function()
    test("GLOBAL[key] に関数が設定されていれば返す", function()
        -- モジュール同期: 同一GLOBALテーブルを参照
        local fresh_ACT = reload_fresh_act()
        local fresh_GLOBAL = require("pasta.global")

        local handler = function() return "L4" end
        fresh_GLOBAL._test_l4_fn = handler

        local act = fresh_ACT.new({})
        act.current_scene = {}

        local result = act:find_act_handler("scene", "_test_l4_fn")
        expect(result):toBe(handler)

        fresh_GLOBAL._test_l4_fn = nil
        package.loaded["pasta.act"] = nil
    end)

    test("word モードでも L4 は GLOBAL 完全一致を検索する", function()
        local fresh_ACT = reload_fresh_act()
        local fresh_GLOBAL = require("pasta.global")

        fresh_GLOBAL._test_l4_word_val = "L4val"

        local act = fresh_ACT.new({})
        act.current_scene = {}

        local result = act:find_act_handler("word", "_test_l4_word_val")
        expect(result):toBe("L4val")

        fresh_GLOBAL._test_l4_word_val = nil
        package.loaded["pasta.act"] = nil
    end)
end)

-- ============================================================================
-- L5: グローバル辞書前方一致（word モード: search_word(nil)）
-- ============================================================================
describe("find_act_handler - L5 word モード (グローバル単語辞書前方一致)", function()
    test("@pasta_search があれば search_word(key, nil) の結果を返す", function()
        local mock_search = {
            search_word = function(self, key, scope)
                if key == "global_word" and scope == nil then
                    return "グローバル語"
                end
                return nil
            end
        }
        local fresh_ACT = reload_act_with_search(mock_search)

        local act = fresh_ACT.new({})
        act.current_scene = {} -- L1 miss、__global_name__なしでL2スキップ

        local result = act:find_act_handler("word", "global_word")
        expect(result):toBe("グローバル語")

        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)
end)

-- ============================================================================
-- L5: グローバル辞書前方一致（scene/expr モード: SCENE.search(key, nil)）
-- ============================================================================
describe("find_act_handler - L5 scene モード (グローバルシーン辞書前方一致)", function()
    local original_search

    test("@pasta_search があり SCENE.search(key, nil) がマッチすれば func を返す", function()
        original_search = SCENE.search
        local handler = function() return "L5-scene" end
        SCENE.search = function(key, scope, attrs)
            if key == "global_scene" and scope == nil then
                return { func = handler }
            end
            return nil
        end
        local fresh_ACT = reload_act_with_search(dummy_search_module())

        local act = fresh_ACT.new({})
        act.current_scene = {} -- L1 miss、L2はL3-4もスキップ想定

        local result = act:find_act_handler("scene", "global_scene")
        expect(result):toBe(handler)

        SCENE.search = original_search
        package.loaded["@pasta_search"] = nil
        package.loaded["pasta.act"] = nil
    end)
end)

-- ============================================================================
-- nil return: いずれもマッチしない場合
-- ============================================================================
describe("find_act_handler - nil return", function()
    test("全レベルがミスしたとき nil を返す", function()
        local act = ACT.new({})
        act.current_scene = {}

        local result = act:find_act_handler("scene", "nonexistent_zzz")
        expect(result):toBe(nil)
    end)

    test("word モードでも全ミスのとき nil を返す", function()
        local act = ACT.new({})
        act.current_scene = {}

        local result = act:find_act_handler("word", "nonexistent_zzz")
        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- find_handler が find_act_handler への thin wrapper であること
-- ============================================================================
describe("ACT_IMPL.find_handler (thin wrapper)", function()
    test("find_handler('scene', key) が find_act_handler('scene', key) と同じ結果を返す", function()
        local act = ACT.new({})
        local handler = function() return "via_find_handler" end
        act.current_scene = { my_fn = handler }

        local result1 = act:find_handler("scene", "my_fn")
        local result2 = act:find_act_handler("scene", "my_fn")
        expect(result1):toBe(result2)
        expect(result1):toBe(handler)
    end)

    test("find_handler('word', key) が find_act_handler('word', key) と同じ結果を返す", function()
        local act = ACT.new({})
        act.current_scene = { val = "test_val" }

        local result1 = act:find_handler("word", "val")
        local result2 = act:find_act_handler("word", "val")
        expect(result1):toBe(result2)
        expect(result1):toBe("test_val")
    end)
end)

-- ============================================================================
-- フォールバック優先順位の確認
-- ============================================================================
describe("find_act_handler - フォールバック優先順位", function()
    test("L1（scene）は L3（act.XX）より優先される", function()
        local act = ACT.new({})

        local scene_handler = function() return "L1_winner" end
        act.current_scene = { _prio_test = scene_handler }

        local act_handler = function() return "L3_loser" end
        ACT.IMPL._prio_test = act_handler

        local result = act:find_act_handler("scene", "_prio_test")
        expect(result):toBe(scene_handler)

        ACT.IMPL._prio_test = nil
    end)

    test("L3（act.XX）は L4（GLOBAL）より優先される", function()
        local fresh_ACT = reload_fresh_act()
        local fresh_GLOBAL = require("pasta.global")

        local global_handler = function() return "L4_loser" end
        fresh_GLOBAL._prio_l3_l4 = global_handler

        local act_handler = function() return "L3_winner" end
        fresh_ACT.IMPL._prio_l3_l4 = act_handler -- fresh_ACT.IMPL を使う

        local act = fresh_ACT.new({})
        act.current_scene = {}

        local result = act:find_act_handler("scene", "_prio_l3_l4")
        expect(result):toBe(act_handler)

        fresh_GLOBAL._prio_l3_l4 = nil
        fresh_ACT.IMPL._prio_l3_l4 = nil
        package.loaded["pasta.act"] = nil
    end)
end)
