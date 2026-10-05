--- @module pasta.shiori.sakura_builder
--- さくらスクリプトビルダーモジュール
---
--- グループ化されたトークン配列をさくらスクリプト文字列に変換するモジュール。
--- pasta.shiori.act の build() から呼び出される。

local BUILDER = {}

local SAKURA_SCRIPT = require "@pasta_sakura_script"
local log = require "@pasta_log"
local buf = require("pasta.buf")
local APPEARANCE = require("pasta.shiori.appearance")

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

--- nil ならビルドローカルの空状態（BUILDER.build の複雑度を増やさないため外出し）
local function appearance_or_new(appearance)
    return appearance or APPEARANCE.new()
end

--- アクター切り替え時にスポットタグ `\p[spot]` を出力し、直後に外見の復旧タグを出力する。
--- スポット解決（未設定→0＋警告）・`\p[spot]` 出力・復旧タグ出力のみを行い、
--- 段落区切り改行の判定・出力・保留（pending）はすべて呼び出し側（BUILDER.build ループ）が担う。
--- 復旧タグは talk 経路を通らないため has-text・pending に影響しない。
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor_spots table<string, integer> アクターごとのスポット位置マップ
--- @param actor table 切り替え先アクター（非nil保証は呼び出し元）
--- @param appearance table 外見状態（直接変更される）
--- @param tokens table[] 切り替え先グループの内側トークン列（先頭タグ列の判定用）
--- @return number spot 切り替え後のスポットID
local function emit_actor_switch(buffer, actor_spots, actor, appearance, tokens)
    local actor_name = actor.name
    local spot = actor_spots[actor_name]
    if spot == nil then
        spot = 0
        if actor_name then
            log.warn(string.format("actor_spots fallback: '%s' -> spot=0", actor_name))
        end
    end

    buffer:put(spot_to_tag(spot))
    buffer:put(APPEARANCE.restore(appearance, actor, spot, tokens))
    return spot
end

--- 文字を表示するタグの名前（1.4）
local CHAR_TAGS = { _u = true, _m = true, ["&"] = true }

--- 単位が字を表示するか（文字を表示するタグ、または中身のある囲み）
local function unit_shows_text(name, literal)
    return CHAR_TAGS[name] or (literal ~= nil and literal ~= "")
end

