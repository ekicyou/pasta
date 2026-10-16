-- mouse_test.lua
-- pasta.shiori.event.mouse の候補の名前（M.candidates）とアクター名（M.resolve_actor）の単体テスト
-- touch-poke-shortcuts Task 3.1 (Requirements: 1.1, 1.4–1.6, 3.1, 3.4, 3.5, 4.1–4.5, 6.1, 6.4)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local MOUSE = require("pasta.shiori.event.mouse")
local STORE = require("pasta.store")

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
