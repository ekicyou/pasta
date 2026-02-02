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

--- STORE.co_sceneを統一管理するローカル関数
--- @param co thread|nil コルーチンまたはnil
local function set_co_scene(co)
    -- 1. 引数検証（suspended以外はclose）
    if co and coroutine.status(co) ~= "suspended" then
        coroutine.close(co)
        co = nil
    end

    -- 2. 同一オブジェクトチェック
    if STORE.co_scene == co then
        return
    end

    -- 3. 旧コルーチンをclose（存在すれば無条件）
    if STORE.co_scene then
        coroutine.close(STORE.co_scene)
    end

    -- 4. 上書き（coはsuspendedまたはnil確定）
    STORE.co_scene = co
end

-- 4. 公開関数

--- デフォルトハンドラ（未登録イベント用）
--- シーン関数をイベント名で検索し、見つかった場合はthreadを返す。
--- 見つからない場合はnilを返す（EVENT.fireでRES.no_content()に変換される）。
--- @param act ShioriAct actオブジェクト
--- @return thread|nil シーンコルーチン、またはnil
function EVENT.no_entry(act)
    -- シーン関数をイベント名で検索（遅延ロードで循環参照回避）
    local SCENE = require("pasta.scene")
    local scene_result = SCENE.search(act.req.id, nil, nil)

    if scene_result then
        -- SCENE.search()はSceneSearchResultテーブルを返す
        -- .funcフィールドからシーン関数を取得してthreadを生成
        return coroutine.create(scene_result.func)
    end

    -- シーン関数が見つからない場合はnilを返す
    return nil
end

--- イベント振り分け
--- ハンドラを実行し、コルーチンの場合はresumeして状態管理を行う
--- @param req table リクエストテーブル（req.id にイベント名）
--- @return string SHIORI レスポンス
function EVENT.fire(req)
    -- act オブジェクトを作成
    local act = create_act(req)

    local handler = REG[req.id] or EVENT.no_entry

    -- ハンドラを呼び出し
    -- エラーは SHIORI.request の xpcall でキャッチされる
    local result = handler(act)

    -- 型判定
    if type(result) == "thread" then
        -- コルーチン実行
        local ok, yielded_value = coroutine.resume(result, act)
        if not ok then
            -- エラー処理: close & 例外伝搬
            set_co_scene(result) -- closeされる（dead状態のため）
            error(yielded_value)
        end
        -- 状態保存（set_co_scene内部でstatus判断）
        set_co_scene(result)
        return RES.ok(yielded_value)
    elseif type(result) == "string" then
        -- 既存互換: 文字列をそのまま返す
        return RES.ok(result)
    else
        -- nil
        return RES.no_content()
    end
end

-- 5. 末尾で返却
return EVENT
