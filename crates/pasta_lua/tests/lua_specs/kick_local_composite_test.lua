-- kick_local_composite_test.lua
-- Lua-side BDD tests for KICK.try_dispatch の local-composite (`:parent:local`) 分岐
-- pasta-scene-kick-from-cursor Task 3.1 (Requirements: 2.2, 2.4)
-- scene-identity-format Task 2.2 (Requirements: 5.3, 5.4, 8.4, 8.6)
--
-- 検証契約（composite-string 方式・debug 限定・シーン表への完全一致）:
--   KICK.try_dispatch(act):
--     - scene_name が `:会話_1:挨拶_1` 形式 → local 分岐:
--         SCENE.get("会話_1", "挨拶_1") で親の中のローカルを完全一致で引きコルーチン化
--         （global の __start__ へ潰さず local 同一性を保持・`挨拶_10` を再生しない）。
--     - scene_name に先頭 `:` が無い（`会話_1` 等）→ global 分岐:
--         SCENE.get_start("会話_1")（ローカルは再生しない）。
--     - local-composite が解決不能（親の中に完全一致なし）→ warn + drop + nil。
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

local KICK = require("pasta.shiori.event.kick")
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

--- 最小モック act。ラッパー（wrap_local_func）が co の中で resumed_act:build() を呼ぶ。
--- @return table mock_act
local function make_act()
    local act = {}
    function act.build(_self)
        return "<built>"
    end
    return act
end

--- STORE を初期化して body を実行し、成否にかかわらず初期化し直す
--- （アサーション失敗で後続スイートへシーン表・保留フラグを残さない）
--- @param body fun()
local function with_clean_state(body)
    STORE.reset()
    local ok, err = pcall(body)
    STORE.reset()
    if not ok then error(err, 0) end
end

--- 親 `会話_1` に __start__・`挨拶_1`・`挨拶_10` を登録する。再生されたシーンを ran に記録する。
--- @return table ran 再生されたシーン名の列
local function register_greetings()
    local ran = {}
    SCENE.register("会話_1", "__start__", function() table.insert(ran, "__start__") end)
    SCENE.register("会話_1", "挨拶_1", function() table.insert(ran, "挨拶_1") end)
    SCENE.register("会話_1", "挨拶_10", function() table.insert(ran, "挨拶_10") end)
    return ran
end

describe("KICK.try_dispatch - local-composite (`:parent:local`) 分岐", function()
    test("`:会話_1:挨拶_1` は親の中の `挨拶_1` だけを再生し `挨拶_10` を再生しない（5.3）", function()
        with_clean_state(function()
            local ran = register_greetings()
            STORE.kick_pending = ":会話_1:挨拶_1"
            local act = make_act()

            local co = KICK.try_dispatch(act)

            -- コルーチンが返る
            expect(type(co)):toBe("thread")
            -- フラグ消費
            expect(STORE.kick_pending):toBeNil()

            -- resume で local func が走り、build() の結果が返る（local 同一性を保持）
            local ok, value = coroutine.resume(co, act)
            expect(ok):toBe(true)
            expect(value):toBe("<built>")
            expect(#ran):toBe(1)
            expect(ran[1]):toBe("挨拶_1")
        end)
    end)

    test("local-composite が親の中で完全一致しなければ warn + drop + nil（kick_pending 消費）", function()
        with_clean_state(function()
            local ran = register_greetings()
            -- `挨拶` は `挨拶_1`・`挨拶_10` の前方一致だが、完全一致ではない
            STORE.kick_pending = ":会話_1:挨拶"
            local act = make_act()

            local co = KICK.try_dispatch(act)

            expect(co):toBeNil()
            expect(#ran):toBe(0)
            -- 解決不能でもフラグ消費（再発火しない）
            expect(STORE.kick_pending):toBeNil()
        end)
    end)

    test("先頭 `:` 無しは global 分岐（親の __start__ だけを再生しローカルは再生しない）", function()
        with_clean_state(function()
            local ran = register_greetings()
            STORE.kick_pending = "会話_1"
            local act = make_act()

            local co = KICK.try_dispatch(act)

            expect(type(co)):toBe("thread")
            expect(STORE.kick_pending):toBeNil()
            local ok = coroutine.resume(co, act)
            expect(ok):toBe(true)
            expect(#ran):toBe(1)
            expect(ran[1]):toBe("__start__")
        end)
    end)
end)
