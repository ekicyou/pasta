--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
--- アクターはキャッシュされ、同名のアクターは同一オブジェクトを返す。

local STORE = require("pasta.store")
local WORD = require("pasta.word")
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
--- WORD.create_actor のビルダー経由で STORE.actor_words に登録する（アクターのフィールドには設定しない）
--- @class ActorWordBuilder
--- @field _word_builder WordBuilder 内部のWordBuilder
local ACTOR_WORD_BUILDER_IMPL = {}
ACTOR_WORD_BUILDER_IMPL.__index = ACTOR_WORD_BUILDER_IMPL

--- 値を追加（アクター単語辞書への登録だけを行う）
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
--- @param text any 発話テキスト（nil は空文字扱い）
--- @param var_name string|nil 変数参照由来のとき変数パス（nil 時の警告用）
--- @return nil
function PROXY_IMPL.talk(self, text, var_name)
    self.act:talk(self.actor, text, var_name)
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
--- skip_methods（動的参照用）: A1 を rawget で表自身のフィールドだけに絞る（ACTOR_IMPL の create_word 等に一致させない）
---
--- @param self ActorProxy プロキシオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @param skip_methods boolean|nil true で継承したメソッドに届かせない。nil/false は既存どおり
--- @return any|nil 見つかったハンドラー、またはnil
function PROXY_IMPL.find_actor_handler(self, mode, key, skip_methods)
    -- アクター検索は word モードのみ
    if mode ~= "word" then
        return nil
    end

    -- A1: proxy.actor[key] 完全一致（skip_methods 時は表自身のフィールドだけ）
    local actor_value
    if skip_methods then actor_value = rawget(self.actor, key) else actor_value = self.actor[key] end
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
--- @param skip_methods boolean|nil find_actor_handler・act:find_act_handler の両方へ渡す
--- @return any|nil
function PROXY_IMPL.find_handler(self, mode, key, skip_methods)
    -- まずアクターレベル検索
    local handler = self:find_actor_handler(mode, key, skip_methods)
    if handler ~= nil then
        return handler
    end
    -- マッチしなければ act:find_act_handler に委譲
    return self.act:find_act_handler(mode, key, skip_methods)
end

--- 戻り値の正規化（expr・word 共通）
--- 先頭の戻り値が ACT またはプロキシ自身と同一なら値なし（nil）にする。
--- それ以外は複数の戻り値も含めてそのまま返す。
--- @param self ActorProxy プロキシオブジェクト
--- @param r any 先頭の戻り値
--- @param ... any 残りの戻り値
--- @return any ... 正規化した戻り値
local function drop_self(self, r, ...)
    if r == self.act or r == self then return nil end
    return r, ...
end

--- expr ポストプロセス（expr_fn・expr_fn_var 共通）: function → h(act, ...)、非function → warn+nil
--- 見つかった段にかかわらず第 1 引数は常に ACT。戻り値は drop_self で正規化する
--- @param self ActorProxy プロキシオブジェクト
--- @param key string 関数名
--- @param skip_methods boolean|nil find_handler へ渡す
--- @param ... any 可変引数
--- @return any|nil ハンドラー戻り値、またはnil
local function call_expr(self, key, skip_methods, ...)
    local handler = self:find_handler("expr", key, skip_methods)
    if type(handler) == "function" then
        return drop_self(self, handler(self.act, ...))
    end
    log.warn(string.format("proxy:expr_fn - handler not found: key='%s', mode='expr', via=proxy(%s)",
        tostring(key), tostring(self.actor.name)))
    return nil
end

--- expr 関数呼び出し（find_handler + expr ポストプロセス）
--- @param self ActorProxy プロキシオブジェクト
--- @param key string 関数名
--- @param ... any 可変引数
--- @return any|nil ハンドラー戻り値、またはnil
function PROXY_IMPL.expr_fn(self, key, ...)
    return call_expr(self, key, nil, ...)
end

--- 動的関数呼び出し（アクター付きの行の＠＄名前（…））
--- value を WORD.dynamic_key で関数名にし、継承したメソッドに届かせずに expr_fn と同じ検索・ポストプロセスを行う
--- @param self ActorProxy プロキシオブジェクト
--- @param value any 参照変数の値
--- @param var_path string 参照変数の Lua パス（警告用）
--- @param ... any 可変引数（ハンドラーに伝搬。第 1 引数は ACT）
--- @return any|nil ハンドラー戻り値、またはnil
function PROXY_IMPL.expr_fn_var(self, value, var_path, ...)
    local key = WORD.dynamic_key(value, var_path, "proxy:expr_fn")
    if key == nil then return nil end
    return call_expr(self, key, true, ...)
end

-------------------------------------------
-- PROXY_IMPL:word 段別検索の実装
-------------------------------------------

--- word（find_handler と同じ順序で検索 + word ポストプロセス）
--- 検索順序は find_actor_handler(A1+A2) → act:find_act_handler(L1-L5)
--- ポストプロセス: handler=nil → warn+nil、その他 → tostring(h)
--- function は見つかった段で第 1 引数が変わる: アクターの段 → h(proxy)、act の段 → h(act)。戻り値は drop_self で正規化する
--- var_path があるとき（動的参照）は name を WORD.dynamic_key でキーにし、継承したメソッドに届かせずに検索する
--- @param self ActorProxy プロキシオブジェクト
--- @param name any 単語名（＠なし）。var_path があるときは参照変数の値
--- @param var_path string|nil 動的参照の変数パス。nil なら既存の挙動（空キーは警告なしで nil）
--- @return string|nil 見つかった単語、またはnil
function PROXY_IMPL.word(self, name, var_path)
    local skip_methods = nil
    if var_path ~= nil then
        name = WORD.dynamic_key(name, var_path, "proxy:word")
        if name == nil then return nil end
        skip_methods = true
    elseif not name or name == "" then
        return nil
    end
    local handler, receiver = self:find_actor_handler("word", name, skip_methods), self
    if handler == nil then
        handler, receiver = self.act:find_act_handler("word", name, skip_methods), self.act
    end
    if handler == nil then
        log.warn(string.format("proxy:word - handler not found: key='%s', mode='word', via=proxy(%s)",
            tostring(name), tostring(self.actor.name)))
        return nil
    end
    if type(handler) == "function" then
        return drop_self(self, handler(receiver))
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
