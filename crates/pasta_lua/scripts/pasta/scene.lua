--- @module pasta.scene
--- シーンレジストリモジュール
---
--- シーン関数の登録と検索を担当する。
--- グローバルシーン名（ファイル名）とローカルシーン名（シーン関数名）の階層構造を管理する。
--- カウンタ管理機能により、同名シーンに対して一意な番号を自動割当する。

local STORE = require("pasta.store")
local WORD = require("pasta.word")

local MOD = {}

--- シーンテーブルのメタテーブル（create_word メソッドを提供）
local scene_table_mt = {
    __index = {
        --- ローカル単語ビルダーを作成（MOD:create_word("key") 形式）
        --- @param self table シーンテーブル
        --- @param key string 単語キー
        --- @return WordBuilder ビルダーオブジェクト
        create_word = function(self, key)
            local global_name = self.__global_name__
            return WORD.create_local(global_name, key)
        end
    }
}

--- カウンタを取得してインクリメント
--- @param base_name string ベース名
--- @return number カウンタ値（1から始まる連番）
function MOD.get_or_increment_counter(base_name)
    local current = STORE.counters[base_name] or 0
    current = current + 1
    STORE.counters[base_name] = current
    return current
end

--- シーン登録
--- @param global_name string グローバルシーン名（ファイル名）
--- @param local_name string ローカルシーン名（シーン関数名）
--- @param scene_func function シーン関数
function MOD.register(global_name, local_name, scene_func)
    if not STORE.scenes[global_name] then
        local scene_table = { __global_name__ = global_name }
        setmetatable(scene_table, scene_table_mt)
        STORE.scenes[global_name] = scene_table
    end
    STORE.scenes[global_name][local_name] = scene_func
end

--- グローバルシーンテーブルを作成
--- @param global_name string グローバルシーン名
--- @return table グローバルシーンテーブル
function MOD.create_global_table(global_name)
    if not STORE.scenes[global_name] then
        local scene_table = { __global_name__ = global_name }
        setmetatable(scene_table, scene_table_mt)
        STORE.scenes[global_name] = scene_table
    end
    return STORE.scenes[global_name]
end

--- グローバルシーンテーブルを取得
--- @param global_name string グローバルシーン名
--- @return table|nil グローバルシーンテーブル、またはnil
function MOD.get_global_table(global_name)
    return STORE.scenes[global_name]
end

--- シーン関数を取得
--- @param global_name string グローバルシーン名
--- @param local_name string ローカルシーン名
--- @return function|nil シーン関数、またはnil
function MOD.get(global_name, local_name)
    local global_table = STORE.scenes[global_name]
    if global_table then
        return global_table[local_name]
    end
    return nil
end

--- グローバルシーン名を取得
--- @param scene_table table シーンテーブル
--- @return string|nil グローバルシーン名、またはnil
function MOD.get_global_name(scene_table)
    return scene_table.__global_name__
end

--- エントリーポイント（__start__）を取得
--- @param global_name string グローバルシーン名
--- @return function|nil __start__シーン関数、またはnil
function MOD.get_start(global_name)
    return MOD.get(global_name, "__start__")
end

--- 全シーン情報を取得
--- @return table {global_name: {local_name: func}} 形式のシーンレジストリ
function MOD.get_all_scenes()
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
--- @return table グローバルシーンテーブル
function MOD.create_scene(base_name, local_name, scene_func)
    -- カウンタからグローバルシーン名を生成
    local counter = MOD.get_or_increment_counter(base_name)
    local global_name = base_name .. counter

    if scene_func and local_name then
        MOD.register(global_name, local_name, scene_func)
    end
    return MOD.get_global_table(global_name) or MOD.create_global_table(global_name)
end

return MOD
