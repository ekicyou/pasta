--- @module pasta.store
--- データストアモジュール
---
--- 全てのランタイムデータを一元管理する。
--- 他のモジュールから require されるが、自身は他モジュールを require しない。
--- これにより循環参照を完全に回避する。

--- @class Store
--- @field save table<string, any> 永続変数（セッションを跨いで保持）
--- @field actors table<string, Actor> アクターキャッシュ（名前→アクター）
--- @field scenes table<string, table> シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
--- @field counters table<string, number> シーン名カウンタ（ベース名→カウンタ値）
--- @field global_words table<string, table> グローバル単語レジストリ（key → values[][]）
--- @field local_words table<string, table> ローカル単語レジストリ（scene_name → {key → values[][]}）
--- @field actor_words table<string, table> アクター単語レジストリ（actor_name → {key → values[][]}）
local STORE = {}

--- 永続変数（セッションを跨いで保持）
--- @type table<string, any>
STORE.save = {}

--- アクターキャッシュ（名前→アクター）
--- @type table<string, Actor>
STORE.actors = {}

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

--- 全データをリセット
--- @return nil
function STORE.reset()
    STORE.save = {}
    STORE.actors = {}
    STORE.scenes = {}
    STORE.counters = {}
    STORE.global_words = {}
    STORE.local_words = {}
    STORE.actor_words = {}
end

return STORE
