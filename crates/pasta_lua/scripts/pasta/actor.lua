--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
--- アクターはキャッシュされ、同名のアクターは同一オブジェクトを返す。

local STORE = require("pasta.store")
local WORD = require("pasta.word")
local GLOBAL = require("pasta.global")

--- @class Actor アクターオブジェクト
--- @field name string アクター名
--- @field spot integer|nil 立ち位置（0以上）
local ACTOR = {}
ACTOR.__index = ACTOR

-------------------------------------------
-- ActorWordBuilder - アクター単語ビルダー
-------------------------------------------

--- ActorWordBuilderクラス（WordBuilderを拡張）
--- word.lua辞書への登録とACTORプロパティへの設定を同時に行う
--- @class ActorWordBuilder
--- @field _actor Actor アクターオブジェクト
--- @field _key string 単語キー
--- @field _word_builder WordBuilder 内部のWordBuilder
local ActorWordBuilder = {}
ActorWordBuilder.__index = ActorWordBuilder

--- 値を追加（辞書登録＋ACTORプロパティ設定）
--- @vararg string 可変長引数で値を受け取る
--- @return ActorWordBuilder メソッドチェーン用に自身を返す
function ActorWordBuilder:entry(...)
    local values = { ... }
    if #values > 0 then
        -- word.lua辞書に登録（L2前方一致用）
        self._word_builder:entry(...)

        -- ACTORプロパティに値を追加（L1完全一致用）
        if not self._actor[self._key] then
            self._actor[self._key] = {}
        end
        for _, v in ipairs(values) do
            table.insert(self._actor[self._key], v)
        end
    end
    return self
end

--- アクター単語ビルダーを作成（ACTOR:create_word("key") 形式）
--- @param self Actor アクターオブジェクト
--- @param key string 単語キー
--- @return ActorWordBuilder ビルダーオブジェクト
function ACTOR:create_word(key)
    local builder = setmetatable({}, ActorWordBuilder)
    builder._actor = self
    builder._key = key
    builder._word_builder = WORD.create_actor(self.name, key)
    return builder
end

--- アクターを取得または新規作成
--- @param name string アクター名
--- @return Actor アクターオブジェクト
function ACTOR.get_or_create(name)
    if not STORE.actors[name] then
        local actor = {
            name = name,
            spot = nil,
        }
        setmetatable(actor, ACTOR)
        STORE.actors[name] = actor
    end
    return STORE.actors[name]
end

--- @class ActorProxy アクタープロキシ（actへの逆参照付き）
--- @field actor Actor アクターオブジェクト
--- @field act Act アクションオブジェクト
local PROXY = {}
PROXY.__index = PROXY

--- プロキシを作成
--- @param actor Actor アクターオブジェクト
--- @param act Act アクションオブジェクト
--- @return ActorProxy アクタープロキシ
function ACTOR.create_proxy(actor, act)
    local proxy = {
        actor = actor,
        act = act,
    }
    setmetatable(proxy, PROXY)
    return proxy
end

--- talk（act経由でトークン蓄積）
--- @param text string 発話テキスト
function PROXY:talk(text)
    self.act:talk(self.actor, text)
end

-------------------------------------------
-- 6レベルフォールバック検索ヘルパー
-------------------------------------------

--- 値を解決（関数なら実行、配列なら最初の要素、その他はそのまま）
--- @param value any 検索結果
--- @param act Act アクションオブジェクト
--- @return any 解決後の値
local function resolve_value(value, act)
    if value == nil then
        return nil
    elseif type(value) == "function" then
        return value(act)
    elseif type(value) == "table" then
        -- 配列なら最初の要素を返す（完全一致の場合）
        if #value > 0 then
            return value[1]
        end
        return nil
    else
        return tostring(value)
    end
end

--- 辞書から前方一致検索
--- @param dict table {key → values[][]} 形式の辞書
--- @param prefix string 検索プレフィックス
--- @return table|nil マッチした全候補値の配列、またはnil
local function search_prefix_lua(dict, prefix)
    if not dict or prefix == "" then
        return nil
    end

    local results = {}
    for key, value_arrays in pairs(dict) do
        if key:sub(1, #prefix) == prefix then
            -- value_arrays は [[値1, 値2], [値3]] 形式
            for _, values in ipairs(value_arrays) do
                for _, v in ipairs(values) do
                    table.insert(results, v)
                end
            end
        end
    end
    return #results > 0 and results or nil
end

-------------------------------------------
-- PROXY:word 6レベルフォールバック検索
-------------------------------------------

--- word（6レベルフォールバック検索）
--- @param name string 単語名（＠なし）
--- @return string|nil 見つかった単語、またはnil
function PROXY:word(name)
    if not name or name == "" then
        return nil
    end

    -- Level 1: アクター完全一致（関数 or 値）
    local actor_value = self.actor[name]
    if actor_value ~= nil then
        return resolve_value(actor_value, self.act)
    end

    -- Level 2: アクター辞書（前方一致）
    local actor_dict = WORD.get_actor_words(self.actor.name)
    if actor_dict then
        local candidates = search_prefix_lua(actor_dict, name)
        if candidates and #candidates > 0 then
            return candidates[math.random(#candidates)]
        end
    end

    -- Level 3: シーン完全一致（関数 or 値）
    local scene = self.act.current_scene
    if scene then
        local scene_value = scene[name]
        if scene_value ~= nil then
            return resolve_value(scene_value, self.act)
        end

        -- Level 4: シーン辞書（前方一致）
        local scene_name = scene.__global_name__ or scene.name
        local scene_dict = WORD.get_local_words(scene_name)
        if scene_dict then
            local candidates = search_prefix_lua(scene_dict, name)
            if candidates and #candidates > 0 then
                return candidates[math.random(#candidates)]
            end
        end
    end

    -- Level 5: グローバル完全一致（関数 or 値）
    local global_value = GLOBAL[name]
    if global_value ~= nil then
        return resolve_value(global_value, self.act)
    end

    -- Level 6: グローバル辞書（前方一致）
    local global_dict = WORD.get_global_words()
    local candidates = search_prefix_lua(global_dict, name)
    if candidates and #candidates > 0 then
        return candidates[math.random(#candidates)]
    end

    return nil
end

return ACTOR
