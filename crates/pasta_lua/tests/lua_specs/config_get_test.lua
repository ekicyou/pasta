-- pasta.config（PASTA_CONFIG.get）単体テスト
-- review-improvement-loop cell 3.46 (G1): 直接テストが存在しなかった設定取得ラッパーの挙動を固定する
--
-- pasta.config はロード時に @pasta_config を pcall require して束縛するため、
-- package.loaded を退避→差し替え→再ロード→復元のパターンでバックエンドを制御する。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- @pasta_config を stub に差し替えて pasta.config を新規ロードし、body を実行する。
--- 実行後は package.loaded を必ず元へ復元する（エラー時も復元してから再送出）。
--- @param stub table|nil @pasta_config の代替テーブル（nil = バックエンド不在環境を再現）
--- @param body fun(config: table)
local function with_fresh_config(stub, body)
    local saved_module = package.loaded["pasta.config"]
    local saved_backend = package.loaded["@pasta_config"]
    package.loaded["pasta.config"] = nil
    package.loaded["@pasta_config"] = stub

    local ok, err = pcall(function()
        local config = require("pasta.config")
        body(config)
    end)

    package.loaded["pasta.config"] = saved_module
    package.loaded["@pasta_config"] = saved_backend
    if not ok then error(err, 0) end
end

describe("pasta.config - get（バックエンドあり）", function()
    test("存在するセクション・キーの値を返す", function()
        with_fresh_config({ ghost = { spot_newlines = 2 } }, function(config)
            expect(config.get("ghost", "spot_newlines", 1.5)):toBe(2)
        end)
    end)

    test("false 値は nil ではないためデフォルトに置換されない", function()
        with_fresh_config({ ghost = { flag = false } }, function(config)
            expect(config.get("ghost", "flag", true)):toBe(false)
        end)
    end)

    test("キーが存在しない場合はデフォルト値を返す", function()
        with_fresh_config({ ghost = { spot_newlines = 2 } }, function(config)
            expect(config.get("ghost", "missing_key", "DEFAULT")):toBe("DEFAULT")
        end)
    end)

    test("セクションが存在しない場合はデフォルト値を返す", function()
        with_fresh_config({ ghost = {} }, function(config)
            expect(config.get("no_such_section", "key", 42)):toBe(42)
        end)
    end)

    test("デフォルト省略時の未発見は nil を返す", function()
        with_fresh_config({}, function(config)
            expect(config.get("ghost", "missing_key")):toBeNil()
        end)
    end)

    test("section が nil のときエラーを送出する", function()
        with_fresh_config({}, function(config)
            local ok, err = pcall(config.get, nil, "key", 1)
            expect(ok):toBe(false)
            expect(tostring(err)):toMatch("section is required")
        end)
    end)
end)

describe("pasta.config - get（@pasta_config 不在環境）", function()
    test("バックエンド不在時は空テーブルへフォールバックし常にデフォルト値を返す", function()
        with_fresh_config(nil, function(config)
            expect(config.get("ghost", "spot_newlines", 1.5)):toBe(1.5)
            expect(config.get("anything", "key")):toBeNil()
        end)
    end)
end)
