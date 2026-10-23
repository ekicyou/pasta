--- @module pasta.shiori.event.register
--- イベントハンドラ登録テーブル
---
--- ゴースト開発者がイベントハンドラを登録するためのテーブル。
--- 依存モジュールなし（循環参照回避のため）。
---
--- ハンドラシグネチャ（EVENT.fire は act オブジェクトを渡す）:
---   function(act: ShioriAct) -> string|thread|nil
---
--- act.req テーブル構造（Rust側 parse_request() により生成）:
---   - req.id: イベント名（例: "OnBoot", "OnClose"）
---   - req.method: "get" | "notify"
---   - req.version: 30（SHIORI/3.0）
---   - req.charset: 文字セット（例: "UTF-8"）
---   - req.sender: 送信者名（例: "SSP"）
---   - req.reference: 参照テーブル（reference[0], reference[1], ...）
---   - req.dic: 全ヘッダー辞書
---
--- 注意: act.req テーブルはread-only契約。ハンドラ内で変更しないこと。
---
--- 戻り値の規則（EVENT.fire が応答にする）:
---   - "SHIORI/" で始まる文字列: RES で作った応答全体としてそのまま返す（204・311・312・500 も可）
---   - それ以外の文字列: Value の 200（空文字列なら 204）
---   - nil: 204
---   - thread: EVENT.drive で再開し、出力を接頭辞にかかわらず常に Value の 200 にする（出力が無ければ 204）
---
--- 使用例:
--- ```lua
--- local REG = require("pasta.shiori.event.register")
---
--- -- 返した文字列は Value の 200 になる（SHIORI/ で始まる文字列は応答全体としてそのまま返る）
--- REG.OnBoot = function(act)
---     return [[\0\s[0]こんにちは\e]]
--- end
---
--- REG.OnClose = function(act)
---     return [[\0\s[0]さようなら\e]]
--- end
---
--- REG.OnMouseDoubleClick = function(act)
---     return [[\0\s[0]なあに？\e]]
--- end
--- ```
---
--- 重ねて登録する（鎖）:
---   REG:register(id, fn) は fn をイベント id の鎖に積み、その登録を外す関数を返す。
---   REG.X への代入（鎖の底）とは別の置き場所で、代入も pairs(REG) も鎖に触れない。
---   EVENT.fire は鎖を新しく登録した順に呼び、全部見送れば REG[id] → 同名のシーンへ回る。
---
--- 鎖のハンドラの返り値の読み方:
---   - nil・false（何も返さない）: 見送って次（1 つ前に登録したもの）へ回る
---   - true: そこで止めて 204
---   - 文字列・thread: そこで止めて、上の「戻り値の規則」のとおり（"" も止まって 204）
---   - ほかの真の値（数・表など）: そこで止めて 204
---
--- ```lua
--- local off = REG:register("OnSecondChange", function(act)
---     if SAVE["初回待ち"] then return true end  -- 一時的な門
--- end)
--- off()  -- 外す（2 回目以降は何もしない）
--- ```
---
--- 注意: REG.register に代入すると、生のキーが先に引かれて登録の口が隠れる。

--- @type table<string, fun(act: table): string|thread|nil>
local REG = {}

--- ID → 登録の目印 { fn = fn } の配列（古い順）。読み直せば空になる
local chains = {}

--- 鎖に fn を積み、自分の登録だけを外す関数を返す（REG:register(id, fn) で呼ぶ）
--- @param id string イベントかリソースの ID
--- @param fn fun(act: table): any
--- @return fun() 外す関数
local function register(_, id, fn)
    if type(id) ~= "string" then
        error("REG:register: id must be a string (call as REG:register(id, fn))", 2)
    end
    if type(fn) ~= "function" then
        error("REG:register: fn must be a function", 2)
    end
    local list = chains[id]
    if not list then
        list = {}
        chains[id] = list
    end
    -- 登録ごとに新しい目印にして、同じ関数の 2 回の登録を別々に外せるようにする
    local entry = { fn = fn }
    list[#list + 1] = entry
    return function()
        for i = #list, 1, -1 do
            if list[i] == entry then
                table.remove(list, i)
                return
            end
        end
    end
end

--- 鎖を新しい順に並べた写し（EVENT.fire だけが使う内部の口）。毎回新しい配列を返す
--- @param id string|nil
--- @return function[]
local function handlers(id)
    local out = {}
    local list = id ~= nil and chains[id]
    if list then
        for i = #list, 1, -1 do
            out[#out + 1] = list[i].fn
        end
    end
    return out
end

-- __index は register だけを見せ、handlers は REG.X から見えない欄に置く
setmetatable(REG, { __index = { register = register }, handlers = handlers })

return REG
