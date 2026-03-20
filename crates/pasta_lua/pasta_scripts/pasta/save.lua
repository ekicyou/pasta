--- @module pasta.save
--- 永続化データモジュール
---
--- ランタイム起動時に自動ロードされ、セッションを跨いで保持される。
--- ctx.saveから参照可能。Drop時に自動保存される。

local persistence = require("@pasta_persistence")

--- 永続化データをロードして返す
--- @return table 永続化データテーブル
local save = persistence.load()

return save
