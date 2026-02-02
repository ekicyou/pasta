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
local cached_config = nil  -- 設定キャッシュ
local scene_executor = nil -- テスト用: シーン実行関数のオーバーライド

-- 3. モジュールテーブル宣言
---@class VirtualDispatcher
local M = {}

-- 4. 内部関数

--- 設定を読み込み・キャッシュ
---@return table 設定テーブル
local function get_config()
    if cached_config then return cached_config end

    local ok, config = pcall(require, "@pasta_config")
    if not ok then config = {} end

    local ghost = config.ghost or {}
    cached_config = {
        talk_interval_min = ghost.talk_interval_min or 180,
        talk_interval_max = ghost.talk_interval_max or 300,
        hour_margin = ghost.hour_margin or 30,
    }
    return cached_config
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

--- シーン関数を実行
---@param event_name string イベント名 ("OnTalk" or "OnHour")
---@param act ShioriAct actオブジェクト
---@return string|nil 実行結果（エラー時は nil）
local function execute_scene(event_name, act)
    -- テスト用オーバーライドがあれば使用
    if scene_executor then
        return scene_executor(event_name, act)
    end

    local SCENE = require("pasta.scene")
    local scene_fn = SCENE.search(event_name, nil, nil)

    if not scene_fn then
        return nil
    end

    -- シーン関数を直接実行
    -- エラーは SHIORI.request の xpcall でキャッチされる
    return scene_fn(act)
end

-- 5. 公開関数

--- OnHour 判定・発行
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return string|nil "fired" (発行成功), nil (発行なし)
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

    -- トーク中はスキップ（SSPからの状態情報を使用）
    if act.req.status == "talking" then
        return nil
    end

    -- 次の正時を更新
    next_hour_unix = calculate_next_hour_unix(current_unix)

    -- 日時変数を転記（OnHour発火時のみ）
    -- エラーは SHIORI.request の xpcall でキャッチされる
    if act.transfer_date_to_var then
        act:transfer_date_to_var()
    end

    -- シーン実行（actを渡す）
    local result = execute_scene("OnHour", act)

    return result and "fired" or nil
end

--- OnTalk 判定・発行
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return string|nil "fired" (発行成功), nil (発行なし)
function M.check_talk(act)
    local current_unix = act.req.date.unix
    local cfg = get_config()

    -- トーク中はスキップ（SSPからの状態情報を使用）
    if act.req.status == "talking" then
        return nil
    end

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

    -- シーン実行（actを渡す、OnTalkではtransfer_date_to_varを呼び出さない）
    local result = execute_scene("OnTalk", act)

    -- 次回トーク時刻を再計算（発行成否に関わらず）
    next_talk_time = calculate_next_talk_time(current_unix)

    return result and "fired" or nil
end

--- 仮想イベントディスパッチ（メインエントリポイント）
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス）
---@return string|nil シーン実行結果 or nil
function M.dispatch(act)
    -- act.req.date 存在チェック
    if not act.req or not act.req.date then
        return nil
    end

    -- OnHour 判定（優先）
    local hour_result = M.check_hour(act)
    if hour_result then
        return hour_result
    end

    -- OnTalk 判定
    local talk_result = M.check_talk(act)
    return talk_result
end

-- 6. テスト用関数（内部状態リセット）

--- テスト用: 内部状態をリセット
function M._reset()
    next_hour_unix = 0
    next_talk_time = 0
    cached_config = nil
    scene_executor = nil
end

--- テスト用: 内部状態を取得
---@return table 内部状態
function M._get_internal_state()
    return {
        next_hour_unix = next_hour_unix,
        next_talk_time = next_talk_time,
        cached_config = cached_config,
    }
end

--- テスト用: シーン実行関数をモックで差し替え
---@param executor function|nil シーン実行関数 (event_name, act) -> result
function M._set_scene_executor(executor)
    scene_executor = executor
end

return M
