-- pasta.shiori.stop（止まるエラーの表記）の単体テスト (runtime-error-balloon 3.2)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local STOP = require("pasta.shiori.stop")

--- LuaJIT がエラー文のチャンク名を縮める形（60 バイト以上なら "..." ＋ 末尾 56 バイト）
local function short(path)
    if #path >= 60 then return "..." .. path:sub(-56) end
    return path
end

local GHOST = "C:/Users/maz/AppData/Local/SSP/ghost/hello-pasta/ghost/master"
local CACHE = short(GHOST .. "/profile/pasta/cache/lua/pasta\\scene\\天気.lua")
local RUNTIME = short(GHOST .. "/profile/pasta/pasta_scripts/pasta\\shiori\\event/init.lua")
local AUTHOR_SHORT = short(GHOST .. "/scripts/天気.lua")
local AUTHOR_MIXED = short("C:\\ghost\\master/scripts\\sub/天気.lua")
local LONG_REL = "scripts\\とても長い名前のフォルダーの中の\\さらに長い名前の天気のスクリプト.lua"
local AUTHOR_LONG = short(GHOST .. "/" .. LONG_REL:gsub("\\", "/"))
local DUP = short(GHOST .. "/scripts/dup.lua")

local CHUNKS = {
    [CACHE] = true,
    [AUTHOR_SHORT] = "scripts\\天気.lua",
    ["scripts/plain.lua"] = "scripts\\plain.lua",
    [AUTHOR_MIXED] = "scripts\\sub\\天気.lua",
    [AUTHOR_LONG] = LONG_REL,
    [DUP] = false,
}

local LINE_MAP = "dic\\天気.pasta|30:15,31:15,40:22"

describe("STOP.location（場所の見分け）", function()
    test("縮めていない作者の .lua はゴーストから見た場所:行", function()
        expect(STOP.location("scripts/plain.lua", 3, CHUNKS)):toBe("scripts\\plain.lua:3")
    end)

    test("縮めた作者の .lua（日本語のファイル名）", function()
        expect(AUTHOR_SHORT:sub(1, 3)):toBe("...")
        expect(STOP.location(AUTHOR_SHORT, 12, CHUNKS)):toBe("scripts\\天気.lua:12")
    end)

    test("\\ と / の混在したパス", function()
        expect(STOP.location(AUTHOR_MIXED, 7, CHUNKS)):toBe("scripts\\sub\\天気.lua:7")
    end)

    test("56 バイトを超える相対パスも表の値で引ける", function()
        expect(#LONG_REL > 56):toBe(true)
        expect(STOP.location(AUTHOR_LONG, 4, CHUNKS)):toBe(LONG_REL .. ":4")
    end)

    test("変換した Lua のキャッシュの行は .pasta の行に直る", function()
        expect(STOP.location(CACHE, 30, CHUNKS, LINE_MAP)):toBe("dic\\天気.pasta:15")
        expect(STOP.location(CACHE, 40, CHUNKS, LINE_MAP)):toBe("dic\\天気.pasta:22")
    end)

    test(".pasta のファイルの / は \\ にそろえる・ファイル名の | は最後の | で切る", function()
        expect(STOP.location(CACHE, 5, CHUNKS, "dic/a|b.pasta|5:9")):toBe("dic\\a|b.pasta:9")
    end)

    test("対応に無い行・対応が無いときは nil", function()
        expect(STOP.location(CACHE, 31 + 100, CHUNKS, LINE_MAP)):toBe(nil)
        expect(STOP.location(CACHE, 30, CHUNKS, nil)):toBe(nil)
    end)

    test("標準ランタイム・[string] ・表に無いキー・重複したキーは nil", function()
        expect(STOP.location(RUNTIME, 229, CHUNKS, LINE_MAP)):toBe(nil)
        expect(STOP.location('[string "entry.lua"]', 10, CHUNKS, LINE_MAP)):toBe(nil)
        expect(STOP.location("scripts/unknown.lua", 1, CHUNKS)):toBe(nil)
        expect(STOP.location(DUP, 1, CHUNKS)):toBe(nil)
    end)

    test("表が無いときは nil", function()
        expect(STOP.location("scripts/plain.lua", 3, nil)):toBe(nil)
    end)
end)

describe("STOP.detail（エラー文の整え）", function()
    test("投げ直しの接頭辞が付いた文から、キャッシュの場所を .pasta に直した文が残る", function()
        local err = RUNTIME .. ":229: " .. CACHE .. ":30: attempt to index local 'data' (a nil value)"
        expect(STOP.detail(err, CHUNKS, LINE_MAP))
            :toBe("dic\\天気.pasta:15: attempt to index local 'data' (a nil value)")
    end)

    test("作者の接頭辞は scripts\\天気.lua:12: に直る", function()
        local err = AUTHOR_SHORT .. ":12: attempt to call a nil value"
        expect(STOP.detail(err, CHUNKS)):toBe("scripts\\天気.lua:12: attempt to call a nil value")
    end)

    test("対応に無いキャッシュの行・[string]・重複したキーの接頭辞は落とす", function()
        expect(STOP.detail(CACHE .. ":99: boom", CHUNKS, LINE_MAP)):toBe("boom")
        expect(STOP.detail('[string "entry.lua"]:10: boom', CHUNKS)):toBe("boom")
        expect(STOP.detail(DUP .. ":1: boom", CHUNKS)):toBe("boom")
    end)

    test("文中の 10:30: は取らない", function()
        expect(STOP.detail("10:30: に起きる", CHUNKS)):toBe("10:30: に起きる")
        expect(STOP.detail(AUTHOR_SHORT .. ":12: 10:30: 時刻", CHUNKS))
            :toBe("scripts\\天気.lua:12: 10:30: 時刻")
    end)

    test("最初の行だけを示す", function()
        expect(STOP.detail("boom\nstack traceback:\n\t[C]: in ?", CHUNKS)):toBe("boom")
        expect(STOP.detail("boom\r\nnext", CHUNKS)):toBe("boom")
    end)

    test("文字列でない値・除いた後が空なら nil", function()
        expect(STOP.detail(nil, CHUNKS)):toBe(nil)
        expect(STOP.detail({}, CHUNKS)):toBe(nil)
        expect(STOP.detail(42, CHUNKS)):toBe(nil)
        expect(STOP.detail(RUNTIME .. ":229: ", CHUNKS)):toBe(nil)
        expect(STOP.detail("", CHUNKS)):toBe(nil)
    end)
end)
