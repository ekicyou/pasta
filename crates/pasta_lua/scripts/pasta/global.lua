--- @module pasta.global
--- グローバル関数モジュール
---
--- ユーザー定義のグローバル関数を格納するテーブル。
--- main.lua等から関数を追加することで、単語参照時にL5で検索される。
---
--- 使用例（ユーザーがmain.lua等で定義）:
---   local GLOBAL = require("pasta.global")
---   function GLOBAL.時報(act)
---       return "正午です"
---   end

--- @class Global グローバル関数テーブル
local GLOBAL = {}

--- チェイントーク（継続トーク）関数
--- act:yield() を呼び出し、蓄積トークンを中間出力として返す。
--- Pasta DSL の `＞チェイントーク` で呼び出される。
--- @param act Act ACT オブジェクト
function GLOBAL.チェイントーク(act)
    act:yield()
end

--- yield（継続トーク）関数
--- チェイントーク と同一動作。
--- Pasta DSL の `＞yield` で呼び出される。
--- @param act Act ACT オブジェクト
function GLOBAL.yield(act)
    act:yield()
end

return GLOBAL
