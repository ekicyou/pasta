--- @module pasta.global
--- グローバル関数モジュール
---
--- ユーザー定義のグローバル関数を格納するテーブル。
--- main.lua等から関数を追加することで、単語参照・Call時にL4（GLOBAL[key] の完全一致）で検索される。
---
--- 使用例（ユーザーがmain.lua等で定義）:
---   local GLOBAL = require("pasta.global")
---   function GLOBAL.時報(act)
---       return "正午です"
---   end

--- @class Global グローバル関数テーブル
--- @field [string] fun(act: Act, ...): any
local GLOBAL = {}

local FAILURE = require("pasta.failure")

--- yield（継続トーク）関数
--- チェイントーク と同一動作。
--- Pasta DSL の `＞yield` で呼び出される。
--- @param act Act ACT オブジェクト
function GLOBAL.yield(act)
    act:yield()
end

-- ブラケット記法を使用: 非 ASCII 識別子（GLOBAL.チェイントーク）は LuaJIT 拡張のため、
-- 標準 Lua パーサー（luacheck 等の静的解析）が解釈できる等価表記で定義する。
GLOBAL["チェイントーク"] = GLOBAL.yield

--- シーン数の関数を作る（表記とログには fn_name をそのまま出す）
--- 2 つ目より後の引数は使わない: Call の形は呼んだシーンの引数を後ろに足すため、作者の書いた余分な引数と区別できない。
---@param fn_name string 関数の名前
---@param method string `@pasta_search` のメソッド名
---@return fun(act: table|nil, name: any, ...): integer
local function scene_counter(fn_name, method)
    return function(act, name)
        if type(name) == "number" then return 0 end
        if not FAILURE.is_name(name) then
            FAILURE.report(act, {
                label = FAILURE.LABEL.BAD_NAME,
                subject = fn_name,
                detail = FAILURE.kind_of(name),
                warning = fn_name .. ": bad argument: name=" .. FAILURE.quoted(name),
            })
            return 0
        end
        -- 呼んだ時点で引く（SCENE.search と同じ。読み込みの最後に登録されるため）
        local SEARCH = require("@pasta_search")
        return SEARCH[method](SEARCH, name)
    end
end

--- シーン数（前方一致・完全一致）。`＞名前` のグローバル検索が選べる候補の数
GLOBAL.scene_count = scene_counter("scene_count", "count_scene")
GLOBAL["シーン数"] = scene_counter("シーン数", "count_scene")
GLOBAL.scene_count_exact = scene_counter("scene_count_exact", "count_scene_exact")
GLOBAL["シーン数完全一致"] = scene_counter("シーン数完全一致", "count_scene_exact")

return GLOBAL
