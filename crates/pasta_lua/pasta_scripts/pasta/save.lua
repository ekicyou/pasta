--- @module pasta.save
--- 永続化データモジュール
---
--- 最初に require された時点で @pasta_persistence.load() により読み込まれ、セッションを跨いで保持される。
--- act.save（act:init_scene の戻り値 save）から参照可能。ランタイムの Drop 時に自動保存される。

local persistence = require("@pasta_persistence")

--- 永続化データをロードして返す
--- @return table 永続化データテーブル
local save = persistence.load()

return save
