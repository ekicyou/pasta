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
local STYLE = require("pasta.shiori.actor_style")
local FAILURE = require("pasta.failure")

--- \q[display,target] 内のデリミタ文字をエスケープ
--- @param s string エスケープ対象の文字列
--- @return string エスケープ済み文字列
local function escape_choice(s)
    s = s:gsub("\\", "\\\\")
    s = s:gsub("%]", "\\]")
    s = s:gsub(",", "\\,")
    return s
end

--- choiceトークンを \![*]\q[display,target(,scope)] へ変換する
--- scope（選択肢を出したグローバルシーン名）は OnChoiceSelectEx の Reference2 として戻る。
--- On・script: で始まる ID は SSP が第 3 引数以降を別の意味で使うため付けない。
--- @param inner table choiceトークン
--- @return string さくらスクリプト
local function choice_to_script(inner)
    local target = inner.target
    local s = "\\![*]\\q[" .. escape_choice(inner.display) .. "," .. escape_choice(target)
    if type(inner.scope) == "string"
        and target:sub(1, 2) ~= "On" and target:sub(1, 7) ~= "script:" then
        s = s .. "," .. escape_choice(inner.scope)
    end
    return s .. "]"
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
--- スポット解決（未設定→0＋警告。未登録アクターは目印で報告済みなので警告しない）・`\p[spot]` 出力・復旧タグ出力のみを行い、
--- 段落区切り改行の判定・出力・保留（pending）はすべて呼び出し側（BUILDER.build ループ）が担う。
--- 復旧タグは talk 経路を通らないため has-text・pending に影響しない。
--- @param buffer table 出力バッファ（pasta.buf 互換）
--- @param actor_spots table<string, integer> アクターごとのスポット位置マップ
--- @param actor table 切り替え先アクター（非nil保証は呼び出し元）
--- @param appearance table 外見状態（直接変更される）
--- @param tokens table[] 切り替え先グループの内側トークン列（先頭タグ列の判定用）
--- @param style table 装いの状態（pasta.shiori.actor_style。直接変更される）。復旧タグの後に装いの出し入れのタグを出す
--- @return number spot 切り替え後のスポットID
local function emit_actor_switch(buffer, actor_spots, actor, appearance, tokens, style)
    local actor_name = actor.name
    local spot = actor_spots[actor_name]
    if spot == nil then
        spot = 0
        if actor_name and not FAILURE.is_unregistered_actor(actor) then
            log.warn(string.format("actor_spots fallback: '%s' -> spot=0", actor_name))
        end
    end

    buffer:put(spot_to_tag(spot))
    buffer:put(APPEARANCE.restore(appearance, actor, spot, tokens))
    buffer:put(STYLE.switch(style, actor, spot))
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

--- テキストの中の、引数の無い `\c` の場所（字の判定と同じく APPEARANCE.tag_at で読む。
--- エスケープと囲みの中はタグとして読まない）
--- @param s string
--- @return integer|nil first 最初の `\c` の位置（無ければ nil）
--- @return integer|nil after 最後の `\c` の直後の位置
local function clear_span(s)
    local first, after
    local pos = 1
    while true do
        local i = s:find("\\", pos, true)
        if not i then
            return first, after
        end
        local name, arg, next_pos = APPEARANCE.tag_at(s, i)
        if name == "c" and arg == nil then
            first = first or i
            after = next_pos
        end
        pos = next_pos
    end
end

