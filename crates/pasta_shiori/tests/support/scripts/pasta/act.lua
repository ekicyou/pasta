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

--- __indexメタメソッド: メソッド検索とアクタープロキシ動的生成
--- @param key string
--- @return any
function ACT:__index(key)
    -- 1. ACTメソッドを検索
    local method = ACT[key]
    if method then return method end

    -- 2. アクター名としてプロキシ生成
    local actor = self.ctx.actors[key]
    if actor then
        return ACTOR.create_proxy(actor, self)
    end

    return nil
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

--- raw_scriptトークン蓄積
--- @param text string 生スクリプト文字列
function ACT:raw_script(text)
    table.insert(self.token, { type = "raw_script", text = text })
end

--- 単語検索（アクター非依存、4レベル検索）
--- 検索順序:
--- 1. シーンテーブル完全一致 (current_scene[name])
--- 2. GLOBAL完全一致 (GLOBAL[name])
--- 3. シーンローカル辞書前方一致 (SEARCH:search_word)
--- 4. グローバル辞書前方一致 (SEARCH:search_word)
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function ACT:word(name)
    if not name or name == "" then
        return nil
    end

    local WORD = require("pasta.word")

    -- 1. シーンテーブル完全一致
    if self.current_scene and self.current_scene[name] ~= nil then
        local value = self.current_scene[name]
        return WORD.resolve_value(value, self)
    end

    -- 2. GLOBAL完全一致
    local GLOBAL = require("pasta.global")
    if GLOBAL[name] ~= nil then
        local value = GLOBAL[name]
        return WORD.resolve_value(value, self)
    end

    -- 3 & 4. SEARCH APIを通じた前方一致検索（利用可能な場合）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if ok and SEARCH then
        -- 3. シーンローカル辞書
        local scene_name = self.current_scene and self.current_scene.__global_name__
        if scene_name then
            local result = SEARCH:search_word(name, scene_name)
            if result then
                return result
            end
        end

        -- 4. グローバル辞書
        local result = SEARCH:search_word(name, nil)
        if result then
            return result
        end
    end

    return nil
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
