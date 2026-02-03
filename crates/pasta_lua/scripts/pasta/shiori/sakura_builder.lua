--- @module pasta.shiori.sakura_builder
--- さくらスクリプトビルダーモジュール
---
--- トークン配列をさくらスクリプト文字列に変換する純粋関数モジュール。
--- pasta.shiori.act の build() から呼び出される。

local BUILDER = {}

--- さくらスクリプト用エスケープ処理
--- @param text string 入力テキスト
--- @return string エスケープ済みテキスト
local function escape_sakura(text)
    if not text then return "" end
    local escaped = text:gsub("\\", "\\\\")
    escaped = escaped:gsub("%%", "%%%%")
    return escaped
end

--- spotからスポットID番号を決定
--- @param spot any スポット値
--- @return number スポットID番号
local function spot_to_id(spot)
    if spot == "sakura" or spot == 0 then
        return 0
    elseif spot == "kero" or spot == 1 then
        return 1
    elseif type(spot) == "number" then
        return spot
    elseif type(spot) == "string" then
        -- "char2" → 2, "char10" → 10
        local n = spot:match("^char(%d+)$")
        if n then
            return tonumber(n)
        end
    end
    return 0 -- デフォルトはsakura
end

--- スポットタグを生成（SSP ukadoc準拠: 常に\p[ID]形式）
--- @param spot_id number スポットID番号
--- @return string スポットタグ
local function spot_to_tag(spot_id)
    return string.format("\\p[%d]", spot_id)
end

--- @class BuildConfig
--- @field spot_newlines number スポット変更時の改行量（デフォルト1.5）

--- トークン配列をさくらスクリプト文字列に変換
--- @param tokens table[] トークン配列
--- @param config BuildConfig|nil 設定
--- @return string さくらスクリプト文字列
function BUILDER.build(tokens, config)
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = {}

    -- ビルダー内部状態（build()呼び出しごとにリセット）
    local actor_spots = {} -- {[actor_name]: spot_id} actor位置マップ
    local last_actor = nil -- 最後に発言したActor
    local last_spot = nil  -- 最後のスポットID

    for _, token in ipairs(tokens) do
        local t = token.type

        if t == "spot" then
            -- spotトークン処理: actor_spots[actor.name] = spot
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end
        elseif t == "clear_spot" then
            -- clear_spotトークン処理: 状態リセット
            actor_spots = {}
            last_actor = nil
            last_spot = nil
        elseif t == "talk" then
            -- talkトークン処理: actor切り替え検出と出力
            local actor = token.actor
            local actor_name = actor and actor.name

            if actor and last_actor ~= actor then
                -- actor切り替え検出 → スポットタグ出力
                local spot = actor_spots[actor_name] or 0

                -- spot変更時に段落区切り改行を出力
                if last_spot ~= nil and last_spot ~= spot then
                    local percent = math.floor(spot_newlines * 100)
                    table.insert(buffer, string.format("\\n[%d]", percent))
                end

                table.insert(buffer, spot_to_tag(spot))
                last_actor = actor
                last_spot = spot
            end

            -- テキスト出力
            table.insert(buffer, escape_sakura(token.text))

            -- 以下は既存トークン処理（後方互換性のため維持）
        elseif t == "actor" then
            local spot_id = spot_to_id(token.actor.spot)
            table.insert(buffer, spot_to_tag(spot_id))
        elseif t == "spot_switch" then
            local percent = math.floor(spot_newlines * 100)
            table.insert(buffer, string.format("\\n[%d]", percent))
        elseif t == "surface" then
            table.insert(buffer, string.format("\\s[%s]", tostring(token.id)))
        elseif t == "wait" then
            table.insert(buffer, string.format("\\w[%d]", token.ms))
        elseif t == "newline" then
            for _ = 1, token.n do
                table.insert(buffer, "\\n")
            end
        elseif t == "clear" then
            table.insert(buffer, "\\c")
        elseif t == "sakura_script" then
            table.insert(buffer, token.text)
        end
        -- yield は無視
    end

    return table.concat(buffer) .. "\\e"
end

return BUILDER
