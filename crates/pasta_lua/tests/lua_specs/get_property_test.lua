-- get_property メソッドのテスト
-- SHIORI_ACT_IMPL.get_property() のバリデーション・タグ発行・多値返却を検証する
-- Requirements: 2.1, 3.1, 4.1, 4.2, 4.3, 5.4
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect
local mocks = require("lua_test.mocks")

-- モックアクターの生成
local function create_mock_actors()
    return {
        ["さくら"] = { name = "さくら", spot = 0 },
    }
end

local function setup()
    mocks.reset()
    mocks.install()
    package.loaded["pasta.shiori.event"] = nil
    package.loaded["pasta.shiori.event.callback"] = nil
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.shiori.res"] = nil
    package.loaded["pasta.shiori.act"] = nil
    package.loaded["pasta.act"] = nil
    require("pasta.store")
    local CALLBACK = require("pasta.shiori.event.callback")
    CALLBACK.reset()
end

-- コルーチン内で新規 act を生成し get_property を呼び出す共通ヘルパー
-- SHIORI_ACT の require は従来どおりコルーチン内（setup() のリロード後）で行う
-- 戻り値: resume の ok・yield 値またはエラー・コルーチン本体
local function resume_get_property(...)
    local n = select("#", ...)
    local args = { ... }
    local co = coroutine.create(function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())
        act:get_property(unpack(args, 1, n))
    end)
    local ok, result = coroutine.resume(co)
    return ok, result, co
end

