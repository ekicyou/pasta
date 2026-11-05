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

-- 組み立てに行の改行の値を直接渡して読む（shiori act からの受け渡しは別の作業）。
-- act の印は常に立ててトークンに継ぎ目を付け、組み立ての line_newlines だけを変える
-- sakura_extra はさくらの設定に足す欄（budoux など）
local function build_lines(line_newlines, fill, sakura_extra, config_extra)
    local BUILDER = require("pasta.shiori.sakura_builder")
    local ACT = require("pasta.act")
    local sakura = { name = "さくら", spot = 0 }
    for k, v in pairs(sakura_extra or {}) do
        sakura[k] = v
    end
    local act = ACT.new({
        sakura = sakura,
        kero = { name = "うにゅう", spot = 1 },
    })
    act._line_newlines = 1.8
    fill(act, act.actors.sakura, act.actors.kero)
    local config = { line_newlines = line_newlines, columns_out = {} }
    for k, v in pairs(config_extra or {}) do
        config[k] = v
    end
    local out = BUILDER.build(act:build(), config, { ["さくら"] = 0, ["うにゅう"] = 1 })
    return out, config.columns_out
end

describe("組み立て - 行の改行の差し込み (2.1, 2.2, 2.4, 2.5)", function()
    test("同じ話し手の継ぎ目で 1 つ入る（結合した台詞の途中）", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "A"):line():talk(s, "B")
        end)
        expect(out):toBe("\\p[0]A\\n[180]B\\e")
    end)

    test("字の無い行の後に空の列ができる", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "なら"):line():talk(s, "\\s[5]"):line():talk(s, "次の話")
        end)
        expect(out):toBe("\\p[0]なら\\s[5]\\n[180]\\n[180]次の話\\e")
    end)

    test("字の無い行がさくらスクリプトでも同じ（保留を次の継ぎ目の位置に出す）", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "なら"):line():sakura_script(s, "\\s[5]"):line():talk(s, "次の話")
        end)
        expect(out):toBe("\\p[0]なら\\s[5]\\n[180]\\n[180]次の話\\e")
    end)

    test("さくらスクリプトの頭の継ぎ目でも、最初の見える字の前に入る", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "A"):line():sakura_script(s, "\\s[5]\\_u[0x3042]")
        end)
        expect(out):toBe("\\p[0]A\\s[5]\\n[180]\\_u[0x3042]\\e")
    end)

    test("Call 先の台詞の後の継ぎ目でも入る", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "A") -- 呼ぶ側の行
            act:line():talk(s, "C") -- Call 先の行
            act:line():talk(s, "B") -- 戻った後の行
        end)
        expect(out):toBe("\\p[0]A\\n[180]C\\n[180]B\\e")
    end)

    test("値が 1.0 なら \\n を入れる", function()
        local out = build_lines(1.0, function(act, s)
            act:line():talk(s, "A"):line():talk(s, "B")
        end)
        expect(out):toBe("\\p[0]A\\nB\\e")
    end)
end)

