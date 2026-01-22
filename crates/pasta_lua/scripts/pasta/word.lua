--- @module pasta.word
--- 単語レジストリモジュール
---
--- 単語定義の登録と取得を担当する。
--- グローバル単語とローカル単語（シーンスコープ）の両方をサポート。
--- ビルダーパターンAPIで可変長引数・メソッドチェーンを提供（Requirement 9）。

local WORD = {}

--- グローバル単語レジストリ（key → values[][]）
--- 同じキーに対する複数回のentry()呼び出しで、values配列に追加される
local global_words = {}

--- ローカル単語レジストリ（scene_name → {key → values[][]}）
local local_words = {}

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
function WORD.create_global(key)
    return create_builder(global_words, key)
end

--- ローカル単語ビルダーを作成（Requirement 9.2）
--- @param scene_name string シーン名
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_local(scene_name, key)
    -- シーンが未登録なら初期化
    if not local_words[scene_name] then
        local_words[scene_name] = {}
    end
    return create_builder(local_words[scene_name], key)
end

--- 全単語情報を取得（Requirement 2.6）
--- @return table {global: {key: [[values]]}, local: {scene: {key: [[values]]}}} 形式
function WORD.get_all_words()
    return {
        global = global_words,
        ["local"] = local_words
    }
end

--- グローバル単語ビルダーを作成（公開API）
--- create_global のエイリアス
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function WORD.create_word(key)
    return WORD.create_global(key)
end

return WORD