--- テキストに字が 1 文字以上あるか（1.1〜1.7, 1.10）。
--- `\` 以外の文字・エスケープ・単位にならない `\`・字を表示する単位があれば真
--- @param s string
--- @return boolean
local function has_text(s)
    local pos = 1
    while pos <= #s do
        if s:sub(pos, pos) ~= "\\" then
            return true
        end
        local name, _, next_pos, literal = APPEARANCE.tag_at(s, pos)
        if not name or unit_shows_text(name, literal) then
            return true
        end
        pos = next_pos
    end
    return false
end

--- テキスト中のタグと囲みに字を表示するものがあるか（1.9, 1.10。タグ以外の文字とエスケープは数えない）
--- @param s string
--- @return boolean
local function script_shows_text(s)
    local pos = 1
    while true do
        local i = s:find("\\", pos, true)
        if not i then
            return false
        end
        local name, _, next_pos, literal = APPEARANCE.tag_at(s, i)
        if name and unit_shows_text(name, literal) then
            return true
        end
        pos = next_pos
    end
end

--- 内側トークンが字を出すか（S3 の条件。判定は talk_to_script の前のテキストに対して行う。1.8）
--- @param inner table 内側トークン
--- @return boolean
local function emits_text(inner)
    if inner.text == nil then
        return false
    end
    local text = tostring(inner.text) -- 数値などは talk_to_script と同じく文字列として読む
    if inner.type == "talk" then
        return has_text(text)
    elseif inner.type == "sakura_script" then
        return script_shows_text(text)
    end
    return false -- それ以外の型は字を出さない（3.4）
end

--- actorグループ内の単一トークンをさくらスクリプト文字列へ変換する
--- @param actor table|nil グループの発言アクター
--- @param inner table グループ内トークン
--- @return string 変換結果（下記以外の型のトークンは空文字列）
local function inner_token_to_string(actor, inner)
    local inner_type = inner.type

    if inner_type == "talk" or inner_type == "sakura_script" then
        return SAKURA_SCRIPT.talk_to_script(actor, inner.text)
    elseif inner_type == "surface" then
        return string.format("\\s[%s]", tostring(inner.id))
    elseif inner_type == "wait" then
        return string.format("\\_w[%d]", inner.ms)
    elseif inner_type == "newline" then
        return string.rep("\\n", inner.n)
    elseif inner_type == "clear" then
        return "\\c"
    elseif inner_type == "raw_script" then
        return inner.text
    elseif inner_type == "choice" then
        return "\\![*]\\q[" .. escape_choice(inner.display) .. "," .. escape_choice(inner.target) .. "]"
    elseif inner_type == "choice_timeout" then
        local ms = inner.seconds and math.floor(inner.seconds * 1000) or 0
        return "\\![set,choicetimeout," .. ms .. "]"
    end
    -- 上記以外の型は出力しない
    return ""
end

--- actorグループ内の単一トークンをさくらスクリプトへ変換して出力し、外見状態に観測させる
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor table|nil グループの発言アクター
--- @param inner table グループ内トークン
--- @param appearance table 外見状態（直接変更される）
--- @param spot integer|nil 現在スコープの解決済みスポットID
local function emit_inner_token(buffer, actor, inner, appearance, spot)
    local s = inner_token_to_string(actor, inner)
    buffer:put(s)
    -- raw_script はアクター未指定の生さくらスクリプトとして観測する（1.8）
    local speaker = actor
    if inner.type == "raw_script" then
        speaker = nil
    end
    APPEARANCE.observe(appearance, speaker, spot, s)
end

--- @class BuildConfig
--- @field spot_newlines number スポット変更時の改行量（デフォルト1.5）
--- @field buffer_factory (fun(): table)|nil 出力バッファの生成関数（デフォルト pasta.buf.new。テスト用）

--- グループ化されたトークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens table[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @param input_actor_spots table<string, integer>|nil アクターごとのスポット位置マップ（直接変更される）
--- @param appearance table|nil 外見状態（直接変更される。nil ならビルドローカルの空状態）
--- @return string さくらスクリプト文字列（\e終端）
-- ponytail: トークン種別の分岐が集まるため複雑度 22 > 15 を許容。分割はリファクタ時に（特性化テスト先行）。
function BUILDER.build(grouped_tokens, config, input_actor_spots, appearance) -- luacheck: ignore 561
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = (config.buffer_factory or buf.new)()

    -- input_actor_spots を直接変更する（nilの場合は内部で空テーブルを作成）
    local actor_spots = input_actor_spots or {}
    -- 外見状態も直接変更する（nilの場合はビルドローカル。clear_spot では破棄しない）
    appearance = appearance_or_new(appearance)
    -- ビルドローカル状態機械（sakura-script-newline / 完全遅延方式）
    local last_actor = nil    -- 最後に発言したActor
    local last_spot = nil     -- 最後のスポットID（＝現在スコープ）
    -- 解決済みスポットIDごとに、同一ビルド内で字を出すトークン（emits_text）を出力済みか。
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
                local spot = emit_actor_switch(buffer, actor_spots, actor, appearance, token.tokens)
                pending_break = (spot_has_text[spot] == true)
                last_spot = spot
                last_actor = actor
            end

            for _, inner in ipairs(token.tokens) do
                if inner.type == "clear" then
                    -- S4b: `\c` を従来どおり出力（emit_inner_token 経由で不変）したうえで、
                    -- 現在スポットの has-text を偽へリセットし pending を破棄する。
                    -- クリアで区切るべき先行テキストが消えるため、後続テキスト直前に改行を出さない。
                    emit_inner_token(buffer, actor, inner, appearance, last_spot)
                    pending_break = false
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = false
                    end
                elseif emits_text(inner) then
                    -- S3: 字を出すトークン（字のある talk・字を表示する sakura_script）。
                    -- 改行 → has-text 設定 → 本文 の順を厳守。
                    if pending_break then
                        buffer:put(string.format("\\n[%d]", math.floor(spot_newlines * 100)))
                        pending_break = false
                    end
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = true
                    end
                    emit_inner_token(buffer, actor, inner, appearance, last_spot)
                else
                    -- S4: 字の無い talk・字を表示しない sakura_script・surface・wait・newline・
                    -- choice・choice_timeout・raw_script。変換出力のみ（has-text・pending 不変）。
                    emit_inner_token(buffer, actor, inner, appearance, last_spot)
                end
            end
        elseif t == "raw_script" then
            buffer:put(token.text)
            APPEARANCE.observe(appearance, nil, nil, token.text)
        end
    end

    -- S7: ループ終端。未フラッシュの pending は出力せず破棄してから `\e` を付与。
    buffer:put("\\e")
    return buffer:tostring()
end

return BUILDER
