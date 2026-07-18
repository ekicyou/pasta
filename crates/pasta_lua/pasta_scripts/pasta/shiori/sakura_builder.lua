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

--- アクター切り替え時にスポットタグ `\p[spot]` を出力する。
--- スポット解決（未設定→0＋警告）と `\p[spot]` 出力のみを行い、
--- 段落区切り改行の判定・出力・保留（pending）はすべて呼び出し側（BUILDER.build ループ）が担う。
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor_spots table<string, integer> アクターごとのスポット位置マップ
--- @param actor table 切り替え先アクター（非nil保証は呼び出し元）
--- @return number spot 切り替え後のスポットID
local function emit_actor_switch(buffer, actor_spots, actor)
    local actor_name = actor.name
    local spot = actor_spots[actor_name]
    if spot == nil then
        spot = 0
        if actor_name then
            log.warn(string.format("actor_spots fallback: '%s' -> spot=0", actor_name))
        end
    end

    buffer:put(spot_to_tag(spot))
    return spot
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
    -- ビルドローカル状態機械（sakura-script-newline / 完全遅延方式）
    local last_actor = nil    -- 最後に発言したActor
    local last_spot = nil     -- 最後のスポットID（＝現在スコープ）
    -- 解決済みスポットIDごとに、同一ビルド内で一般文字列（非空 talk）を出力済みか。
    -- 段落区切り改行の判定材料（「バルーンに既にテキストがあるか」の意味論）。
    local spot_has_text = {}  -- table<integer, boolean>
    -- 現在スコープ（last_spot）に対する段落区切り改行の保留フラグ。
    -- 切替のたびに破棄→切替先の has-text で再評価されるため単一 boolean で表現できる。
    local pending_break = false

    for _, token in ipairs(grouped_tokens) do
        local t = token.type

        if t == "spot" then
            -- S6: spotトークン処理: actor_spots[actor.name] = spot（状態不変）
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end
        elseif t == "clear_spot" then
            -- S5: clear_spotトークン処理（has-text 全体と pending もリセット）
            clear_spots(actor_spots)
            last_actor = nil
            last_spot = nil
            spot_has_text = {}
            pending_break = false
        elseif t == "actor" then
            -- actorトークン処理: アクター切り替え検出後、グループ内のトークンを順次処理
            local actor = token.actor

            if actor and last_actor ~= actor then
                -- S1: アクター切替検出。スポット解決＋`\p[spot]` 出力。
                -- 切替先スポットの has-text で pending を再評価する（旧 pending は暗黙破棄）。
                -- 先出し版の last_spot ~= spot / last_spot == spot ガードは復活させない。
                local spot = emit_actor_switch(buffer, actor_spots, actor)
                pending_break = (spot_has_text[spot] == true)
                last_spot = spot
                last_actor = actor
            end

            for _, inner in ipairs(token.tokens) do
                if inner.type == "clear" then
                    -- S4b: `\c` を従来どおり出力（emit_inner_token 経由で不変）したうえで、
                    -- 現在スポットの has-text を偽へリセットし pending を破棄する。
                    -- クリアで区切るべき先行テキストが消えるため、後続テキスト直前に改行を出さない。
                    emit_inner_token(buffer, actor, inner)
                    pending_break = false
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = false
                    end
                elseif inner.type == "talk" and inner.text ~= nil and inner.text ~= "" then
                    -- S3: 非空 talk（一般文字列）。改行 → has-text 設定 → talk本文 の順を厳守。
                    if pending_break then
                        buffer:put(string.format("\\n[%d]", math.floor(spot_newlines * 100)))
                        pending_break = false
                    end
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = true
                    end
                    emit_inner_token(buffer, actor, inner)
                else
                    -- S4: 空 talk・surface・wait・sakura_script・newline・choice・
                    -- choice_timeout・raw_script・yield。変換出力のみ（has-text・pending 不変）。
                    emit_inner_token(buffer, actor, inner)
                end
            end
        elseif t == "raw_script" then
            buffer:put(token.text)
        end
    end

    -- S7: ループ終端。未フラッシュの pending は出力せず破棄してから `\e` を付与。
    buffer:put("\\e")
    return buffer:tostring()
end

return BUILDER
