---@module pasta.shiori.event.choice_select
--- OnChoiceSelectEx デフォルトハンドラ
---
--- SSP が発火する OnChoiceSelectEx を受信し、選択IDでシーンを自動ルーティングする。
--- 明示的な ＊OnChoiceSelectEx シーンが存在すれば優先実行する。
--- マッチシーンが見つかればコルーチンを返し、見つからなければ nil を返す（204 委譲）。
--- ゴースト開発者は REG.OnChoiceSelectEx を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

--- 選択IDの Reference インデックス
--- ukadoc 仕様: \q[title,id] → Reference0=id, Reference1=title
--- SSP 実装差異があれば定数変更で吸収する
local CHOICE_ID_REF_INDEX = 0

--- SCENE.co_exec と同等のコルーチン作成（スコープ付き SCENE.search 用）
--- co_exec/find_scene は global_scene_name を透過しないため、
--- SCENE.search を直接呼んでコルーチン化する。
--- @param search_result SceneSearchResult SCENE.search の戻り値
--- @return thread シーンコルーチン（act は EVENT.fire の初回 resume 時に渡される）
local function create_scene_coroutine(search_result)
    local fn = search_result.func
    local function wrapped_fn(act, ...)
        fn(act, ...)
        local result = act:build()
        if result ~= nil then
            return result
        end
    end
    return coroutine.create(wrapped_fn)
end

---OnChoiceSelectEx デフォルトハンドラ
---明示シーン優先 → 選択ID前方一致検索 → 非マッチ時 nil（204 委譲）
---@param act ShioriAct actオブジェクト（act.req でリクエスト情報にアクセス可能）
---@return thread|nil シーンコルーチン、またはnil
REG.OnChoiceSelectEx = function(act)
    -- 1. 明示的 ＊OnChoiceSelectEx シーンが存在すれば優先実行（3.5）
    local explicit = SCENE.co_exec(act, "OnChoiceSelectEx", nil, nil)
    if explicit then
        return explicit
    end

    -- 2. Reference から選択IDを取得
    local ref = act.req.reference
    if not ref then
        return nil
    end
    local choice_id = ref[CHOICE_ID_REF_INDEX]
    if not choice_id or choice_id == "" then
        return nil
    end

    -- 3. 選択IDでシーンを前方一致検索（ローカル→グローバル、3.1/3.4）
    --    SCENE.search を直接呼び出し、STORE.last_global_scene をスコープに使用
    --    co_exec/find_scene は global_scene_name を透過しないため直接検索が必須
    --    シャッフル＆順次消費は SCENE.search 内部で処理される（3.2）
    local result = SCENE.search(choice_id, STORE.last_global_scene)
    if not result or type(result.func) ~= "function" then
        return nil
    end

    -- 4. コルーチン化して返す（SCENE.co_exec と同じラッパーパターン）
    return create_scene_coroutine(result)
end

return REG
