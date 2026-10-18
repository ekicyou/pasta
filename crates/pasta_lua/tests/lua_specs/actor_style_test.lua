-- actor_style_test.lua
-- pasta.shiori.actor_style 装いの出し入れ（状態表・出し直し・素通し）テスト
-- actor-display-style Task 2.3 (Requirements: 2.2, 2.3, 2.6, 2.8, 2.9, 3.1, 3.2, 4.2, 4.3, 5.1, 5.2)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local STYLE = require("pasta.shiori.actor_style")

local F1 = "\\f[color,90,110,140]\\f[bold,1]"
local F2 = "\\f[color,200,0,0]"
local RESET = "\\f[default]"

local OJI = { name = "おじさん", font = F1 }
local GAL = { name = "ギャル", font = F2 }
local PASTA = { name = "ぱすた" }

--- 装いのあるアクターが現れたビルドの状態（スポット 0 を Styled にする）
local function styled_state()
    local state = STYLE.new()
    STYLE.switch(state, OJI, 0)
    return state
end

describe("STYLE.new", function()
    test("ビルドごとに新しい空の状態を返す（2.8）", function()
        local s1 = STYLE.new()
        local s2 = STYLE.new()
        expect(s1 == s2):toBe(false)
        expect(next(s1.used)):toBe(nil)
        expect(next(s1.styled)):toBe(nil)
        expect(s1.any):toBe(false)
    end)
end)

describe("STYLE.switch - 状態表", function()
    test("Fresh → 装いのあるアクター: 装いのタグだけ", function()
        local state = STYLE.new()
        expect(STYLE.switch(state, OJI, 0)):toBe(F1)
        expect(state.used[0]):toBe(true)
        expect(state.styled[0]):toBe(true)
        expect(state.any):toBe(true)
    end)

    test("Fresh → 装いの無いアクター: 何も出さない", function()
        local state = STYLE.new()
        expect(STYLE.switch(state, PASTA, 0)):toBe("")
        expect(state.used[0]):toBe(true)
        expect(state.styled[0]):toBe(nil)
        expect(state.any):toBe(false)
    end)

    test("UsedPlain → 装いのあるアクター: \\f[default] ＋ 装い（4.3）", function()
        local state = STYLE.new()
        STYLE.switch(state, PASTA, 0)
        expect(STYLE.switch(state, OJI, 0)):toBe(RESET .. F1)
        expect(state.styled[0]):toBe(true)
    end)

    test("UsedPlain → 装いの無いアクター: 何も出さない（5.2）", function()
        local state = STYLE.new()
        STYLE.switch(state, PASTA, 0)
        expect(STYLE.switch(state, { name = "B" }, 0)):toBe("")
    end)

    test("Styled → 装いのあるアクター: \\f[default] ＋ 装い（2.6）", function()
        local state = styled_state()
        expect(STYLE.switch(state, GAL, 0)):toBe(RESET .. F2)
    end)

    test("Styled → 装いの無いアクター: \\f[default]（2.2・2.3）", function()
        local state = styled_state()
        expect(STYLE.switch(state, PASTA, 0)):toBe(RESET)
        expect(state.styled[0]):toBe(true)
    end)

    test("別のスポットの状態は見ない", function()
        local state = styled_state()
        expect(STYLE.switch(state, PASTA, 1)):toBe("")
        expect(STYLE.switch(state, GAL, 1)):toBe(RESET .. F2)
    end)

    test("font が空でない文字列のときだけ装いあり（2.9）", function()
        local state = styled_state()
        for _, actor in ipairs({
            { name = "未登録" },
            { name = "空", font = "" },
            { name = "表", font = { color = "1,2,3" } },
            { name = "数", font = 1 },
        }) do
            expect(STYLE.switch(state, actor, 1)):toBe("")
        end
        expect(state.styled[1]):toBe(nil)
    end)
end)

describe("STYLE.through - 素通し", function()
    test("装いのあるアクターが現れないビルドでは入力をそのまま返す（5.1）", function()
        local state = STYLE.new()
        STYLE.switch(state, PASTA, 0)
        local s = "あ\\cい\\xう"
        expect(STYLE.through(state, PASTA, 0, s)):toBe(s)
        expect(STYLE.through(state, OJI, 0, s)):toBe(s)
        expect(state.used[0]):toBe(true)
    end)

    test("話し手なしのグループはそのまま返す", function()
        local state = styled_state()
        expect(STYLE.through(state, nil, 0, "あ\\cい")):toBe("あ\\cい")
    end)
end)

describe("STYLE.through - 装いのあるアクターの出し直し", function()
    test("\\c の直後に装いを差し込む（3.2）", function()
        local state = styled_state()
        expect(STYLE.through(state, OJI, 0, "あ\\cい")):toBe("あ\\c" .. F1 .. "い")
        expect(state.styled[0]):toBe(true)
    end)

    test("\\x の直後に装いを差し込み、スポット 0 も Styled にする（3.1）", function()
        local state = STYLE.new()
        STYLE.switch(state, OJI, 1)
        expect(STYLE.through(state, OJI, 1, "あ\\xい")):toBe("あ\\x" .. F1 .. "い")
        expect(state.used[0]):toBe(true)
        expect(state.styled[0]):toBe(true)
        expect(STYLE.switch(state, PASTA, 0)):toBe(RESET)
    end)

    test("\\x[noclear]・\\c[char,1]・エスケープ・囲みの中では何もしない", function()
        local state = STYLE.new()
        STYLE.switch(state, OJI, 1)
        for _, s in ipairs({
            "あ\\x[noclear]い",
            "あ\\c[char,1]い",
            "あ\\\\cい",
            "あ\\\\xい",
            "\\_?\\c\\x\\_?い",
        }) do
            expect(STYLE.through(state, OJI, 1, s)):toBe(s)
        end
        expect(state.styled[0]):toBe(nil)
    end)

    test("文字を消さず挿入だけ（4.2）", function()
        local state = styled_state()
        local s = "\\f[italic,1]あ\\c\\c\\_?\\c\\_?い\\x\\f[bold,0]う\\\\c"
        local out = STYLE.through(state, OJI, 0, s)
        expect(out):toBe(
            "\\f[italic,1]あ\\c" .. F1 .. "\\c" .. F1 .. "\\_?\\c\\_?い\\x" .. F1 .. "\\f[bold,0]う\\\\c"
        )
        local stripped = out:gsub(F1:gsub("[%[%]%\\]", "%%%0"), "")
        expect(stripped):toBe(s)
    end)
end)

describe("STYLE.through - 装いの無いアクター", function()
    test("\\c でそのスポットを Fresh に戻す", function()
        local state = styled_state()
        STYLE.switch(state, PASTA, 0)
        local s = "あ\\cい"
        expect(STYLE.through(state, PASTA, 0, s)):toBe(s)
        expect(state.used[0]):toBe(false)
        expect(state.styled[0]):toBe(false)
        expect(STYLE.switch(state, GAL, 0)):toBe(F2)
    end)

    test("\\x・\\c[char,1]・エスケープでは状態を変えない", function()
        local state = styled_state()
        STYLE.switch(state, PASTA, 0)
        for _, s in ipairs({ "あ\\xい", "あ\\c[char,1]い", "あ\\\\cい" }) do
            expect(STYLE.through(state, PASTA, 0, s)):toBe(s)
        end
        expect(state.styled[0]):toBe(true)
    end)
end)
