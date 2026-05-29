-- act:choice / act:choice_timeout トークン蓄積テスト
-- choice-definition-dsl Task 3.2: 選択肢トークン蓄積メソッド
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = 0 },
    }
end

-- ============================================================================
-- act:choice
-- ============================================================================

describe("ACT - choice()", function()
    test("target と display を指定して構造化トークンを蓄積する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("target_scene", "表示テキスト")

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("choice")
        expect(act.token[1].target):toBe("target_scene")
        expect(act.token[1].display):toBe("表示テキスト")
    end)

    test("display が nil の場合 target を display として使用する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("target_scene", nil)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("choice")
        expect(act.token[1].target):toBe("target_scene")
        expect(act.token[1].display):toBe("target_scene")
    end)

    test("display 省略時も target を display として使用する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("some_target")

        expect(act.token[1].display):toBe("some_target")
    end)

    test("さくらスクリプト文字列を含まず構造化データのみ保持する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("jump_here", "選んでね")

        local token = act.token[1]
        -- type, target, display 以外のフィールドが存在しないことを確認
        local field_count = 0
        for _ in pairs(token) do field_count = field_count + 1 end
        expect(field_count):toBe(3)
        -- さくらスクリプトタグ（\q[...]等）が含まれていないことを確認
        expect(type(token.target)):toBe("string")
        expect(type(token.display)):toBe("string")
    end)

    test("メソッドチェーン用に self を返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:choice("target", "display")

        expect(returned):toBe(act)
    end)

    test("複数回呼び出しでトークンが順番に蓄積される", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("t1", "d1")
        act:choice("t2", "d2")

        expect(#act.token):toBe(2)
        expect(act.token[1].target):toBe("t1")
        expect(act.token[2].target):toBe("t2")
    end)
end)

-- ============================================================================
-- act:choice_timeout
-- ============================================================================

describe("ACT - choice_timeout()", function()
    test("秒数を指定して構造化トークンを蓄積する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice_timeout(5)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("choice_timeout")
        expect(act.token[1].seconds):toBe(5)
    end)

    test("seconds が nil の場合はそのまま nil を保持する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice_timeout(nil)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("choice_timeout")
        expect(act.token[1].seconds):toBe(nil)
    end)

    test("さくらスクリプト文字列を含まず構造化データのみ保持する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice_timeout(10)

        local token = act.token[1]
        local field_count = 0
        for _ in pairs(token) do field_count = field_count + 1 end
        expect(field_count):toBe(2) -- type, seconds のみ
    end)

    test("メソッドチェーン用に self を返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:choice_timeout(5)

        expect(returned):toBe(act)
    end)
end)

-- ============================================================================
-- choice + choice_timeout 混合
-- ============================================================================

describe("ACT - choice と choice_timeout の混合", function()
    test("choice と choice_timeout を混合して蓄積できる", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:choice("option_a", "選択肢A")
        act:choice("option_b", "選択肢B")
        act:choice_timeout(10)

        expect(#act.token):toBe(3)
        expect(act.token[1].type):toBe("choice")
        expect(act.token[2].type):toBe("choice")
        expect(act.token[3].type):toBe("choice_timeout")
    end)
end)
