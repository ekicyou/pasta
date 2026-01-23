--- @module pasta.act
--- アクションオブジェクトモジュール
---
--- トランスパイラー出力のシーン関数から第1引数として受け取るオブジェクト。
--- トークン蓄積、アクタープロキシ動的生成、シーン制御を担当する。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")

--- @class Act アクションオブジェクト
--- @field ctx CTX 環境オブジェクト
--- @field var table アクションローカル変数
--- @field token table[] 構築中のスクリプトトークン
--- @field now_actor Actor|nil 現在のアクター
--- @field current_scene table|nil 現在のシーンテーブル
local ACT = {}

--- __indexメタメソッド: アクタープロキシ動的生成
--- @param key string
--- @return any
function ACT:__index(key)
    -- アクター名としてプロキシ生成
    local actor = self.ctx.actors[key]
    if actor then
        return ACTOR.create_proxy(actor, self)
    end

    -- フォールバック：ACTメソッド
    return ACT[key]
end

--- 新規Actを作成
--- @param ctx CTX 環境オブジェクト
--- @return Act アクションオブジェクト
function ACT.new(ctx)
    local obj = {
        ctx = ctx,
        var = {},
        token = {},
        now_actor = nil,
        current_scene = nil,
    }
    setmetatable(obj, ACT)
    return obj
end

--- シーン初期化（トランスパイラー出力から呼び出し）
--- @param scene table SCENEテーブル
--- @return table save 永続変数テーブル
--- @return table var アクションローカル変数テーブル
function ACT:init_scene(scene)
    self.current_scene = scene
    return self.ctx.save, self.var
end

--- talkトークン蓄積
--- @param actor Actor アクターオブジェクト
--- @param text string 発話テキスト
function ACT:talk(actor, text)
    if self.now_actor ~= actor then
        table.insert(self.token, { type = "actor", actor = actor })
        self.now_actor = actor
    end
    table.insert(self.token, { type = "talk", text = text })
end

--- sakura_scriptトークン蓄積
--- @param text string さくらスクリプト
function ACT:sakura_script(text)
    table.insert(self.token, { type = "sakura_script", text = text })
end

--- 単語検索（アクター非依存、3レベル検索）
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function ACT:word(name)
    -- Level 2: SCENEfield
    if self.current_scene and self.current_scene[name] then
        return self.current_scene[name]
    end
    -- Level 3: グローバルシーン名での検索（Rust関数呼び出し予定）
    -- Level 4: 全体検索（Rust関数呼び出し予定）
    return nil -- TODO: Rust search_word 統合
end

--- トークン出力とyield
function ACT:yield()
    table.insert(self.token, { type = "yield" })
    self.ctx:yield(self)
end

--- アクション終了
function ACT:end_action()
    table.insert(self.token, { type = "end_action" })
    self.ctx:end_action(self)
end

--- シーン呼び出し
--- @param search_result table {global_name, local_name}
--- @param opts table|nil オプション
function ACT:call(search_result, opts, ...)
    local global_name, local_name = search_result[1], search_result[2]
    local scene_func = SCENE.get(global_name, local_name)
    if scene_func then
        scene_func(self, ...)
    end
end

--- スポット設定
--- @param name string アクター名
--- @param number integer 位置
function ACT:set_spot(name, number)
    local actor = self.ctx.actors[name]
    if actor then
        actor.spot = number
    end
end

--- 全スポットクリア
function ACT:clear_spot()
    for _, actor in pairs(self.ctx.actors) do
        actor.spot = nil
    end
end

return ACT
