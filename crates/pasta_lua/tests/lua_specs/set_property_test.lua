-- set_property メソッドおよび escape_tag_arg 関数のテスト
-- SHIORI_ACT_IMPL.set_property() のトークン生成・バリデーション・エスケープ・統合ビルドを検証する
-- escape_tag_arg は shiori/act.lua のローカル関数のため set_property 経由でのみ検証する
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- モックアクターの生成
local function create_mock_actors()
    return {
        ["さくら"] = { name = "さくら", spot = 0 },
    }
end

-- ============================================================================
-- Group A: set_property 単体テスト（トークン生成）
-- Requirement 1.1, 1.2, 2.2, 2.3, 2.4
-- ============================================================================

describe("SHIORI_ACT_IMPL.set_property() - token generation", function()
    test("通常のname/valueでraw_scriptトークンが生成される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("prop.name", "value")

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("raw_script")
        expect(act.token[1].text):toBe("\\![set,property,prop.name,value]")
    end)

    test("valueがnilの場合、空文字列として処理され ,] で終わるトークンを生成する", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("mykey", nil)

        expect(#act.token):toBe(1)
        expect(act.token[1].type):toBe("raw_script")
        expect(act.token[1].text):toBe("\\![set,property,mykey,]")
    end)

    test("valueが数値の場合、tostringで文字列に変換される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("count", 42)

        expect(#act.token):toBe(1)
        expect(act.token[1].text):toBe("\\![set,property,count,42]")
    end)

    test("戻り値がact自身であり、メソッドチェーンが可能", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local act = SHIORI_ACT.new(actors)

        local returned = act:set_property("k", "v")

        expect(returned):toBe(act)
    end)

    test("valueが空文字列の場合、空のvalue部分を持つタグを生成する", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("mykey", "")

        expect(act.token[1].text):toBe("\\![set,property,mykey,]")
    end)

    test("複数回呼び出すと呼び出し順に2件のraw_scriptトークンが生成される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("k1", "v1")
        act:set_property("k2", "v2")

        expect(#act.token):toBe(2)
        expect(act.token[1].text):toBe("\\![set,property,k1,v1]")
        expect(act.token[2].text):toBe("\\![set,property,k2,v2]")
    end)
end)

-- ============================================================================
-- Group B: バリデーションテスト
-- Requirement 2.1
-- ============================================================================

describe("SHIORI_ACT_IMPL.set_property() - validation", function()
    test("nameがnilの場合errorが発生する", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        local ok, err = pcall(function()
            act:set_property(nil, "value")
        end)

        expect(ok):toBe(false)
        expect(err ~= nil):toBe(true)
    end)

    test("nameが空文字列の場合errorが発生する", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        local ok, err = pcall(function()
            act:set_property("", "value")
        end)

        expect(ok):toBe(false)
        expect(err ~= nil):toBe(true)
    end)
end)

-- ============================================================================
-- Group C: escape_tag_arg テスト（set_property 経由で検証）
-- Requirement 2.5
-- ============================================================================

describe("escape_tag_arg() - via set_property token.text", function()
    test("バックスラッシュ \\ が \\\\ にエスケープされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- value = "\" (シングルバックスラッシュ、Luaリテラルで "\\" )
        act:set_property("name", "\\")

        -- escaped value: \\ (2文字)
        -- tag: \![set,property,name,\\]
        -- Luaリテラル: "\\![set,property,name,\\\\]"
        expect(act.token[1].text):toBe("\\![set,property,name,\\\\]")
    end)

    test("% が \\% にエスケープされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("name", "%value%")

        -- %value% → \%value\%
        -- Luaリテラル: "\\![set,property,name,\\%value\\%]"
        expect(act.token[1].text):toBe("\\![set,property,name,\\%value\\%]")
    end)

    test("] が \\] にエスケープされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("name", "a]b")

        -- a]b → a\]b
        -- tag: \![set,property,name,a\]b]
        expect(act.token[1].text):toBe("\\![set,property,name,a\\]b]")
    end)

    test(", を含む場合 \"\" でクォートされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("name", "foo,bar")

        -- foo,bar → "foo,bar"
        expect(act.token[1].text):toBe('\\![set,property,name,"foo,bar"]')
    end)

    test('" を含む場合 "" でクォートされ、内部 " が "" に変換される', function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- value: say "hello"  (ダブルクォートを含む)
        act:set_property("name", 'say "hello"')

        -- say "hello" → "say ""hello"""
        expect(act.token[1].text):toBe('\\![set,property,name,"say ""hello"""]')
    end)

    test(", と \\ が混在する場合、文字エスケープ後にクォートされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- value: "a,b\c"  (カンマとバックスラッシュが混在)
        act:set_property("name", "a,b\\c")

        -- Step1: a,b\c → a,b\\c  (\を\\にエスケープ)
        -- Step2: カンマあり → "a,b\\c"  (クォートで囲む)
        -- tag: \![set,property,name,"a,b\\c"]
        -- Luaリテラル: '\\![set,property,name,"a,b\\\\c"]'
        expect(act.token[1].text):toBe('\\![set,property,name,"a,b\\\\c"]')
    end)

    test("特殊文字を含まない値はエスケープされない", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("sakura.name", "Alice")

        expect(act.token[1].text):toBe("\\![set,property,sakura.name,Alice]")
    end)

    test("nameに特殊文字を含む場合nameもエスケープされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        -- name: "a\b" → "a\\b"（バックスラッシュエスケープ）
        act:set_property("a\\b", "v")

        expect(act.token[1].text):toBe("\\![set,property,a\\\\b,v]")
    end)
end)

-- ============================================================================
-- Group D: 統合ビルドテスト
-- Requirement 1.1, 1.3, 1.4
-- ============================================================================

describe("set_property() - integration build via SHIORI_ACT", function()
    test("set_property 単独でビルドした出力が \\![set,property,name,value]\\e 形式になる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local act = SHIORI_ACT.new(create_mock_actors())

        act:set_property("prop.name", "value")
        local result = act:build()

        expect(result):toBe("\\![set,property,prop.name,value]\\e")
    end)

    test("talk と set_property を混在させた場合、set_property が先に出力される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local actors = create_mock_actors()
        local sakura = actors["さくら"]
        local act = SHIORI_ACT.new(actors)

        act:set_property("mood", "happy")
        act:talk(sakura, "こんにちは")
        local result = act:build()

        -- set_property タグが出力に含まれる
        local prop_pos = result:find("\\!%[set,property,mood,happy%]")
        expect(prop_pos ~= nil):toBe(true)
        -- talk テキストが出力に含まれる
        local talk_pos = result:find("こんにちは")
        expect(talk_pos ~= nil):toBe(true)
        -- set_property が talk より先に出力される
        expect(prop_pos < talk_pos):toBe(true)
        -- \e で終わる
        expect(result:sub(-2)):toBe("\\e")
    end)
end)
