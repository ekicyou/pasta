--- @module pasta.shiori.event.callback
--- コールバック登録・ルーティング・タイムアウト sweep・ユニーク ID 生成モジュール
---
--- コールバック関連の状態とロジックを集約し、EVENT.fire への変更を局所化する。
--- SHIORI_ACT.get_property 等のコンシューマが stage_pending → consume_staged パターンで
--- コールバック待ちコルーチンを登録し、try_route で到着イベントとマッチングする。
---
--- 循環参照回避: このモジュールは pasta.shiori.act を require しない。

local STORE = require("pasta.store")

local CALLBACK = {}

-- Module-local state
local _next_id = 0
local _staged = nil

--- コールバック待ちコルーチンのレジストリ
--- @type table<string, {co: thread, act: table, timeout_at: number, on_timeout: string|nil}>
CALLBACK.pending = {}

--- ユニークなコールバックイベント ID を生成
--- @return string event_id "OnPastaCallBack{N}" 形式
function CALLBACK.next_event_id()
    _next_id = _next_id + 1
    return "OnPastaCallBack" .. _next_id
end

--- コールバック登録意図をステージング（yield 直前に呼び出す）
--- 単一スロット。consume_staged で消費されるまで上書き不可（多重ステージング検出）
--- @param event_id string ユニークイベント ID
--- @param timeout_at number タイムアウト絶対時刻（os.time() ベース）
--- @param on_timeout string|nil タイムアウト時のエラー理由文字列（nil で静かに消える）
function CALLBACK.stage_pending(event_id, timeout_at, on_timeout)
    if _staged ~= nil then
        error("CALLBACK: multiple staging detected (previous: " .. _staged.event_id .. ", new: " .. event_id .. ")")
    end
    _staged = {
        event_id = event_id,
        timeout_at = timeout_at,
        on_timeout = on_timeout,
    }
end

--- ステージング状態を消費し、resume されたコルーチンをペンディングテーブルに登録
--- EVENT.fire が resume 直後に呼び出す
--- @param co thread resume されたコルーチン
--- @param act table コルーチンに紐づく act オブジェクト
--- @return boolean staged_consumed true: コールバック待ちとして登録, false: ステージングなし（通常チェーントーク）
function CALLBACK.consume_staged(co, act)
    if _staged == nil then
        return false
    end
    local staged = _staged
    _staged = nil
    CALLBACK.pending[staged.event_id] = {
        co = co,
        act = act,
        timeout_at = staged.timeout_at,
        on_timeout = staged.on_timeout,
    }
    STORE.co_callback = co
    return true
end

--- 全状態リセット（テスト用）
function CALLBACK.reset()
    _next_id = 0
    _staged = nil
    CALLBACK.pending = {}
    STORE.co_callback = nil
end

return CALLBACK
