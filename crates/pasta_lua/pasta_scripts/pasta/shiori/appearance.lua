--- @module pasta.shiori.appearance
--- 外見状態（サーフェス・着せ替え）の観測・復旧モジュール（actor-surface-restore）
---
--- 出力済みさくらスクリプト文字列からサーフェス変更・スコープ切替タグを検出し（observe）、
--- アクター切替時の復旧タグ列を返す（restore）。
--- いずれも引数で受け取った状態テーブルをその場で更新する。
--- モジュール自身は状態を持たず、STORE・@pasta_* を require しない。
--- 出力文字列は読み取るのみで変更しない。
---
--- タグの読み取り（tag_at）は Rust Tokenizer::SAKURA_TAG_PATTERN
--- （src/sakura_script/tokenizer.rs）と同じ規則の写しで、左から 1 回読むだけで後戻りしない。
--- 一致は適合テスト tests/sakura_script/conformance_test.rs で保つ。

local APPEARANCE = {}

--- 空の外見状態を生成する
--- @return table AppearanceState
function APPEARANCE.new()
    return { actors = {}, spots = {}, owners = {}, last_spots = {} }
end

--- 位置 k の `"` から始まる引用の直後の位置（中の `""` は 1 文字）。閉じなければ nil
local function quote_end(s, k)
    local q = s:find('"', k + 1, true)
    while q and s:sub(q + 1, q + 1) == '"' do
        q = s:find('"', q + 2, true)
    end
    return q and q + 1
end

