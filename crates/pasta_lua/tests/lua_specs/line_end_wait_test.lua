-- 行末と作者の待ちの判定（find_line_ends）のテスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local function find_line_ends(tokens)
    return require("pasta.shiori.sakura_builder")._internal.find_line_ends(tokens)
end

-- 単語（＠名前）の展開先。生成コードは act:talk(actor, act:word(名前)) と書く
local WORDS = {
    ["笑顔"] = "\\s[5]",
    ["左"] = "\\_l[-283,0]",
    ["語尾"] = "だよ？",
    ["間"] = "\\_w[500]",
}

-- 台詞・タグのテキストに行末の印を差し込んで並べる（作者の待ちが続けば [EW]、続かなければ [E]）。
-- テキストの無いトークンは <型>
local function render(tokens, ends)
    local parts = {}
    for k, inner in ipairs(tokens) do
        if inner.text == nil or (inner.type ~= "talk" and inner.type ~= "sakura_script") then
            parts[#parts + 1] = "<" .. inner.type .. ">"
        else
            local text, out, from = tostring(inner.text), {}, 1
            for _, e in ipairs(ends[k] or {}) do
                out[#out + 1] = text:sub(from, e.pos) .. (e.author_wait and "[EW]" or "[E]")
                from = e.pos + 1
            end
            out[#out + 1] = text:sub(from)
            parts[#parts + 1] = table.concat(out)
        end
    end
    return table.concat(parts, " ")
end

-- 本物の act に積んで組み立て、アクターのまとまりごとに行末を求めて並べる
local function lines(fill, line_newlines)
    local ACT = require("pasta.act")
    local act = ACT.new({
        sakura = { name = "さくら", spot = 0 },
        kero = { name = "うにゅう", spot = 1 },
    })
    act._line_newlines = line_newlines
    act.current_scene = {}
    for k, v in pairs(WORDS) do
        act.current_scene[k] = v
    end
    fill(act, act.actors.sakura, act.actors.kero)
    local out = {}
    for _, group in ipairs(act:build()) do
        if group.type == "actor" then
            out[#out + 1] = render(group.tokens, find_line_ends(group.tokens))
        end
    end
    return out
end

describe("find_line_ends - 戻り値の形", function()
    test("トークンの添字ごとに、元のテキストの行末の最後のバイトと作者の待ちの真偽", function()
        local tokens = {
            { type = "talk", text = "一行目。二行目", seams = { 1, 13 } },
            { type = "sakura_script", text = "\\_w[500]" },
        }
        expect(find_line_ends(tokens)):toEqual({
            [1] = { { pos = 12, author_wait = false }, { pos = 21, author_wait = true } },
        })
    end)

    test("空の並びでは空の表", function()
        expect(find_line_ends({})):toEqual({})
    end)
end)

describe("find_line_ends - 行末の判定", function()
    test("アクション行と継続行の最後の見える字が、どちらも行末", function()
        expect(lines(function(act, s)
            act:line():talk(s, "一行目。"):line():talk(s, "二行目")
        end)):toEqual({ "一行目。[E]二行目[E]" })
    end)

    test("話し手の切り替え・同じ話し手・まとまりの最後を区別しない", function()
        expect(lines(function(act, s, k)
            act:line():talk(s, "A。"):line():talk(s, "B")
            act:line():talk(k, "C？")
            act:line():talk(s, "D")
        end)):toEqual({ "A。[E]B[E]", "C？[E]", "D[E]" })
    end)

    test("行の印の無い台詞は、まとまりの最後の見える字だけが行末", function()
        expect(lines(function(act, s)
            act:talk(s, "ab。"):talk(s, "cd")
        end)):toEqual({ "ab。cd[E]" })
    end)

    test("line_newlines の有無と値によらず同じ", function()
        local function fill(act, s)
            act:line():talk(s, "なら。"):talk(s, act:word("笑顔"))
            act:line():talk(s, act:word("左")):talk(s, "続き"):sakura_script(s, "\\_w[500]")
        end
        local want = { "なら。[E]\\s[5]\\_l[-283,0]続き[EW] \\_w[500]" }
        for _, ratio in ipairs({ false, 0, 1.0, 1.8 }) do
            expect(lines(fill, ratio or nil)):toEqual(want)
        end
    end)
end)

describe("find_line_ends - 見える字を出さないもの", function()
    test("タグだけが続いて終わる行は、その前の見える字が行末", function()
        expect(lines(function(act, s)
            act:line():talk(s, "行ってきます。"):talk(s, act:word("笑顔"))
            act:line():talk(s, "次"):sakura_script(s, "\\s[5]\\n")
        end)):toEqual({ "行ってきます。[E]\\s[5]次[E] \\s[5]\\n" })
    end)

    test("＠左 で始まる次の行は、前の行の行末を変えない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "じゃあね。")
            act:line():talk(s, act:word("左")):talk(s, "続き")
        end)):toEqual({ "じゃあね。[E]\\_l[-283,0]続き[E]" })
    end)

    test("表情だけの行は行末を持たない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "A。"):line():talk(s, act:word("笑顔")):line():talk(s, "B")
        end)):toEqual({ "A。[E]\\s[5]B[E]" })
        expect(lines(function(act, s)
            act:line():talk(s, "A。"):line():sakura_script(s, "\\s[5]")
        end)):toEqual({ "A。[E] \\s[5]" })
    end)

    test("単語の展開した字を見える字に数える", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨"):talk(s, act:word("語尾"))
        end)):toEqual({ "雨だよ？[E]" })
    end)

    test("字を表示するタグと中身のある囲みは見える字", function()
        expect(lines(function(act, s)
            act:line():talk(s, "A"):sakura_script(s, "\\_u[0x3042]")
            act:line():talk(s, "B"):sakura_script(s, "\\_?x\\_?\\s[5]")
        end)):toEqual({ "A \\_u[0x3042][E] B \\_?x\\_?[E]\\s[5]" })
    end)

    test("表情・待ち・raw_script・改行・消去・選択肢の時間切れを読み飛ばす", function()
        expect(lines(function(act, s)
            act:line():talk(s, "A。")
            act:surface(5):wait(100):raw_script("字"):newline():clear():choice_timeout(3)
        end)):toEqual({ "A。[E] <surface> <wait> <raw_script> <newline> <clear> <choice_timeout>" })
    end)
