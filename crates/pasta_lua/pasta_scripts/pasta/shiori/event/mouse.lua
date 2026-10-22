---@module pasta.shiori.event.mouse
--- つつき・撫での既定ハンドラの部品
---
--- 候補の名前（M.candidates）: 種類・アクター名・部位から 1〜8 段の名前を上から順に並べる。
--- アクター名（M.resolve_actor）: スコープ番号（Reference3）から pasta.toml のアクター名を引く。
--- 撫での判定（M.check_stroke）: 同じ所で途切れなく 2 秒続いた移動を 1 回の「撫でた」と決める。
--- 既定のハンドラ: REG.OnMouseDoubleClick（つつき）・REG.OnMouseMove（撫で）。
--- ゴーストが先に登録していれば上書きしない。event/init.lua は require しない（循環させない）。

local STORE = require("pasta.store")
local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
local CLOCK = require("pasta.shiori.clock")
local log = require("@pasta_log")

local M = {}

--- 種類ごとの接頭辞と英字
local PREFIX = {
    poke = { "つっつき反応", "OnPoke" },
    touch = { "さわり反応", "OnTouch" },
}

--- 候補の名前（1〜8 段のうち飛ばさなかったもの。上の段から順）
--- 部位は加工せずに連結する。部位が その他・Other でも特別扱いしない。
---@param kind "poke"|"touch"
---@param actor string|nil アクター名（決まらなければ nil）
---@param part string|nil 部位（空文字列・nil は「部位なし」）
---@return string[]
function M.candidates(kind, actor, part)
    local ja, en = PREFIX[kind][1], PREFIX[kind][2]
    if part == "" then part = nil end
    local list = {}
    local function add(a, b)
        list[#list + 1] = a
        list[#list + 1] = b
    end
    if actor and part then add(ja .. actor .. part, en .. actor .. part) end
    if actor then add(ja .. actor .. "その他", en .. actor .. "Other") end
    if part then add(ja .. part, en .. part) end
    add(ja .. "その他", en .. "Other")
    return list
end

--- スコープ番号からアクター名を引く
--- spot が 1 人ならその名前、0 人なら nil、2 人以上なら そのスポットで最後に話した人
--- （STORE.appearance.owners）が集めた中にいればその名前。spot は読み込み時の値を見る。
---@param scope_ref string|nil Reference3 の生の値
---@return string|nil
function M.resolve_actor(scope_ref)
    local scope = tonumber(scope_ref)
    if not scope then return nil end
    local ok, config = pcall(require, "@pasta_config")
    if not ok or type(config) ~= "table" or type(config.actor) ~= "table" then return nil end

    local found = {}
    for name, actor in pairs(config.actor) do
        if type(actor) == "table" and actor.spot == scope then
            found[#found + 1] = name
        end
    end
    if #found == 1 then return found[1] end

    local owners = STORE.appearance and STORE.appearance.owners
    local owner = owners and owners[scope]
    for _, name in ipairs(found) do
        if name == owner then return name end
    end
    return nil
end

--- 撫での判定の定数（秒）。設定では変えない
local THRESHOLD = 2 -- 数え始めからこれ以上で「撫でた」
local GAP = 2       -- 前の移動からこれより長いと途切れ

--- 撫での判定の状態（1 つだけ・保存しない）。target が nil なら Off
local state = { target = nil, start = 0, last = 0, fired = false }

--- テスト用: 状態を Off に戻す
function M._reset()
    state = { target = nil, start = 0, last = 0, fired = false }
end

--- テスト用: 状態を読む
---@return { target: string|nil, start: number, last: number, fired: boolean }
function M._get_state()
    return state
end

--- 撫での判定
---@param scope_ref string|nil Reference3
---@param part string|nil Reference4
---@param now number|nil リクエストの時刻（秒・小数つき）。nil なら数え直す
---@param blocked boolean 抑止の状態か
---@return boolean 今回「撫でた」と決めたら true（1 回だけ）
function M.check_stroke(scope_ref, part, now, blocked)
    if blocked or part == nil or part == "" or now == nil then
        M._reset()
        return false
    end
    local target = tostring(scope_ref) .. "\t" .. part
    if target ~= state.target or now - state.last > GAP or now < state.last then
        state = { target = target, start = now, last = now, fired = false }
        return false
    end
    state.last = now
    if state.fired then return false end
    if now - state.start >= THRESHOLD then
        state.fired = true
        return true
    end
    return false
end

--- 候補の名前を上から完全一致で探し、最初に見つかったシーンのコルーチンと名前を返す
---@return thread|nil, string|nil
local function find_shortcut(kind, actor, part)
    local SCENE = require("pasta.scene")
    for _, name in ipairs(M.candidates(kind, actor, part)) do
        local co = SCENE.co_exec_exact(name)
        if co then return co, name end
    end
    return nil, nil
end

--- イベント名のシーン（9 段目。今の no_entry と同じ呼び出し）
local function exec_event_scene(act, event)
    return require("pasta.scene").co_exec(act, event, nil, nil)
end

--- 秒より細かいリクエストの時刻（なめらかな時計。date が無ければ nil）
local function request_time(req)
    local date = req.date
    if type(date) ~= "table" or date.unix == nil then return nil end
    return CLOCK.steady(req) + (date.ns or 0) / 1e9
end

--- つつき: 抑止は見ない。Reference5・6 は読まない。
--- 自動トークを止めている間は 1〜8 段を探さず 9 段目だけ。
---@param act ShioriAct
---@return thread|nil
local function on_poke(act)
    if STORE.auto_talk_paused then
        return exec_event_scene(act, "OnMouseDoubleClick")
    end
    local ref = act.req.reference or {}
    local actor = M.resolve_actor(ref[3])
    local co, name = find_shortcut("poke", actor, ref[4])
    if co then
        log.debug(string.format("つつき: %s/%s → %s", tostring(ref[3]), tostring(ref[4]), name))
        return co
    end
    return exec_event_scene(act, "OnMouseDoubleClick")
end

--- 撫で: 判定は毎回回す。決めたときだけアクター名を引いて 1〜8 段を探す。
--- 自動トークを止めている間は抑止と同じく決めない（判定の状態は Off）。
---@param act ShioriAct
---@return thread|nil
local function on_touch(act)
    local req = act.req
    local ref = req.reference or {}
    log.trace("OnMouseMove")
    if M.check_stroke(ref[3], ref[4], request_time(req),
            STORE.auto_talk_paused or dispatcher.is_blocked(req.status)) then
        local co, name = find_shortcut("touch", M.resolve_actor(ref[3]), ref[4])
        log.debug(string.format("撫で: %s/%s → %s", tostring(ref[3]), tostring(ref[4]), tostring(name)))
        if co then return co end
    end
    return exec_event_scene(act, "OnMouseMove")
end

REG.OnMouseDoubleClick = REG.OnMouseDoubleClick or on_poke
REG.OnMouseMove = REG.OnMouseMove or on_touch

return M
