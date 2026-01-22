--- @module pasta
--- PASTA公開APIモジュール
---
--- トランスパイラー出力から呼び出される公開APIを提供する。
--- このモジュールはpasta言語ランタイムのエントリーポイントとなる。
--- 各機能モジュールへのリダイレクト点として設計されている。

local CTX = require("pasta.ctx")
local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local WORD = require("pasta.word")

local PASTA = {}

--- アクターを作成または取得する
--- @see pasta.actor.get_or_create
PASTA.create_actor = ACTOR.get_or_create

--- シーンを登録し、グローバルシーンテーブルを返す
--- @see pasta.scene.create_scene
PASTA.create_scene = SCENE.create_scene

--- グローバル単語ビルダーを作成
--- @see pasta.word.create_word
PASTA.create_word = WORD.create_word

--- シーン辞書を最終化する（スタブ実装）
---
--- scene_dic.lua から呼び出される。現在はスタブ実装。
--- Rust側からregister_finalize_scene()で上書きされる。
function PASTA.finalize_scene()
    -- Stub implementation: do nothing
    -- This function is overwritten by Rust's register_finalize_scene()
end

--- サブモジュール参照（内部使用・高度な用途向け）
PASTA.CTX = CTX
PASTA.ACTOR = ACTOR
PASTA.SCENE = SCENE
PASTA.WORD = WORD

return PASTA
