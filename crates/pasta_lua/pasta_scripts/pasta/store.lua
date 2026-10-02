--- @module pasta.store
--- データストアモジュール
---
--- 全てのランタイムデータを一元管理する。
--- 他のモジュールから require されるが、自身は他モジュールを require しない。
--- これにより循環参照を完全に回避する。
---
--- 注意: 永続化データ(save)は pasta.save モジュールが持つ。
--- act.save（act:init_scene の戻り値 save）から参照すること。

--- @class Store
--- @field actors table<string, Actor> アクターキャッシュ（名前→アクター）
--- @field actor_spots table<string, integer> アクターごとのスポット位置マップ（名前→スポットID）
--- @field scenes table<string, table> シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
--- @field counters table<string, number> シーン名カウンタ（ベース名→カウンタ値）
--- @field global_words table<string, table> グローバル単語レジストリ（key → values[][]）
--- @field local_words table<string, table> ローカル単語レジストリ（scene_name → {key → values[][]}）
--- @field actor_words table<string, table> アクター単語レジストリ（actor_name → {key → values[][]}）
--- @field app_ctx table アプリケーション実行中の汎用コンテキストデータ
--- @field appearance table 外見状態（セッション常駐・非永続）
--- @field co_callback thread|nil コールバック待ちとして登録したコルーチンの印（下記参照）
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

--- 外見状態（アクター既知状態・スポット表示中状態・スポットの直前発話アクター・アクターの前回発話スポット）
--- セッション常駐のみ。pasta.save（永続化）には置かない。
--- @type table
STORE.appearance = { actors = {}, spots = {}, owners = {}, last_spots = {} }

--- 継続用コルーチン（OnTalkチェイントーク用）
--- @type thread|nil
STORE.co_scene = nil

--- 最後に実行したグローバルシーン名（選択肢コールバックルーティング用）
--- @type string|nil
STORE.last_global_scene = nil

--- 保留中のキックシーン名（次の OnSecondChange で強制起動・無ければ nil）
--- @type string|nil
STORE.kick_pending = nil

--- キック割り込み許可フラグ（is_blocked ワンショット突破用・既定 false）
--- @type boolean
STORE.kick_force = false

-- STORE.co_callback（thread|nil）はこのモジュールでは初期化しない（未設定は nil）。
-- pasta.shiori.event.callback の CALLBACK.consume_staged が設定し、
-- pasta.shiori.event の set_co_scene と CALLBACK.reset が nil に戻す。STORE.reset は触らない。

--- 全データをリセット
--- @return nil
function STORE.reset()
    -- co_sceneのクリーンアップ（coroutine.close がある環境では suspended コルーチンを close）。
    -- LuaJIT 2.1 には coroutine.close が無いため close 分岐は実行されず、参照を外すだけになる
    -- （破棄したコルーチンは GC が回収する）。
    if STORE.co_scene then
        if coroutine.close and coroutine.status(STORE.co_scene) == "suspended" then
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
    STORE.appearance = { actors = {}, spots = {}, owners = {}, last_spots = {} }
    STORE.last_global_scene = nil
    STORE.kick_pending = nil
    STORE.kick_force = false
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
