-- 実行時の失敗 14 か所の現行の挙動の特性化テスト (failure-output-unification 1.1)
-- 失敗の対応表 #1〜#13 と act:failure を Lua の API から直接呼び、ログの警告の文言・戻り値・積むトークンを固定する。
-- 以降のタスクでは、載せ替えた種類の期待値（主に積むトークンの表記）だけを書き換える。
-- 今は警告を出さない状況（値なし・空の名前）で警告も表記も出ないことも固定する（要件 2.11）。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 警告を記録する表を差し込み、pasta.word → pasta.actor → pasta.act を新規ロードして body を実行する。
-- 実行後に元のモジュールとログへ戻す（他スイートを汚さない）
local RELOAD = { "pasta.word", "pasta.actor", "pasta.act" }

--- @param body fun(ACT: table, WORD: table, warns: string[])
local function with_sites(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved = {}
    for _, name in ipairs(RELOAD) do
        saved[name] = package.loaded[name]
        package.loaded[name] = nil
    end
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    local ok, err = pcall(function()
        local ACT = require("pasta.act")
        body(ACT, require("pasta.word"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    for _, name in ipairs(RELOAD) do
        package.loaded[name] = saved[name]
    end
    if not ok then error(err, 0) end
end

local ACTOR_NAME = "fs_さくら"

--- 登録済みアクター 1 人の act を作る
local function new_act(ACT)
    return ACT.new({ [ACTOR_NAME] = { name = ACTOR_NAME } })
end

--- 積んだトークンを "種類|アクター名|テキスト" の配列にする
local function dump(act)
    local lines = {}
    for _, t in ipairs(act.token) do
        local name = t.actor and t.actor.name or "-"
        table.insert(lines, string.format("%s|%s|%s", t.type, name, tostring(t.text)))
    end
    return table.concat(lines, "\n")
end

describe("失敗の対応表 #1 act:talk", function()
    test("変数パス付きの nil は警告 1 行・表記 1 つ・self を返す", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:talk(act.actors[ACTOR_NAME], nil, "var.話題")):toBe(act)
            expect(dump(act)):toBe("raw_script|-|【値の無い変数：＄話題】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:talk - undefined variable: 'var.話題'")
        end)
    end)

    test("変数パスの無い nil（内側が報告済み）は黙る", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:talk(act.actors[ACTOR_NAME], nil)):toBe(act)
            expect(dump(act)):toBe("")
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("失敗の対応表 #2〜#6 見つからない単語・関数", function()
    test("#2 act:word は nil・警告 1 行・表記 1 つ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:word("fs_無い単語")):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【単語が見つからない：＠fs_無い単語】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:word - handler not found: key='fs_無い単語', mode='word', via=act")
        end)
    end)

    test("#3 proxy:word は nil・警告 1 行・表記 1 つ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(proxy:word("fs_無い単語")):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【単語が見つからない：＠fs_無い単語】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(
                "proxy:word - handler not found: key='fs_無い単語', mode='word', via=proxy(fs_さくら)")
        end)
    end)

    test("#4 act:expr_fn は nil・警告 1 行・表記 1 つ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:expr_fn("fs_無い関数", 1)):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【関数が見つからない：＠fs_無い関数（）】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:expr_fn - handler not found: key='fs_無い関数', mode='expr', via=act")
        end)
    end)

    test("#5 proxy:expr_fn は nil・警告 1 行・表記 1 つ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(proxy:expr_fn("fs_無い関数")):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【関数が見つからない：＠fs_無い関数（）】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(
                "proxy:expr_fn - handler not found: key='fs_無い関数', mode='expr', via=proxy(fs_さくら)")
        end)
    end)

    test("#6 act:global_fn は nil・警告 1 行・表記 1 つ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:global_fn("fs_無い関数")):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【関数が見つからない：＠＊fs_無い関数（）】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:global_fn - function not found: key='fs_無い関数'")
        end)
    end)
end)

