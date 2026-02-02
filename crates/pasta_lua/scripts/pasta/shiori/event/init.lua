--- @module pasta.shiori.event
--- イベント振り分けモジュール
---
--- SHIORI リクエストのイベント ID に応じてハンドラを呼び分ける。
--- 未登録イベントはデフォルトハンドラ（no_entry）で処理する。
--- no_entry ではシーン関数フォールバックを試み、見つからなければ 204 を返す。
--- エラーは呼び出し元（SHIORI.request）の xpcall でキャッチされる。
---
--- ハンドラシグネチャ:
---   function(act: ShioriAct) -> string
---
--- act オブジェクト経由で SHIORI リクエスト情報にアクセス:
---   - act.req.id: イベント名（例: "OnBoot", "OnClose"）
---   - act.req.method: "get" | "notify"
---   - act.req.version: 30（SHIORI/3.0）
---   - act.req.charset: 文字セット（例: "UTF-8"）
---   - act.req.sender: 送信者名（例: "SSP"）
---   - act.req.reference: 参照テーブル（reference[0], reference[1], ...）
---   - act.req.dic: 全ヘッダー辞書
---   - act.req.date: 日付情報（date.unix, date.hour, date.min, date.sec, etc.）
---
--- 注意: act.req は読み取り専用として扱うこと。変更は未定義動作となる。
---
--- Rust側統合パターン（main.lua）:
--- ```lua
--- local EVENT = require("pasta.shiori.event")
---
--- function SHIORI.request(req)
---     return EVENT.fire(req)
--- end
--- ```
---
--- 使用例（ハンドラ登録）:
--- ```lua
--- local REG = require("pasta.shiori.event.register")
--- local RES = require("pasta.shiori.res")
---
--- REG.OnBoot = function(act)
---     act.sakura:talk("こんにちは")
---     return RES.ok(act:build())
--- end
--- ```
---
--- シーン関数フォールバック（alpha01）:
--- REG にハンドラが未登録の場合、SCENE.search(req.id) でグローバルシーンを検索。
--- 見つかった場合はシーン関数を実行（alpha01 では戻り値無視、204 返却）。
--- alpha03 で act オブジェクト生成・さくらスクリプト変換を統合予定。
---
--- テスト用reqテーブル:
--- ```lua
--- local test_req = {
---     id = "OnTest",
---     method = "get",
---     version = 30,
--- }
--- ```

-- 1. require文
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local SHIORI_ACT = require("pasta.shiori.act")
local STORE = require("pasta.store")

-- 1.5. デフォルトイベントハンドラをロード
require("pasta.shiori.event.boot")
require("pasta.shiori.event.second_change")

-- 2. モジュールテーブル宣言
--- @class EVENT
local EVENT = {}

-- 3. 内部関数

--- actオブジェクトを作成
--- @param req table SHIORIリクエストテーブル
--- @return ShioriAct actオブジェクト
local function create_act(req)
    return SHIORI_ACT.new(STORE.actors, req)
end

-- 4. 公開関数

--- デフォルトハンドラ（未登録イベント用）
--- シーン関数フォールバックを試み、見つからなければ 204 No Content を返す。
--- @param act ShioriAct actオブジェクト
--- @return string SHIORI レスポンス（204 No Content または 500 Error）
function EVENT.no_entry(act)
    -- シーン関数をイベント名で検索（遅延ロードで循環参照回避）
    local SCENE = require("pasta.scene")
    local scene_result = SCENE.search(act.req.id, nil, nil)

    if scene_result then
        -- シーン関数が見つかった場合、直接実行
        -- エラーは SHIORI.request の xpcall でキャッチされる
        -- alpha01: 戻り値は無視、204 No Content を返す
        -- alpha03: act オブジェクト生成、さくらスクリプト変換を統合予定
        scene_result()
    end

    return RES.no_content()
end

--- イベント振り分け
--- @param req table リクエストテーブル（req.id にイベント名）
--- @return string SHIORI レスポンス
function EVENT.fire(req)
    -- act オブジェクトを作成
    local act = create_act(req)

    local handler = REG[req.id] or EVENT.no_entry

    -- ハンドラを直接呼び出し
    -- エラーは SHIORI.request の xpcall でキャッチされる
    return handler(act)
end

-- 5. 末尾で返却
return EVENT