end)

describe("find_line_ends - Call と選択肢", function()
    test("Call の先のシーンの行は自分の行、Call の前の行の最後の字はその行の行末", function()
        expect(lines(function(act, s)
            act.current_scene["季節"] = function(a)
                a:line():talk(s, "春")
            end
            act:line():talk(s, "窓の下の公園、")
            act:call("g", "季節", {})
            act:line():talk(s, "戻った")
        end)):toEqual({ "窓の下の公園、[E]春[E]戻った[E]" })
    end)

    test("Call の先が行の印を付けない Lua の関数なら、前の行の続き", function()
        expect(lines(function(act, s)
            act.current_scene["季節"] = function(a)
                a:talk(s, "春だね")
            end
            act:line():talk(s, "窓の下の公園、")
            act:call("g", "季節", {})
        end)):toEqual({ "窓の下の公園、春だね[E]" })
    end)

    test("選択肢の前で行を閉じる", function()
        expect(lines(function(act, s)
            act:line():talk(s, "どうする？"):choice("x"):talk(s, "B"):choice("y")
        end)):toEqual({ "どうする？[E] <choice> B[E] <choice>" })
    end)
end)

describe("find_line_ends - 作者の待ち", function()
    test("辞書に直接書いた \\_w[…]（次のタグのトークンの頭）", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):sakura_script(s, "\\_w[500]")
        end)):toEqual({ "雨？[EW] \\_w[500]" })
    end)

    test("＠単語 が展開した \\_w[…]（同じ台詞の中）", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):talk(s, act:word("間"))
        end)):toEqual({ "雨？[EW]\\_w[500]" })
    end)

    test("2 つ続く \\_w", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):sakura_script(s, "\\_w[500]\\_w[300]")
            act:line():talk(s, "晴れ？"):talk(s, act:word("間")):talk(s, act:word("間"))
        end)):toEqual({ "雨？[EW] \\_w[500]\\_w[300] 晴れ？[EW]\\_w[500]\\_w[500]" })
    end)

    test("次の行の頭の \\_w は数えない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):line():sakura_script(s, "\\_w[500]"):talk(s, "次")
        end)):toEqual({ "雨？[E] \\_w[500] 次[E]" })
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):line():talk(s, act:word("間")):talk(s, "次")
        end)):toEqual({ "雨？[E]\\_w[500]次[E]" })
    end)

    test("\\w5・\\__w[…]・act:wait・raw_script は数えない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "A？"):sakura_script(s, "\\w5")
            act:line():talk(s, "B？"):sakura_script(s, "\\__w[500]")
            act:line():talk(s, "C？"):wait(500)
            act:line():talk(s, "D？"):raw_script("\\_w[500]")
            act:line():talk(s, "E？\\w5\\_w[500]")
        end)):toEqual({
            "A？[E] \\w5 B？[E] \\__w[500] C？[E] <wait> D？[E] <raw_script> E？[E]\\w5\\_w[500]",
        })
    end)

    test("間に \\s[5] があれば数えない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):sakura_script(s, "\\s[5]\\_w[500]")
            act:line():talk(s, "晴れ？"):talk(s, act:word("笑顔")):talk(s, act:word("間"))
        end)):toEqual({ "雨？[E] \\s[5]\\_w[500] 晴れ？[E]\\s[5]\\_w[500]" })
    end)

    test("引数の無い \\_w は数えない", function()
        expect(lines(function(act, s)
            act:line():talk(s, "雨？"):sakura_script(s, "\\_w")
        end)):toEqual({ "雨？[E] \\_w" })
    end)
