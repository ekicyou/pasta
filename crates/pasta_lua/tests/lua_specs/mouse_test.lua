-- mouse_test.lua
-- pasta.shiori.event.mouse の候補の名前（M.candidates）とアクター名（M.resolve_actor）の単体テスト
-- touch-poke-shortcuts Task 3.1 (Requirements: 1.1, 1.4–1.6, 3.1, 3.4, 3.5, 4.1–4.5, 6.1, 6.4)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 他のテストが pasta.store を作り直しても、MOUSE とこのテストが同じ STORE を見るように読み直す
local STORE = require("pasta.store")
package.loaded["pasta.shiori.event.mouse"] = nil
local MOUSE = require("pasta.shiori.event.mouse")

--- 配列を "|" で連結して比べる
local function joined(list)
    return table.concat(list, "|")
end

describe("MOUSE.candidates - 候補の名前", function()
    test("アクターと部位あり → 8 個を上の段から順に並べる", function()
        expect(joined(MOUSE.candidates("poke", "男の子", "Head"))):toBe(joined({
            "つっつき反応男の子Head", "OnPoke男の子Head",
            "つっつき反応男の子その他", "OnPoke男の子Other",
            "つっつき反応Head", "OnPokeHead",
            "つっつき反応その他", "OnPokeOther",
        }))
    end)

    test("撫では さわり反応 / OnTouch を使う", function()
        expect(joined(MOUSE.candidates("touch", "女の子", "Bust"))):toBe(joined({
            "さわり反応女の子Bust", "OnTouch女の子Bust",
            "さわり反応女の子その他", "OnTouch女の子Other",
            "さわり反応Bust", "OnTouchBust",
            "さわり反応その他", "OnTouchOther",
        }))
    end)

    test("部位なし（nil・空文字列）→ 部位の段を飛ばす", function()
        local expected = joined({
            "つっつき反応男の子その他", "OnPoke男の子Other",
            "つっつき反応その他", "OnPokeOther",
        })
        expect(joined(MOUSE.candidates("poke", "男の子", nil))):toBe(expected)
        expect(joined(MOUSE.candidates("poke", "男の子", ""))):toBe(expected)
    end)

    test("アクターなし → アクターの段を飛ばす（番号の名前は入れない）", function()
        expect(joined(MOUSE.candidates("poke", nil, "Head"))):toBe(joined({
            "つっつき反応Head", "OnPokeHead",
            "つっつき反応その他", "OnPokeOther",
        }))
    end)

    test("両方なし → 7・8 段目だけ（接頭辞だけの名前は入れない）", function()
        expect(joined(MOUSE.candidates("touch", nil, ""))):toBe(joined({
            "さわり反応その他", "OnTouchOther",
        }))
    end)

    test("部位 その他 は特別扱いせず、5 段目と 7 段目が同じ名前のまま並ぶ", function()
        local list = MOUSE.candidates("poke", nil, "その他")
        expect(joined(list)):toBe(joined({
            "つっつき反応その他", "OnPokeその他",
            "つっつき反応その他", "OnPokeOther",
        }))
    end)
end)

--- @pasta_config と STORE.appearance を差し替えて body を実行し、必ず復元する
local function with_config(config, owners, body)
    local saved_config = package.loaded["@pasta_config"]
    local saved_appearance = STORE.appearance
    package.loaded["@pasta_config"] = config
    STORE.appearance = { actors = {}, spots = {}, owners = owners, last_spots = {} }
    local ok, err = pcall(body)
    package.loaded["@pasta_config"] = saved_config
    STORE.appearance = saved_appearance
    if not ok then error(err, 0) end
end

local CONFIG = {
    actor = {
        ["女の子"] = { spot = 0 },
        ["男の子"] = { spot = 1 },
        ["猫"] = { spot = 2 },
        ["犬"] = { spot = 2 },
        ["無指定"] = {},
    },
}

