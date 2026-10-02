-- 動的参照（＠＄名前・＠＄名前（…））のランタイムテスト (dynamic-word-reference)
-- WORD.dynamic_key: 参照変数の値 → 検索キー（使えない値は警告して nil）
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- 警告を記録するログを差し込んだ pasta.word を新規ロードして body を実行し、
--- 実行後に元のモジュールへ戻す（他スイートが保持する WORD/log を汚さない）
--- @param body fun(WORD: table, warns: string[])
local function with_captured_word(body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_word = package.loaded["pasta.word"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["pasta.word"] = nil
    local ok, err = pcall(function()
        body(require("pasta.word"), warns)
    end)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.word"] = saved_word
    if not ok then error(err, 0) end
end

describe("WORD.dynamic_key - 検索キーになる値", function()
    test("空でない文字列はそのまま返し、警告しない", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("挨拶", "var.x", "act:word")):toBe("挨拶")
            expect(#warns):toBe(0)
        end)
    end)

    test("文字列を DSL として読み直さない（＠・＄を含んでもそのままキーにする）", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("＠＄y", "var.x", "act:word")):toBe("＠＄y")
            expect(#warns):toBe(0)
        end)
    end)

    test("数値は変数展開（tostring）と同じ表記の文字列にする", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(1, "var.n", "act:word")):toBe("1")
            expect(WORD.dynamic_key(1.5, "var.n", "act:word")):toBe(tostring(1.5))
            expect(WORD.dynamic_key(-3, "var.n", "act:word")):toBe(tostring(-3))
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("WORD.dynamic_key - 使えない値は nil と区別できる警告", function()
    test("未代入（nil）は undefined variable 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(nil, "var.x", "act:word")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:word - undefined variable: 'var.x'")
        end)
    end)

    test("空文字列は empty variable 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key("", "save.x", "proxy:word")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("proxy:word - empty variable: 'save.x'")
        end)
    end)

    test("真偽値は型名付きの unsupported value type 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key(true, "args[1]", "act:expr_fn")):toBeNil()
            expect(WORD.dynamic_key(false, "var.x", "act:word")):toBeNil()
            expect(warns[1]):toBe("act:expr_fn - unsupported value type: 'args[1]' (boolean)")
            expect(warns[2]):toBe("act:word - unsupported value type: 'var.x' (boolean)")
        end)
    end)

    test("テーブル・関数・thread は型名付きの unsupported value type 警告", function()
        with_captured_word(function(WORD, warns)
            expect(WORD.dynamic_key({}, "var.t", "proxy:expr_fn")):toBeNil()
            expect(WORD.dynamic_key(function() end, "var.f", "act:word")):toBeNil()
            expect(WORD.dynamic_key(coroutine.create(function() end), "var.c", "act:word")):toBeNil()
            expect(warns[1]):toBe("proxy:expr_fn - unsupported value type: 'var.t' (table)")
            expect(warns[2]):toBe("act:word - unsupported value type: 'var.f' (function)")
            expect(warns[3]):toBe("act:word - unsupported value type: 'var.c' (thread)")
        end)
    end)

    test("__tostring を持つテーブルも型不正（文字列化しない）", function()
        with_captured_word(function(WORD, warns)
            local obj = setmetatable({}, { __tostring = function() return "挨拶" end })
            expect(WORD.dynamic_key(obj, "var.o", "act:word")):toBeNil()
            expect(warns[1]):toBe("act:word - unsupported value type: 'var.o' (table)")
        end)
    end)
end)

describe("WORD.dynamic_key - 依存方向", function()
    test("pasta.word の読み込みで pasta.act・pasta.actor を読み込まない", function()
        local saved = {
            act = package.loaded["pasta.act"],
            actor = package.loaded["pasta.actor"],
        }
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.actor"] = nil
        local ok, err = pcall(function()
            with_captured_word(function(WORD)
                expect(type(WORD.dynamic_key)):toBe("function")
            end)
            expect(package.loaded["pasta.act"]):toBeNil()
            expect(package.loaded["pasta.actor"]):toBeNil()
        end)
        package.loaded["pasta.act"] = saved.act
        package.loaded["pasta.actor"] = saved.actor
        if not ok then error(err, 0) end
    end)
end)

-- ============================================================================
-- act の動的参照（task 2.2）: find_act_handler の skip_methods・word(値, パス)・expr_fn_var
-- ============================================================================

local GLOBAL = require("pasta.global")
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

