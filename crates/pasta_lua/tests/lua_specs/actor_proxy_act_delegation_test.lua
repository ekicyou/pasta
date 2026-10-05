-- アクタープロキシの word / expr_fn / expr_fn_var が呼ぶ関数の第 1 引数と戻り値のテスト (actor-proxy-act-delegation)
-- 規則: 行のアクターの表のフィールドの関数はプロキシを、それ以外（シーンテーブル・act のメソッド・GLOBAL）の関数は ACT を受け取る。
-- 関数が ACT・プロキシそのものを返したときは nil にする。
--
-- 「修正で直る挙動」の describe は修正前の actor.lua では失敗する（不具合の再現）。
-- 「現行どおりの挙動」の describe は修正の前後で通る。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 依存閉包を整合した同一インスタンス集合として一括新規ロードする（kick_install_test と同じ規約）。
-- entry を最初に require し、GLOBAL.ゴースト終了 を登録した GLOBAL を act と共有させる。
local RELOAD = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.shiori.event.kick",
    "pasta.shiori.res",
    "pasta.shiori.act",
    "pasta.shiori.sakura_builder",
    "pasta.act",
    "pasta.actor",
    "pasta.scene",
    "pasta.word",
    "pasta.global",
    "pasta.store",
    "pasta.config",
    "pasta.save",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

require("pasta.shiori.entry")
local ACT = require("pasta.act")
local GLOBAL = require("pasta.global")
-- pasta.actor が読み込み時に保持したログと同じ表（警告の記録に使う）
local log = require("@pasta_log")

local ACTOR_NAME = "apd_さくら"

--- 登録済みのアクターの act とプロキシを作る（actor_fields はアクターの表のフィールド）
--- @param actor_fields table|nil
--- @return table act
--- @return table proxy
local function registered(actor_fields)
    local actor = { name = ACTOR_NAME }
    for k, v in pairs(actor_fields or {}) do actor[k] = v end
    local act = ACT.new({ [ACTOR_NAME] = actor })
    return act, act:actor_proxy(ACTOR_NAME)
end

--- 未登録のアクターの act と、act:actor_proxy のその場限りのプロキシを作る
--- @return table act
--- @return table proxy
local function unregistered()
    local act = ACT.new({})
    return act, act:actor_proxy("apd_未登録さん")
end

--- 値が act・プロキシのどちらかを表す文字列（失敗時に表を丸ごと出さないため）
local function kind(v, act, proxy)
    if v == act then return "ACT" end
    if v == proxy then return "proxy" end
    return type(v)
end

--- GLOBAL の 1 キーを一時的に差し替えて body を実行し、元に戻す
local function with_global(key, value, body)
    local saved = GLOBAL[key]
    GLOBAL[key] = value
    local ok, err = pcall(body)
    GLOBAL[key] = saved
    if not ok then error(err, 0) end
end

--- 警告を記録して body を実行し、ログを元に戻す
--- @param body fun(warns: string[])
local function capture_warns(body)
    local saved = log.warn
    local warns = {}
    log.warn = function(msg) table.insert(warns, msg) end
    local ok, err = pcall(body, warns)
    log.warn = saved
    if not ok then error(err, 0) end
end

