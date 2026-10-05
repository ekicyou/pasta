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

-- ヘルパー: テストごとの共通セットアップ（act と actors を返す）
-- リロード規約（先行スイートの package.loaded 操作）を保存するため、
-- require はファイル先頭ではなくテスト実行時に行う
local function new_act(req)
    local SHIORI_ACT = require("pasta.shiori.act")
    local actors = create_mock_actors()
    return SHIORI_ACT.new(actors, req), actors
end

-- Test inheritance from pasta.act
describe("SHIORI_ACT - inheritance", function()
    test("inherits ACT.IMPL methods", function()
        local act = new_act()

        -- raw_script() is inherited from ACT_IMPL
        act:raw_script("\\e")
        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("raw_script")
    end)

    test("has IMPL field for further inheritance", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        -- Check IMPL exists and is a table (avoid deep inspection)
        expect(type(SHIORI_ACT.IMPL)):toBe("table")
        -- Check __index is set (use rawget to avoid metatable traversal)
        expect(rawget(SHIORI_ACT.IMPL, "__index") ~= nil):toBe(true)
    end)

    test("inherits word() method", function()
        local act = new_act()

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
        local act, actors = new_act()

        act:talk(actors.sakura, "Hello")
        local result = act:build()

        -- Should contain: \p[0] (default spot) + Hello + \e
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("appends scope tag on actor switch", function()
        local act, actors = new_act()

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
        local act, actors = new_act()

        act:talk(actors.sakura, "Hello")
        act:talk(actors.sakura, "World")
        local result = act:build()

        -- \p[0] should appear only once
        local _, count = result:gsub("\\p%[0%]", "")
        expect(count):toBe(1)
    end)

    test("uses \\p[N] for char2+ actors", function()
        local act, actors = new_act()

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        act:set_spot("char2", 2)

        act:talk(actors.char2, "Third character")
        local result = act:build()

        expect(result:find("\\p%[2%]")):toBeTruthy()
    end)

    test("adds newline only on the return turn of an A→B→A round-trip", function()
        local act, actors = new_act()

        -- 新アーキテクチャ: set_spot()でスポット位置を明示的に設定
        -- sakura=spot0, kero=spot1 の異なるスポットで往復
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        -- A→B→A 往復。完全遅延方式では、スポットに既発話がある状態へ
        -- 戻ってきた手番の直前にのみ段落改行 \n[150] が出力される。
        act:talk(actors.sakura, "Hello") -- A1
        act:talk(actors.kero, "Hi")      -- B1
        act:talk(actors.sakura, "Back")  -- A2 (戻り手番)
        local result = act:build()

        -- \n[150] は往復全体で1回だけ出現する
        local _, newline_count = result:gsub("\\n%[150%]", "")
        expect(newline_count):toBe(1)

        -- \n[150] は戻り手番(A2)のテキスト直前にのみ出現する:
        -- \p[0]\n[150]Back の順で連続して現れる
        expect(result:find("\\p%[0%]\\n%[150%]Back")):toBeTruthy()

        -- A1・B1 の前には \n[150] が出ない:
        -- 最初の \p[0] の直後は Hello（改行なし）、B1 の \p[1] の直後は Hi
        expect(result:find("\\p%[0%]Hello")):toBeTruthy()
        expect(result:find("\\p%[1%]Hi")):toBeTruthy()
    end)

    test("4.4: tag-only talk (word reference path) gets no paragraph break", function()
        local act, actors = new_act()
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)

        act:talk(actors.sakura, "A1")
        act:talk(actors.kero, "B1")
        act:talk(actors.sakura, "\\s[1000]") -- タグだけの行（単語参照と同じ talk トークン）
        act:talk(actors.kero, "B2")

        -- \p[0]\s[1000] の直後に \n[150] が無く、\p[1]\n[150]B2 が 1 回だけ出る
        expect(act:build()):toBe("\\p[0]A1\\p[1]B1\\p[0]\\s[1000]\\p[1]\\n[150]B2\\e")
    end)

    test("supports method chaining", function()
        local act, actors = new_act()

        local returned = act:talk(actors.sakura, "Hello")
        expect(returned):toBe(act)
    end)

    test("also updates token buffer (parent behavior)", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "Hello")
        expect(#act.token):toBeGraterThan(0)
    end)
end)

