--- @module pasta.shiori.actor_style
--- アクターの装い（`actor.font`）の出し入れモジュール（actor-display-style）
---
--- 話者の切り替えで出す戻すタグ（`\f[default]`）と装いのタグを返し（switch）、
--- 内側のトークンの変換後の文字列で、引数の無い `\c`・`\x` の直後に装いを出し直す（through）。
--- いずれも引数で受け取ったビルドごとの状態テーブルをその場で更新する。
--- モジュール自身は状態を持たず、STORE・@pasta_* を require しない。
--- タグの読み取りは pasta.shiori.appearance の tag_at だけを使う。

local APPEARANCE = require("pasta.shiori.appearance")

local STYLE = {}

local RESET = "\\f[default]"

--- 装いのタグ列。`actor.font` が空でない文字列のときだけ返す（それ以外は装い無し）
--- @param actor table|nil
--- @return string|nil
local function font_of(actor)
    local font = actor and actor.font
    if type(font) == "string" and font ~= "" then
        return font
    end
    return nil
end

--- ビルドごとの空の装いの状態を生成する
--- @return table ActorStyleState { used, styled, any }
function STYLE.new()
    return { used = {}, styled = {}, any = false }
end

--- 話者の切り替えで出すタグ（`\p[spot]` と立ち絵の復旧の後に置く）
--- @param state table ActorStyleState（その場で更新する）
--- @param actor table 切り替え先アクター
--- @param spot integer 解決済みスポット
--- @return string 戻すタグと装いのタグ。何も要らなければ ""
function STYLE.switch(state, actor, spot)
    local font = font_of(actor)
    local out = ""
    if font then
        out = (state.used[spot] and RESET or "") .. font
        state.styled[spot] = true
        state.any = true
    elseif state.styled[spot] then
        out = RESET
    end
    state.used[spot] = true
    return out
end

--- 内側のトークンを変換した文字列を、出力の前に通す
--- @param state table ActorStyleState（その場で更新する）
--- @param actor table|nil グループの話者（nil は話し手なしのグループ）
--- @param spot integer|nil 現在のスポット
--- @param s string 変換後の文字列
--- @return string 装いのあるアクターなら引数の無い `\c`・`\x` の直後に装いを差し込んだ文字列。それ以外は s
function STYLE.through(state, actor, spot, s)
    if not state.any or not actor then
        return s
    end
    local font = font_of(actor)
    local parts, last, pos = {}, 1, 1
    while true do
        local i = s:find("\\", pos, true)
        if not i then
            break
        end
        local name, arg, next_pos = APPEARANCE.tag_at(s, i)
        if (name == "c" or name == "x") and arg == nil then
            if font then
                parts[#parts + 1] = s:sub(last, next_pos - 1) .. font
                last = next_pos
                if name == "x" then
                    state.used[0], state.styled[0] = true, true
                end
            elseif name == "c" and spot ~= nil then
                state.used[spot], state.styled[spot] = false, false
            end
        end
        pos = next_pos
    end
    if last == 1 then
        return s
    end
    parts[#parts + 1] = s:sub(last)
    return table.concat(parts)
end

return STYLE