--- 内側トークンの字と、台詞に書いた `\c`（吹き出しの消去）を読む。
--- `\c` を読むのは talk・sakura_script のテキストだけ（raw_script・選択肢の表示は読まない）
--- @param inner table 内側トークン
--- @return boolean lead 最初の `\c` より前に字があるか（`\c` が無ければ emits_text と同じ）
--- @return boolean cleared 引数の無い `\c` があるか
--- @return boolean trail 最後の `\c` より後ろに字があるか
local function text_parts(inner)
    local t = inner.type
    if inner.text == nil or (t ~= "talk" and t ~= "sakura_script") then
        return false, false, false
    end
    local text = tostring(inner.text)
    local first, after = clear_span(text)
    if not first then
        return emits_text(inner), false, false
    end
    return emits_text({ type = t, text = text:sub(1, first - 1) }), true,
        emits_text({ type = t, text = text:sub(after) })
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
        return choice_to_script(inner)
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
--- @param style table 装いの状態（直接変更される）。変換後の文字列を出力の前に STYLE.through に通す
local function emit_inner_token(buffer, actor, inner, appearance, spot, style)
    local s = inner_token_to_string(actor, inner)
    s = STYLE.through(style, actor, spot, s)
    buffer:put(s)
    -- raw_script はアクター未指定の生さくらスクリプトとして観測する（1.8）
    local speaker = actor
    if inner.type == "raw_script" then
        speaker = nil
    end
    APPEARANCE.observe(appearance, speaker, spot, s)
end

--- 幅つきの改行のタグ。spot_newlines と choice_newlines の両方がこれを使う
--- @param ratio number 行の高さに対する倍率
--- @return string `\n[N]`（N は ratio を 100 倍して小数点以下を切り捨てた整数）
local function newline_tag(ratio)
    return string.format("\\n[%d]", math.floor(ratio * 100))
end

--- 選択肢の直前に出す区切り
--- @param tail "text"|"choice"|nil その吹き出しに最後に表示したもの
--- @param choice_newlines number 字の後の選択肢の前の改行幅
--- @return string 空文字列（最初の選択肢）、`\n[N]`（字の後）、`\n`（選択肢の後）のどれか
local function choice_break(tail, choice_newlines)
    if tail == "text" then
        return newline_tag(choice_newlines)
    elseif tail == "choice" then
        return "\\n"
    end
    return ""
end

--- @class BuildConfig
--- @field spot_newlines number|nil スポット変更時の改行量（デフォルト1.5）
--- @field choice_newlines number|nil 字の後の選択肢の前の改行量（デフォルト1.5）
--- @field buffer_factory (fun(): table)|nil 出力バッファの生成関数（デフォルト pasta.buf.new。テスト用）

