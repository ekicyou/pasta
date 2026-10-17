-- pasta.failure（失敗の報告の唯一の出口）の単体テスト (failure-output-unification 1.2・2.1〜2.3)
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

describe("FAILURE.report（act を通じた報告）", function()
    local REC = { label = "数にできない値", subject = "＄y", detail = '"abc"', warning = "w1" }

    test("act あり・actor なし: 警告 1 回、表記を raw_script として 1 つ積み、nil を返す", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            local ret = FAILURE.report(act, REC)
            expect(ret):toBe(nil)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("w1")
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("raw_script")
            expect(act.token[1].text):toBe('【数にできない値：＄y（"abc"）】')
            expect(act.token[1].actor):toBe(nil)
        end)
    end)

    test("act あり・actor あり: 目印のトークンに actor を付ける", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            local actor = { name = "ぱすたー" }
            FAILURE.report(act, {
                label = FAILURE.LABEL.UNREGISTERED_ACTOR, subject = "ぱすたー", warning = "w", actor = actor,
            })
            expect(#warns):toBe(1)
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("raw_script")
            expect(act.token[1].text):toBe("【未登録アクター：ぱすたー】")
            expect(act.token[1].actor):toBe(actor)
        end)
    end)

    test("act なし（actor の有無にかかわらず）: 警告 1 回だけで、nil を返しエラーにしない", function()
        with_log(function(FAILURE, warns)
            expect(FAILURE.report(nil, REC)):toBe(nil)
            expect(FAILURE.report(nil, { label = "x", warning = "w2", actor = {} })):toBe(nil)
            expect(#warns):toBe(2)
            expect(warns[2]):toBe("w2")
        end)
    end)

    test("表記はエスケープしてから枠を付ける", function()
        with_log(function(FAILURE)
            local act = { token = {} }
            FAILURE.report(act, { label = FAILURE.LABEL.WORD_NOT_FOUND, subject = "＠%x" })
            expect(act.token[1].text):toBe([[【単語が見つからない：＠\%x】]])
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

    test("label 以外が文字列でなくても形を作り、例外を投げない", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            FAILURE.report(act, { label = "x", subject = 1, detail = false, warning = 2 })
            expect(act.token[1].text):toBe("【x：1（false）】")
            expect(warns[1]):toBe("2")
        end)
    end)

    test("同じ失敗を 2 回報告すると 2 回出る（抑止しない）", function()
        with_log(function(FAILURE, warns)
            local act = { token = {} }
            FAILURE.report(act, REC)
            FAILURE.report(act, REC)
            expect(#warns):toBe(2)
            expect(#act.token):toBe(2)
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

describe("FAILURE.notation（act の無い場所からの報告）", function()
    test("actor なし: 警告 1 回、表記の文字列を返す", function()
        with_log(function(FAILURE, warns)
            expect(FAILURE.notation({ label = "値の無い変数", subject = "＄x", warning = "w" }))
                :toBe("【値の無い変数：＄x】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("w")
        end)
    end)

    test("actor あり: 積む先が無いので文字列を返すだけ", function()
        with_log(function(FAILURE, warns)
            local actor = {}
            expect(FAILURE.notation({ label = "未登録アクター", subject = "a", warning = "w", actor = actor }))
                :toBe("【未登録アクター：a】")
            expect(#warns):toBe(1)
        end)
    end)

    test("warning が nil ならログを出さない", function()
        with_log(function(FAILURE, warns)
            expect(FAILURE.notation({ label = "x" })):toBe("【x】")
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("act:failure は報告の出口を通る", function()
    test("act:failure(text, warning) が警告 1 回と【text】をエスケープして積む", function()
        with_log(function(_, warns)
            local act = require("pasta.act").new({})
            expect(act:failure([[t\%]], "w")):toBe(nil)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("w")
            local token = act.token[#act.token]
            expect(token.type):toBe("raw_script")
            expect(token.text):toBe([[【t\\\%】]])
        end)
    end)
end)

describe("FAILURE.bind / running_act（実行中の act の対応表）", function()
    local FAILURE = require("pasta.failure")

    test("シーンのコルーチンの中で bind した act を、同じコルーチンの続きで引ける", function()
        local act = {}
        local co = coroutine.create(function()
            FAILURE.bind(act)
            coroutine.yield(FAILURE.running_act())
            return FAILURE.running_act()
        end)
        local _, first = coroutine.resume(co)
        expect(first):toBe(act)
        local _, after_resume = coroutine.resume(co)
        expect(after_resume):toBe(act)
    end)

    test("別のコルーチンでは別の act、対応の無いコルーチンでは nil", function()
        local a, b = {}, {}
        local function bound(act)
            return coroutine.create(function()
                if act then FAILURE.bind(act) end
                coroutine.yield()
                return FAILURE.running_act()
            end)
        end
        local ca, cb, cn = bound(a), bound(b), bound(nil)
        coroutine.resume(ca)
        coroutine.resume(cb)
        coroutine.resume(cn)
        expect(select(2, coroutine.resume(ca))):toBe(a)
        expect(select(2, coroutine.resume(cb))):toBe(b)
        expect(select(2, coroutine.resume(cn))):toBe(nil)
    end)

    test("コルーチンの外（メインスレッド）: running() が thread なら引け、次の bind で上書きされる", function()
        local co = coroutine.running()
        local a, b = {}, {}
        FAILURE.bind(a)
        if co == nil then
            -- running() が nil の環境ではログだけ（対応を作らない）
            expect(FAILURE.running_act()):toBe(nil)
        else
            expect(FAILURE.running_act()):toBe(a)
            FAILURE.bind(b)
            expect(FAILURE.running_act()):toBe(b)
        end
    end)

    test("coroutine.running() が nil を返すとき、bind は何もせず running_act は nil", function()
        local saved = coroutine.running
        local saved_act = FAILURE.running_act()
        coroutine.running = function() return nil end -- luacheck: ignore 122
        local ok, err = pcall(function()
            FAILURE.bind({})
            expect(FAILURE.running_act()):toBe(nil)
        end)
        coroutine.running = saved -- luacheck: ignore 122
        if not ok then error(err, 0) end
        -- 差し替え中の bind は実際のスレッドの対応を変えていない
        expect(FAILURE.running_act()):toBe(saved_act)
    end)
end)

describe("FAILURE.act_of（コルーチンの act を外から読む）", function()
    local FAILURE = require("pasta.failure")

    test("コルーチンの中で bind した act を、コルーチンの外から引ける（エラーで止まった後も）", function()
        local act = {}
        local co = coroutine.create(function()
            FAILURE.bind(act)
            error("boom")
        end)
        expect(coroutine.resume(co)):toBe(false)
        expect(coroutine.status(co)):toBe("dead")
        expect(FAILURE.act_of(co)):toBe(act)
    end)

    test("bind していないコルーチンは nil", function()
        local co = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(co)
        expect(FAILURE.act_of(co)):toBe(nil)
    end)

    test("読むだけで対応を変えない", function()
        local act = {}
        local co = coroutine.create(function()
            FAILURE.bind(act)
            coroutine.yield()
            return FAILURE.running_act()
        end)
        coroutine.resume(co)
        expect(FAILURE.act_of(co)):toBe(act)
        expect(FAILURE.act_of(co)):toBe(act)
        expect(select(2, coroutine.resume(co))):toBe(act)
    end)
end)

describe("FAILURE.set_fn_failed / take_fn_failed（報告済みの印）", function()
    local FAILURE = require("pasta.failure")

    test("書いた印を読み、読んだら偽に戻る", function()
        local act = {}
        expect(FAILURE.take_fn_failed(act)):toBe(false)
        FAILURE.set_fn_failed(act, true)
        expect(FAILURE.take_fn_failed(act)):toBe(true)
        expect(FAILURE.take_fn_failed(act)):toBe(false)
    end)

    test("偽を書けば偽、act ごとに独立", function()
        local a, b = {}, {}
        FAILURE.set_fn_failed(a, true)
        FAILURE.set_fn_failed(a, false)
        FAILURE.set_fn_failed(b, true)
        expect(FAILURE.take_fn_failed(a)):toBe(false)
        expect(FAILURE.take_fn_failed(b)):toBe(true)
    end)
end)

describe("FAILURE.mark_unregistered_actor / is_unregistered_actor（未登録アクターの印）", function()
    local FAILURE = require("pasta.failure")

    test("印を付けたアクターだけが真、nil は偽", function()
        local marked, other = { name = "a" }, { name = "b" }
        FAILURE.mark_unregistered_actor(marked)
        expect(FAILURE.is_unregistered_actor(marked)):toBe(true)
        expect(FAILURE.is_unregistered_actor(other)):toBe(false)
        expect(FAILURE.is_unregistered_actor(nil)):toBe(false)
    end)

    test("印はアクターの表のフィールドに書かない", function()
        local actor = { name = "a" }
        FAILURE.mark_unregistered_actor(actor)
        local keys = {}
        for k in pairs(actor) do table.insert(keys, k) end
        expect(#keys):toBe(1)
        expect(keys[1]):toBe("name")
        expect(getmetatable(actor)):toBe(nil)
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

    test("種類の名前の表は 10 種", function()
        local n = 0
        for _ in pairs(L) do n = n + 1 end
        expect(n):toBe(10)
    end)

    test("スクリプトのエラー（実行時エラーで止まった）", function()
        expect(L.SCRIPT_ERROR):toBe("スクリプトのエラー")
        expect(FAILURE.format(L.SCRIPT_ERROR)):toBe("【スクリプトのエラー】")
        expect(FAILURE.format(L.SCRIPT_ERROR, "＊会話（3 行目）", "attempt to call a nil value"))
            :toBe("【スクリプトのエラー：＊会話（3 行目）（attempt to call a nil value）】")
    end)

    test("時間切れ", function()
        expect(L.TIMEOUT):toBe("時間切れ")
        expect(FAILURE.format(L.TIMEOUT, "OnTalk")):toBe("【時間切れ：OnTalk】")
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

--- [logging] balloon_warning を value にした pasta.config の代役で body を実行し、元へ戻す
--- （value が nil なら書かれていない）
--- @param value any
--- @param body fun()
local function with_balloon_warning(value, body)
    local saved = package.loaded["pasta.config"]
    package.loaded["pasta.config"] = {
        get = function(section, key, default)
            if section == "logging" and key == "balloon_warning" and value ~= nil then return value end
            return default
        end,
    }
    local ok, err = pcall(body)
    package.loaded["pasta.config"] = saved
    if not ok then error(err, 0) end
end

describe("FAILURE.enabled（[logging] balloon_warning）", function()
    local FAILURE = require("pasta.failure")
    test("書かれていなければ true", function()
        with_balloon_warning(nil, function() expect(FAILURE.enabled()):toBe(true) end)
    end)
    test("false なら false、true なら true", function()
        with_balloon_warning(false, function() expect(FAILURE.enabled()):toBe(false) end)
        with_balloon_warning(true, function() expect(FAILURE.enabled()):toBe(true) end)
    end)
    test("真偽値でない値（文字列・数・表）は既定値 true", function()
        for _, v in ipairs({ "false", 0, {} }) do
            with_balloon_warning(v, function() expect(FAILURE.enabled()):toBe(true) end)
        end
    end)
    test("報告のたびに読む（キャッシュしない）", function()
        with_balloon_warning(false, function() expect(FAILURE.enabled()):toBe(false) end)
        with_balloon_warning(nil, function() expect(FAILURE.enabled()):toBe(true) end)
    end)
end)

describe("出さない設定（balloon_warning = false）での報告", function()
    test("report: 警告は出し、actor の無い表記は積まない", function()
        with_balloon_warning(false, function()
            with_log(function(FAILURE, warns)
                local act = { token = {} }
                expect(FAILURE.report(act, { label = "x", subject = "＄y", warning = "w" })):toBe(nil)
                expect(#warns):toBe(1)
                expect(#act.token):toBe(0)
            end)
        end)
    end)
    test("report: 未登録アクターの目印は空の raw_script で積む", function()
        with_balloon_warning(false, function()
            with_log(function(FAILURE, warns)
                local act = { token = {} }
                local actor = { name = "ぱすたー" }
                FAILURE.report(act, {
                    label = FAILURE.LABEL.UNREGISTERED_ACTOR, subject = "ぱすたー", warning = "w", actor = actor,
                })
                expect(#warns):toBe(1)
                expect(#act.token):toBe(1)
                expect(act.token[1].type):toBe("raw_script")
                expect(act.token[1].text):toBe("")
                expect(act.token[1].actor):toBe(actor)
            end)
        end)
    end)
    test("report: act なしでも警告だけ出す", function()
        with_balloon_warning(false, function()
            with_log(function(FAILURE, warns)
                expect(FAILURE.report(nil, { label = "x", warning = "w", actor = {} })):toBe(nil)
                expect(#warns):toBe(1)
            end)
        end)
    end)
    test("notation: 警告は出し、nil を返す", function()
        with_balloon_warning(false, function()
            with_log(function(FAILURE, warns)
                expect(FAILURE.notation({ label = "x", warning = "w" })):toBe(nil)
                expect(#warns):toBe(1)
            end)
        end)
    end)
    test("型違いの値なら表記を出す", function()
        with_balloon_warning("no", function()
            with_log(function(FAILURE)
                expect(FAILURE.notation({ label = "x" })):toBe("【x】")
            end)
        end)
    end)
end)

describe("ACT.build: トークンがすべて空の raw_script なら nil", function()
    local ACT = require("pasta.act")
    test("空の目印だけなら nil を返し、トークンを空にする", function()
        local act = ACT.new({})
        table.insert(act.token, { type = "raw_script", text = "", actor = { name = "a" } })
        table.insert(act.token, { type = "raw_script", text = "" })
        expect(act:build()):toBe(nil)
        expect(#act.token):toBe(0)
    end)
    test("空でない raw_script が 1 つでもあれば組み立てる", function()
        local act = ACT.new({})
        table.insert(act.token, { type = "raw_script", text = "", actor = { name = "a" } })
        table.insert(act.token, { type = "raw_script", text = "x" })
        expect(act:build() ~= nil):toBe(true)
    end)
end)
