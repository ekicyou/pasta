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
--- @field spot_switch_newlines number スポット切り替え時の改行量（デフォルト1.5）

--- トークン配列をさくらスクリプト文字列に変換
--- @param tokens table[] トークン配列
--- @param config BuildConfig|nil 設定
--- @return string さくらスクリプト文字列
function BUILDER.build(tokens, config)
    config = config or {}
    local spot_switch_newlines = config.spot_switch_newlines or 1.5
    local buffer = {}

    for _, token in ipairs(tokens) do
        local t = token.type
        if t == "actor" then
            local spot_id = spot_to_id(token.actor.spot)
            table.insert(buffer, spot_to_tag(spot_id))
        elseif t == "spot_switch" then
            local percent = math.floor(spot_switch_newlines * 100)
            table.insert(buffer, string.format("\\n[%d]", percent))
        elseif t == "talk" then
            table.insert(buffer, escape_sakura(token.text))
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
