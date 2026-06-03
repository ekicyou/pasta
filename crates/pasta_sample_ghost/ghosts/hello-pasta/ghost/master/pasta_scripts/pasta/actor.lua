--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
--- アクターはキャッシュされ、同名のアクターは同一オブジェクトを返す。

local STORE = require("pasta.store")
local WORD = require("pasta.word")
local GLOBAL = require("pasta.global")
local log = require "@pasta_log"

--- @class Actor アクターオブジェクト
--- @field name string アクター名
--- @field spot integer|nil 立ち位置（0以上）
local ACTOR = {}

--- アクター実装メタテーブル
local ACTOR_IMPL = {}
ACTOR_IMPL.__index = ACTOR_IMPL

-------------------------------------------
-- ACTOR_WORD_BUILDER_IMPL - アクター単語ビルダー
-------------------------------------------

--- ActorWordBuilderクラス実装メタテーブル（WordBuilderを拡張）
--- word.lua辞書への登録とACTORプロパティへの設定を同時に行う
--- @class ActorWordBuilder
--- @field _actor Actor アクターオブジェクト
--- @field _key string 単語キー
--- @field _word_builder WordBuilder 内部のWordBuilder
local ACTOR_WORD_BUILDER_IMPL = {}
ACTOR_WORD_BUILDER_IMPL.__index = ACTOR_WORD_BUILDER_IMPL

--- 値を追加（辞書登録＋ACTORプロパティ設定）
--- @param self ActorWordBuilder ビルダーオブジェクト
--- @param ... string 可変長引数で値を受け取る
--- @return ActorWordBuilder メソッドチェーン用に自身を返す
function ACTOR_WORD_BUILDER_IMPL.entry(self, ...)
    local values = { ... }
    if #values > 0 then
        -- Rust WordTableにのみ登録（シャッフル＆順次消費はWordTableが担当）
        self._word_builder:entry(...)
    end
    return self
end

--- アクター単語ビルダーを作成（ACTOR:create_word("key") 形式）
--- @param self Actor アクターオブジェクト
--- @param key string 単語キー
--- @return ActorWordBuilder ビルダーオブジェクト
function ACTOR_IMPL.create_word(self, key)
    local builder = {
        _actor = self,
        _key = key,
        _word_builder = WORD.create_actor(self.name, key),
    }
    return setmetatable(builder, ACTOR_WORD_BUILDER_IMPL)
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
        setmetatable(actor, ACTOR_IMPL)
        STORE.actors[name] = actor
    end
    return STORE.actors[name]
end

-------------------------------------------
-- PROXY_IMPL - アクタープロキシ実装
-------------------------------------------

--- @class ActorProxy アクタープロキシ（actへの逆参照付き）
--- @field actor Actor アクターオブジェクト
--- @field act Act アクションオブジェクト
local PROXY_IMPL = {}
PROXY_IMPL.__index = PROXY_IMPL

--- プロキシを作成
--- @param actor Actor アクターオブジェクト
--- @param act Act アクションオブジェクト
--- @return ActorProxy アクタープロキシ
function ACTOR.create_proxy(actor, act)
    local proxy = {
        actor = actor,
        act = act,
    }
    return setmetatable(proxy, PROXY_IMPL)
end

--- talk（act経由でトークン蓄積）
--- @param self ActorProxy プロキシオブジェクト
--- @param text string 発話テキスト
--- @return nil
function PROXY_IMPL.talk(self, text)
    self.act:talk(self.actor, text)
end

--- sakura_script（act経由でトークン蓄積）
--- @param self ActorProxy プロキシオブジェクト
--- @param text string さくらスクリプトタグ文字列
--- @return nil
function PROXY_IMPL.sakura_script(self, text)
    self.act:sakura_script(self.actor, text)
end

-------------------------------------------
-- PROXY_IMPL:find_actor_handler / find_handler
-------------------------------------------

--- アクタースコープのフォールバック検索（word モード限定）
---
--- word モード以外は即 nil を返す。
--- A1: proxy.actor[key] 完全一致
--- A2: アクター単語辞書前方一致 ("__actor_{name}__" スコープ)
--- @pasta_search 未利用時は A2 をスキップ
---
--- @param self ActorProxy プロキシオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @return any|nil 見つかったハンドラー、またはnil
function PROXY_IMPL.find_actor_handler(self, mode, key)
    -- アクター検索は word モードのみ
    if mode ~= "word" then
        return nil
    end

    -- A1: proxy.actor[key] 完全一致
    local actor_value = self.actor[key]
    if actor_value ~= nil then
        return actor_value
    end

    -- A2: アクター辞書前方一致（@pasta_search 利用可能時のみ）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if ok and SEARCH then
        local actor_scope = "__actor_" .. self.actor.name .. "__"
        local result = SEARCH:search_word(key, actor_scope)
        if result ~= nil then return result end
    end

    return nil
end

--- 統一ハンドラー検索エントリ（PROXY 経由）
---
--- まずアクターレベル検索（find_actor_handler）を実行し、
--- マッチしなければ act:find_act_handler に委譲する。
---
--- @param self ActorProxy プロキシオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @return any|nil
function PROXY_IMPL.find_handler(self, mode, key)
    -- まずアクターレベル検索
    local handler = self:find_actor_handler(mode, key)
    if handler ~= nil then
        return handler
    end
    -- マッチしなければ act:find_act_handler に委譲
    return self.act:find_act_handler(mode, key)
end

--- expr 関数呼び出し（find_handler + expr ポストプロセス）
--- @param self ActorProxy プロキシオブジェクト
--- @param key string 関数名
--- @param ... any 可変引数
--- @return any|nil ハンドラー戻り値、またはnil
function PROXY_IMPL.expr_fn(self, key, ...)
    local handler = self:find_handler("expr", key)
    if type(handler) == "function" then
        return handler(self, ...)
    end
    log.warn(string.format("proxy:expr_fn - handler not found: key='%s', mode='expr', via=proxy(%s)",
        tostring(key), tostring(self.actor.name)))
    return nil
end

-------------------------------------------
-- PROXY_IMPL:word find_handler ベース実装
-------------------------------------------

--- word（find_handler + word ポストプロセス）
--- 検索順序は find_handler → find_actor_handler(A1+A2) → act:find_act_handler(L1-L5)
--- ポストプロセス: handler=nil → warn+nil、function → h(self)、その他 → tostring(h)
--- @param self ActorProxy プロキシオブジェクト
--- @param name string 単語名（＠なし）
--- @return string|nil 見つかった単語、またはnil
function PROXY_IMPL.word(self, name)
    if not name or name == "" then
        return nil
    end
    local handler = self:find_handler("word", name)
    if handler == nil then
        log.warn(string.format("proxy:word - handler not found: key='%s', mode='word', via=proxy(%s)",
            tostring(name), tostring(self.actor.name)))
        return nil
    end
    if type(handler) == "function" then
        return handler(self)
    end
    return tostring(handler)
end

-- CONFIG 由来アクターへのメタテーブル設定
-- STORE.actors の各テーブル要素に ACTOR_IMPL メタテーブルを設定し、name を補完
for name, actor in pairs(STORE.actors) do
    if type(actor) == "table" then
        -- name フィールドがなければ辞書キーから補完
        if actor.name == nil then
            actor.name = name
        end
        setmetatable(actor, ACTOR_IMPL)
    end
end

return ACTOR
