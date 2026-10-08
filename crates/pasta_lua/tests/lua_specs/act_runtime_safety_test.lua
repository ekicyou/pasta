-- 生成コードが呼ぶ存在確認付き act メソッドのランタイムテスト (dsl-codegen-runtime-safety)
-- act:global_fn: ＠＊名前（…）の呼び出し（未定義・関数でないときは警告して値なし）
-- act:actor_proxy: アクション行のアクター解決（未登録は目印付きのその場限りのアクター）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local GLOBAL = require("pasta.global")

--- 警告を記録するログを差し込んだ pasta.act を新規ロードして body を実行し、
--- 実行後に元のモジュールと GLOBAL のテスト用キーを戻す（他スイートを汚さない）
--- @param body fun(ACT: table, warns: string[])
local function with_captured_act(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_act = package.loaded["pasta.act"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.act"] = nil
    local ok, err = pcall(function()
        body(require("pasta.act"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.act"] = saved_act
    GLOBAL["rs_関数"] = nil
    GLOBAL["rs_値"] = nil
    GLOBAL["rs_失敗"] = nil
    if not ok then error(err, 0) end
end

describe("act:global_fn - 定義済みのグローバル関数", function()
    test("act を第 1 引数、続けて引数を渡して呼び、戻り値をすべて返す（警告なし）", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local got_self, got_a, got_b
            GLOBAL["rs_関数"] = function(a, x, y)
                got_self, got_a, got_b = a, x, y
                return x + y, "二つ目"
            end
            local r1, r2 = act:global_fn("rs_関数", 1, 2)
            expect(r1):toBe(3)
            expect(r2):toBe("二つ目")
            expect(got_self):toBe(act)
            expect(got_a):toBe(1)
            expect(got_b):toBe(2)
            expect(#warns):toBe(0)
        end)
    end)

    test("関数の中で起きたエラーは握りつぶさず伝わる", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            GLOBAL["rs_失敗"] = function() error("中のエラー") end
            local ok, err = pcall(act.global_fn, act, "rs_失敗")
            expect(ok):toBe(false)
            expect(tostring(err):find("中のエラー", 1, true) ~= nil):toBe(true)
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("act:global_fn - 未定義・関数でない値", function()
    test("未定義は値なしを返し、関数名を含む警告を 1 行出す", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            expect(select("#", act:global_fn("rs_関数", 1))):toBe(1)
            expect(act:global_fn("rs_関数", 1)):toBeNil()
            expect(#warns):toBe(2)
            expect(warns[1]):toBe("act:global_fn - function not found: key='rs_関数'")
        end)
    end)

    test("関数でない値は値なしを返し、関数名を含む警告を 1 行出す", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            GLOBAL["rs_値"] = "文字列"
            expect(act:global_fn("rs_値")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:global_fn - function not found: key='rs_値'")
        end)
    end)

    test("警告は act:expr_fn の未検出と同じレベル（warn）で出る", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            act:expr_fn("rs_存在しない")
            local expr_warned = false
            for _, w in ipairs(warns) do
                if w:find("act:expr_fn", 1, true) then expr_warned = true end
            end
            expect(expr_warned):toBe(true)
            local before = #warns
            act:global_fn("rs_関数")
            expect(#warns):toBe(before + 1)
            expect(warns[#warns]):toBe("act:global_fn - function not found: key='rs_関数'")
        end)
    end)
end)

-- act:actor_proxy: アクション行のアクター解決（登録済みは act.名前 と同じ・未登録はその場限りのアクター）
local STORE = require("pasta.store")

--- act の自身のキー一覧（フィールドを足していないことの確認用）
local function own_keys(t)
    local keys = {}
    for k in pairs(t) do table.insert(keys, tostring(k)) end
    table.sort(keys)
    return table.concat(keys, ",")
end

describe("act:actor_proxy - 登録済みアクター", function()
    test("act.名前 と同じアクターのトークンを積み、ログ・トークンを足さない", function()
        with_captured_act(function(ACT, warns)
            local actors = { ["さくら"] = { name = "さくら" } }
            local act = ACT.new(actors)
            local p = act:actor_proxy("さくら")
            expect(#act.token):toBe(0)
            p:talk("こんにちは")
            act["さくら"]:talk("こんにちは")
            expect(#act.token):toBe(2)
            expect(act.token[1].actor):toBe(actors["さくら"])
            expect(act.token[1].actor):toBe(act.token[2].actor)
            expect(act.token[1].text):toBe(act.token[2].text)
            expect(#warns):toBe(0)
        end)
    end)

    test("talk・var・save・actors という名前の登録済みアクターが話せる", function()
        with_captured_act(function(ACT, warns)
            local names = { "talk", "var", "save", "actors" }
            local actors = {}
            for _, n in ipairs(names) do actors[n] = { name = n } end
            local act = ACT.new(actors)
            for _, n in ipairs(names) do
                act:actor_proxy(n):talk(n .. "です")
            end
            expect(#act.token):toBe(4)
            for i, n in ipairs(names) do
                expect(act.token[i].type):toBe("talk")
                expect(act.token[i].actor):toBe(actors[n])
                expect(act.token[i].text):toBe(n .. "です")
            end
            expect(#warns):toBe(0)
        end)
    end)

    test("talk という名前のアクターを登録しても、手書き Lua の act.talk はメソッドを返す", function()
        with_captured_act(function(ACT)
            local act = ACT.new({ talk = { name = "talk" } })
            expect(act.talk):toBe(ACT.IMPL.talk)
        end)
    end)
end)

describe("act:actor_proxy - 未登録アクター", function()
    test("1 行の複数アクションで目印と警告は 1 回、同じ actor テーブルで積む", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local keys_before = own_keys(act)
            act:actor_proxy("rs_謎"):talk("こんにちは")
            act:actor_proxy("rs_謎"):sakura_script("\n")
            act:actor_proxy("rs_謎"):talk("さよなら")
            expect(#act.token):toBe(4)
            local adhoc = act.token[1].actor
            expect(act.token[1].type):toBe("talk")
            expect(act.token[1].text):toBe("【未登録アクター：rs_謎】")
            expect(act.token[2].text):toBe("こんにちは")
            expect(act.token[3].type):toBe("sakura_script")
            for i = 2, 4 do expect(act.token[i].actor):toBe(adhoc) end
            expect(adhoc.name):toBe("rs_謎")
            expect(getmetatable(adhoc)):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:actor_proxy - unregistered actor: name='rs_謎'")
            expect(own_keys(act)):toBe(keys_before)
        end)
    end)

    test("実行後も STORE.actors[名前]・act.名前・act.actors[名前] は nil のまま", function()
        with_captured_act(function(ACT)
            local act = ACT.new({})
            act:actor_proxy("rs_謎"):talk("やあ")
            expect(STORE.actors["rs_謎"]):toBeNil()
            expect(act["rs_謎"]):toBeNil()
            expect(act.actors["rs_謎"]):toBeNil()
        end)
    end)

    test("話者が別のアクターに切り替わってから戻ると、目印と警告がもう一度出る", function()
        with_captured_act(function(ACT, warns)
            local actors = { ["さくら"] = { name = "さくら" } }
            local act = ACT.new(actors)
            act:actor_proxy("rs_謎"):talk("一")
            act:actor_proxy("さくら"):talk("二")
            act:actor_proxy("rs_謎"):talk("三")
            act:actor_proxy("rs_別"):talk("四")
            expect(#act.token):toBe(7)
            expect(act.token[4].text):toBe("【未登録アクター：rs_謎】")
            expect(act.token[4].actor == act.token[1].actor):toBe(false)
            expect(act.token[6].text):toBe("【未登録アクター：rs_別】")
            expect(#warns):toBe(3)
        end)
    end)

    test("未登録アクターの行の＠関数（）は act の検索へ委譲されて解決し、関数は ACT を受け取る", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local received = nil
            GLOBAL["rs_関数"] = function(a, x)
                received = a
                return "結果:" .. x
            end
            local p = act:actor_proxy("rs_謎")
            p:talk(p:expr_fn("rs_関数", "x"))
            expect(received):toBe(act)
            expect(act.token[2].text):toBe("結果:x")
            expect(#warns):toBe(1)
        end)
    end)

    test("直前の話者が sakura_script のトークンでも切り替わりとして扱う", function()
        with_captured_act(function(ACT, warns)
            local actors = { ["さくら"] = { name = "さくら" } }
            local act = ACT.new(actors)
            act:actor_proxy("さくら"):talk("a")
            act:actor_proxy("rs_謎"):talk("b")
            act:actor_proxy("さくら"):sakura_script("\n")
            act:actor_proxy("rs_謎"):talk("c")
            expect(act.token[5].text):toBe("【未登録アクター：rs_謎】")
            expect(#warns):toBe(2)
        end)
    end)

    test("話者を持たないトークンは飛ばして直前の話者を探し、build 後は目印が再び付く", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            act:actor_proxy("rs_謎"):talk("一")
            act:surface(1)
            act:actor_proxy("rs_謎"):talk("二")
            expect(#act.token):toBe(4)
            expect(act.token[4].actor):toBe(act.token[1].actor)
            expect(#warns):toBe(1)
            act:build()
            act:actor_proxy("rs_謎"):talk("三")
            expect(act.token[1].text):toBe("【未登録アクター：rs_謎】")
            expect(#warns):toBe(2)
        end)
    end)
end)

-- act:arith: 算術式の生成コードが呼ぶ数値の二項演算（数値にできない被演算子は値なし＋警告）
local NATIVE_OPS = {
    ["+"] = function(a, b) return a + b end,
    ["-"] = function(a, b) return a - b end,
    ["*"] = function(a, b) return a * b end,
    ["/"] = function(a, b) return a / b end,
    ["%"] = function(a, b) return a % b end,
}

--- 変更前の生成コード（Lua のネイティブ演算）の結果。エラーなら ok=false
local function native(op, a, b)
    return pcall(NATIVE_OPS[op], a, b)
end

describe("act:arith - 数値にできる被演算子（ネイティブ演算と同じ結果）", function()
    test("数値・数値文字列・「1」＋2・負数の剰余・0 除算がネイティブ演算と一致し、警告なし", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local cases = {
                { "+", 1, 2 }, { "-", 1, 2 }, { "*", 3, 4 }, { "/", 7, 2 }, { "%", 7, 3 },
                { "/", 1, 3 }, { "+", "1", 2 }, { "+", 1, "2" }, { "*", "1.5", "2" },
                { "%", -7, 3 }, { "%", 7, -3 }, { "%", -7.5, 2 },
                { "/", 1, 0 }, { "/", -1, 0 }, { "%", 5, 0 }, { "/", 0, 0 },
            }
            for _, c in ipairs(cases) do
                local op, a, b = c[1], c[2], c[3]
                local ok, want = native(op, a, b)
                expect(ok):toBe(true)
                local got = act:arith(op, a, b)
                if want ~= want then
                    expect(got ~= got):toBe(true)
                else
                    expect(got):toBe(want)
                end
            end
            expect(act:arith("+", "1", 2)):toBe(3)
            expect(act:arith("/", 1, 2)):toBe(0.5)
            expect(#warns):toBe(0)
        end)
    end)

    test("文字列の数値化の範囲が変更前の暗黙変換と一致する（16 進・指数・空白・全角・空文字列）", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local inputs = {
                "0x10", "0XfF", "1e2", "1E-1", "2.5e+1", " 1 ", "\t2\n", "  3", "4  ",
                "-3", "+3", ".5", "5.", "１２", "１", "", "   ", "abc", "1a", "0x", "1e", "1 2",
            }
            for _, s in ipairs(inputs) do
                local ok, want = native("+", s, 0)
                local before = #warns
                local got = act:arith("+", s, 0)
                if ok then
                    expect(got):toBe(want)
                    expect(#warns):toBe(before)
                else
                    expect(got):toBeNil()
                    expect(#warns):toBe(before + 1)
                end
            end
            -- 全角数字・空文字列は変換しない（3.7）
            expect(act:arith("+", "１２", 0)):toBeNil()
            expect(act:arith("+", "", 0)):toBeNil()
            expect(act:arith("+", "0x10", 0)):toBe(16)
            expect(act:arith("+", " 1 ", 0)):toBe(1)
            expect(act:arith("+", "1e2", 0)):toBe(100)
        end)
    end)
end)

describe("act:arith - 数値にできない被演算子", function()
    test("説明ありの nil は値なし＋演算子と説明を含む警告", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            expect(act:arith("+", nil, 1, "var.x")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:arith - operand is not a number: op='+', operand='var.x', value=nil")
        end)
    end)

    test("説明なしの非数値文字列は値なし＋値と種類を含む警告（文字列どうしの＋は連結しない）", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            expect(act:arith("+", "a", 1)):toBeNil()
            expect(warns[1]):toBe("act:arith - operand is not a number: op='+', value='a' (string)")
            expect(act:arith("+", "a", "b")):toBeNil()
            expect(#warns):toBe(3)
            expect(warns[2]):toBe("act:arith - operand is not a number: op='+', value='a' (string)")
            expect(warns[3]):toBe("act:arith - operand is not a number: op='+', value='b' (string)")
        end)
    end)

    test("真偽値・全角数字は値なし＋警告、説明ありなら説明と値を含む", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            expect(act:arith("*", 2, true)):toBeNil()
            expect(warns[1]):toBe("act:arith - operand is not a number: op='*', value=true (boolean)")
            expect(act:arith("-", "１２", 1, "var.y")):toBeNil()
            expect(warns[2]):toBe("act:arith - operand is not a number: op='-', operand='var.y', value='１２' (string)")
            expect(#warns):toBe(2)
        end)
    end)

    test("テーブルは数値にできない扱いで、メタメソッドを呼ばない", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local called = false
            local mt = {
                __add = function() called = true; return 1 end,
                __tostring = function() called = true; return "x" end,
            }
            local t = setmetatable({}, mt)
            expect(act:arith("+", t, 1)):toBeNil()
            expect(called):toBe(false)
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:arith - operand is not a number: op='+', value=(table)")
        end)
    end)

    test("入れ子で内側が失敗すると外側は値なし＋追加の警告なし", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            -- （＄x＋1）＊2 ／ 1＋2＊＄y
            expect(act:arith("*", (act:arith("+", nil, 1, "var.x")), 2)):toBeNil()
            expect(#warns):toBe(1)
            expect(act:arith("+", 1, act:arith("*", 2, nil, nil, "var.y"))):toBeNil()
            expect(#warns):toBe(2)
            expect(warns[2]):toBe("act:arith - operand is not a number: op='*', operand='var.y', value=nil")
        end)
    end)

    test("未知の演算子は警告して値なし、act の状態に触れない", function()
        with_captured_act(function(ACT, warns)
            local act = ACT.new({})
            local keys_before = own_keys(act)
            expect(act:arith("^", 2, 3)):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:arith - unknown operator: op='^'")
            expect(act:arith("+", 1, 2)):toBe(3)
            expect(#act.token):toBe(0)
            expect(own_keys(act)):toBe(keys_before)
        end)
    end)
end)

-- PASTA.num・PASTA.str: 算術・連結の被演算子を必ず数値・文字列にする変換（生成コードが pasta モジュールから呼ぶ）

--- with_captured_act と同じく警告を記録し、pasta（init.lua）も新規ロードして body に渡す
--- @param body fun(PASTA: table, warns: string[])
local function with_captured_pasta(body)
    local saved_pasta = package.loaded["pasta"]
    package.loaded["pasta"] = nil
    local ok, err = pcall(with_captured_act, function(_, warns)
        body(require("pasta"), warns)
    end)
    package.loaded["pasta"] = saved_pasta
    if not ok then error(err, 0) end
end

describe("PASTA.num・PASTA.str - 被演算子の変換（基本の契約）", function()
    test("pasta.act の ACT.num・ACT.str と同一関数として公開される", function()
        with_captured_pasta(function(PASTA)
            local ACT = require("pasta.act")
            expect(PASTA.num):toBe(ACT.num)
            expect(PASTA.str):toBe(ACT.str)
        end)
    end)

    test("num: 数値・数字だけの文字列は数値、nil は黙って 0、変換できない値は従来の警告 1 行で 0", function()
        with_captured_pasta(function(PASTA, warns)
            expect(PASTA.num("+", 3)):toBe(3)
            expect(PASTA.num("+", "0x10")):toBe(16)
            expect(PASTA.num("+", nil)):toBe(0)
            expect(PASTA.num("-", nil, "var.x")):toBe(0)
            expect(#warns):toBe(0)
            expect(PASTA.num("*", "abc", "var.y")):toBe(0)
            expect(warns[1]):toBe("act:arith - operand is not a number: op='*', operand='var.y', value='abc' (string)")
            local called = false
            local mt = { __add = function() called = true end, __tostring = function() called = true end }
            local t = setmetatable({}, mt)
            expect(PASTA.num("+", t)):toBe(0)
            expect(warns[2]):toBe("act:arith - operand is not a number: op='+', value=(table)")
            expect(called):toBe(false)
            expect(#warns):toBe(2)
        end)
    end)

    test("str: 文字列・数値は文字列、nil は黙って空文字列、変換できない値は従来の警告 1 行で空文字列", function()
        with_captured_pasta(function(PASTA, warns)
            expect(PASTA.str("合計")):toBe("合計")
            expect(PASTA.str(3.5)):toBe("3.5")
            expect(PASTA.str(nil)):toBe("")
            expect(PASTA.str(nil, "var.x")):toBe("")
            expect(#warns):toBe(0)
            expect(PASTA.str(true, "var.b")):toBe("")
            expect(warns[1]):toBe(
                "act:concat - operand is not a string or number: op='&', operand='var.b', value=true (boolean)")
            local called = false
            local mt = { __concat = function() called = true end, __tostring = function() called = true end }
            local t = setmetatable({}, mt)
            expect(PASTA.str(t)):toBe("")
            expect(warns[2]):toBe("act:concat - operand is not a string or number: op='&', value=(table)")
            expect(called):toBe(false)
            expect(#warns):toBe(2)
        end)
    end)
end)
