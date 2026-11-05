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

describe("merge_consecutive_talks - 継ぎ目の位置を残す (2.1, 2.5)", function()
    test("3 つの台詞を結合すると、継ぎ目が各台詞の始まりのバイト位置を指す", function()
        local act, sakura = new_act(1.8)
        act:line():talk(sakura, "あ"):line():talk(sakura, "いい"):line():talk(sakura, "う")
        local tokens = act:build()[1].tokens
        expect(#tokens):toBe(1)
        expect(tokens[1].text):toBe("あいいう")
        expect(tokens[1].seams):toEqual({ 1, 4, 10 })
    end)

    test("継ぎ目の無い台詞を飛ばして、後の台詞の位置だけを足す", function()
        local act, sakura = new_act(1.8)
        act:talk(sakura, "ab"):line():talk(sakura, "c"):talk(sakura, "d")
        local tokens = act:build()[1].tokens
        expect(tokens[1].text):toBe("abcd")
        expect(tokens[1].seams):toEqual({ 3 })
    end)

    test("空の台詞の継ぎ目はテキストの終わり（#text + 1）を指せる", function()
        local act, sakura = new_act(1.8)
        act:talk(sakura, "ab"):line():talk(sakura, "")
        local tokens = act:build()[1].tokens
        expect(tokens[1].text):toBe("ab")
        expect(tokens[1].seams):toEqual({ 3 })
    end)

    test("元のトークンの継ぎ目の表を書き換えない", function()
        local act, sakura = new_act(1.8)
        act:line():talk(sakura, "あ"):line():talk(sakura, "い")
        local first, second = act.token[1], act.token[2]
        act:build()
        expect(first.seams):toEqual({ 1 })
        expect(second.seams):toEqual({ 1 })
    end)

    test("継ぎ目が無ければ結合した台詞に seams の欄が無い", function()
        local act, sakura = new_act(0)
        act:line():talk(sakura, "あ"):line():talk(sakura, "い")
        local tokens = act:build()[1].tokens
        expect(tokens[1]):toEqual({ type = "talk", actor = sakura, text = "あい" })
    end)
end)

describe("組み立て - 列の動きの分類（Rust の column_move の写し）(4.1, 4.2)", function()
    -- Rust の test_column_move_classification と同じ単位の並び
    local moving = {
        { "\\n", "next" },
        { "\\n[180]", "next" },
        { "\\n[half]", "next" },
        { "\\c", "first" },
        { "\\_l[-295,0]", "jump" },
    }
    local still = {
        "\\c[line,1]", "\\x", "\\x[noclear]", "\\_q", "\\_w[50]", "\\s[1]", "\\0",
        "\\p[1]", "\\b[2]", "\\\\", "\\%", "\\_?\\n\\_?", "\\_!\\c\\_!",
    }

    local function classify(unit)
        local BUILDER = require("pasta.shiori.sakura_builder")
        local APPEARANCE = require("pasta.shiori.appearance")
        local name, arg = APPEARANCE.tag_at(unit, 1)
        return BUILDER._internal.column_move(name, arg)
    end

    test("列を動かす単位を「次」「最初」「飛ぶ」に分ける", function()
        for _, case in ipairs(moving) do
            expect(classify(case[1])):toBe(case[2])
        end
    end)

    test("列を動かさない単位は nil", function()
        for _, unit in ipairs(still) do
            expect(classify(unit)):toBeNil()
        end
    end)

    test("分類が talk_to_script の列の動きと一致する", function()
        local SAKURA_SCRIPT = require("@pasta_sakura_script")
        local actor = { budoux = { 40 }, budoux_jump = { 6 } }
        -- 分類ごとに、始めの列（line 3）から動いた後の line・jump（nil は jump を変えない）
        local want_line = { next = 4, first = 1, jump = 1 }
        local want_jump = { first = false, jump = true }
        local units = {}
        for _, case in ipairs(moving) do
            units[#units + 1] = case[1]
        end
        for _, unit in ipairs(still) do
            units[#units + 1] = unit
        end
        for _, start_jump in ipairs({ false, true }) do
            for _, unit in ipairs(units) do
                local column = { line = 3, width = 0, jump = start_jump }
                SAKURA_SCRIPT.talk_to_script(actor, unit, column)
                local m = classify(unit)
                expect(column.line):toBe(want_line[m] or 3)
                local jump = want_jump[m]
                if jump == nil then
                    jump = start_jump
                end
                expect(column.jump):toBe(jump)
            end
        end
    end)
end)

describe("組み立て - 行の改行のタグ (1.3, 1.4)", function()
    local function tag(ratio)
        return require("pasta.shiori.sakura_builder")._internal.line_newline_tag(ratio)
    end

    test("N が 100 以外なら \\n[N]", function()
        expect(tag(1.8)):toBe("\\n[180]")
        expect(tag(0.5)):toBe("\\n[50]")
    end)

    test("N が 100 なら \\n", function()
        expect(tag(1.0)):toBe("\\n")
    end)

    test("N が 0 以下なら空", function()
        expect(tag(0.009)):toBe("")
        expect(tag(-1)):toBe("")
    end)
end)