describe("失敗の対応表 #7 WORD.dynamic_key（動的参照の値が名前にならない）", function()
    test("Lua から直接呼ぶとログだけ（nil・警告 1 行）", function()
        with_sites(function(_, WORD, warns)
            expect(WORD.dynamic_key(nil, "var.x", "act:word")):toBeNil()
            expect(WORD.dynamic_key("", "save.x", "act:word")):toBeNil()
            expect(WORD.dynamic_key(true, "args[1]", "act:word")):toBeNil()
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:word - undefined variable: 'var.x'",
                "act:word - empty variable: 'save.x'",
                "act:word - unsupported value type: 'args[1]' (boolean)",
            }, "\n"))
        end)
    end)

    test("act を渡すと表記を 1 つ積み、戻り値と警告の文言は act なしと同じ", function()
        with_sites(function(ACT, WORD, warns)
            local act = new_act(ACT)
            expect(WORD.dynamic_key(nil, "var.x", "act:word", act)):toBeNil()
            expect(WORD.dynamic_key("", "save.x", "act:word", act)):toBeNil()
            expect(WORD.dynamic_key(true, "args[1]", "act:word", act)):toBeNil()
            expect(WORD.dynamic_key("名前", "var.x", "act:word", act)):toBe("名前")
            expect(WORD.dynamic_key(1, "var.x", "act:word", act)):toBe("1")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【名前にならない値：＄x（値なし）】",
                "raw_script|-|【名前にならない値：＄＊x（空文字列）】",
                "raw_script|-|【名前にならない値：＄０（真偽値）】",
            }, "\n"))
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:word - undefined variable: 'var.x'",
                "act:word - empty variable: 'save.x'",
                "act:word - unsupported value type: 'args[1]' (boolean)",
            }, "\n"))
        end)
    end)

    test("act・proxy の動的参照は表記 1 つ・警告 1 行で、外側の検索の「見つからない」を重ねない", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(act:word(nil, "var.x")):toBeNil()
            expect(act:expr_fn_var("", "var.x")):toBeNil()
            expect(proxy:word(true, "var.x")):toBeNil()
            expect(proxy:expr_fn_var(nil, "args[2]")):toBeNil()
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【名前にならない値：＄x（値なし）】",
                "raw_script|-|【名前にならない値：＄x（空文字列）】",
                "raw_script|-|【名前にならない値：＄x（真偽値）】",
                "raw_script|-|【名前にならない値：＄１（値なし）】",
            }, "\n"))
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:word - undefined variable: 'var.x'",
                "act:expr_fn - empty variable: 'var.x'",
                "proxy:word - unsupported value type: 'var.x' (boolean)",
                "proxy:expr_fn - undefined variable: 'args[2]'",
            }, "\n"))
        end)
    end)

    test("変数パス付きの act:call_key は警告 1 行と Call の失敗表記 1 つ・「呼ばない」印", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local key = act:call_key(nil, "var.行き先")
            expect(type(key)):toBe("table")
            expect(act:call(nil, key)):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【名前にならない値：＄行き先（値なし）】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:call - undefined variable: 'var.行き先'")
        end)
    end)

    test("動的な関数の参照が名前にならないとき（＞＠＄名前（））は call_key が重ねて出さない", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(type(act:call_key(act:expr_fn_var(nil, "var.x"), nil, "@$var.x()"))):toBe("table")
            expect(type(act:call_key(proxy:expr_fn_var(true, "var.y"), nil, "@$var.y()"))):toBe("table")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【名前にならない値：＄x（値なし）】",
                "raw_script|-|【名前にならない値：＄y（真偽値）】",
            }, "\n"))
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:expr_fn - undefined variable: 'var.x'",
                "proxy:expr_fn - unsupported value type: 'var.y' (boolean)",
            }, "\n"))
        end)
    end)
end)

