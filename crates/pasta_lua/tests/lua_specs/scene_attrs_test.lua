-- SCENE.attrs・SCENE.set_attrs・STORE.scene_attrs のテスト (scene-attribute-store 4.2)
--
-- STORE.scenes はスイート間で共有されるため、衝突しない接頭辞 "sat42_" を使う。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 先行スイートの個別リロードで STORE の参照が分かれないよう、一括で読み直す（既存スイートと同じ規約）
package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil
package.loaded["pasta.scene"] = nil
package.loaded["pasta.act"] = nil
package.loaded["pasta"] = nil

local STORE = require("pasta.store")
local SCENE = require("pasta.scene")
local PASTA = require("pasta")

local function count_keys(t)
    local n = 0
    for _ in pairs(t) do
        n = n + 1
    end
    return n
end

-- 生成コードと同じ手順で、属性のあるシーンを 1 つ作る
local function make_scene(base)
    local scene = PASTA.create_scene(base)
    function scene.__start__() end
    scene["選択肢_1"] = function() end
    scene["無印_1"] = function() end
    PASTA.set_scene_attrs(scene, "__start__", {
        { key = "背景", value = "教室" },
        { key = "＄＊経験値", op = ">=", value = 500 },
        { key = "＄＊初回待ち" },
        { key = "率", value = 1.5 },
    })
    PASTA.set_scene_attrs(scene, "選択肢_1", { { key = "優先", value = 3 } })
    return scene
end

describe("SCENE.attrs - 3 種の項目と書いた順", function()
    test("宣言・条件・フラグを書いた順で、値の型のまま返す", function()
        local scene = make_scene("sat42_会話")
        local items = SCENE.attrs(scene.__global_name__, nil)
        expect(items):toEqual({
            { key = "背景", value = "教室" },
            { key = "＄＊経験値", op = ">=", value = 500 },
            { key = "＄＊初回待ち" },
            { key = "率", value = 1.5 },
        })
        expect(type(items[2].value)):toBe("number")
        expect(type(items[1].value)):toBe("string")
        expect(items[3].op):toBeNil()
        expect(items[3].value):toBeNil()
    end)

    test("\"__start__\" は nil と同じくグローバル自身を指す", function()
        local scene = make_scene("sat42_開始")
        expect(SCENE.attrs(scene.__global_name__, "__start__"))
            :toEqual(SCENE.attrs(scene.__global_name__, nil))
    end)

    test("ローカルは登録名で読む", function()
        local scene = make_scene("sat42_ローカル")
        expect(SCENE.attrs(scene.__global_name__, "選択肢_1")):toEqual({ { key = "優先", value = 3 } })
    end)

    test("実行中のグローバルシーンを act.current_scene.__global_name__ で読む", function()
        local ACT = require("pasta.act")
        local scene = make_scene("sat42_実行中")
        local act = ACT.new({})
        act:init_scene(scene)
        expect(SCENE.attrs(act.current_scene.__global_name__, nil)[1]):toEqual({ key = "背景", value = "教室" })
    end)
end)

describe("SCENE.attrs - 空と nil", function()
    test("属性の無い登録済みシーンは空の配列", function()
        local scene = make_scene("sat42_空")
        expect(SCENE.attrs(scene.__global_name__, "無印_1")):toEqual({})
        local plain = PASTA.create_scene("sat42_属性なし")
        function plain.__start__() end
        expect(SCENE.attrs(plain.__global_name__, nil)):toEqual({})
    end)

    test("登録の無いシーンは nil", function()
        local scene = make_scene("sat42_未登録")
        expect(SCENE.attrs("sat42_どこにも無い_1", nil)):toBeNil()
        expect(SCENE.attrs(scene.__global_name__, "無い_1")):toBeNil()
    end)

    test("文字列でない引数は nil を返し、エラーを出さない", function()
        local scene = make_scene("sat42_型")
        expect(SCENE.attrs(nil, nil)):toBeNil()
        expect(SCENE.attrs(1, nil)):toBeNil()
        expect(SCENE.attrs({}, nil)):toBeNil()
        expect(SCENE.attrs(scene.__global_name__, 1)):toBeNil()
    end)
end)

describe("SCENE.attrs - 倉を守る", function()
    test("戻りの配列と項目を書き換えても倉は変わらない", function()
        local scene = make_scene("sat42_写し")
        local first = SCENE.attrs(scene.__global_name__, nil)
        first[1].value = "書き換え"
        first[2].op = "<"
        table.insert(first, { key = "足した" })
        table.remove(first, 3)
        local second = SCENE.attrs(scene.__global_name__, nil)
        expect(second[1].value):toBe("教室")
        expect(second[2].op):toBe(">=")
        expect(#second):toBe(4)
        expect(second[3]):toEqual({ key = "＄＊初回待ち" })
    end)

    test("属性を書いてもシーンの表と一覧に偽のシーンが出ない", function()
        local before = count_keys(SCENE.get_all_scenes())
        local scene = make_scene("sat42_偽なし")
        local after = count_keys(SCENE.get_all_scenes())
        expect(after):toBe(before + 1)
        expect(count_keys(scene)):toBe(4) -- __global_name__・__start__・選択肢_1・無印_1
        expect(STORE.scene_attrs[scene.__global_name__]).not_:toBeNil()
    end)

    test("STORE.reset() で倉が空になる", function()
        make_scene("sat42_リセット")
        STORE.reset()
        expect(count_keys(STORE.scene_attrs)):toBe(0)
    end)
end)