-- ============================================================================
-- Group A: バリデーションテスト（コルーチン内）
-- Requirement 2.1
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - validation in coroutine", function()
    test("引数なしでエラーが発生する（type error）", function()
        setup()
        local ok, err = resume_get_property()
        expect(ok):toBe(false)
        expect(tostring(err):find("first argument must be a property name")).not_:toBe(nil)
    end)

    test("nilでエラーが発生する（type error）", function()
        setup()
        local ok, err = resume_get_property(nil)
        expect(ok):toBe(false)
        expect(tostring(err):find("first argument must be a property name")).not_:toBe(nil)
    end)

    test("空文字列でエラーが発生する（name must not be nil or empty）", function()
        setup()
        local ok, err = resume_get_property("")
        expect(ok):toBe(false)
        expect(tostring(err):find("name must not be nil or empty")).not_:toBe(nil)
    end)

    test("数値引数でエラーが発生する（type error）", function()
        setup()
        local ok, err = resume_get_property(42)
        expect(ok):toBe(false)
        expect(tostring(err):find("first argument must be a property name")).not_:toBe(nil)
    end)

    test("boolean引数でエラーが発生する（type error）", function()
        setup()
        local ok, err = resume_get_property(true)
        expect(ok):toBe(false)
        expect(tostring(err):find("first argument must be a property name")).not_:toBe(nil)
    end)

    test("空テーブルでエラーが発生する（at least one property name required）", function()
        setup()
        local ok, err = resume_get_property({})
        expect(ok):toBe(false)
        expect(tostring(err):find("at least one property name required")).not_:toBe(nil)
    end)

    test("テーブル内に空文字列を含む場合エラーが発生する", function()
        setup()
        local ok, err = resume_get_property({ "valid", "" })
        expect(ok):toBe(false)
        expect(tostring(err):find("name must not be nil or empty")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Group B: メインスレッド呼び出しテスト
-- Requirement 2.1
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - main thread error", function()
    test("メインスレッドから呼び出すとエラーが発生する", function()
        setup()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())
        local ok, err = pcall(function() act:get_property("name") end)
        expect(ok):toBe(false)
        expect(tostring(err):find("must be called inside a scene coroutine")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Group C: タグ生成テスト（単一 string 引数）
-- Requirements: 3.1, 4.1
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - single string tag generation", function()
    test("単一 string 引数で get,property タグが蓄積されyieldされる", function()
        setup()
        local ok, yielded = resume_get_property("baseware.version")
        expect(ok):toBe(true)
        -- yielded にはビルド結果が含まれ、get,property タグが入っている
        expect(type(yielded)):toBe("string")
        expect(yielded:find("\\!%[get,property,OnPastaCallBack1,baseware%.version%]")).not_:toBe(nil)
    end)

    test("ステージングが発生し _staged が設定される", function()
        setup()
        local CALLBACK = require("pasta.shiori.event.callback")

        resume_get_property("test.prop")
        -- consume_staged が呼ばれていないため pending には登録されない
        -- ただし _staged は内部で設定されている（consume_staged を呼ぶまで残る）
        -- 直接 pending を確認：consume_staged 未呼び出しなので nil
        expect(CALLBACK.pending["OnPastaCallBack1"]):toBe(nil)
    end)

    test("連続する get_property 呼び出しで ID がインクリメントされる", function()
        setup()
        local CALLBACK = require("pasta.shiori.event.callback")

        -- 1回目
        local ok1, yielded1, co1 = resume_get_property("prop1")
        expect(ok1):toBe(true)
        expect(yielded1:find("OnPastaCallBack1")).not_:toBe(nil)

        -- consume_staged しないと次の stage_pending でエラーになるため、手動で消費
        CALLBACK.consume_staged(co1, {})

        -- 2回目
        local ok2, yielded2 = resume_get_property("prop2")
        expect(ok2):toBe(true)
        expect(yielded2:find("OnPastaCallBack2")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Group D: タグ生成テスト（table 引数・多値）
-- Requirements: 4.2, 4.3
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - table argument tag generation", function()
    test("table 引数 {\"n1\",\"n2\"} で複数プロパティ名を含むタグが生成される", function()
        setup()
        local ok, yielded = resume_get_property({ "n1", "n2" })
        expect(ok):toBe(true)
        expect(type(yielded)):toBe("string")
        expect(yielded:find("\\!%[get,property,OnPastaCallBack1,n1,n2%]")).not_:toBe(nil)
    end)

    test("table 引数 {\"a\",\"b\",\"c\"} で3つのプロパティ名がタグに含まれる", function()
        setup()
        local ok, yielded = resume_get_property({ "a", "b", "c" })
        expect(ok):toBe(true)
        expect(yielded:find("\\!%[get,property,OnPastaCallBack1,a,b,c%]")).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Group E: escape_tag_arg テスト（get_property 経由）
-- Requirement 4.3
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - escape_tag_arg via tag", function()
    test("カンマを含むプロパティ名がクォートされる", function()
        setup()
        local ok, yielded = resume_get_property("foo,bar")
        expect(ok):toBe(true)
        -- foo,bar → "foo,bar"
        expect(yielded:find('"foo,bar"')).not_:toBe(nil)
    end)

    test("引用符を含むプロパティ名がダブルクォートでエスケープされる", function()
        setup()
        local ok, yielded = resume_get_property('say "hi"')
        expect(ok):toBe(true)
        -- say "hi" → "say ""hi"""
        expect(yielded:find('"say ""hi"""')).not_:toBe(nil)
    end)
end)

-- ============================================================================
-- Group F: timeout デフォルトメッセージテスト
-- Requirement 5.4
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - default timeout_message", function()
    test("timeout のみ指定時にデフォルト timeout_message がステージングに反映される", function()
        setup()
        local CALLBACK = require("pasta.shiori.event.callback")

        local _, _, co = resume_get_property("test.prop", 10) -- timeout=10, timeout_message=nil

        -- consume_staged で pending に移し、on_timeout を検証
        CALLBACK.consume_staged(co, {})
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"].on_timeout):toBe("callback timeout: get_property")
    end)

    test("timeout_message を明示指定した場合その値が使われる", function()
        setup()
        local CALLBACK = require("pasta.shiori.event.callback")

        local _, _, co = resume_get_property("test.prop", 10, "custom timeout")

        CALLBACK.consume_staged(co, {})
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"].on_timeout):toBe("custom timeout")
    end)

    test("timeout 未指定時のデフォルト timeout_message も正しい", function()
        setup()
        local CALLBACK = require("pasta.shiori.event.callback")

        local _, _, co = resume_get_property("test.prop") -- timeout=nil, timeout_message=nil

        CALLBACK.consume_staged(co, {})
        expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
        expect(CALLBACK.pending["OnPastaCallBack1"].on_timeout):toBe("callback timeout: get_property")
    end)
end)

-- ============================================================================
-- Group G: 時刻の飛びをまたぐ待ち時間（tutorial-ssp-gaps Task 8.3）
-- Requirements: 2.5, 7.3, 7.5
-- 締め切りと掃引の今は なめらかな時計（CLOCK.steady）で数える。os.time() は
-- SSP の req.date.unix と同じ OS の時計なので、飛びに合わせて差し替える。
-- ============================================================================

describe("SHIORI_ACT_IMPL.get_property() - 時刻の飛びをまたぐ待ち時間", function()
    local function mk_date(u)
        local t = os.date("*t", u)
        return {
            unix = u, year = t.year, month = t.month, day = t.day, hour = t.hour,
            min = t.min, sec = t.sec, ns = 0, yday = t.yday, ordinal = t.yday,
            wday = t.wday - 1, num_days_from_sunday = t.wday - 1,
        }
    end

    test("飛ぶ前に待ち始めたコールバックは、飛んだ直後でなく残りの実時間の後に打ち切られる", function()
        setup()
        package.loaded["pasta.shiori.event.second_change"] = nil
        package.loaded["pasta.shiori.event.register"] = nil
        local CLOCK = require("pasta.shiori.clock")
        CLOCK._reset()
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        dispatcher._reset()
        dispatcher._set_scene_executor(function() return nil end)
        require("pasta.shiori.event.second_change")
        local REG = require("pasta.shiori.event.register")
        local SHIORI_ACT = require("pasta.shiori.act")
        local CALLBACK = require("pasta.shiori.event.callback")

        local real_time = os.time
        local fake = 1000
        os.time = function(t) -- luacheck: ignore 122
            if t ~= nil then return real_time(t) end
            return fake
        end

        local function tick(u)
            fake = u
            local req = { id = "OnSecondChange", status = "idle", date = mk_date(u) }
            CLOCK.observe(req)
            return REG.OnSecondChange(SHIORI_ACT.new(create_mock_actors(), req))
        end

        local ok, err = pcall(function()
            tick(1000)
            local act = SHIORI_ACT.new(create_mock_actors(), { id = "OnTalk", date = mk_date(1000) })
            local co = coroutine.create(function() act:get_property("x", 5) end)
            assert(coroutine.resume(co))
            CALLBACK.consume_staged(co, act)

            expect(tick(1001)):toBeNil()
            -- 1 秒たって 3 時間先へ設定し直す（飛び 1 回。なめらかな時計は 1002）
            expect(tick(1002 + 3 * 3600)):toBeNil()
            expect(CLOCK._get_state().epoch):toBe(1)
            expect(CALLBACK.pending["OnPastaCallBack1"]).not_:toBe(nil)
            -- なめらかな時計で 1003〜1005 は待ち続け、1006 で打ち切る
            for s = 1003, 1005 do
                expect(tick(s + 3 * 3600)):toBeNil()
            end
            expect(tick(1006 + 3 * 3600)).not_:toBeNil()
            expect(CALLBACK.pending["OnPastaCallBack1"]):toBeNil()
        end)
        os.time = real_time -- luacheck: ignore 122
        CLOCK._reset()
        dispatcher._reset()
        if not ok then error(err, 0) end
    end)
end)
