--- @module pasta.shiori.appearance
--- 外見状態（サーフェス・着せ替え）の観測・復旧モジュール（actor-surface-restore）
---
--- 出力済みさくらスクリプト文字列からサーフェス変更・スコープ切替タグを検出し（observe）、
--- アクター切替時の復旧タグ列を返す（restore）。
--- いずれも引数で受け取った状態テーブルをその場で更新する。
--- モジュール自身は状態を持たず、STORE・@pasta_* を require しない。
--- 出力文字列は読み取るのみで変更しない。

local APPEARANCE = {}

--- タグ名の文字集合（Rust Tokenizer::SAKURA_TAG_PATTERN と同じ）
local NAME_PATTERN = "^[0-9a-zA-Z_!+*?&%-]+"
--- 角括弧引数（最初の ] まで。SAKURA_TAG_PATTERN と同じ）
local ARG_PATTERN = "^%[([^%]]*)%]"

--- 空の外見状態を生成する
--- @return table AppearanceState
function APPEARANCE.new()
    return { actors = {}, spots = {}, owners = {}, last_spots = {} }
end

--- 位置 i の `\` から始まるタグを読む
--- @param s string
--- @param i integer
--- @return string|nil name タグ名（`\` を除く）。タグでなければ nil
--- @return string|nil arg 角括弧の中身（無ければ nil）
--- @return integer|nil next_pos タグ直後の位置
local function tag_at(s, i)
    local name = s:match(NAME_PATTERN, i + 1)
    if name then
        local j = i + 1 + #name
        local arg = s:match(ARG_PATTERN, j)
        return name, arg, j + (arg and #arg + 2 or 0)
    end
    return nil
end

--- pos 以降の次のタグを返す（戻り値は tag_at と同じ）。`\\` は 2 文字読み飛ばす。
--- @param s string
--- @param pos integer
local function next_tag(s, pos)
    while true do
        local i = s:find("\\", pos, true)
        if not i then
            return nil
        end
        local name, arg, next_pos = tag_at(s, i)
        if name then
            return name, arg, next_pos
        end
        -- `\\` はエスケープ。それ以外（`\` + 非タグ文字）は 1 文字進める
        pos = i + ((s:sub(i + 1, i + 1) == "\\") and 2 or 1)
    end
end

--- 名前の先頭 1 文字だけで決まるスコープ切替タグ（\0 \1 \h \u）
local SCOPE_HEAD = { ["0"] = true, ["1"] = true, h = true, u = true }

--- サーフェス変更なら ID を返す（\s[ID] / \sN）。空 ID は無視する
local function surface_id(name, arg)
    local id = (name == "s" and arg) or name:match("^s(%d)")
    if id ~= "" then
        return id
    end
    return nil
end

--- スコープ切替タグか（\0 \1 \h \u \p[N] \pN）
local function is_scope(name, arg)
    if SCOPE_HEAD[name:sub(1, 1)] then
        return true
    end
    return (name == "p" and arg ~= nil) or name:match("^p%d") ~= nil
end

--- bind タグか（\![bind,...] / \![bind-noevent,...]）
local function is_bind(name, arg)
    return name == "!" and arg ~= nil and (arg:match("^bind,") or arg:match("^bind%-noevent,")) ~= nil
end

--- タグを分類する
--- @param name string
--- @param arg string|nil
--- @return string|nil kind "surface" | "scope" | "bind" | nil（無関係なタグ）
--- @return string|nil value surface の ID、bind の引数
local function classify(name, arg)
    local id = surface_id(name, arg)
    if id then
        return "surface", id
    elseif is_scope(name, arg) then
        return "scope"
    elseif is_bind(name, arg) then
        return "bind", arg
    end
    return nil
end

--- t[key] のエントリを取得（無ければ生成）
local function entry(t, key)
    local e = t[key]
    if not e then
        e = { binds = {} }
        t[key] = e
    end
    return e
end

--- 明示 bind の数値
local BIND_VALUES = { ["0"] = 0, ["1"] = 1 }

--- bind 引数を `,` の単純分割で解釈する。
--- `"`・`\` を含む、またはカテゴリ名が空なら解釈不能として nil
--- @return string|nil cat, string part, string val（省略は ""）
local function parse_bind(arg)
    if arg:find('["\\]') then
        return nil
    end
    local f = {}
    for field in (arg .. ","):gmatch("([^,]*),") do
        f[#f + 1] = field
    end
    if f[2] == "" then
        return nil
    end
    return f[2], f[3] or "", f[4] or ""
end

--- スポット側パーツの実効値: 0/1 はその値、false は不明、nil は "" の値（無ければ不明）
local function spot_effective(t, part)
    if not t then
        return nil
    end
    local v = t[part]
    if v == nil then
        return t[""]
    end
    return v or nil
end

--- bind の結果値（0|1）。トグルはスポット側の実効値の反転（3.12）。確定できなければ nil
local function bind_value(sp, cat, part, val)
    if part ~= "" and val == "" then
        local cur = spot_effective(sp and sp.binds[cat], part)
        return cur and (1 - cur)
    end
    return BIND_VALUES[val]
end

--- 数値1の明示 bind 後のスポット側カテゴリ（3.13）。他パーツは不明、"" = 0 は残す。
--- "" があるとき着衣（1）・不明（false）だった他パーツは false にして "" へ落とさない
local function spot_after_wear(t)
    local out = { [""] = t[""] }
    if t[""] ~= nil then
        for q, v in pairs(t) do
            if q ~= "" and v ~= 0 then
                out[q] = false
            end
        end
    end
    return out
end

--- 明示 bind を記録する（3.1, 3.13）
local function record_bind(sp, ac, cat, part, v)
    if sp then
        local t = sp.binds[cat] or {}
        if v == 1 then
            t = spot_after_wear(t)
        end
        t[part] = v
        sp.binds[cat] = t
    end
    if ac then
        local t = ac.binds[cat] or {} -- false（不明）からは新規に記録し直す
        t[part] = v
        ac.binds[cat] = t
        local order = ac.order[cat] or {}
        for i = #order, 1, -1 do
            if order[i] == part then
                table.remove(order, i)
            end
        end
        order[#order + 1] = part
        ac.order[cat] = order
    end
end

--- カテゴリを置き換える。strip = true は全脱衣 { [""] = 0 }（3.11）、false は不明（3.4）
local function reset_category(sp, ac, cat, strip)
    if sp then
        sp.binds[cat] = strip and { [""] = 0 } or nil
    end
    if ac then
        -- 3.4: 不明は false（既定着せ替えも使わない）
        ac.binds[cat] = strip and { [""] = 0 } or false
        ac.order[cat] = strip and {} or nil
    end
end

--- 着せ替え指定を観測する（actor 非 nil）
local function observe_bind(state, actor, spot, arg)
    local sp = spot ~= nil and entry(state.spots, spot) or nil
    local cat, part, val = parse_bind(arg)
    if not cat then
        -- 解釈不能: 発話スポットの着せ替えのみ全カテゴリ不明（アクター既知状態は不変）
        if sp then
            sp.binds = {}
        end
        return
    end
    local ac = nil
    if actor.name ~= nil then
        ac = entry(state.actors, actor.name)
        ac.order = ac.order or {}
    end
    local v = bind_value(sp, cat, part, val)
    if v == nil or (part == "" and v ~= 0) then
        reset_category(sp, ac, cat, false)
    elseif part == "" then
        reset_category(sp, ac, cat, true)
    else
        record_bind(sp, ac, cat, part, v)
    end
end

--- アクター未指定の生スクリプト: 外見タグを含めば全スポット不明化（1.8）
local function observe_raw(state, text)
    local name, arg, pos = next_tag(text, 1)
    while name do
        if classify(name, arg) then
            state.spots = {}
            return
        end
        name, arg, pos = next_tag(text, pos)
    end
end

--- 出力済み文字列を観測して状態へ反映する。text は変更しない。
--- @param state table AppearanceState
--- @param actor table|nil 発話アクター。nil はアクター未指定の生さくらスクリプト
--- @param spot integer|nil 発話アクターの解決済みスポット（actor が nil のとき不使用）
--- @param text string 出力した文字列
function APPEARANCE.observe(state, actor, spot, text)
    if not text:find("\\", 1, true) then
        return
    end
    -- 1.7: スコープ切替後は次の restore（\p[spot]）までアクター未指定として扱う
    if actor == nil or state.detached then
        observe_raw(state, text)
        return
    end
    local name, arg, pos = next_tag(text, 1)
    while name do
        local kind, value = classify(name, arg)
        if kind == "scope" then
            -- 1.7: 全スポット不明化し、残りは記録しない（owners・last_spots は保持）。
            -- DSL はタグごとに別トークンのため、後続の observe 呼び出しも detached で止める
            state.spots = {}
            state.detached = true
            return
        elseif kind == "surface" then
            -- 1.2〜1.5: 出現順に上書き（最後が残る）。ID は文字列のまま
            if actor.name ~= nil then
                entry(state.actors, actor.name).surface = value
            end
            if spot ~= nil then
                entry(state.spots, spot).surface = value
            end
        elseif kind == "bind" then
            observe_bind(state, actor, spot, value)
        end
        name, arg, pos = next_tag(text, pos)
    end
end

--- 文字列の先頭タグ列を走査し、分類済みタグを found へ出現順に追加する
--- @return boolean 一般文字・スコープ切替タグに達したら true（先頭タグ列の終端）
local function scan_leading_text(s, found)
    local pos = 1
    while pos <= #s do
        if s:sub(pos, pos) ~= "\\" then
            return true
        end
        local name, arg, next_pos = tag_at(s, pos)
        if not name then
            return true -- `\\` 等はタグでなく一般文字
        end
        local kind, value = classify(name, arg)
        if kind == "scope" then
            return true
        elseif kind then
            found[#found + 1] = { kind = kind, value = value }
        end
        pos = next_pos
    end
    return false
end

--- 切替先グループの先頭タグ列（最初の一般文字・スコープ切替タグ・raw_script まで）の
--- サーフェス変更・bind を出現順に返す。tokens は読み取りのみ。
--- @param tokens table[]
--- @return table[] { kind = "surface"|"bind", value = string }
local function leading_tags(tokens)
    local found = {}
    for _, t in ipairs(tokens) do
        local ty = t.type
        if ty == "surface" then
            local id = surface_id("s", tostring(t.id))
            if id then
                found[#found + 1] = { kind = "surface", value = id }
            end
        elseif ty == "talk" or ty == "sakura_script" then
            -- talk も走査する（＠単語参照が `\s[ID]` へ展開され talk に結合されるため）。空 talk は素通り
            if scan_leading_text(t.text or "", found) then
                return found
            end
        elseif ty == "raw_script" then
            return found
        end
    end
    return found
end

--- 既定サーフェス（数値・非空文字列のみ。それ以外の型は無視）
local function default_surface(actor)
    local v = actor.surface
    if type(v) == "number" or (type(v) == "string" and v ~= "") then
        return tostring(v)
    end
    return nil
end

--- サーフェス復旧タグを返し、スポットの表示中サーフェスを更新する（2.1〜2.4, 2.8〜2.10, 4.4）
local function restore_surface(state, actor, spot, leading)
    local rec = state.actors[actor.name]
    local known = (rec and rec.surface) or default_surface(actor)
    if known == nil then
        return ""
    end
    local cur = state.spots[spot]
    if cur and cur.surface == known then
        return ""
    end
    for _, tag in ipairs(leading) do
        if tag.kind == "surface" then
            return ""
        end
    end
    entry(state.spots, spot).surface = known
    return "\\s[" .. known .. "]"
end

--- アクター切替時の復旧タグ列を返し、スポットの表示中状態を更新する。
--- @param state table AppearanceState
--- @param actor table 切替先アクター（name / surface を参照）
--- @param spot integer 解決済みスポットID
--- @param tokens table[] 切替先グループの内側トークン列（先頭タグ列の走査用・読み取りのみ）
--- @return string 復旧タグ列（復旧不要なら空文字列）
function APPEARANCE.restore(state, actor, spot, tokens)
    -- 1.7: \p[spot] 出力でスコープ切替の影響は終わる（継続でも解除する）
    state.detached = nil
    local name = actor.name
    -- 2.11: 同一アクターの継続（スポットの直前発話者が自分で、前回も同じスポット）なら復旧しない
    local continuing = name ~= nil and state.owners[spot] == name and state.last_spots[name] == spot
    state.owners[spot] = name
    if name == nil or continuing then
        return ""
    end
    state.last_spots[name] = spot
    return restore_surface(state, actor, spot, leading_tags(tokens))
end

return APPEARANCE