--- 警告を記録するログと @pasta_search（nil で不在固定）を差し込み、pasta.word・pasta.act を
--- 新規ロードして body を実行し、実行後に元のモジュールへ戻す
--- @param search table|nil @pasta_search の差し替え
--- @param body fun(ACT: table, warns: string[])
local function with_act(search, body)
    local names = { "@pasta_log", "@pasta_search", "pasta.word", "pasta.act" }
    local saved = {}
    for _, n in ipairs(names) do saved[n] = package.loaded[n] end
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop,
        debug = noop,
        info = noop,
        warn = function(msg) table.insert(warns, msg) end,
        error = noop,
    }
    package.loaded["@pasta_search"] = search
    package.loaded["pasta.word"] = nil
    package.loaded["pasta.act"] = nil
    local ok, err = pcall(function()
        body(require("pasta.act"), warns)
    end)
    for _, n in ipairs(names) do package.loaded[n] = saved[n] end
    if not ok then error(err, 0) end
end

--- 単語キーごとに候補を順に返す @pasta_search の代役（巡回はキーとスコープの組で 1 本）
--- @param entries table<string, string[]> 単語キー → 候補
--- @return table search
--- @return table[] calls search_word の呼び出し記録 { key, scope }
local function rotating_search(entries)
    local cursor = {}
    local calls = {}
    local search = {
        search_word = function(_, key, scope)
            table.insert(calls, { key = key, scope = scope })
            local list = entries[key]
            if not list then return nil end
            local slot = tostring(scope) .. "/" .. key
            local i = (cursor[slot] or 0) % #list + 1
            cursor[slot] = i
            return list[i]
        end,
        search_scene = function() return nil end,
    }
    return search, calls
end

--- 実物のシーンテーブル（SCENE_TABLE_IMPL を継承）を STORE に残さずに作る
--- @param fields table 表自身のフィールド
--- @return table シーンテーブル
local function real_scene_table(fields)
    local name = "spec_dynamic_ref_scene"
    local mt = getmetatable(SCENE.create_global_table(name))
    STORE.scenes[name] = nil
    fields.__global_name__ = name
    return setmetatable(fields, mt)
end

--- GLOBAL の 1 キーを一時的に差し替えて（nil で削除して）body を実行し、元に戻す
--- @param key string
--- @param value any
--- @param body fun()
local function with_global(key, value, body)
    local saved = GLOBAL[key]
    GLOBAL[key] = value
    local ok, err = pcall(body)
    GLOBAL[key] = saved
    if not ok then error(err, 0) end
end

describe("find_act_handler - skip_methods（継承したメソッドに届かせない）", function()
    test("act のメソッド名（talk）は L3 を飛ばし、指定なしでは従来どおり L3 に一致する", function()
        with_act(nil, function(ACT)
            local act = ACT.new({})
            expect(act:find_act_handler("word", "talk", true)):toBeNil()
            expect(act:find_handler("word", "talk", true)):toBeNil()
            expect(act:find_act_handler("word", "talk")):toBe(ACT.IMPL.talk)
        end)
    end)

    test("yield は act のメソッドではなく L4（GLOBAL）へ進む", function()
        with_act(nil, function(ACT)
            local handler = ACT.new({}):find_handler("word", "yield", true)
            expect(handler).not_:toBe(ACT.IMPL.yield)
            expect(handler):toBe(GLOBAL.yield)
        end)
    end)

    test("シーンテーブルの組み込みメソッド（create_word）には一致せず、指定なしでは従来どおり一致する", function()
        with_act(nil, function(ACT)
            local act = ACT.new({})
            act.current_scene = real_scene_table({})
            expect(act:find_handler("expr", "create_word", true)):toBeNil()
            expect(type(act:find_handler("expr", "create_word"))):toBe("function")
        end)
    end)

    test("シーンテーブル自身のフィールドは指定ありでも一致する", function()
        with_act(nil, function(ACT)
            local act = ACT.new({})
            local fn = function() return "シーン関数" end
            act.current_scene = real_scene_table({ ["場面"] = fn })
            expect(act:find_handler("word", "場面", true)):toBe(fn)
        end)
    end)
end)

