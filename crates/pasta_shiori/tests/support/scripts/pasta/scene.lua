--- @module pasta.scene
--- シーンレジストリモジュール
---
--- シーン関数の登録と検索を担当する。
--- グローバルシーン名（ファイル名）とローカルシーン名（シーン関数名）の階層構造を管理する。

local SCENE = {}

--- シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
local registry = {}

--- シーン登録
--- @param global_name string グローバルシーン名（ファイル名）
--- @param local_name string ローカルシーン名（シーン関数名）
--- @param scene_func function シーン関数
function SCENE.register(global_name, local_name, scene_func)
    if not registry[global_name] then
        registry[global_name] = { __global_name__ = global_name }
    end
    registry[global_name][local_name] = scene_func
end

--- グローバルシーンテーブルを作成
--- @param global_name string グローバルシーン名
--- @return table グローバルシーンテーブル
function SCENE.create_global_table(global_name)
    if not registry[global_name] then
        registry[global_name] = { __global_name__ = global_name }
    end
    return registry[global_name]
end

--- グローバルシーンテーブルを取得
--- @param global_name string グローバルシーン名
--- @return table|nil グローバルシーンテーブル、またはnil
function SCENE.get_global_table(global_name)
    return registry[global_name]
end

--- シーン関数を取得
--- @param global_name string グローバルシーン名
--- @param local_name string ローカルシーン名
--- @return function|nil シーン関数、またはnil
function SCENE.get(global_name, local_name)
    local global_table = registry[global_name]
    if global_table then
        return global_table[local_name]
    end
    return nil
end

--- グローバルシーン名を取得
--- @param scene_table table シーンテーブル
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

return SCENE
