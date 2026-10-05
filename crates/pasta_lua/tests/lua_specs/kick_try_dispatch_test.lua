-- kick_try_dispatch_test.lua
-- Lua-side BDD tests for KICK.try_dispatch (保留キックシーン解決 → co 返却)
-- pasta-scene-kick Task 4.2 (Requirements: 3.2, 3.3, 3.5)
-- scene-identity-format Task 2.2 (Requirements: 5.1, 5.2, 5.4, 8.4, 8.6)
--
-- 検証契約（シーン表への完全一致）:
--   KICK.try_dispatch(act):
--     - STORE.kick_pending == nil                     -> nil（何もしない）
--     - kick_pending がシーン表の登録名に完全一致      -> その __start__ だけを再生する co を返す
--       （`会話_1` のキックで `会話_10` を再生しない）
--     - シーン表に一致なし                            -> nil + 警告ログ（seam=kick.unresolved）
--       （前方一致・act:find_scene・GLOBAL の関数は通らない）
--     - いずれも kick_pending を消費する
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 依存閉包を整合した同一インスタンス集合として一括新規ロードする。
local RELOAD = {
    "pasta.shiori.event.kick",
    "pasta.scene",
    "pasta.word",
    "pasta.store",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local SCENE = require("pasta.scene")
local STORE = require("pasta.store")
local GLOBAL = require("pasta.global")

-- 警告を記録するログを差し込んで kick を読み込む。読み込み後はログを元へ戻し、
-- kick もキャッシュから外す（後続スイートは本物のログを持つ kick を新規ロードする）。
local warns = {}
local saved_log = package.loaded["@pasta_log"]
local noop = function() end
package.loaded["@pasta_log"] = {
    trace = noop,
    debug = noop,
    info = noop,
    warn = function(msg) table.insert(warns, msg) end,
    error = noop,
}
local KICK = require("pasta.shiori.event.kick")
package.loaded["@pasta_log"] = saved_log
package.loaded["pasta.shiori.event.kick"] = nil

--- 最小モック act。キックは解決に act を使わないので、find_scene は呼ばれたことだけを記録する。
--- @return table mock_act, table calls（find_scene に渡された name）
local function make_act()
    local calls = { names = {} }
    local act = {}
    function act.find_scene(_self, name, _global_scene_name, _attrs)
        table.insert(calls.names, name)
        return function(_resumed_act) end
    end
    -- コルーチン化ラッパー（wrap_local_func）が末尾で resumed_act:build() を呼ぶ。
    function act.build(_self)
        return nil
    end
    return act, calls
end

--- STORE とログ記録を初期化して body を実行し、成否にかかわらず初期化し直す
--- （アサーション失敗で後続スイートへシーン表・保留フラグを残さない）
--- @param body fun()
local function with_clean_state(body)
    STORE.reset()
    for i = #warns, 1, -1 do warns[i] = nil end
    local ok, err = pcall(body)
    STORE.reset()
    if not ok then error(err, 0) end
end

--- GLOBAL の 1 キーを一時的に差し替えて body を実行し、成否にかかわらず元に戻す
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

--- `会話_1`・`会話_10` をシーン表に登録する。再生されたシーンを ran に記録する。
--- @return table ran 再生された登録名の列
local function register_conversations()
    local ran = {}
    SCENE.register("会話_1", "__start__", function() table.insert(ran, "会話_1") end)
    SCENE.register("会話_10", "__start__", function() table.insert(ran, "会話_10") end)
    return ran
end

describe("KICK.try_dispatch - 保留キックシーン解決", function()
    test("kick_pending=nil のとき nil を返し何もしない", function()
        with_clean_state(function()
            local act = make_act()
            local co = KICK.try_dispatch(act)
            expect(co):toBeNil()
            expect(STORE.kick_pending):toBeNil()
        end)
    end)

    test("`会話_1` のキックは `会話_1` だけを再生し `会話_10` を再生しない（5.1）", function()
        with_clean_state(function()
            local ran = register_conversations()
            STORE.kick_pending = "会話_1"
            local act, calls = make_act()

            local co = KICK.try_dispatch(act)

            expect(type(co)):toBe("thread")
            -- フラグ消費（再発火防止）
            expect(STORE.kick_pending):toBeNil()
            -- act:find_scene の探索は通らない
            expect(#calls.names):toBe(0)

            local ok = coroutine.resume(co, act)
            expect(ok):toBe(true)
            expect(#ran):toBe(1)
            expect(ran[1]):toBe("会話_1")
        end)
    end)

    test("GLOBAL に同名の関数を置いても再生しない（シーン表のシーンだけ・5.2）", function()
        with_clean_state(function()
            local global_ran = false
            with_global("会話_1", function() global_ran = true end, function()
                -- シーン表には `会話_10` だけがあり `会話_1` は無い
                SCENE.register("会話_10", "__start__", function() end)
                STORE.kick_pending = "会話_1"
                local act, calls = make_act()

                local co = KICK.try_dispatch(act)

                expect(co):toBeNil()
                expect(global_ran):toBe(false)
                expect(#calls.names):toBe(0)
                expect(STORE.kick_pending):toBeNil()
            end)
        end)
    end)

    test("シーン表に一致なし（前方一致の `会話`）は警告ログを残し nil（5.4）", function()
        with_clean_state(function()
            local ran = register_conversations()
            STORE.kick_pending = "会話"
            local act, calls = make_act()

            local co = KICK.try_dispatch(act)

            expect(co):toBeNil()
            expect(#ran):toBe(0)
            expect(#calls.names):toBe(0)
            -- 解決不能でもフラグは消費（再発火しない・前会話は保持）
            expect(STORE.kick_pending):toBeNil()
            expect(#warns):toBe(1)
            expect(string.find(warns[1], "seam=kick.unresolved scene=会話:", 1, true) ~= nil):toBe(true)
        end)
    end)
end)
