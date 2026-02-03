-- sakura_builder module tests
-- Tests for pasta.shiori.sakura_builder module - token to sakura script conversion
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- Requirement 6: sakura_builderモジュール
-- ============================================================================

describe("SAKURA_BUILDER - talk token", function()
    test("talkトークンをエスケープ済みテキストに変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "talk", text = "Hello" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("Hello\\e")
    end)

    test("バックスラッシュをエスケープする", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "talk", text = "path\\to\\file" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("path\\\\to\\\\file")):toBeTruthy()
    end)

    test("パーセントをエスケープする", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "talk", text = "100%" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("100%%%%")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - actor token", function()
    test("actorトークンをスポットタグ \\p[n] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = 0 } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[0]\\e")
    end)

    test("spot=1のactorを \\p[1] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = 1 } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[1]\\e")
    end)

    test("spot='sakura'を \\p[0] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = "sakura" } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[0]\\e")
    end)

    test("spot='kero'を \\p[1] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = "kero" } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[1]\\e")
    end)

    test("spot='char2'を \\p[2] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = "char2" } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[2]\\e")
    end)

    test("spot=nilを \\p[0] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor", actor = { spot = nil } },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\p[0]\\e")
    end)
end)

describe("SAKURA_BUILDER - spot_switch token", function()
    test("spot_switchトークンを段落区切り改行に変換する（デフォルト1.5→150）", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "spot_switch" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\n[150]\\e")
    end)

    test("config.spot_switch_newlinesの設定を反映する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "spot_switch" },
        }
        local result = BUILDER.build(tokens, { spot_switch_newlines = 2.0 })

        expect(result):toBe("\\n[200]\\e")
    end)

    test("spot_switch_newlines=1.0で \\n[100] を出力する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "spot_switch" },
        }
        local result = BUILDER.build(tokens, { spot_switch_newlines = 1.0 })

        expect(result):toBe("\\n[100]\\e")
    end)
end)

describe("SAKURA_BUILDER - surface token", function()
    test("surfaceトークンを \\s[id] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "surface", id = 5 },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\s[5]\\e")
    end)

    test("文字列IDをサポートする", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "surface", id = "smile" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\s[smile]\\e")
    end)
end)

describe("SAKURA_BUILDER - wait token", function()
    test("waitトークンを \\w[ms] に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "wait", ms = 500 },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\w[500]\\e")
    end)
end)

describe("SAKURA_BUILDER - newline token", function()
    test("newlineトークンを \\n に変換する（n=1）", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "newline", n = 1 },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\n\\e")
    end)

    test("複数改行を連続出力する（n=3）", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "newline", n = 3 },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\n\\n\\n\\e")
    end)
end)

describe("SAKURA_BUILDER - clear token", function()
    test("clearトークンを \\c に変換する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "clear" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\c\\e")
    end)
end)

describe("SAKURA_BUILDER - sakura_script token", function()
    test("sakura_scriptトークンをそのまま出力する（エスケープなし）", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "sakura_script", text = "\\![open,calendar]" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\![open,calendar]\\e")
    end)
end)

describe("SAKURA_BUILDER - yield token", function()
    test("yieldトークンは無視される", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "surface", id = 5 },
            { type = "yield" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result):toBe("\\s[5]\\e")
    end)
end)

describe("SAKURA_BUILDER - \\e終端", function()
    test("出力末尾に \\e を付与する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "talk", text = "Hello" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("空トークン配列でも \\e を付与する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local result = BUILDER.build({}, {})

        expect(result):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - 複合シナリオ", function()
    test("複数トークンを正しく連結する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "actor",      actor = { spot = 0 } },
            { type = "talk",       text = "Hello" },
            { type = "surface",    id = 5 },
            { type = "wait",       ms = 100 },
            { type = "actor",      actor = { spot = 1 } },
            { type = "spot_switch" },
            { type = "talk",       text = "Hi" },
            { type = "newline",    n = 1 },
            { type = "clear" },
        }
        local result = BUILDER.build(tokens, { spot_switch_newlines = 1.5 })

        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()
        expect(result:find("\\w%[100%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("\\n%[150%]")):toBeTruthy()
        expect(result:find("Hi")):toBeTruthy()
        expect(result:find("\\n")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - エスケープ処理", function()
    test("複合エスケープを正しく処理する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")

        local tokens = {
            { type = "talk", text = "50% off \\ sale" },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("50%%%%")):toBeTruthy()
        expect(result:find("\\\\")):toBeTruthy()
    end)
end)
