-- pasta（pasta/init.lua 公開 API）と pasta.global のエイリアス同一性テスト
-- review-improvement-loop cell 3.46 (G1): トランスパイラー出力が依存する
-- PASTA 公開 API のリダイレクト束縛と finalize_scene スタブ契約を固定する
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 先行スイートのモジュール個別リロードによる参照分裂を避けるため、
-- pasta と依存モジュールを整合した同一インスタンス集合として一括新規ロードする。
-- pasta を最初に require することで、後続の require が pasta の捕捉した
-- 同一インスタンスを返すことを保証する。
package.loaded["pasta"] = nil
package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil
package.loaded["pasta.scene"] = nil
package.loaded["pasta.global"] = nil
package.loaded["pasta.actor"] = nil

local PASTA = require("pasta")
local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local WORD = require("pasta.word")
local GLOBAL = require("pasta.global")

describe("PASTA - 公開 API リダイレクト", function()
    test("create_actor は ACTOR.get_or_create と同一関数", function()
        expect(PASTA.create_actor):toBe(ACTOR.get_or_create)
    end)

    test("create_scene は SCENE.create_scene と同一関数", function()
        expect(PASTA.create_scene):toBe(SCENE.create_scene)
    end)

    test("create_word は WORD.create_word と同一関数", function()
        expect(PASTA.create_word):toBe(WORD.create_word)
    end)

    test("finalize_scene はスタブとしてエラーなく呼び出せ nil を返す", function()
        -- Rust 側 register_finalize_scene() による上書き前のデフォルト契約
        local ok, result = pcall(PASTA.finalize_scene)
        expect(ok):toBe(true)
        expect(result):toBeNil()
    end)
end)

describe("GLOBAL - チェイントークエイリアス", function()
    test("GLOBAL.チェイントーク は GLOBAL.yield と同一関数", function()
        expect(GLOBAL["チェイントーク"]):toBe(GLOBAL.yield)
    end)
end)
