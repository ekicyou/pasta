-- transfer_req_to_var テストスイート
-- Tests for SHIORI_ACT.transfer_req_to_var() method
-- Requirements: 1.1-1.4, 2.1-2.2, 3.1-3.5, 4.2-4.3, 5.1-5.3
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- Mock actors for testing
local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = "sakura" },
        kero = { name = "うにゅう", spot = "kero" },
    }
end

-- ============================================================================
-- Task 2.1: 基本転記テスト
-- Requirements: 1.1, 1.3, 2.1, 2.2
-- ============================================================================

describe("SHIORI_ACT - transfer_req_to_var() 基本転記", function()
    -- 全角キー（ｒ０〜ｒ９）への転記検証
    test("transfers reference[0]-[9] to fullwidth keys ｒ０-ｒ９", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnMouseClick",
            base_id = "OnMouseClick",
            reference = {
                [0] = "0",
                [1] = "button1",
                [2] = "value2",
                [3] = "value3",
                [4] = "value4",
                [5] = "value5",
                [6] = "value6",
                [7] = "value7",
                [8] = "value8",
                [9] = "value9",
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_req_to_var()

        -- 全角キー確認（ｒ０〜ｒ９）
        expect(act.var["ｒ０"]):toBe("0")
        expect(act.var["ｒ１"]):toBe("button1")
        expect(act.var["ｒ２"]):toBe("value2")
        expect(act.var["ｒ３"]):toBe("value3")
        expect(act.var["ｒ４"]):toBe("value4")
        expect(act.var["ｒ５"]):toBe("value5")
        expect(act.var["ｒ６"]):toBe("value6")
        expect(act.var["ｒ７"]):toBe("value7")
        expect(act.var["ｒ８"]):toBe("value8")
        expect(act.var["ｒ９"]):toBe("value9")
    end)

    -- 半角キー（r0〜r9）への転記検証
    test("transfers reference[0]-[9] to halfwidth keys r0-r9", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnMouseClick",
            base_id = "OnMouseClick",
            reference = {
                [0] = "head",
                [1] = "button2",
                [2] = "extra",
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_req_to_var()

        -- 半角キー確認（r0〜r9）
        expect(act.var.r0):toBe("head")
        expect(act.var.r1):toBe("button2")
        expect(act.var.r2):toBe("extra")
    end)

    -- イベントメタデータ（req_id, req_base_id）の転記検証
    test("transfers req.id to var.req_id and req.base_id to var.req_base_id", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnMouseDoubleClick",
            base_id = "OnMouseClick",
            reference = { [0] = "0" },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_req_to_var()

        expect(act.var.req_id):toBe("OnMouseDoubleClick")
        expect(act.var.req_base_id):toBe("OnMouseClick")
    end)
end)

-- ============================================================================
-- Task 2.2: 境界条件・ガード句テスト
-- Requirements: 1.2, 1.4, 3.5
-- ============================================================================

describe("SHIORI_ACT - transfer_req_to_var() 境界条件", function()
    -- req = nil の場合のガード句動作検証
    test("returns self safely when req is nil", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()

        local act = SHIORI_ACT.new(actors, nil)
        local result = act:transfer_req_to_var()

        -- クラッシュしない、self を返す
        expect(result):toBe(act)
    end)

    -- 部分欠落 reference（疎配列）での正常動作確認
    test("handles sparse reference array correctly (only [0] and [2] exist)", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnMouseClick",
            reference = {
                [0] = "head",
                -- [1] は nil（欠落）
                [2] = "extra",
                -- [3]〜[9] は nil（欠落）
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        act:transfer_req_to_var()

        -- 存在するキーのみ転記される
        expect(act.var["ｒ０"]):toBe("head")
        expect(act.var["ｒ１"]):toBe(nil)
        expect(act.var["ｒ２"]):toBe("extra")
        expect(act.var["ｒ３"]):toBe(nil)
        expect(act.var.r0):toBe("head")
        expect(act.var.r1):toBe(nil)
        expect(act.var.r2):toBe("extra")
        expect(act.var.r3):toBe(nil)
    end)

    -- メソッドチェーンの検証（戻り値が self であること）
    test("returns self for method chaining", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnBoot",
            reference = { [0] = "test" },
        }

        local act = SHIORI_ACT.new(actors, req)
        local result = act:transfer_req_to_var()

        expect(result):toBe(act)
    end)
end)

-- ============================================================================
-- Task 2.3: 統合・副作用テスト
-- Requirements: 3.2, 3.4, 5.1, 5.2
-- ============================================================================

describe("SHIORI_ACT - transfer_req_to_var() 統合テスト", function()
    -- 未呼出時の var 未設定検証
    test("does not set req-derived keys in var when not called", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnMouseClick",
            reference = { [0] = "head", [1] = "button1" },
        }

        local act = SHIORI_ACT.new(actors, req)
        -- transfer_req_to_var() を呼ばない

        -- req 由来キーが var に存在しない
        expect(act.var["ｒ０"]):toBe(nil)
        expect(act.var["ｒ１"]):toBe(nil)
        expect(act.var.r0):toBe(nil)
        expect(act.var.r1):toBe(nil)
        expect(act.var.req_id):toBe(nil)
        expect(act.var.req_base_id):toBe(nil)
    end)

    -- transfer_date_to_var() との共存検証
    test("coexists with transfer_date_to_var() without key conflicts", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local req = {
            id = "OnSecondChange",
            base_id = "OnSecondChange",
            date = {
                year = 2026,
                month = 2,
                day = 7,
                hour = 14,
                min = 30,
                sec = 45,
                wday = 6,
            },
            reference = {
                [0] = "ref0",
                [1] = "ref1",
            },
        }

        local act = SHIORI_ACT.new(actors, req)
        -- 両メソッドを順に呼び出し
        act:transfer_date_to_var()
        act:transfer_req_to_var()

        -- date 由来キーが存在する
        expect(act.var.year):toBe(2026)
        expect(act.var.month):toBe(2)
        expect(act.var.day):toBe(7)
        expect(act.var.hour):toBe(14)
        expect(act.var.min):toBe(30)
        expect(act.var.sec):toBe(45)
        expect(act.var.wday):toBe(6)
        expect(act.var["年"]):toBe("2026年")
        expect(act.var["曜日"]):toBe("土曜日")
        expect(act.var.week):toBe("Saturday")
        expect(act.var["時１２"]):toBe("午後2時")

        -- req 由来キーも存在する（共存）
        expect(act.var["ｒ０"]):toBe("ref0")
        expect(act.var["ｒ１"]):toBe("ref1")
        expect(act.var.r0):toBe("ref0")
        expect(act.var.r1):toBe("ref1")
        expect(act.var.req_id):toBe("OnSecondChange")
        expect(act.var.req_base_id):toBe("OnSecondChange")
    end)
end)
