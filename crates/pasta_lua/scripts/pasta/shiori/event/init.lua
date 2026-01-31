--- @module pasta.shiori.event
--- イベント振り分けモジュール
---
--- SHIORI リクエストのイベント ID に応じてハンドラを呼び分ける。
--- 未登録イベントはデフォルトハンドラ（no_entry）で処理する。
--- no_entry ではシーン関数フォールバックを試み、見つからなければ 204 を返す。
--- エラー発生時は xpcall/pcall でキャッチし、エラーレスポンスに変換する。
---
--- ハンドラシグネチャ:
---   function(req: table) -> string
---
--- reqテーブル構造（Rust側 parse_request() により生成）:
---   - req.id: イベント名（例: "OnBoot", "OnClose"）
---   - req.method: "get" | "notify"
---   - req.version: 30（SHIORI/3.0）
---   - req.charset: 文字セット（例: "UTF-8"）
---   - req.sender: 送信者名（例: "SSP"）
---   - req.reference: 参照テーブル（reference[0], reference[1], ...）
---   - req.dic: 全ヘッダー辞書
---
--- 注意: reqテーブルはread-only契約。ハンドラ内で変更しないこと。
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
--- REG.OnBoot = function(req)
---     return RES.ok([[\0\s[0]こんにちは\e]])
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

-- 1.5. デフォルトイベントハンドラをロード
require("pasta.shiori.event.boot")
require("pasta.shiori.event.second_change")

-- 2. モジュールテーブル宣言
--- @class EVENT
local EVENT = {}

-- 3. 公開関数

--- デフォルトハンドラ（未登録イベント用）
--- シーン関数フォールバックを試み、見つからなければ 204 No Content を返す。
--- @param req table リクエストテーブル
--- @return string SHIORI レスポンス（204 No Content または 500 Error）
function EVENT.no_entry(req)
    -- シーン関数をイベント名で検索（遅延ロードで循環参照回避）
    local SCENE = require("pasta.scene")
    local scene_result = SCENE.search(req.id, nil, nil)

    if scene_result then
        -- シーン関数が見つかった場合、pcall で実行
        -- alpha01: 戻り値は無視、204 No Content を返す
        -- alpha03: act オブジェクト生成、さくらスクリプト変換を統合予定
        local ok, err = pcall(function()
            return scene_result()
        end)
        if not ok then
            -- シーン関数実行時のエラーは 500 で返す
            local err_msg = err
            if type(err) == "string" then
                err_msg = err:match("^[^\n]+") or err
            end
            return RES.err(err_msg)
        end
    end

    return RES.no_content()
end

--- イベント振り分け
--- @param req table リクエストテーブル（req.id にイベント名）
--- @return string SHIORI レスポンス
function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry

    local ok, result = xpcall(function()
        return handler(req)
    end, function(err)
        -- エラーメッセージの最初の行のみを抽出（改行除去）
        -- debug.traceback は ALL_SAFE では使用不可のため、エラーメッセージのみ使用
        if type(err) == "string" then
            return err:match("^[^\n]+")
        else
            return nil
        end
    end)

    if ok then
        return result
    else
        return RES.err(result) -- nil は RES.err 内で "Unknown error" にフォールバック
    end
end

-- 4. 末尾で返却
return EVENT
