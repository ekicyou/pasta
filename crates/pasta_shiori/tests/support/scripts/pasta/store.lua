--- @module pasta.store
--- データストアモジュール
---
--- 全てのランタイムデータを一元管理する。
--- 他のモジュールから require されるが、自身は他モジュールを require しない。
--- これにより循環参照を完全に回避する。

local STORE = {}

--- 永続変数（セッションを跨いで保持）
STORE.save = {}

--- アクターキャッシュ（名前→アクター）
STORE.actors = {}

--- シーンレジストリ（グローバル名→{ローカル名→シーン関数}）
STORE.scenes = {}

--- シーン名カウンタ（ベース名→カウンタ値）
STORE.counters = {}

--- グローバル単語レジストリ（key → values[][]）
STORE.global_words = {}

--- ローカル単語レジストリ（scene_name → {key → values[][]}）
STORE.local_words = {}

--- アクター単語レジストリ（actor_name → {key → values[][]}）
STORE.actor_words = {}

--- 全データをリセット
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
