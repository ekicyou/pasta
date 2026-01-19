--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
--- アクターはキャッシュされ、同名のアクターは同一オブジェクトを返す。

--- @class Actor アクターオブジェクト
--- @field name string アクター名
--- @field spot integer|nil 立ち位置（0以上）
local ACTOR = {}
ACTOR.__index = ACTOR

--- アクターキャッシュ（名前→アクター）
local actor_cache = {}

--- アクターを取得または新規作成
--- @param name string アクター名
--- @return Actor アクターオブジェクト
function ACTOR.get_or_create(name)
    if not actor_cache[name] then
        local actor = {
            name = name,
            spot = nil,
        }
        setmetatable(actor, ACTOR)
        actor_cache[name] = actor
    end
    return actor_cache[name]
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

--- word（4レベル検索）
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function PROXY:word(name)
    -- Level 1: アクターfield
    local actor_value = rawget(self.actor, name)
    if actor_value then
        return actor_value
    end

    -- Level 2: SCENEfield
    local scene = self.act.current_scene
    if scene and scene[name] then
        return scene[name]
    end

    -- Level 3: グローバルシーン名での検索（Rust関数呼び出し予定）
    -- Level 4: 全体検索（Rust関数呼び出し予定）
    return nil -- TODO: Rust search_word 統合
end

return ACTOR
