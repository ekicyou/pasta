-- pasta.act module tests
-- Tests for pasta.act module - token buffer refactoring
-- 親クラスのUI操作トークン蓄積、スポット切り替え検出、build/yield動作を検証
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = 0 },
        kero = { name = "うにゅう", spot = 1 },
        char2 = { name = "キャラ2", spot = 2 },
    }
end

-- ============================================================================
-- Requirement 1: UI操作トークン蓄積
-- ============================================================================

describe("ACT - surface()", function()
    test("蓄積トークンに { type = 'surface', id = id } を追加する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:surface(5)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("surface")
        expect(act.token[1].id):toBe(5)
    end)

    test("文字列IDをサポートする", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:surface("smile")

        expect(act.token[1].id):toBe("smile")
    end)

    test("メソッドチェーン用にselfを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:surface(5)

        expect(returned):toBe(act)
    end)
end)

describe("ACT - wait()", function()
    test("蓄積トークンに { type = 'wait', ms = ms } を追加する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:wait(500)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("wait")
        expect(act.token[1].ms):toBe(500)
    end)

    test("負の値は0に変換する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:wait(-100)

        expect(act.token[1].ms):toBe(0)
    end)

    test("小数点以下を切り捨てる", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:wait(500.7)

        expect(act.token[1].ms):toBe(500)
    end)

    test("nilは0として扱う", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:wait(nil)

        expect(act.token[1].ms):toBe(0)
    end)

    test("メソッドチェーン用にselfを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:wait(500)

        expect(returned):toBe(act)
    end)
end)

describe("ACT - newline()", function()
    test("蓄積トークンに { type = 'newline', n = n } を追加する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:newline(3)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("newline")
        expect(act.token[1].n):toBe(3)
    end)

    test("引数なしの場合はn=1をデフォルトとする", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:newline()

        expect(act.token[1].n):toBe(1)
    end)

    test("メソッドチェーン用にselfを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:newline()

        expect(returned):toBe(act)
    end)
end)

describe("ACT - clear()", function()
    test("蓄積トークンに { type = 'clear' } を追加する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        act:clear()

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("clear")
    end)

    test("メソッドチェーン用にselfを返す", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        local returned = act:clear()

        expect(returned):toBe(act)
    end)
end)

-- ============================================================================
-- Requirement 2: スポット切り替え検出
-- ============================================================================

describe("ACT - スポット切り替え検出", function()
    test("ACT.new()で_current_spot = nilを初期化する", function()
        local ACT = require("pasta.act")
        local act = ACT.new(create_mock_actors())

        expect(act._current_spot):toBe(nil)
    end)

    test("talk()でスポット変更検出時にspot_switchトークンをactorトークン直後に挿入", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")

        -- 期待: actor(sakura), talk(Hello), actor(kero), spot_switch, talk(Hi)
        local types = {}
        for _, t in ipairs(act.token) do
            table.insert(types, t.type)
        end

        expect(types[1]):toBe("actor")
        expect(types[2]):toBe("talk")
        expect(types[3]):toBe("actor")
        expect(types[4]):toBe("spot_switch")
        expect(types[5]):toBe("talk")
    end)

    test("初回のactor追加時はspot_switchを挿入しない", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")

        -- 期待: actor(sakura), talk(Hello) のみ（spot_switchなし）
        local has_spot_switch = false
        for _, t in ipairs(act.token) do
            if t.type == "spot_switch" then
                has_spot_switch = true
            end
        end
        expect(has_spot_switch):toBe(false)
    end)

    test("同一スポットでの連続talkではspot_switchを挿入しない", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.sakura, "World")

        -- 期待: actor(sakura), talk(Hello), talk(World) のみ
        local spot_switch_count = 0
        for _, t in ipairs(act.token) do
            if t.type == "spot_switch" then
                spot_switch_count = spot_switch_count + 1
            end
        end
        expect(spot_switch_count):toBe(0)
    end)

    test("スポット切り替え検出時に_current_spotを更新する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(act._current_spot):toBe(0)

        act:talk(actors.kero, "Hi")
        expect(act._current_spot):toBe(1)
    end)

    test("actor.spotがnilの場合は0として扱う", function()
        local ACT = require("pasta.act")
        local actors = { unknown = { name = "Unknown", spot = nil } }
        local act = ACT.new(actors)

        act:talk(actors.unknown, "Hello")

        expect(act._current_spot):toBe(0)
    end)
