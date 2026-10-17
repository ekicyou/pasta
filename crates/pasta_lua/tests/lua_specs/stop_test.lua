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

-- ============================================================================
-- 控え・名前・応答 (runtime-error-balloon 3.3)
-- ============================================================================

--- 警告を記録するログ・balloon_warning の代役・読み込んだ Lua のファイルの表を差し込んで body を実行し、元へ戻す
---@param balloon any  [logging] balloon_warning の値（nil なら書かない）
---@param body fun(warns: string[])
local function with_env(balloon, body)
    local saved_log = package.loaded["@pasta_log"]
    local saved_config = package.loaded["pasta.config"]
    local saved_chunks = package.loaded["@pasta_chunks"]
    local warns = {}
    local noop = function() end
    package.loaded["@pasta_log"] = {
        trace = noop, debug = noop, info = noop, error = noop,
        warn = function(msg) table.insert(warns, msg) end,
    }
    package.loaded["pasta.config"] = {
        get = function(section, key, default)
            if section == "logging" and key == "balloon_warning" and balloon ~= nil then return balloon end
            return default
        end,
    }
    package.loaded["@pasta_chunks"] = {
        [CACHE] = true,
        ["scripts/boot.lua"] = "scripts\\boot.lua",
    }
    local ok, err = pcall(body, warns)
    package.loaded["@pasta_log"] = saved_log
    package.loaded["pasta.config"] = saved_config
    package.loaded["@pasta_chunks"] = saved_chunks
    STOP.clear()
    if not ok then error(err, 0) end
end

local function actors()
    return { sakura = { name = "さくら", spot = 0 } }
end

--- 応答の Value の行の中身（無ければ nil）
local function value_of(res)
    return res and res:match("\r\nValue: ([^\r\n]*)\r\n")
end

--- 応答の X-Error-Reason の行の中身（無ければ nil）
local function reason_of(res)
    return res and res:match("\r\nX%-Error%-Reason: ([^\r\n]*)\r\n")
end

--- 天気・雨の日で止まった act（書いた名前と行の対応を控えている）
local function weather_act()
    local act = require("pasta.shiori.act").new(actors())
    act.scene_name = "天気"
    act.local_scene_name = "雨の日"
    act.line_map = LINE_MAP
    return act
end

local WEATHER_ERR = RUNTIME .. ":229: " .. CACHE .. ":30: attempt to index a nil value (local 'data')"
local WEATHER_NOTATION =
    [[【スクリプトのエラー：＊天気・雨の日（dic\\天気.pasta:15: attempt to index a nil value (local 'data')）】]]
local BOOT_NOTATION = [[【スクリプトのエラー：OnBoot（scripts\\boot.lua:5: attempt to call a nil value）】]]

--- needle が haystack の中で after より後に出るか
local function appears_after(haystack, after, needle)
    local a = haystack and haystack:find(after, 1, true)
    local n = haystack and haystack:find(needle, 1, true)
    return a ~= nil and n ~= nil and a < n
end

local function contains(haystack, needle)
    return haystack ~= nil and haystack:find(needle, 1, true) ~= nil
end

describe("STOP の控え", function()
    test("hold した控えを take で取り出すと空になる・clear で空になる", function()
        local co = coroutine.create(function() end)
        STOP.clear()
        expect(STOP.take()):toBe(nil)
        STOP.hold(co, "e1")
        local held = STOP.take()
        expect(held.err):toBe("e1")
        expect(held.act):toBe(nil)
        expect(STOP.take()):toBe(nil)
        STOP.hold(co, "e2")
        STOP.clear()
        expect(STOP.take()):toBe(nil)
    end)

    test("hold は止まったコルーチンの act を FAILURE.act_of で引く・前の控えを上書きする", function()
        local FAILURE = require("pasta.failure")
        local act = { token = {} }
        local co = coroutine.create(function() FAILURE.bind(act); error("boom") end)
        coroutine.resume(co)
        STOP.hold(coroutine.create(function() end), "old")
        STOP.hold(co, "new")
        local held = STOP.take()
        expect(held.act):toBe(act)
        expect(held.err):toBe("new")
    end)

    test("ハンドラの act の控えは take_handler で取り出すと空になる・clear で空になる", function()
        local act = {}
        STOP.hold_handler(act)
        expect(STOP.take_handler()):toBe(act)
        expect(STOP.take_handler()):toBe(nil)
        STOP.hold_handler(act)
        STOP.clear()
        expect(STOP.take_handler()):toBe(nil)
    end)
end)

describe("STOP.subject（止まった場所の名前）", function()
    test("書いた名前: ＊グローバル・ローカル", function()
        expect(STOP.subject({ scene_name = "天気", local_scene_name = "雨の日" })):toBe("＊天気・雨の日")
        expect(STOP.subject({ scene_name = "天気" })):toBe("＊天気")
    end)

    test("書いた名前が無ければ登録名から連番を落とした名前", function()
        expect(STOP.subject({ current_scene = { __global_name__ = "天気_3" } })):toBe("＊天気")
        expect(STOP.subject({ current_scene = { __global_name__ = "OnTalk_1_2" } })):toBe("＊OnTalk_1")
        expect(STOP.subject({ current_scene = { __global_name__ = "天気" } })):toBe("＊天気")
    end)

    test("シーンの外・act が無いときは nil", function()
        expect(STOP.subject(nil)):toBe(nil)
        expect(STOP.subject({})):toBe(nil)
        expect(STOP.subject({ current_scene = {} })):toBe(nil)
    end)
end)

