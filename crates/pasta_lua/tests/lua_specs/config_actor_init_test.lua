-- pasta.store / pasta.actor の CONFIG.actor 由来初期化テスト
-- review-improvement-loop cell 3.46 (G1): モジュールロード時に 1 度だけ実行される
-- CONFIG.actor 初期化パス（store.lua の spot 転送・actor.lua のメタテーブル付与と
-- name 補完）が未カバーだった（既存 persist_spot_position_test は CONFIG 未定義時のみ）。
--
-- @pasta_config を stub に差し替えて pasta.store / pasta.actor を新規ロードし、
-- 検証後に package.loaded を必ず復元する（他スイートのモジュール参照を壊さない）。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

--- @pasta_config stub のもとで pasta.store（必要なら pasta.actor も）を新規ロードして
--- body を実行し、終了後に package.loaded を復元する。
--- @param config_stub table @pasta_config の代替テーブル
--- @param load_actor boolean pasta.actor も新規ロードするか
--- @param body fun(fresh_store: table)
local function with_fresh_modules(config_stub, load_actor, body)
    local saved = {
        store = package.loaded["pasta.store"],
        actor = package.loaded["pasta.actor"],
        config = package.loaded["@pasta_config"],
    }
    package.loaded["pasta.store"] = nil
    package.loaded["@pasta_config"] = config_stub
    if load_actor then
        package.loaded["pasta.actor"] = nil
    end

    local ok, err = pcall(function()
        local fresh_store = require("pasta.store")
        if load_actor then
            require("pasta.actor")
        end
        body(fresh_store)
    end)

    package.loaded["pasta.store"] = saved.store
    package.loaded["pasta.actor"] = saved.actor
    package.loaded["@pasta_config"] = saved.config
    if not ok then error(err, 0) end
end

describe("pasta.store - CONFIG.actor 初期化", function()
    test("CONFIG.actor が STORE.actors として採用される", function()
        local config = { actor = { さくら = { spot = 0 } } }
        with_fresh_modules(config, false, function(store)
            expect(store.actors):toBe(config.actor)
            expect(store.actors["さくら"]):toBe(config.actor["さくら"])
        end)
    end)

    test("数値 spot のみ actor_spots へ転送される（欠落・非数値・非テーブルは転送なし）", function()
        local config = {
            actor = {
                さくら = { spot = 0 },
                うにゅう = { spot = 2 },
                無指定 = {},
                文字spot = { spot = "left" },
                非テーブル = "ただの文字列",
            },
        }
        with_fresh_modules(config, false, function(store)
            expect(store.actor_spots["さくら"]):toBe(0)
            expect(store.actor_spots["うにゅう"]):toBe(2)
            expect(store.actor_spots["無指定"]):toBeNil()
            expect(store.actor_spots["文字spot"]):toBeNil()
            expect(store.actor_spots["非テーブル"]):toBeNil()
        end)
    end)
end)

describe("pasta.actor - CONFIG 由来アクターのメタテーブル設定", function()
    test("name 欠落アクターは辞書キーで補完され、ACTOR_IMPL メソッドが利用可能になる", function()
        local config = { actor = { さくら = { spot = 0 } } }
        with_fresh_modules(config, true, function(store)
            local actor = store.actors["さくら"]
            expect(actor.name):toBe("さくら")
            -- メタテーブル経由で create_word メソッドへ到達できる
            expect(type(actor.create_word)):toBe("function")
        end)
    end)

    test("name 既設アクターは上書きされない・非テーブル要素はスキップされる", function()
        local config = {
            actor = {
                別名持ち = { name = "カスタム名" },
                非テーブル = "文字列要素",
            },
        }
        with_fresh_modules(config, true, function(store)
            expect(store.actors["別名持ち"].name):toBe("カスタム名")
            -- 非テーブル要素はメタテーブル設定対象外（エラーなくそのまま保持）
            expect(store.actors["非テーブル"]):toBe("文字列要素")
        end)
    end)
end)
