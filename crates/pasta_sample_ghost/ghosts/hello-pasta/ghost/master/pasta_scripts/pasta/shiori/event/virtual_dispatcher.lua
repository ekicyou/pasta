---@module pasta.shiori.event.virtual_dispatcher
--- 仮想イベント（OnTalk/OnHour）の条件判定・発行モジュール
---
--- OnSecondChange をトリガーとして、以下の仮想イベントを発行:
--- - OnHour: 正時に発行（優先）
--- - OnTalk: 一定時間経過後にランダム発行
---
--- セッション定義: SHIORI load〜unload間。unloadでランタイムドロップにより全状態がリセットされる。
---
--- 使用例:
--- ```lua
--- local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
--- local result = dispatcher.dispatch(act)
--- ```

-- 1. require文（遅延ロードで循環参照回避）

-- 2. モジュールローカル変数（SHIORIセッション中のみ有効）
local next_hour_unix = 0   -- 次の正時タイムスタンプ
local next_talk_time = 0   -- 次回トーク発行予定時刻
local scene_executor = nil -- テスト用: シーン実行関数のオーバーライド

-- 3. モジュールテーブル宣言
---@class VirtualDispatcher
local M = {}

-- 4. 内部関数

-- ブロック対象SSP Statusキーワード一覧
local BLOCKED_STATUSES = {
    "talking",
    "choosing",
    "online",
    "opening",
    "passive",
    "induction",
    "timecritical",
    "nouserbreak",
    "minimizing",
}

--- 設定を読み込む（SAVE > toml > hardcoded の3段フォールバック）
--- キャッシュなし：毎回 SAVE テーブルを読み直すことで実行時変更を即時反映
---@return table 設定テーブル
local function get_config()
    local save = require("pasta.save")

    local ok, config = pcall(require, "@pasta_config")
    if not ok then config = {} end
    local ghost = config.ghost or {}

    --- SAVE > toml > default の優先順位で単一キーを解決
    --- 数値以外・NaN・Inf は無視してフォールバック。小数は floor、10未満は10にクランプ。
    ---@param save_key string SAVEテーブルのキー名（pasta_ プレフィックス付き）
    ---@param toml_key string pasta.toml [ghost] セクションのキー名
    ---@param default number ハードコードデフォルト値
    ---@return number
    local function resolve(save_key, toml_key, default)
        local sv = save[save_key]
        if type(sv) == "number" then
            sv = math.floor(sv)
            if sv < math.huge then return math.max(10, sv) end
        end
        local tv = ghost[toml_key]
        if type(tv) == "number" then
            tv = math.floor(tv)
            if tv < math.huge then return math.max(10, tv) end
        end
        return default
    end

    local min = resolve("pasta_talk_interval_min", "talk_interval_min", 180)
    local max = resolve("pasta_talk_interval_max", "talk_interval_max", 300)
    if min > max then max = min end

    return {
        talk_interval_min = min,
        talk_interval_max = max,
        hour_margin = ghost.hour_margin or 30,
    }
end

--- 次の正時タイムスタンプを計算
---@param current_unix number 現在のUnixタイムスタンプ
---@return number 次の正時のUnixタイムスタンプ
local function calculate_next_hour_unix(current_unix)
    local seconds_into_hour = current_unix % 3600
    return current_unix - seconds_into_hour + 3600
end

--- 次回トーク時刻を計算
---@param current_unix number 現在のUnixタイムスタンプ
---@return number 次回トーク予定時刻
local function calculate_next_talk_time(current_unix)
    local cfg = get_config()
    local interval = math.random(cfg.talk_interval_min, cfg.talk_interval_max)
    return current_unix + interval
end

--- Status 文字列にキーワードが含まれるかを判定する
---@param status string|nil Status ヘッダー値（カンマ区切り複合文字列）
---@param keyword string 検索キーワード（例: "talking", "choosing"）
---@return boolean キーワードが含まれていれば true
local function has_status(status, keyword)
    if not status then return false end
    return status:find(keyword, 1, true) ~= nil
end

