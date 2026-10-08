--- @module pasta
--- PASTA公開APIモジュール
---
--- トランスパイラー出力から呼び出される公開APIを提供する。
--- このモジュールはpasta言語ランタイムのエントリーポイントとなる。
--- 各機能モジュールへのリダイレクト点として設計されている。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local WORD = require("pasta.word")

--- @class Pasta 公開APIモジュール
local PASTA = {}

--- アクターを作成または取得する
--- @see pasta.actor.get_or_create
--- @type fun(name: string): Actor
PASTA.create_actor = ACTOR.get_or_create

--- シーンを登録し、グローバルシーンテーブルを返す
--- @see pasta.scene.create_scene
--- @type fun(base_name: string, local_name?: string, scene_func?: function): SceneTable
PASTA.create_scene = SCENE.create_scene

--- グローバル単語ビルダーを作成
--- @see pasta.word.create_word
--- @type fun(key: string): WordBuilder
PASTA.create_word = WORD.create_word

--- 算術・連結の被演算子の変換（生成コードが呼ぶ）。pasta.act の ACT.num・ACT.str と同一関数
---   PASTA.num(op, v, desc?) -> number : nil は 0、変換できない値は警告して 0
---   PASTA.str(v, desc?) -> string     : nil は ""、変換できない値は警告して ""
--- pasta.act は初めて参照したときに読み込む（この init.lua の読み込みで act.lua のロード順を変えない）
--- @see pasta.act.num
--- @see pasta.act.str
local ACT_REDIRECTS = { num = true, str = true }
setmetatable(PASTA, {
    __index = function(t, key)
        if not ACT_REDIRECTS[key] then return nil end
        local f = require("pasta.act")[key]
        rawset(t, key, f)
        return f
    end,
})

--- シーン辞書を最終化する（スタブ実装）
---
--- scene_dic.lua から呼び出される。現在はスタブ実装。
--- Rust側からregister_finalize_scene()で上書きされる。
--- @return nil
function PASTA.finalize_scene()
    -- Stub implementation: do nothing
    -- This function is overwritten by Rust's register_finalize_scene()
end

return PASTA
