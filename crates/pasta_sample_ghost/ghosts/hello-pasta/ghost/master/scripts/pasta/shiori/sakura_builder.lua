--- @module pasta.shiori.sakura_builder
--- さくらスクリプトビルダーモジュール
---
--- グループ化されたトークン配列をさくらスクリプト文字列に変換する純粋関数モジュール。
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

--- グループ化されたトークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens table[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @return string さくらスクリプト文字列
function BUILDER.build(grouped_tokens, config)
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = {}

    -- ビルダー内部状態（build()呼び出しごとにリセット）
    local actor_spots = {} -- {[actor_name]: spot_id} actor位置マップ
    local last_actor = nil -- 最後に発言したActor
    local last_spot = nil  -- 最後のスポットID

    for _, token in ipairs(grouped_tokens) do
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
        elseif t == "actor" then
            -- actorトークン処理: グループ内のトークンを順次処理
            local actor = token.actor
            local actor_name = actor and actor.name

            -- アクター切り替え検出
            if actor and last_actor ~= actor then
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

            -- グループ内トークンを順次処理
            for _, inner in ipairs(token.tokens) do
                local inner_type = inner.type

                if inner_type == "talk" then
                    table.insert(buffer, escape_sakura(inner.text))
                elseif inner_type == "surface" then
                    table.insert(buffer, string.format("\\s[%s]", tostring(inner.id)))
                elseif inner_type == "wait" then
                    table.insert(buffer, string.format("\\w[%d]", inner.ms))
                elseif inner_type == "newline" then
                    for _ = 1, inner.n do
                        table.insert(buffer, "\\n")
                    end
                elseif inner_type == "clear" then
                    table.insert(buffer, "\\c")
                elseif inner_type == "sakura_script" then
                    table.insert(buffer, inner.text)
                end
                -- yield は無視
            end
        end
    end

    return table.concat(buffer) .. "\\e"
end

return BUILDER
