-- ACT_IMPL.call 4段階検索テスト
-- Task 3: ACT_IMPL.call本体実装のユニットテスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")
local SCENE = require("pasta.scene")
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

-- Task 3.1: Level 1検索の単体テスト
describe("ACT_IMPL.call - Level 1 (current_scene)", function()
    test("current_scene[key]から正しくハンドラーを取得できる", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- current_sceneにハンドラーを設定
        local called = false
        local received_act = nil
        act.current_scene = {
            test_handler = function(a)
                called = true
                received_act = a
                return "level1_result"
            end
        }
        
        local result = act:call("global", "test_handler", {})
        
        expect(called):toBe(true)
        expect(received_act):toBe(act)
        expect(result):toBe("level1_result")
    end)
    
    test("current_scene == nil の場合、Level 2へスキップ（nil安全性）", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- current_sceneはnilのまま
        expect(act.current_scene):toBe(nil)
        
        -- エラーなく動作することを確認（ハンドラー未発見でnilを返す）
        local result = act:call("global", "nonexistent_key", {})
        expect(result):toBe(nil)
    end)
    
    test("current_scene[key] == nil の場合、Level 2へ進む", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- current_sceneは存在するが、keyは存在しない
        act.current_scene = {
            other_key = function() return "other" end
        }
        
        -- ハンドラー未発見でnilを返す
        local result = act:call("global", "nonexistent_key", {})
        expect(result):toBe(nil)
    end)
end)

-- Task 3.3: Level 3検索の単体テスト
describe("ACT_IMPL.call - Level 3 (pasta.global)", function()
    test("pasta.global[key]から正しくハンドラーを取得できる", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- GLOBALにハンドラーを設定
        local called = false
        GLOBAL.global_test_handler = function(a)
            called = true
            return "level3_result"
        end
        
        -- current_sceneにはハンドラーなし
        act.current_scene = {}
        
        local result = act:call("global", "global_test_handler", {})
        
        expect(called):toBe(true)
        expect(result):toBe("level3_result")
        
        -- クリーンアップ
        GLOBAL.global_test_handler = nil
    end)
    
    test("Level 1, 2がnilの場合にLevel 3が呼ばれる", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- Level 1: nil
        act.current_scene = nil
        
        -- Level 2: SCENE.searchはモックされていないのでnilを返す
        
        -- Level 3: GLOBALにハンドラーを設定
        local level3_called = false
        GLOBAL.level3_test = function(a)
            level3_called = true
            return "from_level3"
        end
        
        local result = act:call("global", "level3_test", {})
        
        expect(level3_called):toBe(true)
        expect(result):toBe("from_level3")
        
        -- クリーンアップ
        GLOBAL.level3_test = nil
    end)
end)

-- Task 3.5: 優先順位検証テスト
describe("ACT_IMPL.call - Priority", function()
    test("複数レベルに同じキーが存在する場合、Level 1が優先される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- Level 1
        act.current_scene = {
            priority_test = function(a) return "level1" end
        }
        
        -- Level 3
        GLOBAL.priority_test = function(a) return "level3" end
        
        local result = act:call("global", "priority_test", {})
        
        expect(result):toBe("level1")
        
        -- クリーンアップ
        GLOBAL.priority_test = nil
    end)
    
    test("Level 1がnilでLevel 3が存在する場合、Level 3が使われる", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- Level 1: nilを返す
        act.current_scene = {}
        
        -- Level 3
        GLOBAL.fallback_test = function(a) return "level3_fallback" end
        
        local result = act:call("global", "fallback_test", {})
        
        expect(result):toBe("level3_fallback")
        
        -- クリーンアップ
        GLOBAL.fallback_test = nil
    end)
end)

-- Task 3.6: ハンドラー未発見時の動作テスト
describe("ACT_IMPL.call - Handler Not Found", function()
    test("すべてのレベルでnilの場合、nilを返却する", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- すべてのレベルでハンドラーなし
        act.current_scene = nil
        
        local result = act:call("global", "completely_nonexistent_handler_xyz", {})
        
        expect(result):toBe(nil)
    end)
    
    test("エラーが発生しない（サイレント動作）", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        -- エラーなく実行完了することを確認
        local success, err = pcall(function()
            act:call("global", "nonexistent", {})
        end)
        
        expect(success):toBe(true)
    end)
end)

-- Task 3.7: ハンドラー実行と戻り値のテスト
describe("ACT_IMPL.call - Handler Execution", function()
    test("可変長引数が正しくハンドラーに渡される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        local received_args = {}
        act.current_scene = {
            varargs_test = function(a, arg1, arg2, arg3)
                received_args = {arg1, arg2, arg3}
                return "varargs_ok"
            end
        }
        
        local result = act:call("global", "varargs_test", {}, "first", "second", "third")
        
        expect(result):toBe("varargs_ok")
        expect(received_args[1]):toBe("first")
        expect(received_args[2]):toBe("second")
        expect(received_args[3]):toBe("third")
    end)
    
    test("ハンドラーの戻り値が正しく返却される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        act.current_scene = {
            return_test = function(a)
                return { status = "ok", value = 42 }
            end
        }
        
        local result = act:call("global", "return_test", {})
        
        expect(type(result)):toBe("table")
        expect(result.status):toBe("ok")
        expect(result.value):toBe(42)
    end)
    
    test("ハンドラーがnilを返した場合、nilが正しく返却される", function()
        local ctx = create_mock_ctx()
        local act = ACT.new(ctx)
        
        act.current_scene = {
            nil_return_test = function(a)
                return nil
            end
        }
        
        local result = act:call("global", "nil_return_test", {})
        
        expect(result):toBe(nil)
    end)
end)

print("  ✅ ACT_IMPL.call tests defined")
