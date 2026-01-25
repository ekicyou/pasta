--- @module pasta.word
--- 単語レジストリモジュール
---
--- 単語定義の登録と取得を担当する。
--- グローバル単語、ローカル単語（シーンスコープ）、アクター単語の3種をサポート。
--- ビルダーパターンAPIで可変長引数・メソッドチェーンを提供（Requirement 9）。

local STORE = require("pasta.store")

--- @class Word モジュールテーブル
local WORD = {}

-------------------------------------------
-- WORD_BUILDER_IMPL - ビルダーパターン実装
-------------------------------------------

--- WordBuilderクラス実装メタテーブル
--- @class WordBuilder
--- @field _registry table 登録先レジストリテーブル
--- @field _key string 単語キー
local WORD_BUILDER_IMPL = {}
WORD_BUILDER_IMPL.__index = WORD_BUILDER_IMPL

--- 値を追加（Requirement 9.3, 9.5）
--- @param self WordBuilder ビルダーオブジェクト
--- @param ... string 可変長引数で値を受け取る
--- @return WordBuilder メソッドチェーン用に自身を返す
function WORD_BUILDER_IMPL.entry(self, ...)
    local values = { ... }
    if #values > 0 then
        -- 既存のエントリ配列に新しい値リストを追加
        table.insert(self._registry[self._key], values)
    end
    return self
end

--- WordBuilderを生成
--- @param registry table 登録先レジストリ
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
local function create_builder(registry, key)
    -- キーが未登録なら初期化
    if not registry[key] then
        registry[key] = {}
    end
    local builder = {
        _registry = registry,
        _key = key,
    }
    return setmetatable(builder, WORD_BUILDER_IMPL)
end

-------------------------------------------
-- 公開API
-------------------------------------------

--- グローバル単語ビルダーを作成（Requirement 9.1）
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_global(key)
    return create_builder(STORE.global_words, key)
end

--- ローカル単語ビルダーを作成（Requirement 9.2）
--- @param scene_name string シーン名
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_local(scene_name, key)
    -- シーンが未登録なら初期化
    if not STORE.local_words[scene_name] then
        STORE.local_words[scene_name] = {}
    end
    return create_builder(STORE.local_words[scene_name], key)
end

--- アクター単語ビルダーを作成（actor-word-dictionary）
--- @param actor_name string アクター名
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_actor(actor_name, key)
    -- アクターが未登録なら初期化
    if not STORE.actor_words[actor_name] then
        STORE.actor_words[actor_name] = {}
    end
    return create_builder(STORE.actor_words[actor_name], key)
end

--- 全単語情報を取得（Requirement 2.6）
--- @return table {global: {key: [[values]]}, local: {scene: {key: [[values]]}}, actor: {name: {key: [[values]]}}} 形式
function WORD.get_all_words()
    return {
        global = STORE.global_words,
        ["local"] = STORE.local_words,
        actor = STORE.actor_words
    }
end

--- グローバル単語辞書を取得
--- @return table {key → values[][]} 形式の辞書
function WORD.get_global_words()
    return STORE.global_words
end

--- ローカル単語辞書を取得
--- @param scene_name string シーン名
--- @return table|nil {key → values[][]} 形式の辞書
function WORD.get_local_words(scene_name)
    return STORE.local_words[scene_name]
end

--- アクター単語辞書を取得
--- @param actor_name string アクター名
--- @return table|nil {key → values[][]} 形式の辞書
function WORD.get_actor_words(actor_name)
    return STORE.actor_words[actor_name]
end

--- グローバル単語ビルダーを作成（公開API）
--- create_global のエイリアス
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_word(key)
    return WORD.create_global(key)
end

return WORD