--- 「前」を話してから call を呼び、続けて戻り値と「後」を話すシーンをコルーチンで実行する。
--- 1 回目の resume で中断し、2 回目で終わることを確かめ、call の戻り値を返す。
--- 再開後の act.token が「後」の talk だけであること（戻り値を台詞に出さないこと）も確かめる
--- @param act table
--- @param proxy table
--- @param call fun(): any
--- @return any call の戻り値
local function run_yield_scene(act, proxy, call)
    local result
    local co = coroutine.create(function()
        proxy:talk("前")
        result = call()
        proxy:talk(result)
        proxy:talk("後")
    end)
    local ok1, err1 = coroutine.resume(co)
    if not ok1 then error(err1, 0) end
    expect(coroutine.status(co)):toBe("suspended")
    expect(#act.token):toBe(0) -- 中断までの出力は build されて応答に渡った

    local ok2, err2 = coroutine.resume(co)
    if not ok2 then error(err2, 0) end
    expect(coroutine.status(co)):toBe("dead")
    expect(#act.token):toBe(1)
    expect(act.token[1].text):toBe("後")
    return result
end

--- act.token が待ち（ms）と \- の 2 つであることを確かめる
local function expect_close_ghost_tokens(act, ms)
    expect(#act.token):toBe(2)
    expect(act.token[1].type):toBe("wait")
    expect(act.token[1].ms):toBe(ms)
    expect(act.token[2].type):toBe("raw_script")
    expect(act.token[2].text):toBe("\\-")
end

describe("actor-proxy-act-delegation - アクション行から呼ばれた関数", function()
    -- ========================================================================
    -- 修正で直る挙動（修正前の actor.lua では失敗する）
    -- ========================================================================
    describe("修正で直る挙動 - アクターの外の関数は ACT を受け取る", function()
        test("登録済みアクター: word でシーンテーブルの関数は ACT を受け取る", function()
            local act, p = registered()
            local received
            act.current_scene = { ["場面"] = function(a) received = a return "シーン" end }
            expect(p:word("場面")):toBe("シーン")
            expect(kind(received, act, p)):toBe("ACT")
        end)

        test("登録済みアクター: word で GLOBAL の関数は ACT を受け取る", function()
            local act, p = registered()
            local received
            with_global("apd_時報", function(a) received = a return "正午" end, function()
                expect(p:word("apd_時報")):toBe("正午")
            end)
            expect(kind(received, act, p)):toBe("ACT")
        end)

        test("未登録アクター: word でシーンテーブル・GLOBAL の関数は ACT を受け取る", function()
            local act, p = unregistered()
            local from_scene, from_global
            act.current_scene = { ["場面"] = function(a) from_scene = a return "シーン" end }
            with_global("apd_時報", function(a) from_global = a return "正午" end, function()
                expect(p:word("場面")):toBe("シーン")
                expect(p:word("apd_時報")):toBe("正午")
            end)
            expect(kind(from_scene, act, p)):toBe("ACT")
            expect(kind(from_global, act, p)):toBe("ACT")
        end)

        test("expr_fn はシーンテーブルの関数に ACT と 2 番目以降の引数をそのまま渡す", function()
            local act, p = registered()
            local received, ra, rb
            act.current_scene = { ["加算"] = function(a, x, y) received, ra, rb = a, x, y return x + y end }
            expect(p:expr_fn("加算", 2, 3)):toBe(5)
            expect(kind(received, act, p)):toBe("ACT")
            expect(ra):toBe(2)
            expect(rb):toBe(3)
        end)

        test("expr_fn_var は GLOBAL の関数に ACT と 2 番目以降の引数をそのまま渡す", function()
            local act, p = registered()
            local received, ra, rb
            with_global("apd_連結", function(a, x, y) received, ra, rb = a, x, y return x .. y end, function()
                expect(p:expr_fn_var("apd_連結", "var.f", "あ", "い")):toBe("あい")
            end)
            expect(kind(received, act, p)):toBe("ACT")
            expect(ra):toBe("あ")
            expect(rb):toBe("い")
        end)

        test("未登録アクター: expr_fn・expr_fn_var の GLOBAL の関数は ACT を受け取る", function()
            local act, p = unregistered()
            local got = {}
            with_global("apd_記録", function(a, x) table.insert(got, a) return x end, function()
                expect(p:expr_fn("apd_記録", 1)):toBe(1)
                expect(p:expr_fn_var("apd_記録", "var.f", 2)):toBe(2)
            end)
            expect(kind(got[1], act, p)):toBe("ACT")
            expect(kind(got[2], act, p)):toBe("ACT")
        end)

        test("expr_fn で act のメソッド（wait）を呼ぶと ACT に待ちを積み、nil を返す", function()
            local act, p = registered()
            expect(p:expr_fn("wait", 300)):toBeNil()
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("wait")
            expect(act.token[1].ms):toBe(300)
        end)
    end)

    describe("修正で直る挙動 - ACT・プロキシそのものの戻り値は nil", function()
        test("word: 関数が ACT そのものを返すと nil", function()
            local act, p = registered()
            act.current_scene = { ["自分"] = function() return act end }
            expect(kind(p:word("自分"), act, p)):toBe("nil")
        end)

        test("word: 関数がプロキシそのものを返すと nil", function()
            local act, p = registered()
            p.actor["自分"] = function() return p end
            expect(kind(p:word("自分"), act, p)):toBe("nil")
        end)

        test("expr_fn: 関数が ACT そのもの（と続く値）を返すと nil だけを返す", function()
            local act, p = registered()
            act.current_scene = { ["自分"] = function() return act, "余り" end }
            local n = select("#", p:expr_fn("自分"))
            local r = p:expr_fn("自分")
            expect(kind(r, act, p)):toBe("nil")
            expect(n):toBe(1)
        end)

        test("expr_fn_var: 関数がプロキシそのものを返すと nil", function()
            local act, p = registered()
            with_global("apd_自分", function() return p end, function()
                expect(kind(p:expr_fn_var("apd_自分", "var.f"), act, p)):toBe("nil")
            end)
        end)
    end)

    describe("修正で直る挙動 - 組み込み関数 yield・チェイントーク（コルーチン）", function()
        test("word(\"yield\") は中断し、再開後に nil を返す", function()
            local act, p = registered()
            expect(run_yield_scene(act, p, function() return p:word("yield") end)):toBeNil()
        end)

        test("expr_fn(\"チェイントーク\") は中断し、再開後に nil を返す", function()
            local act, p = registered()
            expect(run_yield_scene(act, p, function() return p:expr_fn("チェイントーク") end)):toBeNil()
        end)

        test("値が yield の動的参照 word(値, パス) は中断し、再開後に nil を返す", function()
            local act, p = registered()
            expect(run_yield_scene(act, p, function() return p:word("yield", "var.x") end)):toBeNil()
        end)

        test("未登録アクター: word(\"yield\") は中断し、再開後に nil を返す", function()
            local act, p = unregistered()
            expect(run_yield_scene(act, p, function() return p:word("yield") end)):toBeNil()
        end)
    end)

    describe("修正で直る挙動 - 組み込み関数 ゴースト終了", function()
        test("expr_fn(\"ゴースト終了\", 500) は待ちと \\- を積み、nil を返す", function()
            local act, p = registered()
            expect(p:expr_fn("ゴースト終了", 500)):toBeNil()
            expect_close_ghost_tokens(act, 500)
        end)

        test("値が ゴースト終了 の expr_fn_var(値, パス, 500) は待ちと \\- を積み、nil を返す", function()
            local act, p = registered()
            expect(p:expr_fn_var("ゴースト終了", "var.x", 500)):toBeNil()
            expect_close_ghost_tokens(act, 500)
        end)

        test("word(\"ゴースト終了\") は \\- だけを積み、nil を返す", function()
            local act, p = registered()
            expect(p:word("ゴースト終了")):toBeNil()
            expect(#act.token):toBe(1)
            expect(act.token[1].type):toBe("raw_script")
            expect(act.token[1].text):toBe("\\-")
        end)
    end)

    -- ========================================================================
    -- 現行どおりの挙動（修正の前後で通る）
    -- ========================================================================
    describe("現行どおりの挙動 - アクターの表の関数はプロキシを受け取る", function()
        test("登録済みアクター: word でアクターのフィールドの関数はプロキシを受け取る", function()
            local received
            local act, p = registered({ ["口上"] = function(x) received = x return "口上です" end })
            expect(p:word("口上")):toBe("口上です")
            expect(kind(received, act, p)):toBe("proxy")
        end)

        test("未登録アクター: word でその場限りのアクターのフィールドの関数はプロキシを受け取る", function()
            local act, p = unregistered()
            local received
            p.actor["口上"] = function(x) received = x return "口上です" end
            expect(p:word("口上")):toBe("口上です")
            expect(kind(received, act, p)):toBe("proxy")
        end)

        test("アクターの段はシーンテーブルより先に探される", function()
            local act, p = registered({ ["名乗り"] = function() return "アクター" end })
            act.current_scene = { ["名乗り"] = function() return "シーン" end }
            expect(p:word("名乗り")):toBe("アクター")
        end)
    end)

    describe("現行どおりの挙動 - 戻り値と関数でない値", function()
        test("expr_fn は複数の戻り値をそのまま返す", function()
            local act, p = registered()
            act.current_scene = { ["二つ"] = function() return "一", 2 end }
            local r1, r2 = p:expr_fn("二つ")
            expect(r1):toBe("一")
            expect(r2):toBe(2)
        end)

        test("expr_fn_var は複数の戻り値をそのまま返す", function()
            local _, p = registered()
            with_global("apd_二つ", function() return "一", 2 end, function()
                local r1, r2 = p:expr_fn_var("apd_二つ", "var.f")
                expect(r1):toBe("一")
                expect(r2):toBe(2)
            end)
        end)

        test("関数の戻り値の文字列・数値・nil はそのまま返る", function()
            local act, p = registered()
            act.current_scene = {
                ["文字"] = function() return "文字列" end,
                ["数"] = function() return 42 end,
                ["無"] = function() return nil end,
            }
            expect(p:word("文字")):toBe("文字列")
            expect(p:word("数")):toBe(42)
            expect(p:word("無")):toBeNil()
            expect(p:expr_fn("文字")):toBe("文字列")
            expect(p:expr_fn("数")):toBe(42)
            expect(p:expr_fn("無")):toBeNil()
        end)

        test("関数でない値は文字列にして返す（アクターのフィールド・シーンテーブル）", function()
            local act, p = registered({ ["年齢"] = 17 })
            act.current_scene = { ["季節"] = "夏" }
            expect(p:word("年齢")):toBe("17")
            expect(p:word("季節")):toBe("夏")
        end)

        test("見つからない名前は word・expr_fn とも警告 1 行と nil", function()
            local _, p = registered()
            capture_warns(function(warns)
                expect(p:word("apd_どこにもない")):toBeNil()
                expect(#warns):toBe(1)
                expect(warns[1]):toBe(string.format(
                    "proxy:word - handler not found: key='apd_どこにもない', mode='word', via=proxy(%s)", ACTOR_NAME))
            end)
            capture_warns(function(warns)
                expect(p:expr_fn("apd_どこにもない", 1)):toBeNil()
                expect(#warns):toBe(1)
                expect(warns[1]):toBe(string.format(
                    "proxy:expr_fn - handler not found: key='apd_どこにもない', mode='expr', via=proxy(%s)", ACTOR_NAME))
            end)
        end)
    end)
end)
