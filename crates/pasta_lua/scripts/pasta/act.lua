--- @module pasta.act
--- アクションオブジェクトモジュール
---
--- トランスパイラー出力のシーン関数から第1引数として受け取るオブジェクト。
--- トークン蓄積、アクタープロキシ動的生成、シーン制御を担当する。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")

--- @class Act アクションオブジェクト
--- @field ctx CTX 環境オブジェクト
--- @field var table アクションローカル変数
--- @field token table[] 構築中のスクリプトトークン
--- @field now_actor Actor|nil 現在のアクター
--- @field current_scene SceneTable|nil 現在のシーンテーブル
local ACT = {}

--- ACT実装メタテーブル
local ACT_IMPL = {}

--- __indexメタメソッド: メソッド検索とアクタープロキシ動的生成
--- @param self Act アクションオブジェクト
--- @param key string キー名
--- @return any メソッドまたはプロキシ
function ACT_IMPL.__index(self, key)
    -- 1. ACT_IMPLメソッドを検索
    local method = ACT_IMPL[key]
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
    return setmetatable(obj, ACT_IMPL)
end

--- シーン初期化（トランスパイラー出力から呼び出し）
--- @param self Act アクションオブジェクト
--- @param scene SceneTable SCENEテーブル
--- @return table save 永続変数テーブル
--- @return table var アクションローカル変数テーブル
function ACT_IMPL.init_scene(self, scene)
    self.current_scene = scene
    return self.ctx.save, self.var
end

--- talkトークン蓄積
--- @param self Act アクションオブジェクト
--- @param actor Actor アクターオブジェクト
--- @param text string 発話テキスト
--- @return nil
function ACT_IMPL.talk(self, actor, text)
    if self.now_actor ~= actor then
        table.insert(self.token, { type = "actor", actor = actor })
        self.now_actor = actor
    end
    table.insert(self.token, { type = "talk", text = text })
end

--- sakura_scriptトークン蓄積
--- @param self Act アクションオブジェクト
--- @param text string さくらスクリプト
--- @return nil
function ACT_IMPL.sakura_script(self, text)
    table.insert(self.token, { type = "sakura_script", text = text })
end

--- 単語検索（アクター非依存、4レベル検索）
--- 検索順序:
--- 1. シーンテーブル完全一致 (current_scene[name])
--- 2. GLOBAL完全一致 (GLOBAL[name])
--- 3. シーンローカル辞書前方一致 (SEARCH:search_word(name, scene_name))
--- 4. グローバル辞書前方一致 (SEARCH:search_word(name, nil))
--- @param self Act アクションオブジェクト
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function ACT_IMPL.word(self, name)
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
    if GLOBAL[name] ~= nil then
        local value = GLOBAL[name]
        return WORD.resolve_value(value, self)
    end

    -- 3 & 4. SEARCH API を使用した前方一致検索（利用可能な場合のみ）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if ok and SEARCH then
        -- 3. シーンローカル辞書（前方一致）
        local scene_name = self.current_scene and self.current_scene.__global_name__
        if scene_name then
            local result = SEARCH:search_word(name, scene_name)
            if result then
                return result -- SEARCH APIは既に文字列を返す
            end
        end

        -- 4. グローバル辞書（前方一致）
        local result = SEARCH:search_word(name, nil)
        if result then
            return result -- SEARCH APIは既に文字列を返す
        end
    end

    return nil
end

--- トークン出力とyield
--- @param self Act アクションオブジェクト
--- @return nil
function ACT_IMPL.yield(self)
    table.insert(self.token, { type = "yield" })
    self.ctx:yield(self)
end

--- アクション終了
--- @param self Act アクションオブジェクト
--- @return nil
function ACT_IMPL.end_action(self)
    table.insert(self.token, { type = "end_action" })
    self.ctx:end_action(self)
end

--- シーン呼び出し（4段階検索）
---
--- トランスパイラ出力から呼び出され、キーに対応するハンドラーを検索して実行する。
--- 4段階の優先順位に従い、最初に見つかった有効な関数を実行する。
---
--- @param self Act アクションオブジェクト
--- @param global_scene_name string|nil グローバルシーン名
--- @param key string 検索キー
--- @param attrs table|nil 属性テーブル（将来拡張用、現在は未使用）
--- @param ... any 可変長引数（ハンドラーに渡す）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
    local handler = nil

    -- Level 1: シーンローカル検索
    if self.current_scene then
        handler = self.current_scene[key]
    end

    -- Level 2: グローバルシーン名スコープ検索
    if not handler then
        local result = SCENE.search(key, global_scene_name, attrs)
        if result then
            handler = result.func
        end
    end

    -- Level 3: グローバル関数モジュール
    if not handler then
        handler = GLOBAL[key]
    end

    -- Level 4: スコープなし全体検索（フォールバック）
    if not handler then
        local result = SCENE.search(key, nil, attrs)
        if result then
            handler = result.func
        end
    end

    -- ハンドラー実行
    if handler then
        return handler(self, ...)
    end

    -- TODO: ハンドラー未発見時のログ出力（将来実装）
    return nil
end

--- スポット設定
--- @param self Act アクションオブジェクト
--- @param name string アクター名
--- @param number integer 位置
--- @return nil
function ACT_IMPL.set_spot(self, name, number)
    local actor = self.ctx.actors[name]
    if actor then
        actor.spot = number
    end
end

--- 全スポットクリア
--- @param self Act アクションオブジェクト
--- @return nil
function ACT_IMPL.clear_spot(self)
    for _, actor in pairs(self.ctx.actors) do
        actor.spot = nil
    end
end

--- 継承用に実装メタテーブルを公開
ACT.IMPL = ACT_IMPL

return ACT
