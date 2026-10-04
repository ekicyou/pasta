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

--- 消費されていない予約を捨てる（予約が無ければ何もしない）
--- EVENT.drive の失敗の経路だけが呼ぶ。pending・STORE には触れない
function CALLBACK.discard_staged()
    _staged = nil
end

--- 到着イベントが pending と一致するなら該当コルーチンを EVENT.drive で再開してレスポンスを返す
--- シーンのエラーは error で伝える
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
    -- ハードニング (3.49 G3): Rust 境界（lua_request.rs）は reference を常時生成するが、
    -- Lua 側から直接呼ばれる経路で reference が欠落しても、pending 削除済みの
    -- 待機コルーチンを孤児化（nil 添字エラー）させず空 refs で継続する
    local reference = req.reference or {}
    local refs = {}
    local i = 0
    while reference[i] ~= nil do
        refs[i + 1] = reference[i]
        i = i + 1
    end

    -- 通常のイベントと同じ再開の手順（予約の消費・継続の更新を含む）で再開する
    -- EVENT は呼び出し時に読み込む（読み込み時に require すると循環する）
    local EVENT = require("pasta.shiori.event")
    local ok, value = EVENT.drive(entry.co, entry.act, refs)
    if not ok then
        error(value)
    end

    return RES.ok(value)
end

--- イベント名の番号（OnPastaCallBack{N} の N）。番号を持たない名前は nil
--- @param event_id string
--- @return number|nil
local function callback_number(event_id)
    return tonumber(event_id:match("^OnPastaCallBack(%d+)$"))
end

--- 番号の小さい順。番号を持たない名前は末尾に置き、名前の文字列順にする
local function event_id_less(a, b)
    local na, nb = callback_number(a), callback_number(b)
    if na and nb then
        return na < nb
    end
    if na or nb then
        return na ~= nil
    end
    return a < b
end

--- 期限切れの待機を集めてから pending から外し、番号順に並べる
--- 再開より前に外すため、再開中に登録された待機は同じ回では扱われない
--- @param now number 現在時刻
--- @return string[] event_ids 並べたイベント名
--- @return table<string, table> entries イベント名 → 外した待機
local function take_expired(now)
    local event_ids = {}
    for event_id, entry in pairs(CALLBACK.pending) do
        if now > entry.timeout_at then
            event_ids[#event_ids + 1] = event_id
        end
    end
    local entries = {}
    for _, event_id in ipairs(event_ids) do
        entries[event_id] = CALLBACK.pending[event_id]
        CALLBACK.pending[event_id] = nil
    end
    table.sort(event_ids, event_id_less)
    return event_ids, entries
end

--- 待機 1 つをタイムアウトとして EVENT.drive で再開し、応答の候補を返す
--- @param event_id string
--- @param entry table pending から外した待機
--- @return string|nil candidate 出力なら 200、理由付きのエラーなら 500、それ以外は nil
local function time_out(event_id, entry)
    local reason = nil
    if type(entry.on_timeout) == "string" then
        reason = entry.on_timeout
        log.warn(event_id .. ": " .. reason)
    end
    -- EVENT は呼び出し時に読み込む（読み込み時に require すると循環する）
    local EVENT = require("pasta.shiori.event")
    local ok, value = EVENT.drive(entry.co, entry.act, nil, reason)
    if not ok then
        if reason then
            -- 理由はシーン内のエラー文字列ではなく timeout_message そのものを使う
            return RES.err(reason)
        end
        -- 静かなタイムアウトのエラーは応答を作らず警告ログだけにする
        log.warn(event_id .. ": scene error after silent timeout: " .. tostring(value))
        return nil
    end
    if value ~= nil and value ~= "" then
        return RES.ok(value)
    end
    return nil
end

--- 期限を過ぎた待機をタイムアウトとして再開し、その回の応答を 1 つ作る
--- 集める → 外す → 番号順に並べる → 順に EVENT.drive で再開する。
--- 最初の候補を応答に採用し、2 番目以降は警告ログを出して捨てる。途中のエラーでも残りを続ける。
--- @param now number 現在時刻（os.time() 戻り値）
--- @return string|nil response その回の応答（200 または 500 の全文）、応答が生まれなければ nil
function CALLBACK.sweep(now)
    local response = nil
    local event_ids, entries = take_expired(now)
    for _, event_id in ipairs(event_ids) do
        local candidate = time_out(event_id, entries[event_id])
        if candidate then
            if response == nil then
                response = candidate
            else
                log.warn(event_id .. ": timeout response discarded (an earlier response was adopted)")
            end
        end
    end
    return response
end

--- 全状態リセット（テスト用）
function CALLBACK.reset()
    _next_id = 0
    _staged = nil
    CALLBACK.pending = {}
    STORE.co_callback = nil
end

return CALLBACK