describe("組み立て - 行の改行を入れない所 (1.3, 3.1, 3.2, 3.3, 3.4)", function()
    local function same_as_off(fill)
        local off = build_lines(nil, fill)
        expect(build_lines(1.8, fill)):toBe(off)
        return off
    end

    test("値が nil・0・N が 0 以下なら出力も列の表も今と同じ", function()
        local function fill(act, s)
            act:line():talk(s, "A"):line():talk(s, "B")
        end
        local off, columns = build_lines(nil, fill)
        expect(off):toBe("\\p[0]AB\\e")
        expect(columns[0]):toEqual({ line = 1, width = columns[0].width, jump = false })
        local zero, zero_columns = build_lines(0, fill)
        expect(zero):toBe(off)
        expect(zero_columns[0]):toEqual(columns[0])
        expect(build_lines(0.009, fill)):toBe(off)
    end)

    test("話し手の切り替えの後は spot_newlines だけ", function()
        local out = same_as_off(function(act, s, k)
            act:line():talk(s, "A"):line():talk(k, "B"):line():talk(s, "C")
        end)
        expect(out):toBe("\\p[0]A\\p[1]B\\p[0]\\n[150]C\\e")
    end)

    test("元の話し手へ戻った頭が表情だけでも、spot_newlines だけ（3.1 の不変条件）", function()
        local out = same_as_off(function(act, s, k)
            act:line():talk(s, "A"):line():talk(k, "B")
            act:line():sakura_script(s, "\\s[5]"):line():talk(s, "C")
        end)
        expect(out):toBe("\\p[0]A\\p[1]B\\p[0]\\s[5]\\n[150]C\\e")
    end)

    test("元の話し手へ戻った後に字の無い行が 2 つ続いても、spot_newlines だけ（3.1）", function()
        same_as_off(function(act, s, k)
            act:line():talk(s, "A"):line():talk(k, "B")
            act:line():sakura_script(s, "\\s[5]"):line():sakura_script(s, "\\s[6]"):line():talk(s, "C")
        end)
        same_as_off(function(act, s, k)
            act:line():talk(s, "A"):line():talk(k, "B")
            act:line():talk(s, "\\s[5]"):line():sakura_script(s, "\\_w[500]"):line():talk(s, "C")
        end)
    end)

    test("選択肢の後は after_choice_newlines だけ", function()
        same_as_off(function(act, s)
            act:line():talk(s, "A"):choice("x"):line():talk(s, "B")
        end)
    end)

    test("区切りの幅が 0 でも、話し手の切り替え・選択肢の後には入らない", function()
        local zero = { spot_newlines = 0, choice_newlines = 0, after_choice_newlines = 0 }
        local fills = {
            function(act, s, k)
                act:line():talk(s, "A"):line():talk(k, "B"):line():talk(s, "C")
            end,
            function(act, s)
                act:line():talk(s, "A"):choice("x"):line():talk(s, "B")
            end,
            function(act, s)
                act:line():talk(s, "A"):line():sakura_script(s, "\\s[5]"):choice("x"):line():talk(s, "B")
            end,
        }
        for _, fill in ipairs(fills) do
            expect(build_lines(1.8, fill, nil, zero)):toBe(build_lines(nil, fill, nil, zero))
        end
    end)

    test("保留のまま選択肢が来たら choice_newlines だけ", function()
        same_as_off(function(act, s)
            act:line():talk(s, "A"):line():sakura_script(s, "\\s[5]"):choice("x")
        end)
    end)

    test("\\c の後（台詞の中・clear のトークン）は入らない", function()
        same_as_off(function(act, s)
            act:line():talk(s, "A\\c"):line():talk(s, "B")
        end)
        same_as_off(function(act, s)
            act:line():talk(s, "A"):clear():line():talk(s, "B")
        end)
    end)

    test("出力の最初の行には入らない", function()
        same_as_off(function(act, s)
            act:line():sakura_script(s, "\\s[5]"):line():talk(s, "A")
        end)
    end)

    test("スポットの割り当てを消した後は入らない", function()
        same_as_off(function(act, s)
            act:line():talk(s, "A")
            act:clear_spot() -- self を返さない
            act:line():talk(s, "B")
        end)
    end)
end)

describe("組み立て - 差し込んだ改行で列が進む (5.1)", function()
    test("列が 1 つ進み、幅は 0 から数え直される", function()
        local _, columns = build_lines(1.8, function(act, s)
            act:line():talk(s, "AAAA"):line():talk(s, "BB")
        end)
        local _, only = build_lines(nil, function(act, s)
            act:talk(s, "BB")
        end)
        expect(columns[0].line):toBe(2)
        expect(columns[0].width):toBe(only[0].width)
    end)
end)

-- 4.1・4.2 で入れない継ぎ目は、line_newlines なしとバイトまで同じ（4.5）
local function same_as_off(fill, sakura_extra)
    local off = build_lines(nil, fill, sakura_extra)
    expect(build_lines(1.8, fill, sakura_extra)):toBe(off)
    return off
end

-- budoux のあるアクター（句読点のウェイトは既定のまま入る）
local BUDOUX = { budoux = { 4 } }