-- Test surface() method (グループ化対応版: surfaceはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - surface()", function()
    test("appends surface tag with number", function()
        local act, actors = new_act()

        -- グループ化後: surfaceはtalkのactorグループ内で処理される
        act:talk(actors.sakura, "")
        act:surface(5)
        local result = act:build()

        expect(result:find("\\s%[5%]")):toBeTruthy()
    end)

    test("appends surface tag with alias string", function()
        local act, actors = new_act()

        -- グループ化後: surfaceはtalkのactorグループ内で処理される
        act:talk(actors.sakura, "")
        act:surface("smile")
        local result = act:build()

        expect(result:find("\\s%[smile%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local act = new_act()

        local returned = act:surface(5)
        expect(returned):toBe(act)
    end)
end)

-- Test wait() method (グループ化対応版: waitはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - wait()", function()
    test("appends wait tag", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:wait(500)
        local result = act:build()

        expect(result:find("\\_w%[500%]")):toBeTruthy()
    end)

    test("handles negative values as 0", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:wait(-100)
        local result = act:build()

        expect(result:find("\\_w%[0%]")):toBeTruthy()
    end)

    test("truncates float to integer", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:wait(500.7)
        local result = act:build()

        expect(result:find("\\_w%[500%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local act = new_act()

        local returned = act:wait(500)
        expect(returned):toBe(act)
    end)
end)

-- Test newline() method (グループ化対応版: newlineはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - newline()", function()
    test("appends single newline by default", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:newline()
        local result = act:build()

        expect(result:find("\\n")):toBeTruthy()
    end)

    test("appends multiple newlines", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:newline(3)
        local result = act:build()

        expect(result:find("\\n\\n\\n")):toBeTruthy()
    end)

    test("does nothing for n < 1", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:newline(0)
        act:newline(-1)
        local result = act:build()

        -- n=0やn=-1のnewlineは出力されない
        -- talkが空文字列なので、スポットタグ + \e のみ
        expect(result:find("\\p%[0%]")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local act = new_act()

        local returned = act:newline()
        expect(returned):toBe(act)
    end)
end)

-- Test clear() method (グループ化対応版: clearはtalkのactorグループ内で処理される)
describe("SHIORI_ACT - clear()", function()
    test("appends clear tag", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:clear()
        local result = act:build()

        expect(result:find("\\c")):toBeTruthy()
    end)

    test("supports method chaining", function()
        local act = new_act()

        local returned = act:clear()
        expect(returned):toBe(act)
    end)
end)

-- Test build() method
describe("SHIORI_ACT - build()", function()
    test("returns nil for empty buffer (act-build-early-return)", function()
        local act = new_act()

        local result = act:build()

        expect(result):toBe(nil)
    end)

    test("appends \\e to end", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "")
        act:surface(5):wait(100)
        local result = act:build()

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("auto-resets after build", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "test")
        act:surface(5)
        local result1 = act:build()
        -- After build(), buffer is auto-reset
        local result2 = act:build()

        expect(result1:find("\\s%[5%]")):toBeTruthy()
        expect(result2):toBe(nil) -- nil after auto-reset (act-build-early-return)
    end)
end)

-- actor-surface-restore Task 3.2: STORE.appearance をビルダーへ渡す（直接変更方式）
describe("SHIORI_ACT - build() STORE.appearance連携 (actor-surface-restore)", function()
    test("2.7/4.1: act 経由でストアの外見状態が更新され、後続ビルドの \\p[0] 直後に \\s[10] を復旧する", function()
        local STORE = require("pasta.store")
        STORE.reset()
        local appearance = STORE.appearance

        -- スポット未設定の2アクターはスポット0を共有する
        local act, actors = new_act()
        act:talk(actors.kero, "B1"):surface(10)
        act:talk(actors.sakura, "A1"):surface(3)
        act:build()

        -- 直接変更方式: 同じ表がその場で更新される（サーフェスIDは出力どおりの文字列で記録: 1.5）
        expect(STORE.appearance):toBe(appearance)
        expect(appearance.actors["うにゅう"].surface):toBe("10")
        expect(appearance.actors["さくら"].surface):toBe("3")
        expect(appearance.spots[0].surface):toBe("3")

        local act2 = new_act()
        act2:talk(actors.kero, "B2")
        expect(act2:build()):toBe("\\p[0]\\s[10]B2\\e")

        STORE.reset()
    end)

    test("ストアに外見状態が無い環境でもビルドローカル状態でエラーなく動く", function()
        local STORE = require("pasta.store")
        STORE.reset()
        STORE.appearance = nil

        local act, actors = new_act()
        act:talk(actors.kero, "B1"):surface(10)
        act:talk(actors.sakura, "A1"):surface(3)
        act:talk(actors.kero, "B2")
        expect(act:build()):toBe("\\p[0]B1\\s[10]\\p[0]\\n[150]A1\\s[3]\\p[0]\\s[10]\\n[150]B2\\e")
        expect(STORE.appearance):toBe(nil)

        STORE.reset()
    end)
end)

-- Test reset() method - REMOVED (reset is no longer a public API)
-- The reset functionality is now integrated into build() method.
-- build() automatically resets the token buffer and spot state.

-- Test talk_to_script conversion (wait insertion)
describe("SHIORI_ACT - talk_to_script変換", function()
    test("通常テキストがそのまま出力される（デフォルト設定）", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "Hello")
        local result = act:build()

        -- デフォルト設定ではウェイトタグなし（effective_wait = 0）
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("句点にはウェイトタグが挿入される", function()
        local act, actors = new_act()

        act:talk(actors.sakura, "あ。")
        local result = act:build()

        -- 句点（。）にはデフォルトでウェイトタグが挿入される
        -- script_wait_period=1000 → effective=950
        expect(result:find("\\_w%[950%]")):toBeTruthy()
    end)

    test("読点にはウェイトタグが挿入される", function()
        local act, actors = new_act()

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
        local act, actors = new_act()

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
        expect(result:find("\\_w%[500%]")):toBeTruthy() -- wait
        expect(result:find("\\p%[1%]")):toBeTruthy()   -- kero spot
        expect(result:find("やあ")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()        -- clear
        expect(result:sub(-2)):toBe("\\e")             -- end
    end)

    test("multiple rounds (build auto-resets)", function()
        local act, actors = new_act()

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
        local req = {
            id = "OnTest",
            method = "get",
            version = 30,
            reference = { "ref0", "ref1" },
        }

        local act = new_act(req)

        expect(act.req):toBe(req)
        expect(act.req.id):toBe("OnTest")
        expect(act.req.method):toBe("get")
        expect(act.req.version):toBe(30)
    end)

    test("req reference is same object (not a copy)", function()
        local req = {
            id = "OnBoot",
            reference = { "value0" },
        }

        local act = new_act(req)

        -- Same reference check
        expect(act.req == req):toBe(true)
        expect(act.req.reference[1]):toBe("value0")
    end)

    test("act.req.date contains date info when provided", function()
        local req = {
            id = "OnSecondChange",
            date = {
                unix = 1704067200,
                hour = 12,
                minute = 0,
            },
        }

        local act = new_act(req)

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

        local act = new_act(req)
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
        local act = new_act(nil)
        local result = act:transfer_date_to_var()

        -- 何もせず正常終了、メソッドチェーン用に self を返す
        expect(result):toBe(act)
    end)

    -- req.date 不在時の安全終了
    test("returns self safely when req.date is nil", function()
        local req = { id = "OnSecondChange" }

        local act = new_act(req)
        local result = act:transfer_date_to_var()

        -- 何もせず正常終了、メソッドチェーン用に self を返す
        expect(result):toBe(act)
    end)

    -- 日本語変数マッピング確認（年月日時分秒）
    test("maps Japanese variable names with formatted strings", function()
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

        local act = new_act(req)
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
        local weekdays_ja = { "日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日" }
        local weekdays_en = { "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" }

        for wday = 0, 6 do
            local req = {
                id = "OnSecondChange",
                date = { year = 2026, month = 2, day = 1, hour = 0, min = 0, sec = 0, wday = wday },
            }
            local act = new_act(req)
            act:transfer_date_to_var()

            expect(act.var["曜日"]):toBe(weekdays_ja[wday + 1])
            expect(act.var.week):toBe(weekdays_en[wday + 1])
        end
    end)

    -- 12時間制変換確認（hour 0, 1, 11, 12, 13, 23 のケース）
    test("converts hour to 12-hour format with 深夜0時/正午 special cases", function()
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
            local act = new_act(req)
            act:transfer_date_to_var()

            expect(act.var["時１２"]):toBe(tc.expected)
        end
    end)

    -- メソッドチェーン用に self を返す
    test("returns self for method chaining", function()
        local req = {
            id = "OnSecondChange",
            date = { year = 2026, month = 2, day = 1, hour = 14, min = 37, sec = 45, wday = 0 },
        }

        local act = new_act(req)
        local result = act:transfer_date_to_var()

        expect(result):toBe(act)
    end)
end)

-- ============================================================================
-- act-token-grouping-fix: さくらスクリプトのバイト比較
-- 先頭（と clear_spot の後、発言より前）の表示制御はアクター未指定として積んだ位置に出る。
-- clear_spot は現在のグループを閉じる。set_spot（spot）はグループを閉じない。
-- 前提: 既定サーフェスを持たないアクター（さくら＝立ち位置 0、うにゅう＝立ち位置 1）。
-- 本文は句読点を含めない（ウェイト挿入を避ける）。
-- ============================================================================

--- STORE.actor_spots・STORE.appearance を初期化し、act と さくら・うにゅう を返す
--- @param sakura_surface number|nil さくらの既定サーフェス（nil は持たない）
local function new_bytes_act(sakura_surface)
    local STORE = require("pasta.store")
    STORE.reset()
    STORE.actor_spots["さくら"] = 0
    STORE.actor_spots["うにゅう"] = 1
    local SHIORI_ACT = require("pasta.shiori.act")
    local actors = {
        ["さくら"] = { name = "さくら", surface = sakura_surface },
        ["うにゅう"] = { name = "うにゅう" },
    }
    return SHIORI_ACT.new(actors), actors["さくら"], actors["うにゅう"], STORE
end

describe("SHIORI_ACT - 先頭の表示制御のバイト比較 (act-token-grouping-fix)", function()
    test("1: yield の区切りの後に積んだ wait は切替タグの前に出る", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:talk(sakura, "X")
        expect(act:build()):toBe("\\p[0]X\\e") -- yield が呼ぶ区切り
        act:wait(500)
        act:talk(sakura, "A")
        expect(act:build()):toBe("\\_w[500]\\p[0]A\\e")
        STORE.reset()
    end)

    test("2: シーン冒頭の表示制御は積んだ順に切替タグなしで出る", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:surface(5)
        act:wait(500)
        act:newline()
        act:clear()
        act:choice("t", "d")
        act:choice_timeout(30)
        act:talk(sakura, "A")
        expect(act:build()):toBe("\\s[5]\\_w[500]\\n\\c\\![*]\\q[d,t]\\![set,choicetimeout,30000]\\p[0]A\\e")
        STORE.reset()
    end)

    test("3: 発言の無い出力でも wait が出る", function()
        local act, _, _, STORE = new_bytes_act()
        act:wait(1000)
        expect(act:build()):toBe("\\_w[1000]\\e")
        STORE.reset()
    end)

    test("4: GLOBAL.close_ghost（ゴースト終了）は wait と \\- を出す", function()
        local act, _, _, STORE = new_bytes_act()
        require("pasta.shiori.entry") -- GLOBAL.close_ghost を定義する
        local GLOBAL = require("pasta.global")
        GLOBAL.close_ghost(act, 1500)
        expect(act:build()):toBe("\\_w[1500]\\-\\e")
        STORE.reset()
    end)

    test("5: raw_script と wait が混在しても積んだ位置に出る", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:raw_script("\\![x]")
        act:wait(100)
        act:raw_script("\\![y]")
        act:talk(sakura, "A")
        expect(act:build()):toBe("\\![x]\\_w[100]\\![y]\\p[0]A\\e")
        STORE.reset()
    end)

    test("6: 先頭の surface はさくらのサーフェスとして記録されず既定サーフェスを復旧する", function()
        local act, sakura, _, STORE = new_bytes_act(0)
        act:surface(5)
        act:talk(sakura, "A")
        expect(act:build()):toBe("\\s[5]\\p[0]\\s[0]A\\e")
        local rec = STORE.appearance.actors["さくら"]
        expect(rec == nil or rec.surface ~= "5"):toBe(true)
        STORE.reset()
    end)

    test("7: 6 の続きの出力で同じアクターが同じスポットで続くと復旧タグは出ない", function()
        local act, sakura, _, STORE = new_bytes_act(0)
        act:surface(5)
        act:talk(sakura, "A")
        expect(act:build()):toBe("\\s[5]\\p[0]\\s[0]A\\e")
        act:surface(5)
        act:talk(sakura, "B")
        expect(act:build()):toBe("\\s[5]\\p[0]B\\e")
        local rec = STORE.appearance.actors["さくら"]
        expect(rec == nil or rec.surface ~= "5"):toBe(true)
        STORE.reset()
    end)
end)

describe("SHIORI_ACT - clear_spot を挟む出力のバイト比較 (act-token-grouping-fix)", function()
    test("8: clear_spot の後の同じ発言者の発言は切替タグの後に出る", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)
        act:talk(sakura, "B")
        expect(act:build()):toBe("\\p[0]A\\p[0]B\\e")
        STORE.reset()
    end)

    test("9: clear_spot の後の立ち位置の入れ替えが発言に効く", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:clear_spot()
        act:set_spot("うにゅう", 0)
        act:set_spot("さくら", 1)
        act:talk(sakura, "B")
        expect(act:build()):toBe("\\p[0]A\\p[1]B\\e")
        STORE.reset()
    end)

    test("10: clear_spot の後に発言者が変わる", function()
        local act, sakura, kero, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)
        act:talk(kero, "B")
        expect(act:build()):toBe("\\p[0]A\\p[1]B\\e")
        STORE.reset()
    end)

    test("11: clear_spot の後は段落区切りの判定を持ち越さない", function()
        local act, sakura, kero, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:talk(kero, "B")
        act:talk(sakura, "C")
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)
        act:talk(sakura, "D")
        expect(act:build()):toBe("\\p[0]A\\p[1]B\\p[0]\\n[150]C\\p[0]D\\e")
        STORE.reset()
    end)

    test("12: clear_spot の後、発言より前の wait は切替タグの前に出る", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)
        act:wait(300)
        act:talk(sakura, "B")
        expect(act:build()):toBe("\\p[0]A\\_w[300]\\p[0]B\\e")
        STORE.reset()
    end)

    test("13: set_spot 単独はグループを閉じない", function()
        local act, sakura, _, STORE = new_bytes_act()
        act:talk(sakura, "A")
        act:set_spot("さくら", 1)
        act:talk(sakura, "B")
        expect(act:build()):toBe("\\p[0]AB\\e")
        STORE.reset()
    end)
end)
