-- external-control-events: 外からの操作のイベントの既定のハンドラの単体テスト
-- まずは単体テストの土台が本物の @json を出していることを確かめる。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("単体テストの土台の @json", function()
    test("null は userdata、false は真偽値で返る", function()
        local json = require("@json")
        expect(type(json.decode("null"))):toBe("userdata")
        expect(json.decode("false")):toBe(false)
    end)
end)