describe("組み立て - 行の頭の位置を決めるタグでは入れない (4.1, 4.5)", function()
    local heads = {
        { "\\n[180]\\_u[0x3000]", "相手役の継続行の ＠改行" },
        { "\\![bind,a,b,1]\\n[180]", "表情の bind の後の ＠改行" },
        { "\\_w[2000]\\c", "＠改ページ" },
        { "\\_l[-283,0]", "＠左" },
    }
    for _, case in ipairs(heads) do
        test(case[2] .. "（同じ台詞の頭）", function()
            for _, extra in ipairs({ {}, BUDOUX }) do
                same_as_off(function(act, s)
                    act:line():talk(s, "長い記憶。"):line():talk(s, case[1] .. "続き、ね。")
                end, extra)
            end
        end)
        test(case[2] .. "（さくらスクリプトの頭）", function()
            for _, extra in ipairs({ {}, BUDOUX }) do
                same_as_off(function(act, s)
                    act:line():talk(s, "長い記憶。"):line():sakura_script(s, case[1]):talk(s, "続き、ね。")
                end, extra)
            end
        end)
    end
end)

describe("組み立て - 前の出力が位置を決めるタグで終わるなら入れない (4.2, 4.3, 4.5)", function()
    for _, tail in ipairs({ "\\n", "\\n[180]", "\\_l[-253,0]", "\\c" }) do
        test(tail .. " が同じ台詞の末尾", function()
            for _, extra in ipairs({ {}, BUDOUX }) do
                same_as_off(function(act, s)
                    act:line():talk(s, "長い記憶。" .. tail):line():talk(s, "続き、ね。")
                end, extra)
            end
        end)
        test(tail .. " が別のさくらスクリプト", function()
            for _, extra in ipairs({ {}, BUDOUX }) do
                same_as_off(function(act, s)
                    act:line():talk(s, "長い記憶。"):sakura_script(s, tail):line():talk(s, "続き、ね。")
                end, extra)
            end
        end)
    end

    test("改行のトークン・clear のトークン", function()
        same_as_off(function(act, s)
            act:line():talk(s, "長い記憶。"):newline():line():talk(s, "続き、ね。")
        end, BUDOUX)
        same_as_off(function(act, s)
            act:line():talk(s, "長い記憶。"):clear():line():talk(s, "続き、ね。")
        end, BUDOUX)
    end)

    test("位置のタグの後に見える字があれば、次の行の頭には入る", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "A\\nB"):line():talk(s, "C")
        end)
        expect(out):toBe("\\p[0]A\\nB\\n[180]C\\e")
    end)

    test("行の末尾の \\_l[-253,0] はその行自身の頭を止めず、次の行の頭は止める", function()
        local out = build_lines(1.8, function(act, s)
            act:line():talk(s, "X"):line():talk(s, "A\\_l[-253,0]"):line():talk(s, "B")
        end)
        expect(out):toBe("\\p[0]X\\n[180]A\\_l[-253,0]B\\e")
    end)
end)

describe("組み立て - raw_script とエスケープ・囲み (4.4, 4.6)", function()
    -- 2 つの行の間に raw_script をはさんだ出力
    local function with_raw(raw, before, after)
        return build_lines(1.8, function(act, s)
            act:line():talk(s, before)
            act:raw_script(raw)
            act:line():talk(s, after)
        end)
    end

    test("raw_script をはさんでも入る（見える字・位置のタグとして読まない）", function()
        for _, raw in ipairs({ "\\0", "\\![bind,a,b,1]", "\\s[5]", "\\_w[100]", "\\n", "\\c", "\\_l[-283,0]" }) do
            expect(with_raw(raw, "A", "B")):toBe("\\p[0]A" .. raw .. "\\n[180]B\\e")
        end
    end)

    test("raw_script をはさんでも、入れない継ぎ目は入れないまま", function()
        expect(with_raw("\\s[5]", "A\\n", "B")):toBe("\\p[0]A\\n\\s[5]B\\e")
    end)

    test("raw_script の字は吹き出しの字に数えない（最初の行には入らない）", function()
        same_as_off(function(act, s)
            act:raw_script("字")
            act:line():talk(s, "B")
        end)
    end)

    test("エスケープの \\\\n と囲みの中の \\n・\\c を位置のタグにしない", function()
        for _, tail in ipairs({ "\\\\n", "\\_?\\n\\_?", "\\_!\\c\\_!" }) do
            local out = build_lines(1.8, function(act, s)
                act:line():talk(s, "A" .. tail):line():talk(s, "B")
            end)
            expect(out):toBe("\\p[0]A" .. tail .. "\\n[180]B\\e")
        end
    end)
end)

