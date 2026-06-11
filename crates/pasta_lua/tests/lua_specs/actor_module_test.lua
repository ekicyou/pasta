-- pasta.actor（ACTOR / ActorWordBuilder / プロキシポストプロセス）単体テスト
-- review-improvement-loop cell 3.46 (G1): 既存 proxy_find_handler_test は検索段のみを
-- カバーしており、get_or_create のキャッシュ・ActorWordBuilder・
-- proxy:word / proxy:expr_fn のポストプロセス分岐が未カバーだった
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 先行スイートのモジュール個別リロードによる STORE 参照分裂を避けるため、
-- 整合した同一インスタンス集合を一括で新規ロードする（既存スイートと同一規約）。
package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil
package.loaded["pasta.scene"] = nil
package.loaded["pasta.global"] = nil
package.loaded["pasta.actor"] = nil
package.loaded["pasta.act"] = nil

local ACTOR = require("pasta.actor")
local ACT = require("pasta.act")
local STORE = require("pasta.store")

describe("ACTOR.get_or_create", function()
    test("新規アクターは name 設定済み・spot nil で生成される", function()
        local actor = ACTOR.get_or_create("spec346_新人")
        expect(actor.name):toBe("spec346_新人")
        expect(actor.spot):toBeNil()
    end)

    test("同名アクターはキャッシュされ同一オブジェクトを返す", function()
        local a1 = ACTOR.get_or_create("spec346_同一")
        local a2 = ACTOR.get_or_create("spec346_同一")
        expect(a1):toBe(a2)
        expect(STORE.actors["spec346_同一"]):toBe(a1)
    end)

    test("異なる名前は異なるオブジェクトを返す", function()
        local a = ACTOR.get_or_create("spec346_甲")
        local b = ACTOR.get_or_create("spec346_乙")
        expect(a == b):toBe(false)
    end)
end)

describe("ACTOR - ActorWordBuilder（actor:create_word）", function()
    test("entry がアクター単語レジストリへ登録される", function()
        local actor = ACTOR.get_or_create("spec346_語り手")
        actor:create_word("好物"):entry("カレー", "ラーメン")

        local words = STORE.actor_words["spec346_語り手"]
        expect(words):toBeTruthy()
        expect(#words["好物"]):toBe(1)
        expect(words["好物"][1][1]):toBe("カレー")
        expect(words["好物"][1][2]):toBe("ラーメン")
    end)

    test("引数なしの entry() は登録せず、ビルダーは self を返す（チェーン可）", function()
        local actor = ACTOR.get_or_create("spec346_寡黙")
        local builder = actor:create_word("口癖")
        local ret = builder:entry()
        expect(ret):toBe(builder)
        expect(#STORE.actor_words["spec346_寡黙"]["口癖"]):toBe(0)

        builder:entry("ふむ"):entry("なるほど")
        expect(#STORE.actor_words["spec346_寡黙"]["口癖"]):toBe(2)
    end)
end)

describe("PROXY - word ポストプロセス", function()
    --- アクターとプロキシを組み立てる（act.アクター名 アクセスでプロキシ生成）
    local function make_proxy(actor_name, actor_fields)
        local actor = ACTOR.get_or_create(actor_name)
        for k, v in pairs(actor_fields or {}) do
            actor[k] = v
        end
        local act = ACT.new({ [actor_name] = actor })
        return act[actor_name], act
    end

    test("name が nil / 空文字列のとき nil を返す", function()
        local proxy = make_proxy("spec346_proxy_nil")
        expect(proxy:word(nil)):toBeNil()
        expect(proxy:word("")):toBeNil()
    end)

    test("A1 ヒットが関数のときプロキシ自身を引数に実行し戻り値を返す", function()
        local received = nil
        local proxy = make_proxy("spec346_proxy_fn", {
            口上 = function(p)
                received = p
                return "口上テキスト"
            end,
        })
        expect(proxy:word("口上")):toBe("口上テキスト")
        expect(received):toBe(proxy)
    end)

    test("A1 ヒットが非関数のとき tostring した文字列を返す", function()
        local proxy = make_proxy("spec346_proxy_num", { 年齢 = 17 })
        expect(proxy:word("年齢")):toBe("17")
    end)
end)

describe("PROXY - expr_fn ポストプロセス", function()
    test("ハンドラー関数にプロキシと可変引数が伝搬し戻り値を返す", function()
        local actor = ACTOR.get_or_create("spec346_式者")
        local act = ACT.new({ spec346_式者 = actor })
        local received_self = nil
        act.current_scene = {
            加算 = function(p, a, b)
                received_self = p
                return a + b
            end,
        }
        local proxy = act.spec346_式者

        expect(proxy:expr_fn("加算", 2, 3)):toBe(5)
        expect(received_self):toBe(proxy)
    end)

    test("ハンドラーが非関数のとき nil を返す（警告のみ）", function()
        local actor = ACTOR.get_or_create("spec346_式者2")
        local act = ACT.new({ spec346_式者2 = actor })
        act.current_scene = { 定数 = "関数ではない" }
        local proxy = act.spec346_式者2

        expect(proxy:expr_fn("定数")):toBeNil()
    end)
end)
