--- @module pasta
--- PASTA公開APIモジュール
---
--- トランスパイラー出力から呼び出される公開APIを提供する。
--- このモジュールはpasta言語ランタイムのエントリーポイントとなる。

local CTX = require("pasta.ctx")
local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")

local PASTA = {}

--- アクターを作成または取得する
--- @param name string アクター名
--- @return Actor アクターオブジェクト
function PASTA.create_actor(name)
    return ACTOR.get_or_create(name)
end

--- シーンを登録し、グローバルシーンテーブルを返す
--- @param global_name string グローバルシーン名（ファイル名）
--- @param local_name string|nil ローカルシーン名（シーン関数名）
--- @param scene_func function|nil シーン関数
--- @return table グローバルシーンテーブル
function PASTA.create_scene(global_name, local_name, scene_func)
    if scene_func and local_name then
        SCENE.register(global_name, local_name, scene_func)
    end
    return SCENE.get_global_table(global_name) or SCENE.create_global_table(global_name)
end

--- シーン辞書を最終化する（スタブ実装）
---
--- scene_dic.lua から呼び出される。現在は警告ログのみ出力。
--- 本実装（シーン辞書構築ロジック）は別仕様で行う。
function PASTA.finalize_scene()
    -- Stub implementation: do nothing
    -- Full implementation will build scene dictionary from registered scenes
end

--- CTXモジュールを公開（ランタイム用）
PASTA.CTX = CTX

return PASTA
