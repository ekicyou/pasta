-- pasta.failure（失敗の報告の唯一の出口）の単体テスト (failure-output-unification 1.2)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んで body を実行し、元のログへ戻す
--- @param body fun(FAILURE: table, warns: string[])
local function with_log(body)
    local saved_log = package.loaded["@pasta_log"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    local ok, err = pcall(body, require("pasta.failure"), warns)
    package.loaded["@pasta_log"] = saved_log
    if not ok then error(err, 0) end
end

describe("FAILURE.report", function()
    test("警告をログへ 1 回出し、【label】を raw_script として積み、nil を返す", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            local ret = FAILURE.report(act, { label = "Call失敗：x", warning = "w1" })
            expect(ret):toBe(nil)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("w1")
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("raw_script")
            expect(act.token[1].text):toBe("【Call失敗：x】")
        end)
    end)

    test("warning が nil ならログを出さず、表記だけ積む", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            FAILURE.report(act, { label = "x" })
            expect(#warns):toBe(0)
            expect(act.token[1].text):toBe("【x】")
        end)
    end)

    test("act が nil なら警告だけ出し、エラーにしない", function()
        with_log(function(FAILURE, warns)
            expect(FAILURE.report(nil, { label = "x", warning = "w" })):toBe(nil)
            expect(#warns):toBe(1)
        end)
    end)

    test("ログはモジュールの読み込み後に差し替えても、呼んだ時点のものを使う", function()
        require("pasta.failure")
        with_log(function(FAILURE, warns)
            FAILURE.report({ token = {} }, { label = "x", warning = "late" })
            expect(warns[1]):toBe("late")
        end)
    end)
end)

describe("act:failure は FAILURE.report へ委ねる", function()
    test("act:failure(text, warning) が同じ警告とトークンを出す", function()
        with_log(function(FAILURE, warns)
            local saved = FAILURE.report
            local seen
            FAILURE.report = function(act, record)
                seen = record
                return saved(act, record)
            end
            local ok, err = pcall(function()
                local act = require("pasta.act").new({})
                expect(act:failure("t", "w")):toBe(nil)
                expect(seen.label):toBe("t")
                expect(seen.warning):toBe("w")
                expect(warns[1]):toBe("w")
                expect(act.token[#act.token].text):toBe("【t】")
            end)
            FAILURE.report = saved
            if not ok then error(err, 0) end
        end)
    end)
end)

describe("依存の向き", function()
    test("pasta.failure は act・actor・word を読み込まない", function()
        local names = { "pasta.failure", "pasta.act", "pasta.actor", "pasta.word" }
        local saved = {}
        for _, n in ipairs(names) do
            saved[n] = package.loaded[n]
            package.loaded[n] = nil
        end
        local ok, err = pcall(function()
            require("pasta.failure")
            expect(package.loaded["pasta.act"]):toBe(nil)
            expect(package.loaded["pasta.actor"]):toBe(nil)
            expect(package.loaded["pasta.word"]):toBe(nil)
        end)
        for _, n in ipairs(names) do
            package.loaded[n] = saved[n]
        end
        if not ok then error(err, 0) end
    end)
end)

describe("FAILURE.format（形の 4 通り）", function()
    local FAILURE = require("pasta.failure")

    test("対象と説明: 【種類：対象（説明）】", function()
        expect(FAILURE.format("種類", "対象", "説明")):toBe("【種類：対象（説明）】")
    end)

    test("説明なし: 【種類：対象】", function()
        expect(FAILURE.format("種類", "対象", nil)):toBe("【種類：対象】")
    end)

    test("対象なし: 【種類：説明】", function()
        expect(FAILURE.format("種類", nil, "説明")):toBe("【種類：説明】")
    end)

    test("両方なし: 【種類】", function()
        expect(FAILURE.format("種類")):toBe("【種類】")
    end)

    test("行番号を足さない（中身は渡したものだけ）", function()
        expect(FAILURE.format("種類", "＄x")):toBe("【種類：＄x】")
    end)

    test("文字列でない値は tostring して形を作り、エラーにしない", function()
        expect(FAILURE.format("種類", 12, true)):toBe("【種類：12（true）】")
    end)
end)

describe("FAILURE.LABEL と要件 3.1 の文言", function()
    local FAILURE = require("pasta.failure")
    local L = FAILURE.LABEL

    test("種類の名前の表は 8 種", function()
        local n = 0
        for _ in pairs(L) do n = n + 1 end
        expect(n):toBe(8)
    end)

    test("値の無い変数", function()
        expect(FAILURE.format(L.UNDEFINED_VAR, "＄話題")):toBe("【値の無い変数：＄話題】")
    end)

    test("単語が見つからない", function()
        expect(FAILURE.format(L.WORD_NOT_FOUND, "＠挨拶")):toBe("【単語が見つからない：＠挨拶】")
    end)

    test("関数が見つからない（ローカル・グローバル）", function()
        expect(FAILURE.format(L.FN_NOT_FOUND, "＠計算（）")):toBe("【関数が見つからない：＠計算（）】")
        expect(FAILURE.format(L.FN_NOT_FOUND, "＠＊計算（）")):toBe("【関数が見つからない：＠＊計算（）】")
    end)

    test("名前にならない値（値なし・空文字列・型の名前）", function()
        expect(FAILURE.format(L.BAD_NAME, "＄名前", "値なし")):toBe("【名前にならない値：＄名前（値なし）】")
        expect(FAILURE.format(L.BAD_NAME, "＄名前", "空文字列")):toBe("【名前にならない値：＄名前（空文字列）】")
        expect(FAILURE.format(L.BAD_NAME, "＄名前", "表")):toBe("【名前にならない値：＄名前（表）】")
    end)

    test("未登録アクター", function()
        expect(FAILURE.format(L.UNREGISTERED_ACTOR, "ぱすたー")):toBe("【未登録アクター：ぱすたー】")
    end)

    test("数にできない値", function()
        expect(FAILURE.format(L.NOT_NUMBER, "＄y", '"abc"')):toBe('【数にできない値：＄y（"abc"）】')
    end)

    test("文字列にできない値", function()
        expect(FAILURE.format(L.NOT_STRING, "＄y", "表")):toBe("【文字列にできない値：＄y（表）】")
    end)

    test("Call のターゲットが見つからない", function()
        expect(FAILURE.format(L.SCENE_NOT_FOUND, "＞会話２")):toBe("【シーンが見つからない：＞会話２】")
    end)

    test("Call の動的キーが名前にならない", function()
        expect(FAILURE.format(L.BAD_NAME, "＄行き先", "値なし")):toBe("【名前にならない値：＄行き先（値なし）】")
    end)
end)

describe("FAILURE.escape（タグとして働かせない）", function()
    local FAILURE = require("pasta.failure")

    test([[\ を \\ に、% を \% にする]], function()
        expect(FAILURE.escape([[\n]])):toBe([[\\n]])
        expect(FAILURE.escape("%username")):toBe([[\%username]])
        expect(FAILURE.escape([[\%]])):toBe([[\\\%]])
    end)

    test("CR・LF を空白にする", function()
        expect(FAILURE.escape("a\r\nb\nc\rd")):toBe("a  b c d")
    end)

    test("それ以外の文字はそのまま", function()
        expect(FAILURE.escape("＄話題【】（）abc")):toBe("＄話題【】（）abc")
    end)

    test("format は中身全体をエスケープしてから枠を付ける", function()
        expect(FAILURE.format([[種\類]], [[\s[0]\]], "%x\n"))
            :toBe([[【種\\類：\\s[0]\\（\%x ）】]])
    end)

    test([[タグを含む名前は、エスケープされない \ と % を残さない]], function()
        local s = FAILURE.format(FAILURE.LABEL.WORD_NOT_FOUND, [[＠\s[10]\e%month%]] .. "\r\n")
        expect(s):toBe([[【単語が見つからない：＠\\s[10]\\e\%month\%  】]])
        -- 「\\」「\%」の対を取り除くと、\・%・CR・LF は 1 つも残らない
        local rest = s:gsub([[\\]], ""):gsub([[\%%]], "")
        expect(rest:find("[\\%%\r\n]")):toBe(nil)
    end)
end)

describe("FAILURE.var（変数パス → 辞書の書き方）", function()
    local FAILURE = require("pasta.failure")
    local cases = {
        { "var.話題", "＄話題" },
        { "save.回数", "＄＊回数" },
        { "args[1]", "＄０" },
        { "args[2]", "＄１" },
        { "args[11]", "＄１０" },
    }
    for _, c in ipairs(cases) do
        test(c[1] .. " → " .. c[2], function()
            expect(FAILURE.var(c[1])):toBe(c[2])
        end)
    end

    test("知らない形はそのまま返す", function()
        for _, s in ipairs({ "x", "var.", "save.", "args[0]", "args[x]", "args[1]x", "@f()", "" }) do
            expect(FAILURE.var(s)):toBe(s)
        end
    end)
end)

describe("FAILURE.fn（関数・被演算子の説明 → 辞書の書き方）", function()
    local FAILURE = require("pasta.failure")
    local cases = {
        { "@計算()", "＠計算（）" },
        { "@*計算()", "＠＊計算（）" },
        { "@$var.f()", "＠＄f（）" },
        { "@$save.f()", "＠＄＊f（）" },
        { "@$args[2]()", "＠＄１（）" },
        { "var.y", "＄y" },
        { "save.y", "＄＊y" },
        { "args[3]", "＄２" },
    }
    for _, c in ipairs(cases) do
        test(c[1] .. " → " .. c[2], function()
            expect(FAILURE.fn(c[1])):toBe(c[2])
        end)
    end

    test("知らない形はそのまま返す", function()
        for _, s in ipairs({ "f", "@f", "@()", "@*()", "@$x()", "@$()", "f()", "" }) do
            expect(FAILURE.fn(s)):toBe(s)
        end
    end)
end)

describe("FAILURE.kind_of（名前にならない値の説明）", function()
    local FAILURE = require("pasta.failure")
    test("値なし・空文字列・真偽値・表・関数", function()
        expect(FAILURE.kind_of(nil)):toBe("値なし")
        expect(FAILURE.kind_of("")):toBe("空文字列")
        expect(FAILURE.kind_of(true)):toBe("真偽値")
        expect(FAILURE.kind_of(false)):toBe("真偽値")
        expect(FAILURE.kind_of({})):toBe("表")
        expect(FAILURE.kind_of(print)):toBe("関数")
    end)
    test("それ以外は Lua の型の名前", function()
        expect(FAILURE.kind_of(coroutine.create(function() end))):toBe("thread")
        expect(FAILURE.kind_of(1)):toBe("number")
    end)
end)

describe("FAILURE.value_of（数・文字列にできない値の説明）", function()
    local FAILURE = require("pasta.failure")
    test("文字列は引用符付き", function()
        expect(FAILURE.value_of("abc")):toBe('"abc"')
        expect(FAILURE.value_of("")):toBe('""')
    end)
    test("真偽値は true / false", function()
        expect(FAILURE.value_of(true)):toBe("true")
        expect(FAILURE.value_of(false)):toBe("false")
    end)
    test("それ以外は型の名前", function()
        expect(FAILURE.value_of({})):toBe("表")
        expect(FAILURE.value_of(print)):toBe("関数")
        expect(FAILURE.value_of(coroutine.create(function() end))):toBe("thread")
    end)
    test("format と組み合わせて要件 3.1 の文言になる", function()
        expect(FAILURE.format(FAILURE.LABEL.NOT_NUMBER, FAILURE.fn("var.y"), FAILURE.value_of("abc")))
            :toBe('【数にできない値：＄y（"abc"）】')
    end)
end)