--- グループ化されたトークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens table[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @param input_actor_spots table<string, integer>|nil アクターごとのスポット位置マップ（直接変更される）
--- @param appearance table|nil 外見状態（直接変更される。nil ならビルドローカルの空状態）
--- @return string さくらスクリプト文字列（\e終端）
-- ponytail: トークン種別の分岐が集まるため複雑度 28 > 15 を許容。分割はリファクタ時に（特性化テスト先行）。
function BUILDER.build(grouped_tokens, config, input_actor_spots, appearance) -- luacheck: ignore 561
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local choice_newlines = config.choice_newlines or 1.5
    local buffer = (config.buffer_factory or buf.new)()

    -- input_actor_spots を直接変更する（nilの場合は内部で空テーブルを作成）
    local actor_spots = input_actor_spots or {}
    -- 外見状態も直接変更する（nilの場合はビルドローカル。clear_spot では破棄しない）
    appearance = appearance_or_new(appearance)
    -- 装いの状態はビルドごとに作り、次のビルドへ持ち越さない（actor-display-style）
    local style = STYLE.new()
    -- ビルドローカル状態機械（sakura-script-newline / 完全遅延方式）
    local last_actor = nil    -- 最後に発言したActor
    local last_spot = nil     -- 最後のスポットID（＝現在スコープ）
    -- 解決済みスポットIDごとに、同一ビルド内で字を出すトークン（emits_text）を出力済みか。
    -- 段落区切り改行の判定材料（「バルーンに既にテキストがあるか」の意味論）。
    local spot_has_text = {}  -- table<integer, boolean>
    -- 現在スコープ（last_spot）に対する段落区切り改行の保留フラグ。
    -- 切替のたびに破棄→切替先の has-text で再評価されるため単一 boolean で表現できる。
    local pending_break = false
    -- スポットIDごとに、その吹き出しに最後に表示したもの（"text"・"choice"。無ければ未表示）。
    -- 選択肢の前と、選択肢の後の字の前の区切りの判定材料。
    -- 話し手なしのグループの中（last_spot が nil の間）はスポット 0 として数える。
    local spot_tail = {}      -- table<integer, "text"|"choice">

    for _, token in ipairs(grouped_tokens) do
        local t = token.type

        if t == "spot" then
            -- S6: spotトークン処理: actor_spots[actor.name] = spot（状態不変）
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end
        elseif t == "clear_spot" then
            -- S5: clear_spotトークン処理（has-text 全体・最後に表示したもの全体と pending もリセット）
            clear_spots(actor_spots)
            last_actor = nil
            last_spot = nil
            spot_has_text = {}
            spot_tail = {}
            pending_break = false
        elseif t == "actor" then
            -- actorトークン処理: アクター切り替え検出後、グループ内のトークンを順次処理
            local actor = token.actor

            if actor and last_actor ~= actor then
                -- S1: アクター切替検出。スポット解決＋`\p[spot]` 出力。
                -- 切替先スポットの has-text で pending を再評価する（旧 pending は暗黙破棄）。
                -- 先出し版の last_spot ~= spot / last_spot == spot ガードは復活させない。
                local spot = emit_actor_switch(buffer, actor_spots, actor, appearance, token.tokens, style)
                pending_break = (spot_has_text[spot] == true)
                last_spot = spot
                last_actor = actor
            end

            local tail_spot = last_spot or 0 -- 話し手なしのグループはスポット 0 の吹き出しに出る
            for _, inner in ipairs(token.tokens) do
                -- lead: 字を出すトークンか（台詞に書いた `\c` があれば、最初の `\c` より前の字だけを見る）
                local lead, cleared, trail = text_parts(inner)
                if inner.type == "clear" then
                    -- S4b: `\c` を従来どおり出力（emit_inner_token 経由で不変）したうえで、
                    -- 現在スポットの has-text を偽へリセットし pending を破棄する。
                    -- クリアで区切るべき先行テキストが消えるため、後続テキスト直前に改行を出さない。
                    -- 最後に表示したものも消す（消した吹き出しの最初の選択肢を区切らない）。
                    emit_inner_token(buffer, actor, inner, appearance, last_spot, style)
                    pending_break = false
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = false
                    end
                    spot_tail[tail_spot] = nil
                elseif inner.type == "choice" then
                    -- S4c: 選択肢。直前に区切り（字の後は choice_newlines の幅、選択肢の後は `\n`）を出す。
                    -- 字には数えない（has-text・pending 不変。保留は次の字の直前で出る）。
                    buffer:put(choice_break(spot_tail[tail_spot], choice_newlines))
                    spot_tail[tail_spot] = "choice"
                    emit_inner_token(buffer, actor, inner, appearance, last_spot, style)
                elseif lead then
                    -- S3: 字を出すトークン（字のある talk・字を表示する sakura_script）。
                    -- 改行 → has-text 設定 → 本文 の順を厳守。
                    -- 区切りは 1 つだけ出す（保留があればそれだけ。無くて選択肢の後なら `\n`）。
                    if pending_break then
                        buffer:put(newline_tag(spot_newlines))
                        pending_break = false
                    elseif spot_tail[tail_spot] == "choice" then
                        buffer:put("\\n")
                    end
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = true
                    end
                    spot_tail[tail_spot] = "text"
                    emit_inner_token(buffer, actor, inner, appearance, last_spot, style)
                else
                    -- S4: 字の無い talk・字を表示しない sakura_script・surface・wait・newline・
                    -- choice_timeout・raw_script。変換出力のみ（has-text・pending・最後に表示したもの不変）。
                    emit_inner_token(buffer, actor, inner, appearance, last_spot, style)
                end
                if cleared then
                    -- S4d: 台詞に書いた `\c`（S3・S4 のどちらを通ったトークンにも当てる）。
                    -- clear のトークン（S4b）と同じに状態を戻す。最後の `\c` より後ろの字は、
                    -- 消した吹き出しの最初の字として記録する（区切りは出さない）。
                    pending_break = false
                    if last_spot ~= nil then
                        spot_has_text[last_spot] = trail
                    end
                    spot_tail[tail_spot] = trail and "text" or nil
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
