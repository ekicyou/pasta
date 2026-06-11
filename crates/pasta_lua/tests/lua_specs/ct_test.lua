-- ct.lua（キャンセルトークン）単体テスト
-- review-improvement-loop cell 3.46 (G1): 完全未テストだった ct モジュールの公開挙動を固定する
--
-- 注意: ct.lua の IMPL には __index が設定されていないため、
-- インスタンス経由のメソッド呼び出し（obj:defer(...)）は現状利用できない（潜在バグ・申し送り）。
-- 本テストは getmetatable 経由で実装関数を直接呼び出し、現行挙動を文書化する。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local new_ct = require("ct")

describe("ct - new()", function()
    test("new() がテーブルを返し、初期状態は _cleanups 空・_cancelled false", function()
        local ct = new_ct()
        expect(type(ct)):toBe("table")
        expect(type(ct._cleanups)):toBe("table")
        expect(#ct._cleanups):toBe(0)
        expect(ct._cancelled):toBe(false)
    end)

    test("メタテーブルが __close / defer / cancel を提供する", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        expect(type(mt.__close)):toBe("function")
        expect(type(mt.defer)):toBe("function")
        expect(type(mt.cancel)):toBe("function")
    end)

    test("【現行挙動の文書化】IMPL に __index がなくインスタンス経由のメソッド参照は nil", function()
        -- 潜在バグ: setmetatable(obj, IMPL) だが IMPL.__index 未設定のため
        -- obj:defer(...) / obj:cancel() は呼び出せない（CONCERNS で申し送り）。
        -- この挙動が変更された（修正された）場合、本テストを意図的に更新すること。
        local ct = new_ct()
        expect(ct.defer):toBeNil()
        expect(ct.cancel):toBeNil()
    end)
end)

describe("ct - defer / __close", function()
    test("defer は関数を登録し self を返す（チェーン可）", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local fn = function() end
        local ret = mt.defer(ct, fn)
        expect(ret):toBe(ct)
        expect(#ct._cleanups):toBe(1)
        expect(ct._cleanups[1]):toBe(fn)
    end)

    test("__close は登録逆順（LIFO）でクリーンアップを実行する", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local order = {}
        mt.defer(ct, function() table.insert(order, "a") end)
        mt.defer(ct, function() table.insert(order, "b") end)
        mt.defer(ct, function() table.insert(order, "c") end)

        mt.__close(ct, nil)

        expect(#order):toBe(3)
        expect(order[1]):toBe("c")
        expect(order[2]):toBe("b")
        expect(order[3]):toBe("a")
    end)

    test("__close はエラー引数を各クリーンアップ関数へ伝搬する", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local received = "unset"
        mt.defer(ct, function(err) received = err end)

        mt.__close(ct, "boom")

        expect(received):toBe("boom")
    end)

    test("クリーンアップ中のエラーは pcall で隔離され残りも実行される", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local ran = {}
        mt.defer(ct, function() table.insert(ran, "first") end)
        mt.defer(ct, function() error("cleanup failure") end)
        mt.defer(ct, function() table.insert(ran, "last") end)

        -- __close 自体はエラーを送出しない
        mt.__close(ct, nil)

        -- LIFO: last → (エラー) → first の順で、エラー後も first が実行される
        expect(#ran):toBe(2)
        expect(ran[1]):toBe("last")
        expect(ran[2]):toBe("first")
    end)

    test("__close 後はリストがクリアされ、2回目の __close は何も実行しない", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local count = 0
        mt.defer(ct, function() count = count + 1 end)

        mt.__close(ct, nil)
        mt.__close(ct, nil)

        expect(count):toBe(1)
        expect(#ct._cleanups):toBe(0)
    end)
end)

describe("ct - cancel", function()
    test("cancel 後の __close はクリーンアップを実行しない", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        local count = 0
        mt.defer(ct, function() count = count + 1 end)

        mt.cancel(ct)
        mt.__close(ct, nil)

        expect(count):toBe(0)
    end)

    test("cancel はフラグを立てリストを空にする（不可逆: 以後の defer も実行されない）", function()
        local ct = new_ct()
        local mt = getmetatable(ct)
        mt.defer(ct, function() end)

        mt.cancel(ct)
        expect(ct._cancelled):toBe(true)
        expect(#ct._cleanups):toBe(0)

        -- cancel 後に defer しても _cancelled は維持され __close は早期リターンする
        local count = 0
        mt.defer(ct, function() count = count + 1 end)
        mt.__close(ct, nil)
        expect(count):toBe(0)
    end)
end)
