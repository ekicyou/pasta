-- ACT:build() 早期リターンテスト
-- act-build-early-return 機能: トークン0件時のnilリターン検証
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = 0 },
        kero = { name = "うにゅう", spot = 1 },
    }
end

-- ============================================================================
-- R1: ACT:build()の早期リターン（撮影トークン0件時）
-- ============================================================================

describe("ACT:build() - 早期リターン（トークン0件時）", function()
    test("トークン0件時にnilを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        -- トークン未追加の状態でbuild()を呼び出し
        local result = act:build()

        expect(result):toBe(nil)
    end)

    test("nilリターン後もself.tokenが空テーブルにリセットされている", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        -- トークン未追加の状態でbuild()を呼び出し
        local _ = act:build()

        -- self.tokenは空テーブル{}であること
        expect(type(act.token)):toBe("table")
        expect(#act.token):toBe(0)
    end)

    test("トークン1件以上の場合は配列を返す（既存動作継続確認）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        -- トークンを1件追加
        act:talk(actors.sakura, "こんにちは")
        local result = act:build()

        -- nilではなく配列を返す
        expect(result ~= nil):toBe(true)
        expect(type(result)):toBe("table")
        expect(#result >= 1):toBe(true)
    end)

    test("2回連続build()で両方nilを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        -- 1回目: トークンなし
        local result1 = act:build()
        expect(result1):toBe(nil)

        -- 2回目: build()後もトークンなし
        local result2 = act:build()
        expect(result2):toBe(nil)
    end)

    test("トークン追加→build()→トークンなし→build()の流れで2回目はnil", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        -- 1回目: トークンあり
        act:talk(actors.sakura, "Hello")
        local result1 = act:build()
        expect(result1 ~= nil):toBe(true)

        -- 2回目: build()後はトークンリセットされているのでnil
        local result2 = act:build()
        expect(result2):toBe(nil)
    end)
end)

-- ============================================================================
-- R2: SHIORI_ACT:build()の早期リターン（ACT:build()がnil時）
-- ============================================================================

describe("SHIORI_ACT:build() - 早期リターン（トークン0件時）", function()
    test("トークン0件時にnilを返す", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- トークン未追加の状態でbuild()を呼び出し
        local result = act:build()

        expect(result):toBe(nil)
    end)

    test("トークン1件以上の場合はさくらスクリプト文字列を返す（既存動作継続確認）", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- トークンを1件追加
        act:talk(actors.sakura, "こんにちは")
        local result = act:build()

        -- nilではなく文字列を返す
        expect(result ~= nil):toBe(true)
        expect(type(result)):toBe("string")
        -- さくらスクリプト終端記号を含む
        expect(result:find("\\e")):toBeTruthy()
    end)

    test("2回連続build()で両方nilを返す", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- 1回目: トークンなし
        local result1 = act:build()
        expect(result1):toBe(nil)

        -- 2回目: build()後もトークンなし
        local result2 = act:build()
        expect(result2):toBe(nil)
    end)

    test("トークン追加→build()→トークンなし→build()の流れで2回目はnil", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- 1回目: トークンあり
        act:talk(actors.sakura, "Hello")
        local result1 = act:build()
        expect(result1 ~= nil):toBe(true)
        expect(type(result1)):toBe("string")

        -- 2回目: build()後はトークンリセットされているのでnil
        local result2 = act:build()
        expect(result2):toBe(nil)
    end)
end)
