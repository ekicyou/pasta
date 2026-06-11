-- pasta.act - word / expr_fn ポストプロセス・call nil キーガードのテスト
-- review-improvement-loop cell 3.46 (G1): 既存 act_find_act_handler_test は検索段のみを
-- カバーしており、act:word / act:expr_fn のポストプロセス分岐と
-- act:call の nil キーガードが未カバーだった
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local ACT = require("pasta.act")

-- スイート実行順に依存しないよう @pasta_search を明示的に不在へ固定して body を実行し、
-- 実行後に元のキャッシュ状態へ復元する
-- （pcall require 失敗 → L2/L5 スキップで決定的に全ミスとなる）
local function with_search_absent(body)
    local saved_search = package.loaded["@pasta_search"]
    package.loaded["@pasta_search"] = nil
    local ok, err = pcall(body)
    package.loaded["@pasta_search"] = saved_search
    if not ok then error(err, 0) end
end

describe("ACT - word ポストプロセス", function()
    test("name が nil / 空文字列のとき nil を返す", function()
        local act = ACT.new({})
        expect(act:word(nil)):toBeNil()
        expect(act:word("")):toBeNil()
    end)

    test("ハンドラーが関数のとき act 自身を引数に実行し戻り値を返す", function()
        local act = ACT.new({})
        local received = nil
        -- 注: 非 ASCII キーは luacheck 標準パーサー非対応のためブラケット記法で記述
        -- （ドット記法と等価・キー文字列は同一）
        act.current_scene = {
            ["挨拶"] = function(a)
                received = a
                return "こんにちは"
            end,
        }
        expect(act:word("挨拶")):toBe("こんにちは")
        expect(received):toBe(act)
    end)

    test("ハンドラーが数値のとき tostring した文字列を返す", function()
        local act = ACT.new({})
        act.current_scene = { ["回数"] = 42 }
        expect(act:word("回数")):toBe("42")
    end)

    test("ハンドラーが false のとき 'false' を返す（L1 は false ~= nil でヒット扱い）", function()
        local act = ACT.new({})
        act.current_scene = { ["フラグ"] = false }
        expect(act:word("フラグ")):toBe("false")
    end)

    test("全レベルミスのとき nil を返す（警告のみ・エラーなし）", function()
        with_search_absent(function()
            local act = ACT.new({})
            act.current_scene = nil
            expect(act:word("spec346_存在しない単語")):toBeNil()
        end)
    end)
end)

describe("ACT - expr_fn ポストプロセス", function()
    test("ハンドラー関数に act と可変引数が伝搬し戻り値を返す", function()
        local act = ACT.new({})
        local received_self = nil
        act.current_scene = {
            ["加算"] = function(a, x, y)
                received_self = a
                return x + y
            end,
        }
        expect(act:expr_fn("加算", 10, 20)):toBe(30)
        expect(received_self):toBe(act)
    end)

    test("ハンドラーが非関数（文字列）のとき nil を返す", function()
        local act = ACT.new({})
        act.current_scene = { ["定数"] = "関数ではない" }
        expect(act:expr_fn("定数")):toBeNil()
    end)

    test("ハンドラー未発見のとき nil を返す", function()
        with_search_absent(function()
            local act = ACT.new({})
            expect(act:expr_fn("spec346_未定義関数")):toBeNil()
        end)
    end)
end)

describe("ACT - call の nil キーガード", function()
    test("key が nil のときシーン検索せず nil を返す（エラーなし）", function()
        local act = ACT.new({})
        local ok, result = pcall(function()
            return act:call(nil, nil, nil)
        end)
        expect(ok):toBe(true)
        expect(result):toBeNil()
    end)

    test("key が nil でもトークンに副作用を残さない", function()
        local act = ACT.new({})
        act:call(nil, nil, nil)
        expect(#act.token):toBe(0)
    end)
end)
