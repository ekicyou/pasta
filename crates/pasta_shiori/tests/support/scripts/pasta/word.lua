--- @module pasta.word
--- 単語レジストリモジュール
---
--- 単語定義の登録と取得を担当する。
--- グローバル単語、ローカル単語（シーンスコープ）、アクター単語の3種をサポート。
--- ビルダーパターンAPIで可変長引数・メソッドチェーンを提供（Requirement 9）。

local STORE = require("pasta.store")

local MOD = {}

-------------------------------------------
-- WordBuilder - ビルダーパターン実装
-------------------------------------------

--- WordBuilderクラス
--- @class WordBuilder
--- @field _registry table 登録先レジストリテーブル
--- @field _key string 単語キー
local WordBuilder = {}
WordBuilder.__index = WordBuilder

--- 値を追加（Requirement 9.3, 9.5）
--- @vararg string 可変長引数で値を受け取る
--- @return WordBuilder メソッドチェーン用に自身を返す
function WordBuilder:entry(...)
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
    local builder = setmetatable({}, WordBuilder)
    builder._registry = registry
    builder._key = key
    return builder
end

-------------------------------------------
-- 公開API
-------------------------------------------

--- グローバル単語ビルダーを作成（Requirement 9.1）
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function MOD.create_global(key)
    return create_builder(STORE.global_words, key)
end

--- ローカル単語ビルダーを作成（Requirement 9.2）
--- @param scene_name string シーン名
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function MOD.create_local(scene_name, key)
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
function MOD.create_actor(actor_name, key)
    -- アクターが未登録なら初期化
    if not STORE.actor_words[actor_name] then
        STORE.actor_words[actor_name] = {}
    end
    return create_builder(STORE.actor_words[actor_name], key)
end

--- 全単語情報を取得（Requirement 2.6）
--- @return table {global: {key: [[values]]}, local: {scene: {key: [[values]]}}, actor: {name: {key: [[values]]}}} 形式
function MOD.get_all_words()
    return {
        global = STORE.global_words,
        ["local"] = STORE.local_words,
        actor = STORE.actor_words
    }
end

--- グローバル単語辞書を取得
--- @return table {key → values[][]} 形式の辞書
function MOD.get_global_words()
    return STORE.global_words
end

--- ローカル単語辞書を取得
--- @param scene_name string シーン名
--- @return table|nil {key → values[][]} 形式の辞書
function MOD.get_local_words(scene_name)
    return STORE.local_words[scene_name]
end

--- アクター単語辞書を取得
--- @param actor_name string アクター名
--- @return table|nil {key → values[][]} 形式の辞書
function MOD.get_actor_words(actor_name)
    return STORE.actor_words[actor_name]
end

--- グローバル単語ビルダーを作成（公開API）
--- create_global のエイリアス
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function MOD.create_word(key)
    return MOD.create_global(key)
end

-------------------------------------------
-- resolve_value - 共通値解決関数
-------------------------------------------

--- 値を解決（関数なら実行、配列なら最初の要素、その他はそのまま）
--- ACT_IMPL.word と PROXY_IMPL.word から共通利用
--- @param value any 検索結果
--- @param act Act アクションオブジェクト
--- @return any 解決後の値
function MOD.resolve_value(value, act)
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

return MOD
