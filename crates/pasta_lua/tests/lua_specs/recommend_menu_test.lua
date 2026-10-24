-- recommend-menu-scenes: おすすめメニューの既定のハンドラと組み込みのテスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local MODULE = "pasta.shiori.event.recommend"

--- @pasta_log を警告を集める代役に差し替えて fn を走らせ、必ず元に戻す
---@return string[] warns
local function with_warns(fn)
    local warns = {}
    local noop = function() end
    local saved = package.loaded["@pasta_log"]
    package.loaded["@pasta_log"] = {
        trace = noop, debug = noop, info = noop, error = noop,
        warn = function(msg) warns[#warns + 1] = msg end,
    }
    local ok, err = pcall(fn)
    package.loaded["@pasta_log"] = saved
    if not ok then error(err, 0) end
    return warns
end

--- 組み込みを読み込んで GLOBAL を返す
local function load_global()
    package.loaded[MODULE] = nil
    require(MODULE)
    return require("pasta.global")
end

--- 入れ物を手で置いた act（メニューの受け手の代わり）
local function menu_act(id)
    return { token = {}, _recommend_items = { id = id or "sakura.recommendsites", calls = 0 } }
end

-- ============================================================================
-- 3.1: ＞おすすめ項目・＞おすすめ区切り
-- ============================================================================

describe("recommend_menu - 組み込みの名前", function()
    test("英語の名前と日本語の名前が同じ関数を指す", function()
        local GLOBAL = load_global()
        expect(type(GLOBAL.recommend_item)):toBe("function")
        expect(type(GLOBAL.recommend_separator)):toBe("function")
        expect(GLOBAL["おすすめ項目"]):toBe(GLOBAL.recommend_item)
        expect(GLOBAL["おすすめ区切り"]):toBe(GLOBAL.recommend_separator)
    end)
end)

describe("recommend_menu - 項目を積む", function()
    test("項目と区切り線を呼んだ順に加える", function()
        local GLOBAL = load_global()
        local act = menu_act()
        local warns = with_warns(function()
            GLOBAL.recommend_item(act, "公式", "https://example.com/", "公式の話", "banner.png")
            GLOBAL.recommend_separator(act)
            GLOBAL["おすすめ項目"](act, "二つ目", "https://example.org/")
        end)
        local holder = act._recommend_items
        expect(#warns):toBe(0)
        expect(#holder):toBe(3)
        expect(holder.calls):toBe(3)
        expect(holder[1].name):toBe("公式")
        expect(holder[1].url):toBe("https://example.com/")
        expect(holder[1].scene):toBe("公式の話")
        expect(holder[1].banner):toBe("banner.png")
        expect(holder[2].name):toBe("-")
        expect(holder[2].url):toBe("")
        expect(holder[2].scene):toBe(nil)
        expect(holder[3].name):toBe("二つ目")
        expect(#act.token):toBe(0)
    end)

    test("3・4 つ目は省略・空文字列で無しになる", function()
        local GLOBAL = load_global()
        local act = menu_act()
        with_warns(function()
            GLOBAL.recommend_item(act, "a", "u")
            GLOBAL.recommend_item(act, "b", "u", "", "")
            GLOBAL.recommend_item(act, "c", "u", nil, "bn.png")
        end)
        local holder = act._recommend_items
        expect(#holder):toBe(3)
        expect(holder[1].scene):toBe(nil)
        expect(holder[1].banner):toBe(nil)
        expect(holder[2].scene):toBe(nil)
        expect(holder[2].banner):toBe(nil)
        expect(holder[3].scene):toBe(nil)
        expect(holder[3].banner):toBe("bn.png")
    end)

    test("数値は文字列にして加える", function()
        local GLOBAL = load_global()
        local act = menu_act()
        local warns = with_warns(function()
            GLOBAL.recommend_item(act, 1, 2, 3, 4)
        end)
        local item = act._recommend_items[1]
        expect(#warns):toBe(0)
        expect(item.name):toBe("1")
        expect(item.url):toBe("2")
        expect(item.scene):toBe("3")
        expect(item.banner):toBe("4")
    end)
end)

describe("recommend_menu - メニューの外", function()
    test("入れ物が無ければ何もしない（トークも警告も出さない）", function()
        local GLOBAL = load_global()
        local act = { token = { { type = "talk", text = "前から" } } }
        local warns = with_warns(function()
            GLOBAL.recommend_item(act, "a", "u")
            GLOBAL.recommend_item(act, nil, nil)
            GLOBAL.recommend_separator(act)
        end)
        expect(#warns):toBe(0)
        expect(#act.token):toBe(1)
        expect(act.token[1].text):toBe("前から")
        expect(rawget(act, "_recommend_items")):toBe(nil)
    end)
end)

describe("recommend_menu - 誤った項目", function()
    --- 1 項目を呼び、積んだ数と警告を返す
    local function one(...)
        local GLOBAL = load_global()
        local act = menu_act("kero.recommendsites")
        local args = table.pack(...)
        local warns = with_warns(function()
            GLOBAL.recommend_separator(act)
            GLOBAL.recommend_item(act, table.unpack(args, 1, args.n))
        end)
        return #act._recommend_items - 1, warns, act._recommend_items
    end

    test("名前が nil なら警告して加えない", function()
        local n, warns = one(nil, "u")
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad name (nil); item skipped")
    end)

    test("URL が空文字列なら警告して加えない", function()
        local n, warns = one("a", "")
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad url (empty string); item skipped")
    end)

    test("名前が真偽値なら警告して加えない", function()
        local n, warns = one(true, "u")
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad name (boolean); item skipped")
    end)

    test("シーンが表なら警告して加えない", function()
        local n, warns = one("a", "u", {})
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad scene (table); item skipped")
    end)

    test("バイト値 1 を含めば警告して加えない", function()
        local n, warns = one("a", "u", nil, "b\1n")
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad banner (contains byte 1); item skipped")
    end)

    test("バイト値 2 を含めば警告して加えない", function()
        local n, warns = one("a\2", "u")
        expect(n):toBe(0)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: bad name (contains byte 2); item skipped")
    end)

    test("5 つ目から後は警告して捨て、項目は加える", function()
        local n, warns, holder = one("a", "u", "s", "b", "x", "y")
        expect(n):toBe(1)
        expect(#warns):toBe(1)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: 2 extra argument(s) ignored")
        expect(holder[2].banner):toBe("b")
    end)

    test("5 つ目が nil でも数える", function()
        local n, warns = one("a", "u", nil, nil, nil)
        expect(n):toBe(1)
        expect(warns[1]):toBe("recommend_item: kero.recommendsites item #2: 1 extra argument(s) ignored")
    end)
end)

-- ============================================================================
-- 3.2: メニューの問い合わせ（MenuReceiver・その場で動かす最小の道）
-- STORE.scenes はスイート間で共有されるため、一意プレフィックス "rms_" を使う。
-- ============================================================================

local SCENE = require("pasta.scene")

--- package.loaded の複数のキーを差し替えて fn を走らせ、必ず元に戻す（false は引けない状態）
local function with_loaded(stubs, fn)
    local saved = {}
    for name, stub in pairs(stubs) do
        saved[name] = package.loaded[name]
        package.loaded[name] = stub
    end
    local ok, err = pcall(fn)
    for name in pairs(stubs) do
        package.loaded[name] = saved[name]
    end
    if not ok then error(err, 0) end
end

--- 書いた名前 → 大域の登録名の並びを引く @pasta_search の代役（同じ名前は呼ぶたびに順に選ぶ。
--- 本物の選び方の差し替え set_scene_selector の代わり）
---@param scenes table<string, string[]>
---@param aliases table|nil scene_aliases の戻り値（nil なら scene_aliases を持たない）
local function make_search(scenes, aliases)
    local picks = {}
    local function pick(_, name)
        local list = scenes[name]
        if not list then return nil end
        picks[name] = (picks[name] or 0) % #list + 1
        return list[picks[name]], "__start__"
    end
    local search = { search_scene_exact = pick, search_scene = pick }
    if aliases then
        search.scene_aliases = function() return aliases end
    end
    return search
end

local MENU_IDS = { "sakura.recommendsites", "kero.recommendsites", "sakura.portalsites" }

--- 作者のハンドラが無い状態で受け手を置き直し、REG を返す
local function load_receivers(extra_ids)
    local REG = require("pasta.shiori.event.register")
    for _, id in ipairs(MENU_IDS) do REG[id] = nil end
    for _, id in ipairs(extra_ids or {}) do REG[id] = nil end
    package.loaded[MODULE] = nil
    require(MODULE)
    return REG
end

--- 本物の ShioriAct（GET のリクエスト）
local function shiori_act(id)
    return require("pasta.shiori.act").new({}, { id = id, method = "get", reference = {} })
end

--- 差し替えの検索で受け手を置き、id の受け手を 1 回呼ぶ
local function ask(id, scenes, opts)
    opts = opts or {}
    local result
    with_loaded({ ["@pasta_search"] = make_search(scenes, opts.aliases) }, function()
        local REG = load_receivers(opts.extra_ids)
        result = REG[id](shiori_act(id))
    end)
    return result
end

describe("recommend_menu - メニューの値", function()
    test("項目と区切り線を SSP の値の形にする（URL・バナーは書いたまま、トークの欄は空）", function()
        SCENE.register("rms_v_1", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "カクヨム", "https://kakuyomu.jp/")
            act:call(nil, "おすすめ区切り", {})
            act:call(nil, "おすすめ項目", {}, "話す", "script:\\0\\s[0]やあ\\e", "話す話", "img/b.png")
        end)
        local result = ask("sakura.recommendsites", { ["sakura.recommendsites"] = { "rms_v_1" } })
        expect(result):toBe("カクヨム\1https://kakuyomu.jp/\1\1\2"
            .. "-\1\1\1\2"
            .. "話す\1script:\\0\\s[0]やあ\\e\1img/b.png\1\2")
    end)

    test("台詞は値に出さない", function()
        SCENE.register("rms_t_1", "__start__", function(act)
            act:raw_script("台詞")
            act:call(nil, "おすすめ項目", {}, "a", "u")
            act:raw_script("後の台詞")
        end)
        local result = ask("kero.recommendsites", { ["kero.recommendsites"] = { "rms_t_1" } })
        expect(result):toBe("a\1u\1\1\2")
    end)

    test("呼んだシーンの項目も加わる", function()
        SCENE.register("rms_c_1", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "親", "u1")
            act:call(nil, "rms_子", {})
        end)
        SCENE.register("rms_c_2", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "子", "u2")
        end)
        local result = ask("sakura.portalsites", {
            ["sakura.portalsites"] = { "rms_c_1" },
            ["rms_子"] = { "rms_c_2" },
        })
        expect(result):toBe("親\1u1\1\1\2子\1u2\1\1\2")
    end)

    test("同じ名前のシーンが複数あれば、問い合わせのたびに引き直す", function()
        SCENE.register("rms_s_1", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "一", "u")
        end)
        SCENE.register("rms_s_2", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "二", "u")
        end)
        local got = {}
        with_loaded({
            ["@pasta_search"] = make_search({ ["sakura.recommendsites"] = { "rms_s_1", "rms_s_2" } }),
        }, function()
            local REG = load_receivers()
            for i = 1, 3 do
                got[i] = REG["sakura.recommendsites"](shiori_act("sakura.recommendsites"))
            end
        end)
        expect(got[1]):toBe("一\1u\1\1\2")
        expect(got[2]):toBe("二\1u\1\1\2")
        expect(got[3]):toBe("一\1u\1\1\2")
    end)
end)

describe("recommend_menu - 204", function()
    test("シーンが無ければ nil", function()
        expect(ask("sakura.recommendsites", {})):toBeNil()
    end)

    test("項目が 0 なら nil（台詞だけのシーン）", function()
        SCENE.register("rms_z_1", "__start__", function(act)
            act:raw_script("台詞だけ")
        end)
        expect(ask("sakura.recommendsites", { ["sakura.recommendsites"] = { "rms_z_1" } })):toBeNil()
    end)

    test("シーン関数が登録されていなければ nil", function()
        expect(ask("sakura.recommendsites", { ["sakura.recommendsites"] = { "rms_none_1" } })):toBeNil()
    end)
end)

describe("recommend_menu - 受け手を置く ID", function()
    test("決まった 3 つに置く（scene_aliases が無い検索でも）", function()
        with_loaded({ ["@pasta_search"] = make_search({}) }, function()
            local REG = load_receivers()
            for _, id in ipairs(MENU_IDS) do
                expect(type(rawget(REG, id))):toBe("function")
            end
        end)
    end)

    test("@pasta_search が引けなくても決まった 3 つに置く", function()
        with_loaded({ ["@pasta_search"] = false }, function()
            local REG = load_receivers()
            for _, id in ipairs(MENU_IDS) do
                expect(type(rawget(REG, id))):toBe("function")
            end
        end)
    end)

    test("別名の表の置き換え先の charN.recommendsites にも置く（ほかの置き換え先には置かない）", function()
        SCENE.register("rms_k_1", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "三人目", "u")
        end)
        local others = { "OnTalk", "char2.recommendsites.caption" }
        local result = ask("char2.recommendsites", { ["char2.recommendsites"] = { "rms_k_1" } }, {
            aliases = {
                ["char2.recommendsites"] = { "三人目のおすすめ" },
                ["OnTalk"] = { "会話" },
                ["char2.recommendsites.caption"] = { "三人目のボタン" },
            },
            extra_ids = { "char2.recommendsites", others[1], others[2] },
        })
        expect(result):toBe("三人目\1u\1\1\2")
        local REG = require("pasta.shiori.event.register")
        for _, id in ipairs(others) do
            expect(rawget(REG, id)):toBeNil()
        end
        REG["char2.recommendsites"] = nil
    end)

    test("作者が先に代入した REG[id] が勝つ", function()
        local REG = load_receivers()
        local mine = function() return "mine" end
        REG["sakura.recommendsites"] = mine
        package.loaded[MODULE] = nil
        require(MODULE)
        expect(rawget(REG, "sakura.recommendsites")):toBe(mine)
        load_receivers()
    end)
end)

describe("recommend_menu - EVENT.fire を通す", function()
    test("REG:register の鎖が先に呼ばれ、nil なら組み込みの受け手が答える", function()
        SCENE.register("rms_f_1", "__start__", function(act)
            act:call(nil, "おすすめ項目", {}, "組み込み", "u")
        end)
        -- イベントのモジュールを読み直し、終わったら元のものに戻す（ほかのスイートの REG を残す）
        with_loaded({
            ["@pasta_search"] = make_search({ ["sakura.recommendsites"] = { "rms_f_1" } }),
            ["pasta.shiori.event"] = false,
            ["pasta.shiori.event.register"] = false,
            [MODULE] = false,
        }, function()
            local EVENT = require("pasta.shiori.event")
            local REG = require("pasta.shiori.event.register")
            local req = { id = "sakura.recommendsites", method = "get", reference = {} }
            local seen = 0
            local off = REG:register("sakura.recommendsites", function() seen = seen + 1 end)
            local res = EVENT.fire(req)
            expect(seen):toBe(1)
            expect(res:find("200 OK", 1, true) ~= nil):toBe(true)
            expect(res:find("Value: 組み込み\1u\1\1\2", 1, true) ~= nil):toBe(true)
            off()
            REG:register("sakura.recommendsites", function() return "鎖" end)
            res = EVENT.fire(req)
            expect(res:find("Value: 鎖", 1, true) ~= nil):toBe(true)
        end)
    end)
end)

-- ============================================================================
-- 3.3: ほかの状態に触れずにその場で 1 回だけ動かす（InPlaceRun）
-- 依存閉包を同一インスタンス集合として読み直す（chain_talk_interrupt_test と同じ）。
-- ============================================================================

local RELOAD = {
    "pasta.shiori.entry", "pasta.shiori.event", "pasta.shiori.event.register",
    "pasta.shiori.event.callback", "pasta.shiori.event.boot", "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change", "pasta.shiori.event.mouse", "pasta.shiori.event.scene_play",
    "pasta.shiori.event.kick", "pasta.shiori.event.external_control",
    "pasta.shiori.event.virtual_dispatcher", MODULE, "pasta.shiori.res", "pasta.shiori.stop",
    "pasta.shiori.act", "pasta.shiori.sakura_builder", "pasta.act", "pasta.actor", "pasta.scene",
    "pasta.word", "pasta.global", "pasta.store", "pasta.config", "pasta.save",
}

--- 状態の写し（関数・スレッド・表の同一性も字にして比べる）
--- 写しのために作る外側の表（root）は同一性を見ない
local function snapshot(value, seen)
    if type(value) ~= "table" then return tostring(value) end
    local root = seen == nil
    seen = seen or {}
    if seen[value] then return "<cycle>" end
    seen[value] = true
    local keys = {}
    for k in pairs(value) do keys[#keys + 1] = k end
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    local parts = { root and "" or tostring(value) }
    for _, k in ipairs(keys) do
        parts[#parts + 1] = tostring(k) .. "=" .. snapshot(value[k], seen)
    end
    return "{" .. table.concat(parts, ",") .. "}"
end

--- メニューのシーンが触れてはならない状態の写し
local function guarded(STORE, CALLBACK)
    return snapshot({
        co_scene = STORE.co_scene, co_preempt = STORE.co_preempt,
        actor_spots = STORE.actor_spots, appearance = STORE.appearance,
        last_global_scene = STORE.last_global_scene, auto_talk_paused = STORE.auto_talk_paused,
        pending = CALLBACK.pending, staged = CALLBACK.has_staged(),
    })
end

--- 読み直した集合で状態を作り、id のメニューのシーン fn を置いて body(ctx) を走らせる
--- ctx: STORE・CALLBACK・ENTRY・REG・ask()（受け手を 1 回呼ぶ）・warns・before（呼ぶ前の写し）
local function with_menu(fn, body)
    local id = "sakura.recommendsites"
    for _, name in ipairs(RELOAD) do package.loaded[name] = nil end
    local names = { [id] = { "rms_ip_1" }, ["チェイントーク中断"] = { "rms_ip_chudan" } }
    local chudan_ran = false
    local ctx = { warns = {} }
    local noop = function() end
    with_loaded({
        ["@pasta_search"] = make_search(names),
        ["@pasta_log"] = {
            trace = noop, debug = noop, info = noop, error = noop,
            warn = function(msg) ctx.warns[#ctx.warns + 1] = msg end,
        },
    }, function()
        ctx.ENTRY = require("pasta.shiori.entry")
        ctx.STORE = require("pasta.store")
        ctx.CALLBACK = require("pasta.shiori.event.callback")
        ctx.REG = require("pasta.shiori.event.register")
        local STORE = ctx.STORE
        STORE.reset()
        ctx.CALLBACK.reset()
        -- STORE.reset が登録したシーンも消すので、その後に登録する
        local scene = require("pasta.scene")
        scene.register("rms_ip_1", "__start__", fn)
        scene.register("rms_ip_chudan", "__start__", function() chudan_ran = true end)
        STORE.actors = { sakura = { name = "さくら", spot = 0 } }
        -- 待っているチェイントークの続き・持ち越し・最後のグローバルシーン・ほかの待ちを置く
        STORE.co_scene = coroutine.create(function() coroutine.yield() end)
        coroutine.resume(STORE.co_scene)
        STORE.co_preempt[STORE.co_scene] = true
        STORE.actor_spots = { ["さくら"] = 1 }
        STORE.last_global_scene = "rms_before"
        ctx.CALLBACK.pending["999"] = { co = STORE.co_scene, timeout_at = 1 }
        ctx.before = guarded(STORE, ctx.CALLBACK)
        ctx.ask = function()
            return ctx.REG[id](require("pasta.shiori.act").new(STORE.actors,
                { id = id, method = "get", reference = {} }))
        end
        local ok, err = pcall(body, ctx)
        expect(chudan_ran):toBe(false)
        STORE.reset()
        ctx.CALLBACK.reset()
        if not ok then error(err, 0) end
    end)
end

describe("recommend_menu - ほかの状態に触れない", function()
    test("区切り（＞チェイントーク）で打ち切り、続き・持ち越し・待ちは変わらない", function()
        with_menu(function(act)
            local STORE = require("pasta.store")
            act:call(nil, "おすすめ項目", {}, "前", "u")
            act:set_spot("sakura", 2)
            act:talk(STORE.actors.sakura, "TALK")
            act:surface(5)
            STORE.last_global_scene = "rms_menu"
            act:call(nil, "チェイントーク", {})
            act:call(nil, "おすすめ項目", {}, "後", "u")
        end, function(ctx)
            expect(ctx.ask()):toBe("前\1u\1\1\2")
            expect(guarded(ctx.STORE, ctx.CALLBACK)):toBe(ctx.before)
        end)
    end)

    test("自動トークを止めているときの区切りは再開しない", function()
        with_menu(function(act)
            act:call(nil, "おすすめ項目", {}, "前", "u")
            act:yield()
        end, function(ctx)
            ctx.STORE.auto_talk_paused = true
            ctx.before = guarded(ctx.STORE, ctx.CALLBACK)
            local GLOBAL = require("pasta.global")
            local resumed = 0
            local real = GLOBAL.resume_auto_talk
            GLOBAL.resume_auto_talk = function() resumed = resumed + 1 end
            local result = ctx.ask()
            GLOBAL.resume_auto_talk = real
            expect(result):toBe("前\1u\1\1\2")
            expect(resumed):toBe(0)
            expect(ctx.STORE.auto_talk_paused):toBe(true)
            expect(guarded(ctx.STORE, ctx.CALLBACK)):toBe(ctx.before)
            expect(#ctx.warns):toBe(0)
        end)
    end)

    test("get_property で打ち切り、warn を 1 行出して予約を残さない", function()
        with_menu(function(act)
            act:call(nil, "おすすめ項目", {}, "前", "u")
            act:get_property("currentghost.name")
            act:call(nil, "おすすめ項目", {}, "後", "u")
        end, function(ctx)
            expect(ctx.ask()):toBe("前\1u\1\1\2")
            expect(ctx.CALLBACK.has_staged()):toBe(false)
            expect(guarded(ctx.STORE, ctx.CALLBACK)):toBe(ctx.before)
            expect(#ctx.warns):toBe(1)
            expect(ctx.warns[1]):toBe("recommend menu: sakura.recommendsites: "
                .. "get_property is not available in a menu scene; the scene was cut there")
        end)
    end)

    test("変数の代入は打ち切りの後も残る", function()
        with_menu(function(act)
            act.save.rms_kept = "kept"
            act:call(nil, "おすすめ項目", {}, "前", "u")
            act:yield()
            act.save.rms_kept = "after"
        end, function(ctx)
            ctx.ask()
            local SAVE = require("pasta.save")
            expect(SAVE.rms_kept):toBe("kept")
            SAVE.rms_kept = nil
        end)
    end)

    test("実行時エラーは表記なしの 500 と X-Error-Reason になり、状態は変わらない", function()
        with_menu(function(act)
            local STORE = require("pasta.store")
            act:call(nil, "おすすめ項目", {}, "前", "u")
            STORE.last_global_scene = "rms_menu"
            act:talk(STORE.actors.sakura, "TALK")
            error("rms boom")
        end, function(ctx)
            local res = ctx.ENTRY.request({
                id = "sakura.recommendsites", method = "get", version = 30, reference = {},
                security_level = "local",
            })
            expect(res:find("500 Internal Server Error", 1, true) ~= nil):toBe(true)
            expect(res:find("X%-Error%-Reason: [^\r\n]*rms boom") ~= nil):toBe(true)
            expect(res:find("Value:", 1, true)):toBeNil()
            expect(guarded(ctx.STORE, ctx.CALLBACK)):toBe(ctx.before)
        end)
    end)
end)
