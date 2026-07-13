--- @module pasta.shiori.sakura_builder
--- さくらスクリプトビルダーモジュール
---
--- グループ化されたトークン配列をさくらスクリプト文字列に変換するモジュール。
--- pasta.shiori.act の build() から呼び出される。

local BUILDER = {}

local SAKURA_SCRIPT = require "@pasta_sakura_script"
local log = require "@pasta_log"
local buf = require("pasta.buf")

--- \q[display,target] 内のデリミタ文字をエスケープ
--- @param s string エスケープ対象の文字列
--- @return string エスケープ済み文字列
local function escape_choice(s)
    s = s:gsub("\\", "\\\\")
    s = s:gsub("%]", "\\]")
    s = s:gsub(",", "\\,")
    return s
end

--- スポットタグを生成（SSP ukadoc準拠: 常に\p[ID]形式）
--- @param spot_id number スポットID番号
--- @return string スポットタグ
local function spot_to_tag(spot_id)
    return string.format("\\p[%d]", spot_id)
end

--- actor_spots の全エントリを個別 nil クリアする（テーブル再割り当てを回避）
--- @param actor_spots table<string, integer> アクターごとのスポット位置マップ（直接変更される）
local function clear_spots(actor_spots)
    for name in pairs(actor_spots) do
        actor_spots[name] = nil
    end
end

--- アクター切り替え時のスポットタグ（必要なら段落区切り改行）を出力
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor_spots table<string, integer> アクターごとのスポット位置マップ
--- @param last_spot number|nil 直前のスポットID（nil は未出力）
--- @param actor table 切り替え先アクター（非nil保証は呼び出し元）
--- @param spot_newlines number スポット変更時の改行量
--- @param allow_break boolean 段落区切り改行を許可するか（前回の改行以降に一般文字列が出力済みの場合のみ true）
--- @return number spot 切り替え後のスポットID
--- @return boolean emitted_break 段落区切り改行を出力したか
local function emit_actor_switch(buffer, actor_spots, last_spot, actor, spot_newlines, allow_break)
    local actor_name = actor.name
    local spot = actor_spots[actor_name]
    if spot == nil then
        spot = 0
        if actor_name then
            log.warn(string.format("actor_spots fallback: '%s' -> spot=0", actor_name))
        end
    end

    -- spot変更時に段落区切り改行を出力。
    -- ただし直前の会話が全てさくらスクリプト（一般文字列が1文字も未出力）の場合は
    -- allow_break=false となり、改行を抑制する。
    local emitted_break = false
    if allow_break and last_spot ~= nil and last_spot ~= spot then
        local percent = math.floor(spot_newlines * 100)
        buffer:put(string.format("\\n[%d]", percent))
        emitted_break = true
    end

    buffer:put(spot_to_tag(spot))
    return spot, emitted_break
end

--- actorグループ内の単一トークンをさくらスクリプトへ変換して出力
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor table|nil グループの発言アクター
--- @param inner table グループ内トークン
local function emit_inner_token(buffer, actor, inner)
    local inner_type = inner.type

    if inner_type == "talk" then
        buffer:put(SAKURA_SCRIPT.talk_to_script(actor, inner.text))
    elseif inner_type == "sakura_script" then
        buffer:put(SAKURA_SCRIPT.talk_to_script(actor, inner.text))
    elseif inner_type == "surface" then
        buffer:put(string.format("\\s[%s]", tostring(inner.id)))
    elseif inner_type == "wait" then
        buffer:put(string.format("\\w[%d]", inner.ms))
    elseif inner_type == "newline" then
        for _ = 1, inner.n do
            buffer:put("\\n")
        end
    elseif inner_type == "clear" then
        buffer:put("\\c")
    elseif inner_type == "raw_script" then
        buffer:put(inner.text)
    elseif inner_type == "choice" then
        local display = escape_choice(inner.display)
        local target = escape_choice(inner.target)
        buffer:put("\\![*]\\q[" .. display .. "," .. target .. "]")
    elseif inner_type == "choice_timeout" then
        local ms = inner.seconds and math.floor(inner.seconds * 1000) or 0
        buffer:put("\\![set,choicetimeout," .. ms .. "]")
    end
    -- yield は無視
end

--- @class BuildConfig
--- @field spot_newlines number スポット変更時の改行量（デフォルト1.5）

--- グループ化されたトークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens table[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @param input_actor_spots table<string, integer>|nil アクターごとのスポット位置マップ（直接変更される）
--- @return string さくらスクリプト文字列（\e終端）
function BUILDER.build(grouped_tokens, config, input_actor_spots)
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = (config.buffer_factory or buf.new)()

    -- input_actor_spots を直接変更する（nilの場合は内部で空テーブルを作成）
    local actor_spots = input_actor_spots or {}
    local last_actor = nil -- 最後に発言したActor
    local last_spot = nil  -- 最後のスポットID
    -- 前回の段落区切り改行以降に一般文字列（talk）が1文字でも出力されたか。
    -- アクター切り替え時、これが false（＝直前の会話が全てさくらスクリプト）なら
    -- 段落区切り改行を抑制する（sakura-script-newline）。
    local text_since_break = false

    for _, token in ipairs(grouped_tokens) do
        local t = token.type

        if t == "spot" then
            -- spotトークン処理: actor_spots[actor.name] = spot
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end
        elseif t == "clear_spot" then
            -- clear_spotトークン処理
            clear_spots(actor_spots)
            last_actor = nil
            last_spot = nil
            text_since_break = false
        elseif t == "actor" then
            -- actorトークン処理: アクター切り替え検出後、グループ内のトークンを順次処理
            local actor = token.actor

            if actor and last_actor ~= actor then
                local emitted_break
                last_spot, emitted_break =
                    emit_actor_switch(buffer, actor_spots, last_spot, actor, spot_newlines, text_since_break)
                -- 段落区切り改行を出力したら「前回の改行以降」の起点をリセット
                if emitted_break then
                    text_since_break = false
                end
                last_actor = actor
            end

            for _, inner in ipairs(token.tokens) do
                -- 一般文字列（非空の talk）が出力されたら改行許可フラグを立てる。
                -- surface/wait/sakura_script/choice 等はさくらスクリプト扱いでカウントしない。
                if inner.type == "talk" and inner.text ~= nil and inner.text ~= "" then
                    text_since_break = true
                end
                emit_inner_token(buffer, actor, inner)
            end
        elseif t == "raw_script" then
            buffer:put(token.text)
        end
    end

    buffer:put("\\e")
    return buffer:tostring()
end

return BUILDER
