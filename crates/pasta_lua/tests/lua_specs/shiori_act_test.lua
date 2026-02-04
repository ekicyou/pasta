-- SHIORI Act module tests
-- Tests for pasta.shiori.act module - sakura script builder
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = "sakura" },
        kero = { name = "うにゅう", spot = "kero" },
        char2 = { name = "キャラ2", spot = "char2" },
    }
end

-- Create mock context (full CTX-like structure)
local function create_mock_ctx()
    local ACTOR = require("pasta.actor")
    local sakura = ACTOR.get_or_create("さくら")
    sakura.spot = "sakura"
    local kero = ACTOR.get_or_create("うにゅう")
    kero.spot = "kero"

    return {
        actors = {
            sakura = sakura,
            kero = kero,
        }
    }
end

-- Test inheritance from pasta.act
describe("SHIORI_ACT - inheritance", function()
    test("inherits ACT.IMPL methods", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- sakura_script() is inherited from ACT_IMPL
        act:sakura_script("\\e")
        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("sakura_script")
    end)

    test("has IMPL field for further inheritance", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        -- Check IMPL exists and is a table (avoid deep inspection)
        expect(type(SHIORI_ACT.IMPL)):toBe("table")
        -- Check __index is set (use rawget to avoid metatable traversal)
        expect(rawget(SHIORI_ACT.IMPL, "__index") ~= nil):toBe(true)
    end)

    test("inherits word() method", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- word() method should be accessible (returns nil for unknown word)
        local result = act:word("unknown_word")
        expect(result):toBe(nil)
    end)

    test("supports actor proxy (act.sakura:talk)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx.actors)

        -- act.sakura should create a proxy that redirects to act:talk(sakura, text)
        act.sakura:talk("Hello via proxy")
        local result = act:build()

        -- Should contain scope tag and text (same as direct call)
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello via proxy")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("actor proxy supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local ctx = create_mock_ctx()
        local act = SHIORI_ACT.new(ctx.actors)

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        -- Proxy talk returns nil, but act methods can chain
        act.sakura:talk("First")
        act:surface(5)
        act.kero:talk("Second")
        local result = act:build()

        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("First")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("Second")):toBeTruthy()
    end)
end)

