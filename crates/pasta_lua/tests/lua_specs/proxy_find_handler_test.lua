-- PROXY_IMPL.find_actor_handler / find_handler 単体テスト
-- Task 2.1, 2.2: find_actor_handler (word限定) と find_handler (委譲) を検証する
-- Requirements: 1.2, 1.3, 2.1, 2.2, 2.3
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local ACTOR = require("pasta.actor")

--- テスト用アクタープロキシを作成するヘルパー
local function make_proxy(actor_name)
    local actor = ACTOR.get_or_create(actor_name)
    local act = ACT.new({})
    act.current_scene = {}
    -- ACTOR.create_proxy を使う（actor.luaのPROXY_IMPL経由）
    -- act経由でプロキシ取得: act.{actor_name} → __index → create_proxy
    act.actors[actor_name] = actor
    return act[actor_name], act
end

-- ============================================================================
-- find_actor_handler: word 以外は nil を返す
-- ============================================================================
describe("PROXY_IMPL.find_actor_handler - word 以外は nil", function()
    test("mode が 'scene' のとき nil を返す", function()
        local proxy, act = make_proxy("テスト俳優_scene")

        local result = proxy:find_actor_handler("scene", "any_key")
        expect(result):toBe(nil)
    end)

    test("mode が 'expr' のとき nil を返す", function()
        local proxy, act = make_proxy("テスト俳優_expr")

        local result = proxy:find_actor_handler("expr", "any_key")
        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- find_actor_handler: A1 - proxy.actor[key] 完全一致（word モード）
-- ============================================================================
describe("PROXY_IMPL.find_actor_handler - A1 (proxy.actor[key] 完全一致)", function()
    test("proxy.actor[key] に値があれば返す", function()
        local proxy, act = make_proxy("テスト俳優_a1")
        proxy.actor["一人称"] = "私"

        local result = proxy:find_actor_handler("word", "一人称")
        expect(result):toBe("私")
    end)

    test("proxy.actor[key] に関数があれば返す", function()
        local proxy, act = make_proxy("テスト俳優_a1fn")
        local fn = function() return "計算値" end
        proxy.actor["fn_key"] = fn

        local result = proxy:find_actor_handler("word", "fn_key")
        expect(result):toBe(fn)
    end)

    test("proxy.actor[key] が nil のとき次レベルへ進む（@pasta_search なし → nil）", function()
        local proxy, act = make_proxy("テスト俳優_a1miss")

        local result = proxy:find_actor_handler("word", "nonexistent")
        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- find_actor_handler: A2 - アクター辞書前方一致（word モード）
-- ============================================================================
describe("PROXY_IMPL.find_actor_handler - A2 (アクター辞書前方一致)", function()
    test("@pasta_search があれば __actor_{name}__ スコープで search_word を呼ぶ", function()
        local searched_scope = nil
        local mock_search = {
            search_word = function(self, key, scope)
                searched_scope = scope
                if key == "呼び名" and scope == "__actor_テスト花子__" then
                    return "はなちゃん"
                end
                return nil
            end
        }
        package.loaded["@pasta_search"] = mock_search

        local proxy, act = make_proxy("テスト花子")

        local result = proxy:find_actor_handler("word", "呼び名")
        expect(result):toBe("はなちゃん")
        expect(searched_scope):toBe("__actor_テスト花子__")

        package.loaded["@pasta_search"] = nil
    end)

    test("@pasta_search がなければ A2 をスキップして nil を返す", function()
        package.loaded["@pasta_search"] = nil

        local proxy, act = make_proxy("テスト太郎_nosearch")

        local result = proxy:find_actor_handler("word", "呼び名")
        expect(result):toBe(nil)
    end)
end)

-- ============================================================================
-- PROXY_IMPL.find_handler: アクターレベル先行検索 → act委譲
-- ============================================================================
describe("PROXY_IMPL.find_handler - アクターレベル先行 → act委譲", function()
    test("word モード: actor[key] がマッチすれば即返す（act に委譲しない）", function()
        local proxy, act = make_proxy("テスト委譲_w")
        act.current_scene = { some_word = "from_scene" }
        proxy.actor["some_word"] = "from_actor" -- A1 win

        local result = proxy:find_handler("word", "some_word")
        expect(result):toBe("from_actor")
    end)

    test("word モード: actor がミスしたとき act:find_act_handler に委譲する", function()
        local proxy, act = make_proxy("テスト委譲_w2")
        act.current_scene = { scene_fn = function() return "from_act_L1" end }

        -- actor にはない
        local result = proxy:find_handler("word", "scene_fn")
        expect(result):toBeTruthy() -- act L1 から返ってくる
    end)

    test("scene モード: actor検索をスキップして act:find_act_handler に委譲する", function()
        local proxy, act = make_proxy("テスト委譲_s")
        local handler = function() return "scene_result" end
        act.current_scene = { scene_fn = handler }

        local result = proxy:find_handler("scene", "scene_fn")
        expect(result):toBe(handler)
    end)

    test("expr モード: actor検索をスキップして act:find_act_handler に委譲する", function()
        local proxy, act = make_proxy("テスト委譲_e")
        local handler = function() return "expr_result" end
        act.current_scene = { expr_fn = handler }

        local result = proxy:find_handler("expr", "expr_fn")
        expect(result):toBe(handler)
    end)

    test("全ミスのとき nil を返す", function()
        local proxy, act = make_proxy("テスト委譲_nil")
        act.current_scene = {}

        local result = proxy:find_handler("scene", "nonexistent_zzz")
        expect(result):toBe(nil)
    end)
end)
