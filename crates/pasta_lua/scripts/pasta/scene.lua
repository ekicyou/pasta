--- @module pasta.scene
--- シーンレジストリモジュール
---
--- シーン関数の登録と検索を担当する。
--- グローバルシーン名（ファイル名）とローカルシーン名（シーン関数名）の階層構造を管理する。
--- カウンタ管理機能により、同名シーンに対して一意な番号を自動割当する。

local STORE = require("pasta.store")
local WORD = require("pasta.word")

--- @class Scene モジュールテーブル
local SCENE = {}

--- シーンテーブル実装メタテーブル（create_word メソッドを提供）
--- @class SceneTable
--- @field __global_name__ string グローバルシーン名
local SCENE_TABLE_IMPL = {}

--- ローカル単語ビルダーを作成（scene:create_word("key") 形式）
--- @param self SceneTable シーンテーブル
--- @param key string 単語キー
--- @return WordBuilder ビルダーオブジェクト
function SCENE_TABLE_IMPL.create_word(self, key)
    local global_name = self.__global_name__
    return WORD.create_local(global_name, key)
end

--- シーンテーブルのメタテーブル
local scene_table_mt = {
    __index = SCENE_TABLE_IMPL
}

--- シーン検索結果のメタテーブル（__call で直接呼び出し可能）
--- @class SceneSearchResult
--- @field global_name string グローバルシーン名
--- @field local_name string ローカルシーン名
--- @field func function シーン関数
local scene_result_mt = {
    __call = function(self, ...)
        return self.func(...)
    end
}

--- カウンタを取得してインクリメント
--- @param base_name string ベース名
--- @return number カウンタ値（1から始まる連番）
function SCENE.get_or_increment_counter(base_name)
    local current = STORE.counters[base_name] or 0
    current = current + 1
    STORE.counters[base_name] = current
    return current
end

--- シーン登録
--- @param global_name string グローバルシーン名（ファイル名）
--- @param local_name string ローカルシーン名（シーン関数名）
--- @param scene_func function シーン関数
--- @return nil
function SCENE.register(global_name, local_name, scene_func)
    if not STORE.scenes[global_name] then
        local scene_table = { __global_name__ = global_name }
        setmetatable(scene_table, scene_table_mt)
        STORE.scenes[global_name] = scene_table
    end
    STORE.scenes[global_name][local_name] = scene_func
end

--- グローバルシーンテーブルを作成
--- @param global_name string グローバルシーン名
--- @return SceneTable グローバルシーンテーブル
function SCENE.create_global_table(global_name)
    if not STORE.scenes[global_name] then
        local scene_table = { __global_name__ = global_name }
        setmetatable(scene_table, scene_table_mt)
        STORE.scenes[global_name] = scene_table
    end
    return STORE.scenes[global_name]
end

--- グローバルシーンテーブルを取得
--- @param global_name string グローバルシーン名
--- @return SceneTable|nil グローバルシーンテーブル、またはnil
function SCENE.get_global_table(global_name)
    return STORE.scenes[global_name]
end

--- シーン関数を取得
--- @param global_name string グローバルシーン名
--- @param local_name string ローカルシーン名
--- @return function|nil シーン関数、またはnil
function SCENE.get(global_name, local_name)
    local global_table = STORE.scenes[global_name]
    if global_table then
        return global_table[local_name]
    end
    return nil
end

--- グローバルシーン名を取得
--- @param scene_table SceneTable シーンテーブル
--- @return string|nil グローバルシーン名、またはnil
function SCENE.get_global_name(scene_table)
    return scene_table.__global_name__
end

--- エントリーポイント（__start__）を取得
--- @param global_name string グローバルシーン名
--- @return function|nil __start__シーン関数、またはnil
function SCENE.get_start(global_name)
    return SCENE.get(global_name, "__start__")
end

--- 全シーン情報を取得
--- @return table {global_name: {local_name: func}} 形式のシーンレジストリ
function SCENE.get_all_scenes()
    return STORE.scenes
end

--- シーンを登録し、グローバルシーンテーブルを返す
---
--- カウンタ管理を使用してベース名から一意なグローバルシーン名を生成する。
--- 例: create_scene("メイン") → "メイン1", 2回目 → "メイン2"
---
--- @param base_name string ベース名（シーン名のベース）
--- @param local_name string|nil ローカルシーン名（シーン関数名）
--- @param scene_func function|nil シーン関数
--- @return SceneTable グローバルシーンテーブル
function SCENE.create_scene(base_name, local_name, scene_func)
    -- カウンタからグローバルシーン名を生成
    local counter = SCENE.get_or_increment_counter(base_name)
    local global_name = base_name .. counter

    if scene_func and local_name then
        SCENE.register(global_name, local_name, scene_func)
    end
    return SCENE.get_global_table(global_name) or SCENE.create_global_table(global_name)
end

--- シーンを名前で検索（プレフィックス検索）
---
--- 指定された名前でシーンを検索し、見つかった場合は呼び出し可能な結果オブジェクトを返す。
--- global_scene_name が nil の場合はグローバル検索（__start__ を返す）、
--- 指定された場合はそのシーン内のローカル検索を行う。
---
--- @param name string 検索するシーン名
--- @param global_scene_name string|nil ローカル検索の場合のグローバルシーン名
--- @param attrs table|nil 属性テーブル（将来拡張用、現在は未使用）
--- @return SceneSearchResult|nil 検索結果、またはnil
function SCENE.search(name, global_scene_name, attrs)
    -- 型チェック: name が文字列でない場合は nil を返す
    if type(name) ~= "string" then
        return nil
    end

    -- @pasta_search を遅延ロード（初期化タイミング問題を回避）
    local SEARCH = require("@pasta_search")

    -- @pasta_search を使用してシーンを検索
    local global_name, local_name = SEARCH:search_scene(name, global_scene_name)

    -- 検索結果がなければ nil を返す
    if not global_name then
        return nil
    end

    -- Lua 側で登録されているシーン関数を取得
    local func = SCENE.get(global_name, local_name)

    -- シーン関数が登録されていなければ nil を返す
    if not func then
        return nil
    end

    -- 呼び出し可能な結果オブジェクトを返す
    return setmetatable({
        global_name = global_name,
        local_name = local_name,
        func = func
    }, scene_result_mt)
end

return SCENE