--- シーン関数からthreadを生成（実行しない）
---@param event_name string イベント名 ("OnTalk" or "OnHour")
---@param act ShioriAct actオブジェクト（未使用、将来拡張用）
---@return thread|nil コルーチンまたはnil
local function create_scene_thread(event_name, act)
    -- テスト用オーバーライドがあれば使用
    if scene_executor then
        return scene_executor(event_name, act)
    end

    local SCENE = require("pasta.scene")
    return SCENE.co_exec(act, event_name, nil, nil)
end

-- 5. 公開関数

--- Status文字列にブロック対象キーワードが含まれるか判定
---@param status string|nil act.req.status値
---@return boolean true=発行ブロック, false=発行許可
function M.is_blocked(status)
    for _, keyword in ipairs(BLOCKED_STATUSES) do
        if has_status(status, keyword) then
            return true
        end
    end
    return false
end

--- OnHour 判定・発行（thread返却形式）
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return thread|nil コルーチンまたはnil
function M.check_hour(act)
    local current_unix = act.req.date.unix

    -- 初回起動: 次の正時を計算して設定、発行スキップ
    if next_hour_unix == 0 then
        next_hour_unix = calculate_next_hour_unix(current_unix)
        return nil
    end

    -- 正時未到達
    if current_unix < next_hour_unix then
        return nil
    end

    -- 次の正時を更新
    next_hour_unix = calculate_next_hour_unix(current_unix)

    -- 日時変数を転記（OnHour発火時のみ）
    -- エラーは SHIORI.request の xpcall でキャッチされる
    if act.transfer_date_to_var then
        act:transfer_date_to_var()
    end

    -- 4段階フォールバックチェーンでシーンを解決
    local hh = string.format("%02d", act.req.date.hour)
    local candidates = { "時報" .. hh, "OnHour" .. hh, "時報その他", "OnHourOther" }
    for _, name in ipairs(candidates) do
        local t = create_scene_thread(name, act)
        if t then return t end
    end
    return nil
end

--- OnTalk 判定・発行（thread返却形式、チェイントーク対応）
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return thread|nil コルーチンまたはnil
function M.check_talk(act)
    local current_unix = act.req.date.unix
    local cfg = get_config()

    -- 初回または次回トーク時刻未設定
    if next_talk_time == 0 then
        next_talk_time = calculate_next_talk_time(current_unix)
        return nil
    end

    -- 次回トーク時刻未到達
    if current_unix < next_talk_time then
        return nil
    end

    -- 時報マージンチェック（正時が近い場合はスキップ）
    local time_to_hour = next_hour_unix - current_unix
    if time_to_hour > 0 and time_to_hour < cfg.hour_margin then
        return nil
    end

    -- 次回トーク時刻を再計算（発行成否に関わらず）
    next_talk_time = calculate_next_talk_time(current_unix)

    -- チェイントーク継続確認（OnTalk発動タイミングで判定）
    local STORE = require("pasta.store")
    if STORE.co_scene then
        -- 継続用コルーチンを返す（新規シーン検索スキップ）
        return STORE.co_scene
    end

    -- 新規シーンのthreadを返す（実行しない）
    return create_scene_thread("OnTalk", act)
end

--- 仮想イベントディスパッチ（メインエントリポイント、thread返却形式）
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return thread|nil コルーチンまたはnil
function M.dispatch(act)
    -- act.req.date 存在チェック
    if not act.req or not act.req.date then
        return nil
    end

    -- Statusブロックガード（dispatch入口で一括判定）
    if M.is_blocked(act.req.status) then
        return nil
    end

    -- OnHour 判定（優先）
    local hour_result = M.check_hour(act)
    if hour_result then
        return hour_result
    end

    -- OnTalk 判定
    return M.check_talk(act)
end

-- 6. テスト用関数（内部状態リセット）

--- テスト用: 内部状態をリセット
function M._reset()
    next_hour_unix = 0
    next_talk_time = 0
    scene_executor = nil
end

--- テスト用: 内部状態を取得
---@return table 内部状態
function M._get_internal_state()
    return {
        next_hour_unix = next_hour_unix,
        next_talk_time = next_talk_time,
    }
end

--- テスト用: シーン実行関数をモックで差し替え
---@param executor function|nil シーン実行関数 (event_name, act) -> result
function M._set_scene_executor(executor)
    scene_executor = executor
end

--- テスト用: 設定解決結果を直接取得（get_config の公開参照）
---@return table 設定テーブル
M._get_config = get_config

return M