describe("STOP.response（表記付きの応答）", function()
    test("ステータスはタスク 1 で決めた 200 OK", function()
        expect(STOP.STATUS):toBe("200 OK")
    end)

    test("シーンで止まった: 要件 2.2 の例の文言・ステータス・理由の行", function()
        with_env(nil, function()
            local res = STOP.response({
                reason = "REASON", hold = { act = weather_act(), err = WEATHER_ERR }, fallback = "OnTalk", shown = true,
            })
            expect(res:sub(1, #"SHIORI/3.0 200 OK\r\n")):toBe("SHIORI/3.0 200 OK\r\n")
            expect(contains(value_of(res), WEATHER_NOTATION)):toBe(true)
            expect(reason_of(res)):toBe("REASON")
        end)
    end)

    test("途中の台詞の後に表記が出る", function()
        with_env(nil, function()
            local act = weather_act()
            act:talk(act.actors.sakura, "今日は")
            local value = value_of(STOP.response({
                reason = "R", hold = { act = act, err = WEATHER_ERR }, shown = true,
            }))
            expect(appears_after(value, "今日は", WEATHER_NOTATION)):toBe(true)
        end)
    end)

    test("シーンの外（ハンドラ）は要求の ID で示し、ハンドラの act の台詞の後に積む", function()
        with_env(nil, function()
            local handler = require("pasta.shiori.act").new(actors())
            handler:talk(handler.actors.sakura, "起動")
            local value = value_of(STOP.response({
                reason = "R", handler = handler, err = "scripts/boot.lua:5: attempt to call a nil value",
                fallback = "OnBoot", shown = true,
            }))
            expect(appears_after(value, "起動", BOOT_NOTATION)):toBe(true)
        end)
    end)

    test("act が無ければ新しい act に表記だけを積む", function()
        with_env(nil, function()
            local value = value_of(STOP.response({
                reason = "R", err = "scripts/boot.lua:5: attempt to call a nil value", fallback = "OnBoot",
                shown = true,
            }))
            expect(contains(value, BOOT_NOTATION)):toBe(true)
        end)
    end)

    test("理由付きの時間切れ: 【時間切れ：…】", function()
        with_env(nil, function()
            local value = value_of(STOP.response({
                reason = "R", timeout = "callback timeout: get_property", shown = true,
            }))
            expect(contains(value, "【時間切れ：callback timeout: get_property】")):toBe(true)
        end)
    end)

    test("エラーの値が文字列でない: 【スクリプトのエラー】", function()
        with_env(nil, function()
            local value = value_of(STOP.response({
                reason = "R", hold = { act = weather_act(), err = { code = 1 } }, fallback = "OnTalk", shown = true,
            }))
            expect(contains(value, "【スクリプトのエラー】")):toBe(true)
        end)
    end)

    test("\\・%・改行は書いたとおりに出る形になる", function()
        with_env(nil, function()
            local value = value_of(STOP.response({
                reason = "R", err = [[100% \s0 だめ]], fallback = "OnBoot", shown = true,
            }))
            expect(contains(value, [[【スクリプトのエラー：OnBoot（100\% \\s0 だめ）】]])):toBe(true)
            value = value_of(STOP.response({ reason = "R", timeout = "a\nb", shown = true }))
            expect(contains(value, "【時間切れ：a b】")):toBe(true)
        end)
    end)

    test("理由の行は渡された値のまま・空なら Unknown error", function()
        with_env(nil, function()
            local reason = RUNTIME .. ":229: " .. CACHE .. ":30: boom"
            expect(reason_of(STOP.response({ reason = reason, err = "x", fallback = "OnBoot", shown = true })))
                :toBe(reason)
            expect(reason_of(STOP.response({ err = "x", fallback = "OnBoot", shown = true }))):toBe("Unknown error")
            expect(reason_of(STOP.response({ reason = "", err = "x", fallback = "OnBoot", shown = true })))
                :toBe("Unknown error")
        end)
    end)

    test("出さない設定では nil を返し、警告は 1 行出す", function()
        with_env(false, function(warns)
            expect(STOP.response({
                reason = "R", hold = { act = weather_act(), err = WEATHER_ERR }, shown = true, warning = true,
            })):toBe(nil)
            expect(#warns):toBe(1)
        end)
    end)

    test("On で始まらない ID（shown が偽）では nil を返し、警告は 1 行出す", function()
        with_env(nil, function(warns)
            expect(STOP.response({
                reason = "R", err = "x", fallback = "version", shown = false, warning = true,
            })):toBe(nil)
            expect(#warns):toBe(1)
        end)
    end)

    test("警告は表記と同じ整えた中身（枠・エスケープなし・キャッシュのパスなし）で 1 行", function()
        with_env(nil, function(warns)
            STOP.response({
                reason = "R", hold = { act = weather_act(), err = WEATHER_ERR }, shown = true, warning = true,
            })
            expect(#warns):toBe(1)
            expect(warns[1]):toBe(
                "スクリプトのエラー：＊天気・雨の日（dic\\天気.pasta:15: attempt to index a nil value (local 'data')）")
            expect(warns[1]:find("cache", 1, true)):toBe(nil)
        end)
    end)

    test("警告が文字列ならその文を前に付ける・nil なら出さない", function()
        with_env(nil, function(warns)
            STOP.response({ reason = "R", timeout = "t", shown = true, warning = "時間切れの掃引: " })
            STOP.response({ reason = "R", timeout = "t", shown = true })
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("時間切れの掃引: 時間切れ：t")
        end)
    end)

    test("act の組み立てが失敗したら表記だけを値にする", function()
        with_env(nil, function()
            local broken = { token = {}, build = function() error("build failed") end }
            local value = value_of(STOP.response({
                reason = "R", hold = { act = broken, err = "boom" }, fallback = "OnTalk", shown = true,
            }))
            expect(value):toBe("【スクリプトのエラー：OnTalk（boom）】")
        end)
    end)
end)
