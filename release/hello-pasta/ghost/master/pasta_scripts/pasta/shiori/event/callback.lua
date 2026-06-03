--- @module pasta.shiori.event.callback
--- コールバック登録・ルーティング・タイムアウト sweep・ユニーク ID 生成モジュール
---
--- コールバック関連の状態とロジックを集約し、EVENT.fire への変更を局所化する。
--- SHIORI_ACT.get_property 等のコンシューマが stage_pending → consume_staged パターンで
--- コールバック待ちコルーチンを登録し、try_route で到着イベントとマッチングする。
---
--- 循環参照回避: このモジュールは pasta.shiori.act を require しない。

local STORE = require("pasta.store")
local log = require("@pasta_log")
local RES = require("pasta.shiori.res")

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

--- 到着イベントが pending と一致するなら該当コルーチンを resume してレスポンスを返す
--- @param req table SHIORI リクエスト
--- @return string|nil response 一致時は SHIORI レスポンス文字列、不一致は nil
function CALLBACK.try_route(req)
    local entry = CALLBACK.pending[req.id]
    if entry == nil then
        return nil
    end

    -- Delete entry before resume (coroutine may re-stage)
    CALLBACK.pending[req.id] = nil

    -- Convert 0-indexed reference to 1-based array
    local refs = {}
    local i = 0
    while req.reference[i] ~= nil do
        refs[i + 1] = req.reference[i]
        i = i + 1
    end

    -- Resume coroutine with references
    local ok, yielded = coroutine.resume(entry.co, refs)
    if not ok then
        error(yielded)
    end

    -- R4: callback chaining — coroutine may have called stage_pending again
    if coroutine.status(entry.co) == "suspended" then
        CALLBACK.consume_staged(entry.co, entry.act)
    end

    return RES.ok(yielded)
end

--- タイムアウト時刻超過エントリを掃引
--- @param now number 現在時刻（os.time() 戻り値）
--- @return string|nil response 最初の on_timeout=string エントリの 500 レスポンス、なければ nil
function CALLBACK.sweep(now)
    local timeout_response = nil
    local to_remove = {}
    for event_id, entry in pairs(CALLBACK.pending) do
        if now > entry.timeout_at then
            to_remove[#to_remove + 1] = event_id
            if type(entry.on_timeout) == "string" then
                coroutine.resume(entry.co, nil, entry.on_timeout)
                log.warn(event_id .. ": " .. entry.on_timeout)
                if timeout_response == nil then
                    timeout_response = RES.build("500 Internal Server Error", {
                        ["X-ERROR-REASON"] = entry.on_timeout,
                    })
                end
            else
                coroutine.resume(entry.co, nil)
            end
        end
    end
    for _, event_id in ipairs(to_remove) do
        CALLBACK.pending[event_id] = nil
    end
    return timeout_response
end

--- 全状態リセット（テスト用）
function CALLBACK.reset()
    _next_id = 0
    _staged = nil
    CALLBACK.pending = {}
    STORE.co_callback = nil
end

return CALLBACK