--- 位置 j の `[` から始まる引数の `]` の次の位置。閉じなければ nil。
--- `\` ＋ 1 文字の組は読み飛ばし、引用は引数の先頭（`[` と `,` の直後）でだけ開く
local function args_end(s, j)
    local k, at_start = j + 1, true
    while k <= #s do
        local c = s:sub(k, k)
        if at_start and c == '"' then
            k = quote_end(s, k)
            if not k then
                return nil -- 先頭の引用が閉じない
            end
            at_start = false
        elseif c == "]" then
            return k + 1
        else
            -- 末尾の `\` は k が #s を越えて閉じないことになる
            k = k + ((c == "\\") and 2 or 1)
            at_start = c == ","
        end
    end
    return nil
end

--- 位置 i の `\` から始まる単位（タグ・エスケープ・囲み）を読む
--- @param s string
--- @param i integer s:sub(i, i) == "\\" であること（呼び出し側の責任）
--- @return string|nil name タグ名（`\` を除く。数字付きは "s3"、囲みは "_?"・"_!"）。
---                         エスケープ・単位にならない `\` は nil
--- @return string|nil arg 角括弧の中身（外側の括弧を除く生の文字列。無ければ nil）
--- @return integer next_pos 単位の直後の位置（エスケープは i + 2、単位にならない `\` は i + 1）
--- @return string|nil literal 囲みの中身（囲みのときだけ。空の囲みは ""）
local function tag_at(s, i)
    local c1 = s:sub(i + 1, i + 1)
    if c1 == "\\" or c1 == "%" then
        return nil, nil, i + 2
    end
    local mark = s:sub(i, i + 2)
    if mark == "\\_?" or mark == "\\_!" then
        local e = s:find(mark, i + 3, true)
        if e then
            return mark:sub(2), nil, e + 3, s:sub(i + 3, e - 1)
        end
    end
    local name = s:match("^[spb]%d", i + 1) or s:match("^w[1-9]", i + 1)
    if name then
        return name, nil, i + 3
    end
    name = s:match("^_?_?[0-9A-Za-z!+*?&%-]", i + 1)
    if not name then
        return nil, nil, i + 1
    end
    local j = i + 1 + #name
    local e = s:sub(j, j) == "[" and args_end(s, j)
    if e then
        return name, s:sub(j + 1, e - 2), e
    end
    return name, nil, j
end

--- タグの読み取り（適合テスト tests/sakura_script/conformance_test.rs が使う）
APPEARANCE.tag_at = tag_at

--- pos 以降の次のタグ（囲みを含む）を返す（戻り値は tag_at の先頭 3 つ）。
--- 名前の無い単位は tag_at の次の位置で読み飛ばす。囲みの中は観測しない
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
        pos = next_pos
    end
end

--- 名前だけで決まるスコープ切替タグ（\0 \1 \h \u）
local SCOPE_NAMES = { ["0"] = true, ["1"] = true, h = true, u = true }

--- サーフェス変更なら ID を返す（\s[ID] / \sN）。名前は正確な一致で判定し、空 ID は無視する
local function surface_id(name, arg)
    local id = (name == "s" and arg) or name:match("^s(%d)$")
    if id ~= "" then
        return id
    end
    return nil
end

--- スコープ切替タグか（\0 \1 \h \u \p[N] \pN）。名前は正確な一致で判定する
local function is_scope(name, arg)
    return SCOPE_NAMES[name] or (name == "p" and arg ~= nil) or name:match("^p%d$") ~= nil
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
--- @return boolean 字（一般文字・エスケープ・中身のある囲み）・スコープ切替タグに達したら true（先頭タグ列の終端）
local function scan_leading_text(s, found)
    local pos = 1
    while pos <= #s do
        if s:sub(pos, pos) ~= "\\" then
            return true
        end
        local name, arg, next_pos, literal = tag_at(s, pos)
        if not name or (literal and literal ~= "") then
            return true -- エスケープ・単位にならない `\`・中身のある囲みは字
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

--- 先頭タグ列の明示 bind（カテゴリ・パーツ・数値 0/1 が明示）の集合 cat → part → true（3.10）。
--- トグル・カテゴリ単位 bind は含めない（抑止しない）
local function leading_explicit_binds(leading)
    local set = {}
    for _, tag in ipairs(leading) do
        if tag.kind == "bind" then
            local cat, part, val = parse_bind(tag.value)
            if cat and part ~= "" and BIND_VALUES[val] then
                set[cat] = set[cat] or {}
                set[cat][part] = true
            end
        end
    end
    return set
end

--- 既定着せ替えの 1 カテゴリ（非空文字列パーツで値 0/1 のもののみ。それ以外は無視）
local function default_parts(dressup, cat)
    local out = {}
    local d = type(dressup) == "table" and dressup[cat]
    if type(d) == "table" then
        for part, v in pairs(d) do
            if type(part) == "string" and part ~= "" and (v == 0 or v == 1) then
                out[part] = v == 1 and 1 or 0
            end
        end
    end
    return out
end

--- 復旧対象カテゴリ（既定と記録の和集合。非空文字列のみ）をバイト昇順で返す
local function dressup_categories(dressup, ac_binds)
    local seen, cats = {}, {}
    local function add(cat)
        if type(cat) == "string" and cat ~= "" and not seen[cat] then
            seen[cat] = true
            cats[#cats + 1] = cat
        end
    end
    if type(dressup) == "table" then
        for cat in pairs(dressup) do
            add(cat)
        end
    end
    for cat in pairs(ac_binds) do
        add(cat)
    end
    table.sort(cats)
    return cats
end

--- カテゴリの既知着せ替えを出力順の { part, value } 列で返す:
--- 全脱衣（part = ""）→ 既定パーツ（パーツ名昇順・記録で上書きされたものを除く）→ 記録パーツ（記録順）。
--- アクター側が不明（false）なら空、全脱衣済みなら既定を使わない（3.4, 3.8, 3.11）
local function known_dressup(dressup, ac, cat)
    local rec = ac and ac.binds[cat]
    if rec == false then
        return {}
    end
    rec = rec or {}
    local list = {}
    if rec[""] == 0 then
        list[1] = { part = "", value = 0 }
    else
        local defaults = default_parts(dressup, cat)
        local names = {}
        for part in pairs(defaults) do
            if rec[part] == nil then
                names[#names + 1] = part
            end
        end
        table.sort(names)
        for _, part in ipairs(names) do
            list[#list + 1] = { part = part, value = defaults[part] }
        end
    end
    for _, part in ipairs(ac and ac.order and ac.order[cat] or {}) do
        list[#list + 1] = { part = part, value = rec[part] }
    end
    return list
end

--- 既知着せ替えのいずれかがスポット側の実効値と不一致・不明か（全脱衣は part = "" で同じ規則）
local function dressup_differs(spot_cat, list)
    for _, e in ipairs(list) do
        if spot_effective(spot_cat, e.part) ~= e.value then
            return true
        end
    end
    return false
end

--- 全脱衣したアクター（rec[""] == 0）では、スポット側の他パーツもアクターの実効値と比較する
--- （3.11。他者が着けたパーツ・不明パーツを残さない）。全脱衣でないカテゴリは走査しない（3.5）
local function stripped_differs(rec, spot_cat)
    if not (rec and rec[""] == 0 and spot_cat) then
        return false
    end
    for q in pairs(spot_cat) do
        if q ~= "" and spot_effective(spot_cat, q) ~= (rec[q] or rec[""]) then
            return true
        end
    end
    return false
end

--- カテゴリを丸ごと bind-noevent で返し、スポット側を観測と同じ規則で更新する（3.2, 3.7, 3.10, 3.11）。
--- skip のパーツ（先頭タグ列の明示 bind）は出力しない。アクター側は変えない
local function emit_dressup(sp, cat, list, skip)
    local out = {}
    for _, e in ipairs(list) do
        if e.part == "" or not skip[e.part] then
            out[#out + 1] = "\\![bind-noevent," .. cat .. "," .. e.part .. "," .. e.value .. "]"
            if e.part == "" then
                reset_category(sp, nil, cat, true)
            else
                record_bind(sp, nil, cat, e.part, e.value)
            end
        end
    end
    return table.concat(out)
end

--- 着せ替え復旧タグ列を返し、スポットの適用中着せ替え状態を更新する（3.2, 3.3, 3.5, 3.8〜3.11）。
--- 走査はアクター側の集合のみ（スポット側にのみあるパーツには触れない。全脱衣カテゴリを除く）
local function restore_dressup(state, actor, spot, leading)
    local ac = state.actors[actor.name]
    local skip = leading_explicit_binds(leading)
    local out = {}
    for _, cat in ipairs(dressup_categories(actor.dressup, ac and ac.binds or {})) do
        local list = known_dressup(actor.dressup, ac, cat)
        local cur = state.spots[spot]
        local spot_cat = cur and cur.binds[cat]
        if dressup_differs(spot_cat, list) or stripped_differs(ac and ac.binds[cat], spot_cat) then
            out[#out + 1] = emit_dressup(entry(state.spots, spot), cat, list, skip[cat] or {})
        end
    end
    return table.concat(out)
end

--- アクター切替時の復旧タグ列を返し、スポットの表示中状態を更新する。
--- @param state table AppearanceState
--- @param actor table 切替先アクター（name / surface / dressup を参照）
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
    local leading = leading_tags(tokens)
    -- サーフェス復旧が先、着せ替え復旧が後（3.2）
    return restore_surface(state, actor, spot, leading) .. restore_dressup(state, actor, spot, leading)
end

return APPEARANCE
