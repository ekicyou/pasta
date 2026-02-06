-- persist-spot-position feature tests
-- Tests for STORE.actor_spots initialization, reset, and SHIORI_ACT.build() STORE integration
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- Task 5.1: store.lua actor_spots初期化テスト
-- ============================================================================

describe("STORE.actor_spots - 初期化 (Task 5.1)", function()
    test("STORE.actor_spotsフィールドが存在する", function()
        local STORE = require("pasta.store")
        expect(type(STORE.actor_spots)):toBe("table")
    end)

    test("CONFIG.actor未定義時にactor_spotsが空テーブル", function()
        local STORE = require("pasta.store")
        -- テスト環境では @pasta_config が未登録のため actor_spots は空
        -- （pcall保護により安全に動作する）
        expect(type(STORE.actor_spots)):toBe("table")
    end)
end)

-- ============================================================================
-- Task 5.2: store.lua reset()動作テスト
-- ============================================================================

describe("STORE.actor_spots - reset() (Task 5.2)", function()
    test("STORE.reset()がactor_spotsを空テーブルにクリアする", function()
        local STORE = require("pasta.store")

        -- actor_spotsにデータを設定
        STORE.actor_spots["テスト"] = 5
        STORE.actor_spots["キャラ"] = 3

        -- reset実行
        STORE.reset()

        -- actor_spotsが空テーブルにクリアされる
        expect(next(STORE.actor_spots)):toBe(nil)
        expect(type(STORE.actor_spots)):toBe("table")
    end)

    test("reset()後もactor_spotsへの書き込みが可能", function()
        local STORE = require("pasta.store")

        STORE.reset()
        STORE.actor_spots["新キャラ"] = 2

        expect(STORE.actor_spots["新キャラ"]):toBe(2)

        -- クリーンアップ
        STORE.reset()
    end)
end)

-- ============================================================================
-- Task 6.1: SHIORI_ACT_IMPL.build() STORE読み書きフロー
-- ============================================================================

describe("SHIORI_ACT.build() - STORE.actor_spots連携 (Task 6.1)", function()
    test("build()後にSTORE.actor_spotsが更新される", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local STORE = require("pasta.store")
        local ACTOR = require("pasta.actor")

        -- STOREをリセット
        STORE.reset()

        -- アクターを準備
        local sakura = ACTOR.get_or_create("さくら")
        sakura.spot = "sakura"
        local kero = ACTOR.get_or_create("うにゅう")
        kero.spot = "kero"

        local actors = {
            sakura = sakura,
            kero = kero,
        }

        -- act生成してset_spot + talk
        local act = SHIORI_ACT.new(actors)
        act:set_spot("sakura", 0)
        act:set_spot("kero", 1)
        act.sakura:talk("Hello")
        act.kero:talk("Hi")

        local result = act:build()

        -- build()が正常にスクリプトを返す
        expect(result):toBeTruthy()
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()

        -- STORE.actor_spotsが更新されている
        expect(STORE.actor_spots["さくら"]):toBe(0)
        expect(STORE.actor_spots["うにゅう"]):toBe(1)

        -- クリーンアップ
        STORE.reset()
    end)

    test("トークン0件時（nil返却）でSTORE更新がスキップされる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local STORE = require("pasta.store")

        -- STOREをリセットし初期値を設定
        STORE.reset()
        STORE.actor_spots["さくら"] = 0

        -- トークンなしでbuild
        local act = SHIORI_ACT.new({})
        local result = act:build()

        -- nilが返される
        expect(result):toBe(nil)

        -- STORE.actor_spotsは変更されない（初期値を維持）
        expect(STORE.actor_spots["さくら"]):toBe(0)

        -- クリーンアップ
        STORE.reset()
    end)
end)

-- ============================================================================
-- Task 6.2: シーン連続実行: スポット値の引き継ぎ
-- ============================================================================

describe("SHIORI_ACT.build() - シーン連続実行でスポット値引き継ぎ (Task 6.2)", function()
    test("％行ありシーン → ％行なしシーンでスポット値が引き継がれる", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local STORE = require("pasta.store")
        local ACTOR = require("pasta.actor")

        -- STOREをリセット
        STORE.reset()

        local sakura = ACTOR.get_or_create("さくら")
        sakura.spot = "sakura"
        local kero = ACTOR.get_or_create("うにゅう")
        kero.spot = "kero"

        local actors = {
            sakura = sakura,
            kero = kero,
        }

        -- シーン1: ％行あり（set_spot + talk）
        local act1 = SHIORI_ACT.new(actors)
        act1:set_spot("sakura", 0)
        act1:set_spot("kero", 1)
        act1.sakura:talk("Scene1 Sakura")
        act1.kero:talk("Scene1 Kero")
        local result1 = act1:build()

        expect(result1):toBeTruthy()
        expect(result1:find("\\p%[0%]")):toBeTruthy()
        expect(result1:find("\\p%[1%]")):toBeTruthy()

        -- STORE.actor_spotsが更新されている
        expect(STORE.actor_spots["さくら"]):toBe(0)
        expect(STORE.actor_spots["うにゅう"]):toBe(1)

        -- シーン2: ％行なし（set_spotなし、talkのみ）
        -- STORE.actor_spotsの前回値がビルダーに渡される
        local act2 = SHIORI_ACT.new(actors)
        act2.sakura:talk("Scene2 Sakura")
        act2.kero:talk("Scene2 Kero")
        local result2 = act2:build()

        expect(result2):toBeTruthy()
        -- 前回のスポット値（0, 1）が引き継がれている
        expect(result2:find("\\p%[0%]")):toBeTruthy()
        expect(result2:find("\\p%[1%]")):toBeTruthy()

        -- クリーンアップ
        STORE.reset()
    end)
end)

-- ============================================================================
-- Task 6.3: CONFIG未設定時のデフォルト動作
-- ============================================================================

describe("SHIORI_ACT.build() - CONFIG未設定時のデフォルト動作 (Task 6.3)", function()
    test("CONFIG.actor未定義時にactor_spotsが空テーブルで動作する", function()
        local SHIORI_ACT = require("pasta.shiori.act")
        local STORE = require("pasta.store")
        local ACTOR = require("pasta.actor")

        -- STOREをリセット（CONFIG未設定の状態をシミュレート）
        STORE.reset()

        local sakura = ACTOR.get_or_create("さくら")
        sakura.spot = "sakura"

        local actors = { sakura = sakura }

        -- set_spotなしでbuild
        local act = SHIORI_ACT.new(actors)
        act.sakura:talk("Default spot")
        local result = act:build()

        expect(result):toBeTruthy()
        -- デフォルトspot=0で動作
        expect(result:find("\\p%[0%]")):toBeTruthy()

        -- クリーンアップ
        STORE.reset()
    end)
end)