end)

-- ============================================================================
-- Requirement 5: talk後の固定改行除去
-- ============================================================================

describe("ACT - talk()固定改行除去", function()
    test("talk()後にnewlineトークンを自動挿入しない", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")

        -- newlineトークンが含まれていないことを確認
        local has_newline = false
        for _, t in ipairs(act.token) do
            if t.type == "newline" then
                has_newline = true
            end
        end
        expect(has_newline):toBe(false)
    end)
end)

-- ============================================================================
-- Requirement 7: 親build()メソッド
-- ============================================================================

describe("ACT - build()", function()
    test("トークン配列を返却する", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:surface(5)
        act:wait(100)
        local token = act:build()

        expect(type(token)):toBe("table")
        expect(#token):toBe(2)
        expect(token[1].type):toBe("surface")
        expect(token[2].type):toBe("wait")
    end)

    test("build()後にself.tokenを空配列にリセットする", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:surface(5)
        local _ = act:build()

        expect(#act.token):toBe(0)
    end)

    test("build()後にnow_actorをnilにリセットする", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(act.now_actor):toBe(actors.sakura)

        local _ = act:build()
        expect(act.now_actor):toBe(nil)
    end)

    test("build()後に_current_spotをnilにリセットする", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(act._current_spot):toBe(0)

        local _ = act:build()
        expect(act._current_spot):toBe(nil)
    end)

    test("空のトークン配列を返却できる", function()
        local ACT = require("pasta.act")
        local act = ACT.new({})

        local token = act:build()

        expect(type(token)):toBe("table")
        expect(#token):toBe(0)
    end)
end)

-- ============================================================================
-- Requirement 8: 親yield()責務統一
-- ============================================================================

describe("ACT - yield()", function()
    test("yield()はself:build()を呼び出す", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:surface(5)

        local co = coroutine.create(function()
            return act:yield()
        end)

        local ok, result = coroutine.resume(co)

        expect(ok):toBe(true)
        expect(type(result)):toBe("table") -- build()の結果
        expect(#result):toBe(1)
        expect(result[1].type):toBe("surface")
    end)

    test("yield()後にトークンがリセットされる", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:surface(5)

        local co = coroutine.create(function()
            return act:yield()
        end)

        local _, _ = coroutine.resume(co)

        expect(#act.token):toBe(0)
    end)

    test("yield()はメソッドチェーン用にselfを返す", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:surface(5)

        local co = coroutine.create(function()
            local returned = act:yield()
            return returned
        end)

        -- 最初のresume: yield()でサスペンド
        local ok1, _ = coroutine.resume(co)
        expect(ok1):toBe(true)

        -- 2回目のresume: selfを返す
        local ok2, returned = coroutine.resume(co)
        expect(ok2):toBe(true)
        expect(returned):toBe(act)
    end)

    test("yield()後に_current_spotがリセットされる", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(act._current_spot):toBe(0)

        local co = coroutine.create(function()
            return act:yield()
        end)

        local _, _ = coroutine.resume(co)

        expect(act._current_spot):toBe(nil)
    end)
end)

-- ============================================================================
-- Requirement 11: end_action()の削除
-- ============================================================================

describe("ACT - end_action()削除", function()
    test("end_action()メソッドが存在しない", function()
        local ACT = require("pasta.act")
        local act = ACT.new({})

        expect(act.end_action):toBe(nil)
    end)
end)

-- ============================================================================
-- メソッドチェーンテスト
-- ============================================================================

describe("ACT - メソッドチェーン", function()
    test("複数メソッドを連続呼び出し可能", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        local returned = act
            :surface(5)
            :wait(100)
            :newline(2)
            :clear()

        expect(returned):toBe(act)
        expect(#act.token):toBe(4)
    end)

    test("talk()もメソッドチェーンに対応", function()
        local ACT = require("pasta.act")
        local actors = create_mock_actors()
        local act = ACT.new(actors)

        local returned = act:talk(actors.sakura, "Hello")

        expect(returned):toBe(act)
    end)
end)
