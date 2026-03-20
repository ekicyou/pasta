--- @module pasta.store
--- データストアモジュール
---
--- 全てのランタイムデータを一元管理する。
--- 他のモジュールから require されるが、自身は他モジュールを require しない。
--- これにより循環参照を完全に回避する。
---
--- 注意: 永続化データ(save)はpasta.saveモジュールに移行済み。
--- ctx.saveから参照すること。

--- @class Store
--- @field actors table<string, Actor> アクターキャッシュ（名前→アクター）
--- @field actor_spots table<string, integer> アクターごとのスポット位置マップ（名前→スポットID）
--- @field scenes table<string, table> シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
--- @field counters table<string, number> シーン名カウンタ（ベース名→カウンタ値）
--- @field global_words table<string, table> グローバル単語レジストリ（key → values[][]）
--- @field local_words table<string, table> ローカル単語レジストリ（scene_name → {key → values[][]}）
--- @field actor_words table<string, table> アクター単語レジストリ（actor_name → {key → values[][]}）
--- @field app_ctx table アプリケーション実行中の汎用コンテキストデータ
local STORE = {}

--- アクターキャッシュ（名前→アクター）
--- @type table<string, Actor>
STORE.actors = {}

--- アクターごとのスポット位置マップ（名前→スポットID）
--- @type table<string, integer>
STORE.actor_spots = {}

--- シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
--- @type table<string, table>
STORE.scenes = {}

--- シーン名カウンタ（ベース名→カウンタ値）
--- @type table<string, number>
STORE.counters = {}

--- グローバル単語レジストリ（key → values[][]）
--- @type table<string, table>
STORE.global_words = {}

--- ローカル単語レジストリ（scene_name → {key → values[][]}）
--- @type table<string, table>
STORE.local_words = {}

--- アクター単語レジストリ（actor_name → {key → values[][]}）
--- @type table<string, table>
STORE.actor_words = {}

--- アプリケーション実行中の汎用コンテキストデータ
--- @type table
STORE.app_ctx = {}

--- 継続用コルーチン（OnTalkチェイントーク用）
--- @type thread|nil
STORE.co_scene = nil

--- 全データをリセット
--- @return nil
function STORE.reset()
    -- co_sceneのクリーンアップ（suspendedコルーチンをclose）
    if STORE.co_scene then
        if coroutine.status(STORE.co_scene) == "suspended" then
            coroutine.close(STORE.co_scene)
        end
        STORE.co_scene = nil
    end

    STORE.actors = {}
    STORE.actor_spots = {}
    STORE.scenes = {}
    STORE.app_ctx = {}
    STORE.counters = {}
    STORE.global_words = {}
    STORE.local_words = {}
    STORE.actor_words = {}
end

-- CONFIG.actor からの初期化
-- @pasta_config は Rust 組み込みモジュールのため例外扱い（循環参照回避ポリシーの例外）
-- pcall で保護することで、@pasta_config が無い環境（単体テスト等）でも動作可能にする
local ok, CONFIG = pcall(require, "@pasta_config")
if ok and type(CONFIG.actor) == "table" then
    STORE.actors = CONFIG.actor

    -- CONFIG.actor からのspot値転送（persist-spot-position）
    -- actor.spot が数値型の場合のみ STORE.actor_spots に転送
    for name, actor in pairs(CONFIG.actor) do
        if type(actor) == "table" and type(actor.spot) == "number" then
            STORE.actor_spots[name] = actor.spot
        end
    end
end

return STORE
