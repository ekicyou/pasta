-- actor-line-newlines のテスト
-- act の行の頭の印（act:line）と、次に積む台詞の継ぎ目（seams）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local function new_act(line_newlines)
    local ACT = require("pasta.act")
    local act = ACT.new({ sakura = { name = "さくら", spot = 0 } })
    act._line_newlines = line_newlines
    return act, act.actors.sakura
end

describe("act:line - 行の頭の印 (2.1, 6.1)", function()
    test("自分を返す", function()
        local act = new_act(1.8)
        expect(act:line()):toBe(act)
    end)

    test("値があれば次の台詞の頭に継ぎ目（位置 1）が付き、印は消える", function()
        local act, sakura = new_act(1.8)
        act:line():talk(sakura, "こんにちは"):talk(sakura, "続き")
        expect(act.token[1].seams):toEqual({ 1 })
        expect(act.token[2].seams):toBeNil()
    end)

    test("さくらスクリプトのトークンにも付く", function()
        local act, sakura = new_act(1.8)
        act:line():sakura_script(sakura, "\\s[5]")
        expect(act.token[1].seams):toEqual({ 1 })
    end)

    test("raw_script には付けず、印は次の台詞へ残る", function()
        local act, sakura = new_act(1.8)
        act:line():raw_script("\\![raise,x]"):talk(sakura, "字")
        expect(act.token[1].seams):toBeNil()
        expect(act.token[2].seams):toEqual({ 1 })
    end)

    test("talk(nil) は積まず、印を次の台詞へ持ち越す", function()
        local act, sakura = new_act(1.8)
        act:line():talk(sakura, nil):talk(sakura, "字")
        expect(#act.token):toBe(1)
        expect(act.token[1].seams):toEqual({ 1 })
    end)

    test("値が 0 なら印の後のトークンに継ぎ目の欄が無い", function()
        local act, sakura = new_act(0)
        act:line():talk(sakura, "字"):line():sakura_script(sakura, "\\s[5]")
        expect(act.token[1]):toEqual({ type = "talk", actor = sakura, text = "字" })
        expect(act.token[2]):toEqual({ type = "sakura_script", actor = sakura, text = "\\s[5]" })
    end)

    test("値が nil（基の act）なら印の後のトークンに継ぎ目の欄が無い", function()
        local act, sakura = new_act(nil)
        act:line():talk(sakura, "字")
        expect(act.token[1]):toEqual({ type = "talk", actor = sakura, text = "字" })
    end)
end)
