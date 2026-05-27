--- @module pasta.shiori.entry
--- テストフィクスチャ: 非同期コールバック統合テスト用エントリーポイント
---
--- 実プロダクションの EVENT.fire パイプラインを使用し、
--- get_property コールバック機構の統合テストを行う。

local EVENT = require("pasta.shiori.event")
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

-- グローバル SHIORI テーブルを初期化
SHIORI = SHIORI or {}

--- xpcall用エラーハンドラ
--- @param err any エラーオブジェクト
--- @return string|nil エラーメッセージの最初の行
local function error_handler(err)
    if type(err) == "string" then
        return err:match("^[^\n]+")
    end
    return nil
end

--- SHIORI load
--- @param hinst integer DLLハンドル
--- @param load_dir string ロードディレクトリパス
--- @return boolean success
function SHIORI.load(hinst, load_dir)
    return true
end

--- SHIORI/3.0 リクエスト処理
--- @param req table パース済み SHIORI リクエストテーブル
--- @return string SHIORI/3.0 レスポンス
function SHIORI.request(req)
    local ok, result = xpcall(function()
        return EVENT.fire(req)
    end, error_handler)

    if ok then
        return result
    else
        return RES.err(result)
    end
end

--- SHIORI unload
function SHIORI.unload()
end

-- ============================================================================
-- テストハンドラ登録
-- ============================================================================

--- Scenario 1: シンプルなプロパティ取得（2ラウンド）
--- Round 1: get_property → get タグ付きレスポンス
--- Round 2: コールバック到着 → プロパティ値を含むレスポンス
REG.OnTestSimple = function(act)
    return coroutine.create(function(act)
        local version = act:get_property("baseware.version")
        act:raw_script("version=" .. tostring(version) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- Scenario 2: トーク蓄積 + プロパティ取得
--- Round 1: プレフィックストーク + get タグ
--- Round 2: コールバック後の最終トーク
REG.OnTestAccumulate = function(act)
    return coroutine.create(function(act)
        act:raw_script("\\p[0]checking...")
        local version = act:get_property("baseware.version")
        act:raw_script("\\p[0]result=" .. tostring(version) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- Scenario 4: コールバック待ち中の無関係イベント
--- OnTestSimple と同じだが、途中で無関係イベントを挟む
REG.OnTestWait = function(act)
    return coroutine.create(function(act)
        local version = act:get_property("baseware.version")
        act:raw_script("waited=" .. tostring(version) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- Scenario 3: チェーントーク → コールバック待ち遷移（3ラウンド）
--- Round 1: 通常チェーントーク yield（\e付き）→ STORE.co_scene に保存
--- Round 2: OnResumeChain で co_scene を resume → get_property → get タグ
--- Round 3: コールバック到着 → プロパティ値を含む最終レスポンス
REG.OnTestChain = function(act)
    return coroutine.create(function(act)
        act:raw_script("起動しました\\e")
        coroutine.yield(act:build()) -- yield chain talk, STORE.co_scene set
        -- Resumed by OnResumeChain
        local ver = act:get_property("baseware.version")
        act:raw_script("ver=" .. tostring(ver) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- Resume chain talk handler
--- STORE.co_scene に保存されたコルーチンを返して resume させる
REG.OnResumeChain = function(act)
    local STORE = require("pasta.store")
    return STORE.co_scene -- returns the stored coroutine to resume it
end

--- 複数プロパティ取得
--- get_property にテーブルを渡して複数値を一度に取得
REG.OnTestMultiProp = function(act)
    return coroutine.create(function(act)
        local w, h = act:get_property({ "width", "height" })
        act:raw_script("w=" .. tostring(w) .. ",h=" .. tostring(h) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- タイムアウトテスト用
--- timeout=0 で即座にタイムアウト可能な状態にする
REG.OnTestTimeout = function(act)
    return coroutine.create(function(act)
        local v = act:get_property("some.prop", 0, "test timeout")
        act:raw_script("val=" .. tostring(v) .. "\\e")
        coroutine.yield(act:build())
    end)
end

--- Scenario: talk()チェーン内 get_property 干渉テスト
--- トランスパイル後パターンの再現:
---   act.さくら:talk("名前は")
---   act.さくら:talk(tostring(act:get_property("currentghost.name")))
---   act.さくら:talk("です。")
--- get_property のトークン退避・復元が act.さくら:talk() チェーンに
--- 干渉しないことを検証する。
REG.OnTestTalkChainProperty = function(act)
    return coroutine.create(function(act)
        -- トランスパイル後コードと同一パターン
        act.さくら:talk("名前は")
        act.さくら:talk(tostring(act:get_property("currentghost.name")))
        act.さくら:talk("です。")
        coroutine.yield(act:build())
    end)
end

--- Scenario: 複数アクター talk()チェーン + get_property 干渉テスト
--- さくら→get_property→うにゅう の切り替えパターン:
---   act.さくら:talk("確認中...")
---   local name = act:get_property("currentghost.name")
---   act.うにゅう:talk("名前は" .. tostring(name) .. "だよ。")
--- get_property 前後でアクター切り替えが正しく動作することを検証。
REG.OnTestMultiActorProperty = function(act)
    return coroutine.create(function(act)
        act.さくら:talk("確認中...")
        local name = act:get_property("currentghost.name")
        act.うにゅう:talk("名前は" .. tostring(name) .. "だよ。")
        coroutine.yield(act:build())
    end)
end

return SHIORI