describe("ACT - word(値, 変数パス)", function()
    test("値をキーに検索し、シーン関数を act を引数に呼んだ戻り値を返す", function()
        with_act(nil, function(ACT, warns)
            local act = ACT.new({})
            local received = nil
            act.current_scene = real_scene_table({
                ["挨拶"] = function(a)
                    received = a
                    return "こんにちは"
                end,
            })
            expect(act:word("挨拶", "var.x")):toBe("こんにちは")
            expect(received):toBe(act)
            expect(#warns):toBe(0)
        end)
    end)

    test("GLOBAL の関数は呼ばれる", function()
        with_global("spec_dyn_天気", function() return "晴れ" end, function()
            with_act(nil, function(ACT)
                expect(ACT.new({}):word("spec_dyn_天気", "save.k")):toBe("晴れ")
            end)
        end)
    end)

    test("数値の値は文字列化したキーで検索する", function()
        local search, calls = rotating_search({ ["1"] = { "いち" } })
        with_act(search, function(ACT)
            expect(ACT.new({}):word(1, "var.n")):toBe("いち")
            expect(calls[1].key):toBe("1")
        end)
    end)

    test("値が talk でも act:talk を呼ばず辞書の段へ進む", function()
        local search = rotating_search({ talk = { "辞書のtalk" } })
        with_act(search, function(ACT)
            local act = ACT.new({})
            expect(act:word("talk", "var.x")):toBe("辞書のtalk")
            expect(#act.token):toBe(0)
        end)
    end)

    test("値が yield でも act:yield を呼ばず後段へ進む（GLOBAL に無ければ辞書の段）", function()
        local search = rotating_search({ yield = { "辞書のyield" } })
        with_global("yield", nil, function()
            with_act(search, function(ACT)
                local act = ACT.new({})
                local co = coroutine.create(function() return act:word("yield", "var.x") end)
                local ok, result = coroutine.resume(co)
                expect(ok):toBe(true)
                expect(coroutine.status(co)):toBe("dead")
                expect(result):toBe("辞書のyield")
            end)
        end)
    end)

    test("値が create_word・__index でもシーンが止まらず次の段へ進む", function()
        local search = rotating_search({
            create_word = { "辞書のcreate_word" },
            __index = { "辞書の__index" },
        })
        with_act(search, function(ACT)
            local act = ACT.new({})
            act.current_scene = real_scene_table({})
            expect(act:word("create_word", "var.x")):toBe("辞書のcreate_word")
            expect(act:word("__index", "var.x")):toBe("辞書の__index")
        end)
    end)

    test("動的参照と静的参照は同じ単語キーの巡回を共有する", function()
        local search = rotating_search({ ["挨拶"] = { "おはよう", "こんにちは", "こんばんは" } })
        with_act(search, function(ACT)
            local act = ACT.new({})
            act.current_scene = real_scene_table({})
            expect(act:word("挨拶")):toBe("おはよう")
            expect(act:word("挨拶", "var.x")):toBe("こんにちは")
            expect(act:word("挨拶")):toBe("こんばんは")
            expect(act:word("挨拶", "save.y")):toBe("おはよう")
        end)
    end)

    test("該当なしは静的と同じ警告を出して nil", function()
        with_act(nil, function(ACT, warns)
            expect(ACT.new({}):word("spec_dyn_無い", "var.x")):toBeNil()
            expect(warns[1]):toBe("act:word - handler not found: key='spec_dyn_無い', mode='word', via=act")
        end)
    end)

    test("未代入・空・型不正は検索せず act:word の警告を出して nil", function()
        local search, calls = rotating_search({})
        with_act(search, function(ACT, warns)
            local act = ACT.new({})
            expect(act:word(nil, "var.x")):toBeNil()
            expect(act:word("", "var.x")):toBeNil()
            expect(act:word(true, "var.x")):toBeNil()
            expect(warns[1]):toBe("act:word - undefined variable: 'var.x'")
            expect(warns[2]):toBe("act:word - empty variable: 'var.x'")
            expect(warns[3]):toBe("act:word - unsupported value type: 'var.x' (boolean)")
            expect(#calls):toBe(0)
        end)
    end)

    test("変数パスなしの word(nil)・word('') は従来どおり警告なしで nil", function()
        with_act(nil, function(ACT, warns)
            local act = ACT.new({})
            expect(act:word(nil)):toBeNil()
            expect(act:word("")):toBeNil()
            expect(#warns):toBe(0)
        end)
    end)
end)

describe("ACT - expr_fn_var(値, 変数パス, 引数…)", function()
    test("シーン関数を act と同じ引数で呼び、戻り値を返す", function()
        with_act(nil, function(ACT, warns)
            local act = ACT.new({})
            local received = nil
            act.current_scene = real_scene_table({
                ["加算"] = function(a, x, y)
                    received = a
                    return x + y
                end,
            })
            expect(act:expr_fn_var("加算", "var.f", 10, 20)):toBe(30)
            expect(received):toBe(act)
            expect(#warns):toBe(0)
        end)
    end)

    test("GLOBAL の関数は呼ばれる", function()
        with_global("spec_dyn_倍", function(_, x) return x * 2 end, function()
            with_act(nil, function(ACT)
                expect(ACT.new({}):expr_fn_var("spec_dyn_倍", "args[1]", 21)):toBe(42)
            end)
        end)
    end)

    test("値が talk・create_word でも act・シーンテーブルのメソッドを呼ばない", function()
        with_act(nil, function(ACT, warns)
            local act = ACT.new({})
            act.current_scene = real_scene_table({})
            expect(act:expr_fn_var("talk", "var.f", "台詞")):toBeNil()
            expect(act:expr_fn_var("create_word", "var.f")):toBeNil()
            expect(#act.token):toBe(0)
            expect(warns[1]):toBe("act:expr_fn - handler not found: key='talk', mode='expr', via=act")
            expect(warns[2]):toBe("act:expr_fn - handler not found: key='create_word', mode='expr', via=act")
        end)
    end)

    test("該当なしは静的 expr_fn と同じ警告を出して nil", function()
        with_act(nil, function(ACT, warns)
            local act = ACT.new({})
            expect(act:expr_fn_var("spec_dyn_無い", "var.f")):toBeNil()
            expect(act:expr_fn("spec_dyn_無い")):toBeNil()
            expect(warns[1]):toBe("act:expr_fn - handler not found: key='spec_dyn_無い', mode='expr', via=act")
            expect(warns[2]):toBe(warns[1])
        end)
    end)

    test("未代入は関数を探さず act:expr_fn の警告を出して nil", function()
        with_act(nil, function(ACT, warns)
            expect(ACT.new({}):expr_fn_var(nil, "var.f", 1)):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("act:expr_fn - undefined variable: 'var.f'")
        end)
    end)
end)

-- ============================================================================
-- アクター付きの動的参照（task 2.3）: PROXY_IMPL の skip_methods・word(値, パス)・expr_fn_var
-- ============================================================================

--- with_act に加えて pasta.store（@pasta_config は空の代役）・pasta.actor も新規ロードし、
--- アクター 1 体のプロキシを作って body を実行する（既存のアクターのメタテーブルを汚さない）
--- @param search table|nil @pasta_search の差し替え
--- @param fields table アクターの表自身のフィールド
--- @param body fun(proxy: table, act: table, ACT: table, warns: string[])
local function with_proxy(search, fields, body)
    local names = { "@pasta_config", "pasta.store", "pasta.actor" }
    local saved = {}
    for _, n in ipairs(names) do saved[n] = package.loaded[n] end
    package.loaded["@pasta_config"] = {}
    package.loaded["pasta.store"] = nil
    package.loaded["pasta.actor"] = nil
    local ok, err = pcall(function()
        with_act(search, function(ACT, warns)
            local ACTOR = require("pasta.actor")
            local actor = ACTOR.get_or_create("spec_dyn_俳優")
            for k, v in pairs(fields) do actor[k] = v end
            local act = ACT.new({})
            body(ACTOR.create_proxy(actor, act), act, ACT, warns)
        end)
    end)
    for _, n in ipairs(names) do package.loaded[n] = saved[n] end
    if not ok then error(err, 0) end
end

describe("PROXY - find_handler の skip_methods", function()
    test("アクターの組み込みメソッド（create_word）には一致せず、指定なしでは従来どおり一致する", function()
        with_proxy(nil, {}, function(proxy)
            expect(proxy:find_actor_handler("word", "create_word", true)):toBeNil()
            expect(proxy:find_handler("word", "create_word", true)):toBeNil()
            expect(type(proxy:find_handler("word", "create_word"))):toBe("function")
        end)
    end)

    test("act の検索へも指定を渡す（act のメソッド talk に一致しない）", function()
        with_proxy(nil, {}, function(proxy, _, ACT)
            expect(proxy:find_handler("word", "talk", true)):toBeNil()
            expect(proxy:find_handler("word", "talk")):toBe(ACT.IMPL.talk)
        end)
    end)
end)

describe("PROXY - word(値, 変数パス)", function()
    test("アクターの表のフィールドをシーンテーブルより先に探す", function()
        with_proxy(nil, { ["一人称"] = "わたし" }, function(proxy, act, _, warns)
            act.current_scene = real_scene_table({ ["一人称"] = function() return "シーン側" end })
            expect(proxy:word("一人称", "var.x")):toBe("わたし")
            expect(#warns):toBe(0)
        end)
    end)

    test("アクター辞書をシーンテーブルより先に、アクターのスコープで探す", function()
        local search, calls = rotating_search({ ["呼び名"] = { "はなちゃん" } })
        with_proxy(search, {}, function(proxy, act)
            act.current_scene = real_scene_table({ ["呼び名"] = function() return "シーン側" end })
            expect(proxy:word("呼び名", "var.x")):toBe("はなちゃん")
            expect(calls[1].scope):toBe("__actor_spec_dyn_俳優__")
        end)
    end)

    test("作者定義のアクターのフィールドの関数はプロキシを引数に呼ばれる", function()
        local received = nil
        local fn = function(p)
            received = p
            return "計算値"
        end
        with_proxy(nil, { ["気分"] = fn }, function(proxy)
            expect(proxy:word("気分", "save.k")):toBe("計算値")
            expect(received):toBe(proxy)
        end)
    end)

    test("値が create_word・__index でもアクター・シーンテーブルの組み込みメソッドに一致せず次の段へ進む", function()
        local search = rotating_search({
            create_word = { "辞書のcreate_word" },
            __index = { "辞書の__index" },
        })
        with_proxy(search, {}, function(proxy, act)
            act.current_scene = real_scene_table({})
            expect(proxy:word("create_word", "var.x")):toBe("辞書のcreate_word")
            expect(proxy:word("__index", "var.x")):toBe("辞書の__index")
        end)
    end)

    test("値が talk でも act:talk を呼ばない", function()
        local search = rotating_search({ talk = { "辞書のtalk" } })
        with_proxy(search, {}, function(proxy, act)
            expect(proxy:word("talk", "var.x")):toBe("辞書のtalk")
            expect(#act.token):toBe(0)
        end)
    end)

    test("該当なしは静的と同じ警告を出して nil", function()
        with_proxy(nil, {}, function(proxy, _, _, warns)
            expect(proxy:word("spec_dyn_無い", "var.x")):toBeNil()
            expect(warns[1]):toBe(
                "proxy:word - handler not found: key='spec_dyn_無い', mode='word', via=proxy(spec_dyn_俳優)")
        end)
    end)

    test("未代入・空は検索せず proxy:word の警告を出して nil、変数パスなしは従来どおり警告なし", function()
        local search, calls = rotating_search({})
        with_proxy(search, {}, function(proxy, _, _, warns)
            expect(proxy:word(nil, "var.x")):toBeNil()
            expect(proxy:word("", "var.x")):toBeNil()
            expect(warns[1]):toBe("proxy:word - undefined variable: 'var.x'")
            expect(warns[2]):toBe("proxy:word - empty variable: 'var.x'")
            expect(#calls):toBe(0)
            expect(proxy:word(nil)):toBeNil()
            expect(proxy:word("")):toBeNil()
            expect(#warns):toBe(2)
        end)
    end)
end)

describe("PROXY - expr_fn_var(値, 変数パス, 引数…)", function()
    test("関数はプロキシを第 1 引数に、同じ引数で呼ばれる", function()
        with_proxy(nil, {}, function(proxy, act, _, warns)
            local received = nil
            act.current_scene = real_scene_table({
                ["加算"] = function(p, x, y)
                    received = p
                    return x + y
                end,
            })
            expect(proxy:expr_fn_var("加算", "var.f", 1, 2)):toBe(3)
            expect(received):toBe(proxy)
            expect(#warns):toBe(0)
        end)
    end)

    test("値が talk・create_word でもメソッドを呼ばず、静的 expr_fn と同じ警告を出して nil", function()
        with_proxy(nil, {}, function(proxy, act, _, warns)
            act.current_scene = real_scene_table({})
            expect(proxy:expr_fn_var("talk", "var.f", "台詞")):toBeNil()
            expect(proxy:expr_fn_var("create_word", "var.f")):toBeNil()
            expect(#act.token):toBe(0)
            expect(warns[1]):toBe("proxy:expr_fn - handler not found: key='talk', mode='expr', via=proxy(spec_dyn_俳優)")
            expect(warns[2]):toBe(
                "proxy:expr_fn - handler not found: key='create_word', mode='expr', via=proxy(spec_dyn_俳優)")
        end)
    end)

    test("未代入は関数を探さず proxy:expr_fn の警告を出して nil", function()
        with_proxy(nil, {}, function(proxy, _, _, warns)
            expect(proxy:expr_fn_var(nil, "args[1]")):toBeNil()
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("proxy:expr_fn - undefined variable: 'args[1]'")
        end)
    end)
end)