describe("失敗の対応表 #8 act:actor_proxy（未登録アクター）", function()
    test("話者がその名前に切り替わるたびにアクター付きの目印の raw_script 1 つと警告 1 行", function()
        with_sites(function(ACT, _, warns)
            local FAILURE = require("pasta.failure")
            local act = new_act(ACT)
            local p1 = act:actor_proxy("fs_ぱすたー")
            p1:talk("こんにちは")
            act:actor_proxy("fs_ぱすたー"):talk("続き")
            act:actor_proxy(ACTOR_NAME):talk("やあ")
            act:actor_proxy("fs_ぱすたー")
            expect(dump(act)):toBe(table.concat({
                "raw_script|fs_ぱすたー|【未登録アクター：fs_ぱすたー】",
                "talk|fs_ぱすたー|こんにちは",
                "talk|fs_ぱすたー|続き",
                "talk|fs_さくら|やあ",
                "raw_script|fs_ぱすたー|【未登録アクター：fs_ぱすたー】",
            }, "\n"))
            expect(#warns):toBe(2)
            expect(warns[1]):toBe("act:actor_proxy - unregistered actor: name='fs_ぱすたー'")
            expect(warns[2]):toBe("act:actor_proxy - unregistered actor: name='fs_ぱすたー'")
            expect(act.actors["fs_ぱすたー"]):toBeNil()
            -- その場限りのアクターには未登録の印が付き、登録済みのアクターには付かない
            expect(FAILURE.is_unregistered_actor(act.token[1].actor)):toBe(true)
            expect(FAILURE.is_unregistered_actor(act.token[4].actor)):toBe(false)
        end)
    end)

    test("遡りは目印を話者として数え、アクターの無い失敗表記では止まらない", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act:actor_proxy("fs_ぱすたー"):word("fs_無い単語")
            act:actor_proxy("fs_ぱすたー"):talk("続き")
            expect(dump(act)):toBe(table.concat({
                "raw_script|fs_ぱすたー|【未登録アクター：fs_ぱすたー】",
                "raw_script|-|【単語が見つからない：＠fs_無い単語】",
                "talk|fs_ぱすたー|続き",
            }, "\n"))
            expect(act.token[3].actor):toBe(act.token[1].actor)
            expect(#warns):toBe(2)
        end)
    end)
end)

describe("失敗の対応表 #9・#10 ACT.num・ACT.str", function()
    test("#9 数にできない値は 0・警告 1 行・実行中の act に表記 1 つ、nil は黙って 0", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act:init_scene({})
            expect(ACT.num("+", "abc", "var.x")):toBe(0)
            expect(ACT.num("*", {}, nil)):toBe(0)
            expect(ACT.num("+", nil, "var.x")):toBe(0)
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:arith - operand is not a number: op='+', operand='var.x', value='abc' (string)",
                "act:arith - operand is not a number: op='*', value=(table)",
            }, "\n"))
            expect(dump(act)):toBe(table.concat({
                'raw_script|-|【数にできない値：＄x（"abc"）】',
                "raw_script|-|【数にできない値：表】",
            }, "\n"))
        end)
    end)

    test("#10 文字列にできない値は空文字列・警告 1 行・実行中の act に表記 1 つ、nil は黙って空文字列", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act:init_scene({})
            expect(ACT.str(true, "@真偽()")):toBe("")
            expect(ACT.str(nil, "var.x")):toBe("")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(
                "act:concat - operand is not a string or number: op='&', operand='@真偽()', value=true (boolean)")
            expect(dump(act)):toBe("raw_script|-|【文字列にできない値：＠真偽（）（true）】")
        end)
    end)

    test("#9・#10 実行中の act が無いコルーチンではログだけ（落とさない）", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act:init_scene({})
            local co = coroutine.create(function()
                return ACT.num("+", "abc", "var.x"), ACT.str(true, "var.t")
            end)
            local ok, n, s = coroutine.resume(co)
            expect(ok):toBe(true)
            expect(n):toBe(0)
            expect(s):toBe("")
            expect(#warns):toBe(2)
            expect(#act.token):toBe(0)
        end)
    end)
end)