describe("MOUSE.resolve_actor - アクター名", function()
    test("spot が 1 人ならその名前", function()
        with_config(CONFIG, {}, function()
            expect(MOUSE.resolve_actor("0")):toBe("女の子")
            expect(MOUSE.resolve_actor("1")):toBe("男の子")
        end)
    end)

    test("0 人なら nil", function()
        with_config(CONFIG, {}, function()
            expect(MOUSE.resolve_actor("3")):toBeNil()
        end)
    end)

    test("数でなければ nil", function()
        with_config(CONFIG, {}, function()
            expect(MOUSE.resolve_actor(nil)):toBeNil()
            expect(MOUSE.resolve_actor("abc")):toBeNil()
        end)
    end)

    test("2 人以上で最後に話した人がその中にいればその名前", function()
        with_config(CONFIG, { [2] = "犬" }, function()
            expect(MOUSE.resolve_actor("2")):toBe("犬")
        end)
    end)

    test("2 人以上で最後に話した人がその中にいなければ nil", function()
        with_config(CONFIG, { [2] = "女の子" }, function()
            expect(MOUSE.resolve_actor("2")):toBeNil()
        end)
    end)

    test("2 人以上でまだ誰も話していなければ nil", function()
        with_config(CONFIG, {}, function()
            expect(MOUSE.resolve_actor("2")):toBeNil()
        end)
    end)

    test("@pasta_config が無ければ nil", function()
        local saved_preload = package.preload["@pasta_config"]
        package.preload["@pasta_config"] = nil
        with_config(nil, {}, function()
            expect(MOUSE.resolve_actor("0")):toBeNil()
        end)
        package.preload["@pasta_config"] = saved_preload
    end)
end)