describe("組み立て - 手書きの ＠改行 と同じ出力 (6.3, 3.1)", function()
    test("手書きの ＠改行 を消して設定した出力が、手書きの出力と同じ", function()
        for _, extra in ipairs({ {}, BUDOUX }) do
            local hand = build_lines(nil, function(act, s)
                act:line():talk(s, "なら。")
                act:line():sakura_script(s, "\\![bind,a,b,1]"):talk(s, "\\n[180]みんなに、ね。")
            end, extra)
            local set = build_lines(1.8, function(act, s)
                act:line():talk(s, "なら。")
                act:line():sakura_script(s, "\\![bind,a,b,1]"):talk(s, "みんなに、ね。")
            end, extra)
            expect(set):toBe(hand)
            expect(select(2, hand:gsub("\\n%[180%]", ""))):toBe(1)
        end
    end)

    test("話し手が戻った直後の表情だけの台詞（talk）で spot_newlines だけ", function()
        local out = same_as_off(function(act, s, k)
            act:line():talk(s, "A"):line():talk(k, "B")
            act:line():talk(s, "\\s[5]"):line():talk(s, "C")
        end, BUDOUX)
        -- 同じ話し手の talk は 1 つに結合され、字のある台詞として頭に spot_newlines が出る
        expect(out):toBe("\\p[0]A\\p[1]B\\p[0]\\n[150]\\s[5]C\\e")
    end)
end)

--- @pasta_config を stub に差し替えて pasta.config・pasta.shiori.act を新規ロードし、body を実行する（後で必ず復元）
--- @param stub table @pasta_config の代替テーブル
--- @param body fun(SHIORI_ACT: table)
local function with_shiori_act(stub, body)
    local names = { "@pasta_config", "pasta.config", "pasta.shiori.act" }
    local saved = {}
    for _, name in ipairs(names) do saved[name] = package.loaded[name] end
    package.loaded["pasta.config"] = nil
    package.loaded["pasta.shiori.act"] = nil
    package.loaded["@pasta_config"] = stub
    local ok, err = pcall(function() body(require("pasta.shiori.act")) end)
    for _, name in ipairs(names) do package.loaded[name] = saved[name] end
    if not ok then error(err, 0) end
end

--- 2 行の同じ話し手の台詞を積んで組み立てる
--- @param sakura table|nil 話し手のアクター（省けば name・spot だけのもの）
local function build_two_lines(SHIORI_ACT, sakura)
    sakura = sakura or { name = "さくら", spot = 0 }
    local act = SHIORI_ACT.new({ sakura = sakura })
    act:line():talk(sakura, "一行目。"):line():talk(sakura, "二行目。")
    return act:build(), act
end

describe("SHIORI の act - [ghost] line_newlines を読んで渡す (1.1, 1.5, 1.7)", function()
    test("値を設定した act の出力に行の改行が入る", function()
        with_shiori_act({ ghost = { line_newlines = 1.8 } }, function(SHIORI_ACT)
            local script = build_two_lines(SHIORI_ACT)
            expect(script):toBe("\\p[0]一行目。\\_w[950]\\n[180]二行目。\\_w[950]\\e")
        end)
    end)

    test("書かなければ既定 0 で、改行は入らない", function()
        with_shiori_act({ ghost = {} }, function(SHIORI_ACT)
            local script, act = build_two_lines(SHIORI_ACT)
            expect(act._line_newlines):toBe(0.0)
            expect(script):toBe("\\p[0]一行目。\\_w[950]二行目。\\_w[950]\\e")
        end)
    end)

    test("アクターごとの同名のキーは読まない", function()
        local stub = { ghost = {}, actor = { ["さくら"] = { line_newlines = 1.8 } } }
        with_shiori_act(stub, function(SHIORI_ACT)
            local script = build_two_lines(SHIORI_ACT, { name = "さくら", spot = 0, line_newlines = 1.8 })
            expect(script):toBe("\\p[0]一行目。\\_w[950]二行目。\\_w[950]\\e")
        end)
    end)

    test("数でない値は改行を作る所で Lua のエラーになる（spot_newlines と同じ）", function()
        with_shiori_act({ ghost = { line_newlines = "abc" } }, function(SHIORI_ACT)
            local ok = pcall(build_two_lines, SHIORI_ACT)
            expect(ok):toBe(false)
        end)
    end)
end)
