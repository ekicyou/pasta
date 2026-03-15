-- ACT_IMPL.call Level 4 actメソッドフォールバックテスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local GLOBAL = require("pasta.global")

-- モック用のCTXオブジェクトを作成
local function create_mock_ctx()
    return {
        actors = {},
        save = {},
        yield = function() end,
        end_action = function() end,
    }
end

-- Level 4: actメソッドフォールバック
describe("ACT_IMPL.call - Level 4 (act method fallback)", function()
    test("actメソッドがcall経由で呼び出せる", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        -- ACT_IMPLにテスト用メソッドを追加
        local called = false
        ACT.IMPL._test_method_fallback = function(a)
            called = true
            return "method_result"
        end

        local result = act:call("global", "_test_method_fallback", {})

        expect(called):toBe(true)
        expect(result):toBe("method_result")

        -- クリーンアップ
        ACT.IMPL._test_method_fallback = nil
    end)

    test("引数がactメソッドに正しく渡される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        local received = {}
        ACT.IMPL._test_method_args = function(a, arg1, arg2)
            received = { arg1, arg2 }
            return "args_ok"
        end

        local result = act:call("global", "_test_method_args", {}, "hello", 42)

        expect(result):toBe("args_ok")
        expect(received[1]):toBe("hello")
        expect(received[2]):toBe(42)

        -- クリーンアップ
        ACT.IMPL._test_method_args = nil
    end)

    test("non-function値（テーブル等）はフォールバック対象外", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        -- self[key]がテーブル（例: actorsプロキシ）の場合はスキップ
        -- varはテーブルなので"var"キーではメソッドフォールバックは発動しない
        local result = act:call("global", "var", {})

        expect(result):toBe(nil)
    end)
end)

-- 優先順位: Level 3 (GLOBAL) > Level 4 (actメソッド)
describe("ACT_IMPL.call - GLOBAL優先 over actメソッド", function()
    test("GLOBALとactメソッド両方に同名が存在する場合、GLOBALが優先", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        -- Level 3: GLOBAL
        GLOBAL._test_priority = function(a) return "from_global" end

        -- Level 4: actメソッド
        ACT.IMPL._test_priority = function(a) return "from_method" end

        local result = act:call("global", "_test_priority", {})

        expect(result):toBe("from_global")

        -- クリーンアップ
        GLOBAL._test_priority = nil
        ACT.IMPL._test_priority = nil
    end)

    test("GLOBALにない場合はactメソッドにフォールバック", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)

        ACT.IMPL._test_fallback_only = function(a) return "method_fallback" end

        local result = act:call("global", "_test_fallback_only", {})

        expect(result):toBe("method_fallback")

        -- クリーンアップ
        ACT.IMPL._test_fallback_only = nil
    end)
end)

-- SHIORI_ACT継承チェーンでのフォールバック
describe("ACT_IMPL.call - SHIORI_ACT method fallback", function()
    test("SHIORI_ACT_IMPLのメソッドがcall経由で呼び出せる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ACTOR = require("pasta.actor")

        local sakura = ACTOR.get_or_create("さくら")
        sakura.spot = "sakura"
        local actors = { sakura = sakura }

        local req = {
            reference = { [0] = "click_x", [1] = "click_y" },
            id = "OnMouseClick",
            base_id = "OnMouseClick",
        }
        local act = SHIORI_ACT.new(actors, req)

        -- transfer_req_to_varをcall経由で呼び出し
        act:call("global", "transfer_req_to_var", {})

        -- 転記結果を検証
        expect(act.var["ｒ０"]):toBe("click_x")
        expect(act.var["ｒ１"]):toBe("click_y")
        expect(act.var.r0):toBe("click_x")
        expect(act.var.r1):toBe("click_y")
        expect(act.var.req_id):toBe("OnMouseClick")
    end)
end)

print("  ✅ act_method_fallback tests defined")
