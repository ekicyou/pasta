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
-- PROXY:word 3レベルフォールバック検索
-------------------------------------------

--- word（3レベルフォールバック検索）
--- 検索順序:
--- 1. アクター完全一致 (actor[name])
--- 2. アクター辞書前方一致 (SEARCH:search_word)
--- 3. act:word() に委譲（シーン→グローバル検索）
--- @param name string 単語名（＠なし）
--- @return string|nil 見つかった単語、またはnil
function PROXY:word(name)
    if not name or name == "" then
        return nil
    end

    -- 1. アクター完全一致（関数 or 値）
    local actor_value = self.actor[name]
    if actor_value ~= nil then
        return WORD.resolve_value(actor_value, self.act)
    end

    -- 2 & 3. SEARCH APIを通じた前方一致検索 or act:word() に委譲
    local ok, SEARCH = pcall(require, "@pasta_search")
    if ok and SEARCH then
        -- 2. アクター辞書（前方一致）
        local actor_scope = "__actor_" .. self.actor.name .. "__"
        local result = SEARCH:search_word(name, actor_scope)
        if result then
            return result
        end
    end

    -- 3. act:word() に委譲（シーン→グローバル検索）
    return self.act:word(name)
end

return ACTOR
