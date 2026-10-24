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