-- Test talk() method override
describe("SHIORI_ACT - talk()", function()
    test("appends scope tag on first actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        local result = act:build()

        -- Should contain: \p[0] (default spot) + Hello + \e
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("appends scope tag on actor switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")
        local result = act:build()

        -- Should contain both spot tags (SSP compliant)
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
    end)

    test("does not append scope tag on same actor", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.sakura, "World")
        local result = act:build()

        -- \p[0] should appear only once
        local _, count = result:gsub("\\p%[0%]", "")
        expect(count):toBe(1)
    end)

    test("uses \\p[N] for char2+ actors", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("char2", 2)

        act:talk(actors.char2, "Third character")
        local result = act:build()

        expect(result:find("\\p%[2%]")):toBeTruthy()
    end)

    test("adds newline after spot switch", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        act:talk(actors.sakura, "Hello")
        act:talk(actors.kero, "Hi")
        local result = act:build()

        -- スポット変更時に段落改行が出力される: \n[150]\p[1]
        expect(result:find("\\n%[150%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:talk(actors.sakura, "Hello")
        expect(returned):toBe(act)
    end)

    test("also updates token buffer (parent behavior)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        expect(#act.token):toBeGraterThan(0)
    end)
end)

-- Test surface() method (グループ化対応版: surfaceはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - surface()", function()
    test("appends surface tag with number", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- グループ化後: surfaceはtalkのactorグループ内で処理される
        act:talk(actors.sakura, "")
        act:surface(5)
        local result = act:build()

        expect(result:find("\\s%[5%]")):toBeTruthy()
    end)

    test("appends surface tag with alias string", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- グループ化後: surfaceはtalkのactorグループ内で処理される
        act:talk(actors.sakura, "")
        act:surface("smile")
        local result = act:build()

        expect(result:find("\\s%[smile%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:surface(5)
        expect(returned):toBe(act)
    end)
end)

-- Test wait() method (グループ化対応版: waitはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - wait()", function()
    test("appends wait tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:wait(500)
        local result = act:build()

        expect(result:find("\\w%[500%]")):toBeTruthy()
    end)

    test("handles negative values as 0", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:wait(-100)
        local result = act:build()

        expect(result:find("\\w%[0%]")):toBeTruthy()
    end)

    test("truncates float to integer", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:wait(500.7)
        local result = act:build()

        expect(result:find("\\w%[500%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:wait(500)
        expect(returned):toBe(act)
    end)
end)

-- Test newline() method (グループ化対応版: newlineはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - newline()", function()
    test("appends single newline by default", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:newline()
        local result = act:build()

        expect(result:find("\\n")):toBeTruthy()
    end)

    test("appends multiple newlines", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:newline(3)
        local result = act:build()

        expect(result:find("\\n\\n\\n")):toBeTruthy()
    end)

    test("does nothing for n < 1", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:newline(0)
        act:newline(-1)
        local result = act:build()

        -- n=0やn=-1のnewlineは出力されない
        -- talkが空文字列なので、スポットタグ + \e のみ
        expect(result:find("\\p%[0%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:newline()
        expect(returned):toBe(act)
    end)
end)

-- Test clear() method (グループ化対応版: clearはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - clear()", function()
    test("appends clear tag", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:clear()
        local result = act:build()

        expect(result:find("\\c")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:clear()
        expect(returned):toBe(act)
    end)
end)

-- Test build() method
describe("SHIORI_ACT - build()", function()
    test("returns nil for empty buffer (act-build-early-return)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local result = act:build()

        expect(result):toBe(nil)
    end)

    test("appends \\e to end", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "")
        act:surface(5):wait(100)
        local result = act:build()

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("auto-resets after build", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "test")
        act:surface(5)
        local result1 = act:build()
        -- After build(), buffer is auto-reset
        local result2 = act:build()

        expect(result1:find("\\s%[5%]")):toBeTruthy()
        expect(result2):toBe(nil) -- nil after auto-reset (act-build-early-return)
    end)
end)

-- Test reset() method - REMOVED (reset is no longer a public API)
-- The reset functionality is now integrated into build() method.
-- build() automatically resets the token buffer and spot state.

-- Test talk_to_script conversion (wait insertion)
describe("SHIORI_ACT - talk_to_script変換", function()
    test("通常テキストがそのまま出力される（デフォルト設定）", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "Hello")
        local result = act:build()

        -- デフォルト設定ではウェイトタグなし（effective_wait = 0）
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("句点にはウェイトタグが挿入される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "あ。")
        local result = act:build()

        -- 句点（。）にはデフォルトでウェイトタグが挿入される
        -- script_wait_period=1000 → effective=950
        expect(result:find("\\_w%[950%]")):toBeTruthy()
    end)

    test("読点にはウェイトタグが挿入される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        act:talk(actors.sakura, "あ、")
        local result = act:build()

        -- 読点（、）にはデフォルトでウェイトタグが挿入される
        -- script_wait_comma=500 → effective=450
        expect(result:find("\\_w%[450%]")):toBeTruthy()
    end)
end)

-- E2E scenario test
describe("SHIORI_ACT - E2E scenario", function()
    test("complex script generation", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        act:talk(actors.sakura, "こんにちは")
            :surface(5)
            :wait(500)
            :talk(actors.kero, "やあ")
            :clear()

        local result = act:build()

        -- Verify structure (SSP compliant: \p[ID] format)
        expect(result:find("\\p%[0%]")):toBeTruthy()   -- sakura spot
        expect(result:find("こんにちは")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()   -- surface
        expect(result:find("\\w%[500%]")):toBeTruthy() -- wait
        expect(result:find("\\p%[1%]")):toBeTruthy()   -- kero spot
        expect(result:find("やあ")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()        -- clear
        expect(result:sub(-2)):toBe("\\e")             -- end
    end)

    test("multiple rounds (build auto-resets)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        -- First round
        act:talk(actors.sakura, "First")
        local result1 = act:build()
        expect(result1:find("First")):toBeTruthy()

        -- Second round (build auto-resets, so no manual reset needed)
        -- 新アーキテクチャ: actor_spotsもbuild()ごとにリセットされる
        act:set_spot("kero", 1)
        act:talk(actors.kero, "Second")
        local result2 = act:build()

        expect(result2:find("First")):toBeFalsy()     -- First should be cleared by auto-reset
        expect(result2:find("Second")):toBeTruthy()
        expect(result2:find("\\p%[1%]")):toBeTruthy() -- kero spot
    end)
end)

-- Test act.req field
describe("SHIORI_ACT - req field", function()
    test("stores req parameter in act.req", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnTest",
            method = "get",
            version = 30,
            reference = { "ref0", "ref1" },
        }

        local act = SHIORI_ACT.new(actors, req)

        expect(act.req):toBe(req)
        expect(act.req.id):toBe("OnTest")
        expect(act.req.method):toBe("get")
        expect(act.req.version):toBe(30)
    end)

    test("req reference is same object (not a copy)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnBoot",
            reference = { "value0" },
        }

        local act = SHIORI_ACT.new(actors, req)

        -- Same reference check
        expect(act.req == req):toBe(true)
        expect(act.req.reference[1]):toBe("value0")
    end)

    test("act.req.date contains date info when provided", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnSecondChange",
            date = {
                unix = 1704067200,
                hour = 12,
                minute = 0,
            },
        }

        local act = SHIORI_ACT.new(actors, req)

        expect(act.req.date.unix):toBe(1704067200)
        expect(act.req.date.hour):toBe(12)
    end)
end)

-- ============================================================================
-- Task 4.1: transfer_date_to_var() テスト
-- Requirements: 1.1, 1.2, 1.3, 1.4
-- ============================================================================

describe("SHIORI_ACT - transfer_date_to_var()", function()
    -- 正常系: 全フィールド転記確認（英語・数値型）
    test("transfers all date fields from req.date to var", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnSecondChange",
            date = {
                year = 2026,
                month = 2,
                day = 1,
                hour = 14,
                min = 37,
                sec = 45,
                wday = 0,
                unix = 1769932665,
                ns = 123456789,
                yday = 32,
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_date_to_var()

        -- 英語フィールド（数値型）確認
        expect(act.var.year):toBe(2026)
        expect(act.var.month):toBe(2)
        expect(act.var.day):toBe(1)
        expect(act.var.hour):toBe(14)
        expect(act.var.min):toBe(37)
        expect(act.var.sec):toBe(45)
        expect(act.var.wday):toBe(0)

        -- 転記対象外の確認（unix, ns, yday は転記されない）
        expect(act.var.unix):toBe(nil)
        expect(act.var.ns):toBe(nil)
        expect(act.var.yday):toBe(nil)
    end)

    -- req 不在時の安全終了
    test("returns self safely when req is nil", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()

        local act = SHIORI_ACT.new(actors, nil)
        local result = act:transfer_date_to_var()

        -- 何もせず正常終了、メソッドチェーン用に self を返す
        expect(result):toBe(act)
    end)

    -- req.date 不在時の安全終了
    test("returns self safely when req.date is nil", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = { id = "OnSecondChange" }

        local act = SHIORI_ACT.new(actors, req)
        local result = act:transfer_date_to_var()

        -- 何もせず正常終了、メソッドチェーン用に self を返す
        expect(result):toBe(act)
    end)

    -- 日本語変数マッピング確認（年月日時分秒）
    test("maps Japanese variable names with formatted strings", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnSecondChange",
            date = {
                year = 2026,
                month = 2,
                day = 1,
                hour = 14,
                min = 37,
                sec = 45,
                wday = 0,
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_date_to_var()

        -- 日本語変数（文字列型）確認
        expect(act.var["年"]):toBe("2026年")
        expect(act.var["月"]):toBe("2月")
        expect(act.var["日"]):toBe("1日")
        expect(act.var["時"]):toBe("14時")
        expect(act.var["分"]):toBe("37分")
        expect(act.var["秒"]):toBe("45秒")
    end)

    -- 曜日変換確認（wday 0-6 全パターン）
    test("converts wday to Japanese and English weekday names", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()

        local weekdays_ja = { "日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日" }
        local weekdays_en = { "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" }

        for wday = 0, 6 do
            local req = {
                id = "OnSecondChange",
                date = { year = 2026, month = 2, day = 1, hour = 0, min = 0, sec = 0, wday = wday },
            }
            local act = SHIORI_ACT.new(actors, req)
            act:transfer_date_to_var()

            expect(act.var["曜日"]):toBe(weekdays_ja[wday + 1])
            expect(act.var.week):toBe(weekdays_en[wday + 1])
        end
    end)

    -- 12時間制変換確認（hour 0, 1, 11, 12, 13, 23 のケース）
    test("converts hour to 12-hour format with 深夜0時/正午 special cases", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()

        local test_cases = {
            { hour = 0, expected = "深夜0時" },
            { hour = 1, expected = "午前1時" },
            { hour = 11, expected = "午前11時" },
            { hour = 12, expected = "正午" },
            { hour = 13, expected = "午後1時" },
            { hour = 23, expected = "午後11時" },
        }

        for _, tc in ipairs(test_cases) do
            local req = {
                id = "OnSecondChange",
                date = { year = 2026, month = 2, day = 1, hour = tc.hour, min = 0, sec = 0, wday = 0 },
            }
            local act = SHIORI_ACT.new(actors, req)
            act:transfer_date_to_var()

            expect(act.var["時１２"]):toBe(tc.expected)
        end
    end)

    -- メソッドチェーン用に self を返す
    test("returns self for method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnSecondChange",
            date = { year = 2026, month = 2, day = 1, hour = 14, min = 37, sec = 45, wday = 0 },
        }

        local act = SHIORI_ACT.new(actors, req)
        local result = act:transfer_date_to_var()

        expect(result):toBe(act)
    end)
end)