end)

local function join_lines(inner, column, ratio, ends)
    return require("pasta.shiori.sakura_builder")._internal.join_lines(inner, column, ratio, ends)
end

-- まとまりごとに行末を求め、1 つの列の状態で join_lines を順に通し、写した位置の字を確かめる。
-- 戻り値は差し込んだ後の台詞を並べたもの（行末に [E] を付ける）
local function joined(fill, ratio)
    local ACT = require("pasta.act")
    local act = ACT.new({ sakura = { name = "さくら", spot = 0 } })
    act._line_newlines = ratio
    act.current_scene = {}
    for k, v in pairs(WORDS) do
        act.current_scene[k] = v
    end
    fill(act, act.actors.sakura)
    local out = {}
    for _, group in ipairs(act:build()) do
        if group.type == "actor" then
            local src = find_line_ends(group.tokens)
            local column = { line = 1, width = 0, jump = false }
            local tokens, ends = {}, {}
            for k, inner in ipairs(group.tokens) do
                local orig = inner.text and tostring(inner.text)
                local text, mapped = join_lines(inner, column, ratio, src[k] or {})
                for i, e in ipairs(mapped) do
                    local o = src[k][i].pos
                    local a = o
                    while orig:byte(a) >= 0x80 and orig:byte(a) < 0xC0 do -- UTF-8 の続きのバイト
                        a = a - 1
                    end
                    local ch = orig:sub(a, o)
                    expect(tostring(text):sub(e.pos - #ch + 1, e.pos)):toBe(ch)
                    expect(e.author_wait):toBe(src[k][i].author_wait)
                end
                tokens[k] = { type = inner.type, text = text }
                ends[k] = mapped
            end
            out[#out + 1] = render(tokens, ends)
        end
    end
    return out
end

describe("join_lines - 行末の位置の写し替え", function()
    test("行の改行を差し込むと、行末の位置が差し込んだ後のテキストの行末の字に当たる", function()
        expect(joined(function(act, s)
            act:line():talk(s, "一行目。"):line():talk(s, "二行目。"):line():talk(s, "三行目")
        end, 1.8)):toEqual({ "一行目。[E]\\n[180]二行目。[E]\\n[180]三行目[E]" })
    end)

    test("継続行の頭のタグ・行末の後ろの表情と作者の待ちがあっても当たる", function()
        expect(joined(function(act, s)
            act:line():talk(s, "なら。"):talk(s, act:word("笑顔"))
            act:line():talk(s, act:word("左")):talk(s, "続き"):sakura_script(s, "\\_w[500]")
            act:line():talk(s, "雨？"):talk(s, act:word("間"))
        end, 1.8)):toEqual({ "なら。[E]\\s[5]\\_l[-283,0]続き[EW] \\_w[500] \\n[180]雨？[EW]\\_w[500]" })
    end)

    test("行の印の無い台詞は差し込まず、位置も変わらない", function()
        expect(joined(function(act, s)
            act:talk(s, "ab。"):talk(s, "cd")
        end, 1.8)):toEqual({ "ab。cd[E]" })
    end)

    test("line_newlines が nil・0 なら、テキストも行末も受け取ったまま返す", function()
        for _, ratio in ipairs({ false, 0 }) do
            local inner = { type = "talk", text = "一行目。二行目", seams = { 1, 13 } }
            local ends = { { pos = 12, author_wait = false } }
            local text, out = join_lines(inner, { line = 1, width = 0 }, ratio or nil, ends)
            expect(text):toBe(inner.text)
            expect(out == ends):toBe(true)
        end
    end)

    test("受け取った行末の表を書き換えない", function()
        local ends = { { pos = 12, author_wait = false }, { pos = 21, author_wait = true } }
        local inner = { type = "talk", text = "一行目。二行目", seams = { 1, 13 } }
        local text, out = join_lines(inner, { line = 1, width = 0, joinable = true }, 1.8, ends)
        expect(text):toBe("\\n[180]一行目。\\n[180]二行目")
        expect(ends):toEqual({ { pos = 12, author_wait = false }, { pos = 21, author_wait = true } })
        expect(out):toEqual({ { pos = 19, author_wait = false }, { pos = 35, author_wait = true } })
    end)
end)
