--- @module pasta.word
--- 単語レジストリモジュール
---
--- 単語定義の登録と取得を担当する。
--- グローバル単語、ローカル単語（シーンスコープ）、アクター単語の3種をサポート。
--- ビルダーパターンAPIで可変長引数・メソッドチェーンを提供（Requirement 9）。

local STORE = require("pasta.store")
local log = require "@pasta_log"

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

-------------------------------------------
-- resolve_value - 共通値解決関数
-------------------------------------------

--- 値を解決（関数なら実行、配列なら最初の要素、その他はそのまま）
---
--- 旧 ACT_IMPL.word / PROXY_IMPL.word の検索パスで共通利用されていた汎用ユーティリティ。
--- handler-resolution-fallback 以降 pasta_scripts 内に呼び出し元はないが、
--- 出荷物の公開 API（外部ゴーストスクリプトからの利用面）として維持する。
--- @param value any 検索結果
--- @param act Act アクションオブジェクト
--- @return any 解決後の値
function WORD.resolve_value(value, act)
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

-------------------------------------------
-- dynamic_key - 動的参照のキー解決（dynamic-word-reference）
-------------------------------------------

--- 動的参照（＠＄名前・＠＄名前（…））の参照変数の値を検索キーに変換する。
--- 空でない文字列はそのまま、数値は変数展開と同じ tostring 表記にする（値を DSL として読み直さない）。
--- それ以外（nil・空文字列・真偽値・テーブル（__tostring 付きを含む）・関数・userdata・thread）は
--- 警告して nil を返す。
--- act・actor の両方から使うため、両者を require しないこのモジュールに置く。
--- @param value any 参照変数の値
--- @param var_path string 参照変数の Lua パス（"var.x" / "save.x" / "args[1]"）
--- @param via string 警告の接頭辞（"act:word" / "proxy:word" / "act:expr_fn" / "proxy:expr_fn"）
--- @return string|nil 検索キー。nil のときは警告済み
function WORD.dynamic_key(value, var_path, via)
    local t = type(value)
    if t == "number" then
        return tostring(value)
    elseif t == "string" then
        if value ~= "" then
            return value
        end
        log.warn(string.format("%s - empty variable: '%s'", via, var_path))
    elseif value == nil then
        log.warn(string.format("%s - undefined variable: '%s'", via, var_path))
    else
        log.warn(string.format("%s - unsupported value type: '%s' (%s)", via, var_path, t))
    end
    return nil
end

return WORD
