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

return GLOBAL