describe("失敗の対応表 #11〜#13 Call", function()
    test("#11 act:call(nil) は nil・失敗表記 1 つ・警告 1 行", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:call(nil, nil)):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【名前にならない値：値なし】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:call - nil key (undefined variable?), skipping scene search")
        end)
    end)

    test("#12 見つからないシーンは nil・Call の失敗表記 1 つ・警告 1 行", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:call(nil, "fs_無いシーン")):toBeNil()
            expect(dump(act)):toBe("raw_script|-|【シーンが見つからない：＞fs_無いシーン】")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:call - handler not found: key='fs_無いシーン', mode='scene', via=act")
        end)
    end)

    test("#13 変数パスの無い act:call_key は失敗表記 1 つ・警告 1 行・「呼ばない」印", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(type(act:call_key(nil, nil, "@f()"))):toBe("table")
            expect(type(act:call_key("", nil, "@f()"))):toBe("table")
            expect(type(act:call_key(true, nil, nil))):toBe("table")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【名前にならない値：＠f（）（値なし）】",
                "raw_script|-|【名前にならない値：＠f（）（空文字列）】",
                "raw_script|-|【名前にならない値：真偽値】",
            }, "\n"))
            expect(table.concat(warns, "\n")):toBe(table.concat({
                "act:call - key is not a string or number: operand='@f()', value=nil",
                "act:call - key is not a string or number: operand='@f()', value='' (string)",
                "act:call - key is not a string or number: value=true (boolean)",
            }, "\n"))
        end)
    end)

    test("#13 値 nil で説明も無い（手書きの Lua）ときは警告なしで失敗表記だけ", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(type(act:call_key(nil, nil, nil))):toBe("table")
            expect(dump(act)):toBe("raw_script|-|【名前にならない値：値なし】")
            expect(#warns):toBe(0)
        end)
    end)

    test("名前になる値はそのままキーにして黙る（数値は tostring）", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:call_key("会話", "var.x")):toBe("会話")
            expect(act:call_key(2, nil, "@f()")):toBe("2")
            expect(dump(act)):toBe("")
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("関数の参照の「報告済み」の印と call_key（重ねない規則）", function()
    test("見つからない関数の直後の call_key は黙って「呼ばない」印を返す（act・proxy・グローバル）", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(type(act:call_key(act:expr_fn("fs_無い関数"), nil, "@fs_無い関数()"))):toBe("table")
            expect(type(act:call_key(proxy:expr_fn("fs_無い関数"), nil, "@fs_無い関数()"))):toBe("table")
            expect(type(act:call_key(act:global_fn("fs_無い関数"), nil, "@*fs_無い関数()"))):toBe("table")
            expect(type(act:call_key(act:expr_fn_var("fs_無い関数", "var.x"), nil, "@$var.x()"))):toBe("table")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【関数が見つからない：＠fs_無い関数（）】",
                "raw_script|-|【関数が見つからない：＠fs_無い関数（）】",
                "raw_script|-|【関数が見つからない：＠＊fs_無い関数（）】",
                "raw_script|-|【関数が見つからない：＠fs_無い関数（）】",
            }, "\n"))
            expect(#warns):toBe(4)
        end)
    end)

    test("見つかった関数が nil を返したときは現行どおり call_key が報告する（内側の失敗の印は戻った後に偽へ戻す）", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act.current_scene = {
                ["fs_値なし"] = function(a)
                    a:expr_fn("fs_無い関数") -- 関数の中の失敗が印を真にしても、戻った時点で偽に戻す
                    return nil
                end,
            }
            expect(type(act:call_key(act:expr_fn("fs_値なし"), nil, "@fs_値なし()"))):toBe("table")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【関数が見つからない：＠fs_無い関数（）】",
                "raw_script|-|【名前にならない値：＠fs_値なし（）（値なし）】",
            }, "\n"))
            expect(#warns):toBe(2)
        end)
    end)

    test("印は call_key が読んで偽に戻す（次の call_key は現行どおり報告する）", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            act:call_key(act:expr_fn("fs_無い関数"), nil, "@fs_無い関数()")
            act:call_key(nil, nil, "@f()")
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【関数が見つからない：＠fs_無い関数（）】",
                "raw_script|-|【名前にならない値：＠f（）（値なし）】",
            }, "\n"))
            expect(#warns):toBe(2)
        end)
    end)
end)

describe("act:failure（Lua からの報告）", function()
    test("【text】（エスケープ済み）を raw_script で 1 つ積み、警告があれば 1 行出し、nil を返す", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            expect(act:failure("Lua失敗：\\-", "test - direct failure")):toBeNil()
            expect(act:failure("警告なし")):toBeNil()
            expect(dump(act)):toBe(table.concat({
                "raw_script|-|【Lua失敗：\\\\-】",
                "raw_script|-|【警告なし】",
            }, "\n"))
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("test - direct failure")
        end)
    end)
end)

describe("今は警告を出さない状況（要件 2.11）", function()
    test("静的な単語参照の空の名前・nil は黙って nil", function()
        with_sites(function(ACT, _, warns)
            local act = new_act(ACT)
            local proxy = act:actor_proxy(ACTOR_NAME)
            expect(act:word("")):toBeNil()
            expect(act:word(nil)):toBeNil()
            expect(proxy:word("")):toBeNil()
            expect(proxy:word(nil)):toBeNil()
            expect(dump(act)):toBe("")
            expect(#warns):toBe(0)
        end)
    end)
end)
