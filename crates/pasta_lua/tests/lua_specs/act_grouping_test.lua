-- pasta.act grouping module tests
-- Tests for group_by_actor() and merge_consecutive_talks() functions
-- グループ化機能の単体テストを実装
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
-- 注: ACTのactorsマップはアクター名をキーとして使用
local function create_mock_actors()
    return {
        ["さくら"] = { name = "さくら", spot = 0 },
        ["うにゅう"] = { name = "うにゅう", spot = 1 },
    }
end

-- ============================================================================
-- Phase 1: group_by_actor() テスト
-- Requirement 1.1: アクター切り替え境界でのグループ化
-- ============================================================================

describe("ACT.build() - group_by_actor()", function()
    test("トークン0件でnilを返す (act-build-early-return)", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        -- トークンなしでbuild
        local result = act:build()

        expect(result):toBe(nil)
    end)

    test("単一talkで単一type=actorトークンを生成する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "こんにちは")
        local result = act:build()

        expect(#result):toBe(1)
        expect(result[1].type):toBe("actor")
        expect(result[1].actor):toBe(sakura)
        expect(#result[1].tokens):toBe(1)
        expect(result[1].tokens[1].type):toBe("talk")
        expect(result[1].tokens[1].text):toBe("こんにちは")
    end)

    test("spotトークンは独立トークンとして出力する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local kero = actors["うにゅう"]
        local act = ACT.new(actors)

        act:set_spot("うにゅう", 1)
        act:talk(sakura, "テスト")
        local result = act:build()

        expect(#result):toBe(2)
        expect(result[1].type):toBe("spot")
        expect(result[1].actor):toBe(kero)
        expect(result[1].spot):toBe(1)
        expect(result[2].type):toBe("actor")
    end)

    test("clear_spotトークンは独立トークンとして出力する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:clear_spot()
        act:talk(sakura, "テスト")
        local result = act:build()

        expect(#result):toBe(2)
        expect(result[1].type):toBe("clear_spot")
        expect(result[2].type):toBe("actor")
    end)

    test("同一actorの連続talkは同一type=actorトークン内に格納する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "今日は")
        act:talk(sakura, "晴れ")
        act:talk(sakura, "でした")
        local result = act:build()

        expect(#result):toBe(1)
        expect(result[1].type):toBe("actor")
        expect(result[1].actor):toBe(sakura)
        -- merge_consecutive_talks()により統合される
        expect(#result[1].tokens):toBe(1)
        expect(result[1].tokens[1].text):toBe("今日は晴れでした")
    end)

    test("actorが変化したら新しいtype=actorトークンを開始する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local kero = actors["うにゅう"]
        local act = ACT.new(actors)

        act:talk(sakura, "こんにちは")
        act:talk(kero, "やあ")
        local result = act:build()

        expect(#result):toBe(2)
        expect(result[1].type):toBe("actor")
        expect(result[1].actor):toBe(sakura)
        expect(result[2].type):toBe("actor")
        expect(result[2].actor):toBe(kero)
    end)

    test("断続的に同じactorが話しても別グループになる", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local kero = actors["うにゅう"]
        local act = ACT.new(actors)

        act:talk(sakura, "最初")
        act:talk(kero, "途中")
        act:talk(sakura, "最後")
        local result = act:build()

        expect(#result):toBe(3)
        expect(result[1].actor):toBe(sakura)
        expect(result[2].actor):toBe(kero)
        expect(result[3].actor):toBe(sakura)
    end)

    test("トークン順序を保持する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local kero = actors["うにゅう"]
        local act = ACT.new(actors)

        act:set_spot("うにゅう", 1)
        act:talk(sakura, "A")
        act:clear_spot()
        act:talk(kero, "B")
        local result = act:build()

        expect(#result):toBe(4)
        expect(result[1].type):toBe("spot")
        expect(result[2].type):toBe("actor")
        expect(result[2].tokens[1].text):toBe("A")
        expect(result[3].type):toBe("clear_spot")
        expect(result[4].type):toBe("actor")
        expect(result[4].tokens[1].text):toBe("B")
    end)
end)

-- ============================================================================
-- Phase 2: merge_consecutive_talks() テスト
-- Requirement 1.2: 連続talkの統合
-- ============================================================================

describe("ACT.build() - merge_consecutive_talks()", function()
    test("連続したtalkトークンを単一talkに統合する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "こんにちは")
        act:talk(sakura, "ございます")
        local result = act:build()

        expect(#result):toBe(1)
        expect(result[1].type):toBe("actor")
        expect(#result[1].tokens):toBe(1)
        expect(result[1].tokens[1].type):toBe("talk")
        expect(result[1].tokens[1].text):toBe("こんにちはございます")
    end)

    test("空文字列のtalkも正しく結合する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "")
        act:talk(sakura, "テスト")
        act:talk(sakura, "")
        local result = act:build()

        expect(#result):toBe(1)
        expect(result[1].tokens[1].text):toBe("テスト")
    end)

    test("アクター行動トークンで分離する（surface）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "前半")
        act:surface(5)
        act:talk(sakura, "後半")
        local result = act:build()

        expect(#result):toBe(1)
        expect(#result[1].tokens):toBe(3)
        expect(result[1].tokens[1].type):toBe("talk")
        expect(result[1].tokens[1].text):toBe("前半")
        expect(result[1].tokens[2].type):toBe("surface")
        expect(result[1].tokens[3].type):toBe("talk")
        expect(result[1].tokens[3].text):toBe("後半")
    end)

    test("アクター行動トークンで分離する（wait）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "待機前")
        act:wait(500)
        act:talk(sakura, "待機後")
        local result = act:build()

        expect(#result):toBe(1)
        expect(#result[1].tokens):toBe(3)
        expect(result[1].tokens[2].type):toBe("wait")
        expect(result[1].tokens[2].ms):toBe(500)
    end)

    test("アクター行動トークンで分離する（newline）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "1行目")
        act:newline(2)
        act:talk(sakura, "3行目")
        local result = act:build()

        expect(#result):toBe(1)
        expect(#result[1].tokens):toBe(3)
        expect(result[1].tokens[2].type):toBe("newline")
        expect(result[1].tokens[2].n):toBe(2)
    end)

    test("アクター行動トークンで分離する（clear）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "クリア前")
        act:clear()
        act:talk(sakura, "クリア後")
        local result = act:build()

        expect(#result):toBe(1)
        expect(#result[1].tokens):toBe(3)
        expect(result[1].tokens[2].type):toBe("clear")
    end)

    test("アクター行動トークンで分離する（sakura_script）", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "スクリプト前")
        act:sakura_script("\\![open,notepad]")
        act:talk(sakura, "スクリプト後")
        local result = act:build()

        expect(#result):toBe(1)
        expect(#result[1].tokens):toBe(3)
        expect(result[1].tokens[2].type):toBe("sakura_script")
        expect(result[1].tokens[2].text):toBe("\\![open,notepad]")
    end)

    test("最初のtalkのactor情報を保持する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "結合")
        act:talk(sakura, "テスト")
        local result = act:build()

        expect(result[1].tokens[1].actor):toBe(sakura)
    end)

    test("spot/clear_spotトークンはそのまま出力する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:set_spot("うにゅう", 1)
        act:talk(sakura, "テスト")
        act:clear_spot()
        local result = act:build()

        expect(result[1].type):toBe("spot")
        expect(result[2].type):toBe("actor")
        expect(result[3].type):toBe("clear_spot")
    end)
end)

-- ============================================================================
-- 統合テスト: 複合シナリオ
-- ============================================================================

describe("ACT.build() - 複合シナリオ", function()
    test("実際の会話シナリオを正しくグループ化する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local kero = actors["うにゅう"]
        local act = ACT.new(actors)

        -- 複合シナリオ: さくらが話し、表情変え、また話す。うにゅうが返答。
        act:set_spot("うにゅう", 1)
        act:talk(sakura, "今日は")
        act:talk(sakura, "いい天気ですね")
        act:surface(5)
        act:talk(sakura, "散歩しましょう")
        act:talk(kero, "いいですね")
        local result = act:build()

        -- 期待: spot, actor(さくら), actor(うにゅう)
        expect(#result):toBe(3)
        expect(result[1].type):toBe("spot")

        -- さくらのグループ
        expect(result[2].type):toBe("actor")
        expect(result[2].actor):toBe(sakura)
        -- 連続talkは統合、surfaceで分離
        expect(#result[2].tokens):toBe(3)
        expect(result[2].tokens[1].type):toBe("talk")
        expect(result[2].tokens[1].text):toBe("今日はいい天気ですね")
        expect(result[2].tokens[2].type):toBe("surface")
        expect(result[2].tokens[3].type):toBe("talk")
        expect(result[2].tokens[3].text):toBe("散歩しましょう")

        -- うにゅうのグループ
        expect(result[3].type):toBe("actor")
        expect(result[3].actor):toBe(kero)
        expect(#result[3].tokens):toBe(1)
        expect(result[3].tokens[1].text):toBe("いいですね")
    end)

    test("build後にtokenがリセットされnilを返す (act-build-early-return)", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = ACT.new(actors)

        act:talk(sakura, "テスト")
        local _ = act:build()
        local result2 = act:build()

        expect(result2):toBe(nil)
    end)
end)