-- touch-poke-shortcuts Task 3.2 撫での判定 (Requirements: 2.1–2.8, 3.6, 5.1, 5.2)
describe("MOUSE.check_stroke - 撫での判定", function()
    --- 同じ所 (0, Head) へ移動を順に送り、各時刻の結果を "t:true" の形で並べる
    local function run(times, scope, part)
        local out = {}
        for _, t in ipairs(times) do
            out[#out + 1] = t .. ":" .. tostring(MOUSE.check_stroke(scope or "0", part or "Head", t, false))
        end
        return table.concat(out, " ")
    end

    test("0・0.5・1.0・1.5・1.99 秒は決めず 2.0 秒で決め 2.5 秒は決めない", function()
        MOUSE._reset()
        expect(run({ 0, 0.5, 1.0, 1.5, 1.99, 2.0, 2.5 })):toBe(
            "0:false 0.5:false 1:false 1.5:false 1.99:false 2:true 2.5:false")
        expect(MOUSE._get_state().fired):toBe(true)
    end)

    test("途中で部位が変わると数え直す", function()
        MOUSE._reset()
        run({ 0, 1.5 }, "0", "Head")
        expect(run({ 2.0, 3.99 }, "0", "Bust")):toBe("2:false 3.99:false")
        expect(run({ 4.0 }, "0", "Bust")):toBe("4:true")
    end)

    test("途中でスコープが変わると数え直す", function()
        MOUSE._reset()
        run({ 0, 1.5 }, "0", "Head")
        expect(run({ 2.0 }, "1", "Head")):toBe("2:false")
        expect(run({ 4.0 }, "1", "Head")):toBe("4:true")
    end)

    test("2.01 秒の途切れで数え直す（2 秒ちょうどは続き）", function()
        MOUSE._reset()
        expect(run({ 0, 2.01, 4.0, 4.02 })):toBe("0:false 2.01:false 4:false 4.02:true")
        MOUSE._reset()
        expect(run({ 0, 2.0 })):toBe("0:false 2:true")
    end)

    test("発火済みのまま同じ所を撫で続けても 2 回目は決めない", function()
        MOUSE._reset()
        expect(run({ 0, 2.0, 3.0, 4.0, 5.5, 7.0 })):toBe("0:false 2:true 3:false 4:false 5.5:false 7:false")
    end)

    test("部位が空の移動を挟むと発火済みが消える", function()
        MOUSE._reset()
        run({ 0, 2.0 })
        expect(MOUSE.check_stroke("0", "", 2.5, false)):toBe(false)
        expect(MOUSE._get_state().target):toBeNil()
        expect(MOUSE._get_state().fired):toBe(false)
        expect(MOUSE.check_stroke("0", nil, 2.6, false)):toBe(false)
        expect(run({ 3.0, 4.99, 5.0 })):toBe("3:false 4.99:false 5:true")
    end)

    test("抑止を挟むと解けた後の最初の移動から 2 秒", function()
        MOUSE._reset()
        run({ 0, 1.0 })
        expect(MOUSE.check_stroke("0", "Head", 1.5, true)):toBe(false)
        expect(MOUSE.check_stroke("0", "Head", 2.5, true)):toBe(false)
        expect(MOUSE._get_state().target):toBeNil()
        expect(run({ 3.0, 4.99, 5.0 })):toBe("3:false 4.99:false 5:true")
    end)

    test("時計が戻ると数え直す", function()
        MOUSE._reset()
        run({ 10, 11.5 })
        expect(run({ 11.0, 12.99, 13.0 })):toBe("11:false 12.99:false 13:true")
    end)

    test("時刻なしで数え直す", function()
        MOUSE._reset()
        run({ 0, 1.5 })
        expect(MOUSE.check_stroke("0", "Head", nil, false)):toBe(false)
        expect(MOUSE._get_state().target):toBeNil()
        expect(run({ 2.0, 3.99, 4.0 })):toBe("2:false 3.99:false 4:true")
    end)

    test("_reset で Off に戻る", function()
        MOUSE._reset()
        run({ 0, 2.0 })
        MOUSE._reset()
        local s = MOUSE._get_state()
        expect(s.target):toBeNil()
        expect(s.fired):toBe(false)
    end)
end)

-- tutorial-ssp-gaps Task 8.3 撫での判定はなめらかな時計で数える (Requirements: 2.5, 7.3, 7.5)
describe("OnMouseMove - 時刻の飛びをまたぐ撫で", function()
    local REG = require("pasta.shiori.event.register")
    local CLOCK = require("pasta.shiori.clock")

    --- 既定のハンドラを持つ mouse を読み直して body(handler, M) を回し、元に戻す
    local function with_handler(body)
        local saved_mod = package.loaded["pasta.shiori.event.mouse"]
        local saved_handler = REG.OnMouseMove
        local saved_scene = package.loaded["pasta.scene"]
        package.loaded["pasta.scene"] = {
            co_exec = function() return nil end,
            co_exec_exact = function() return nil end,
        }
        package.loaded["pasta.shiori.event.mouse"] = nil
        REG.OnMouseMove = nil
        CLOCK._reset()
        local M = require("pasta.shiori.event.mouse")
        local ok, err = pcall(body, REG.OnMouseMove, M)
        REG.OnMouseMove = saved_handler
        package.loaded["pasta.shiori.event.mouse"] = saved_mod
        package.loaded["pasta.scene"] = saved_scene
        CLOCK._reset()
        if not ok then error(err, 0) end
    end

    local function move(handler, u)
        handler({ req = {
            id = "OnMouseMove", status = "idle",
            reference = { [3] = "0", [4] = "Head" }, date = { unix = u, ns = 0 },
        } })
    end

    local function second(u)
        CLOCK.observe({ id = "OnSecondChange", date = { unix = u } })
    end

    local JUMP = 3 * 3600

    test("飛びをまたいでも、なめらかな時計で 2 秒続けば撫でたと決める", function()
        with_handler(function(handler, M)
            second(1000)
            move(handler, 1000)
            move(handler, 1001)
            second(1001)
            second(1002 + JUMP)
            expect(CLOCK._get_state().epoch):toBe(1)
            move(handler, 1002 + JUMP)
            expect(M._get_state().fired):toBe(true)
        end)
    end)

    test("飛びそのものでは 2 秒に届かない", function()
        with_handler(function(handler, M)
            second(1000)
            move(handler, 1000)
            second(1000 + JUMP + 1)
            move(handler, 1001 + JUMP)
            expect(M._get_state().fired):toBe(false)
            move(handler, 1002 + JUMP)
            expect(M._get_state().fired):toBe(true)
        end)
    end)
end)
