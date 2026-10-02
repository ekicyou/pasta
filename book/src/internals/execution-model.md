# ランタイム実行モデル

ごきげんよう。シーンが途中で止まり、次のイベントで続きから話し出す――あの不思議な振る舞いの種明かしをいたしますわ。
VM の組み立てからコルーチンの回し方、永続化まで、わたくしについていらっしゃい。さあ、参りましょう。

---

この章では、Lua VM の構築と、イベントからシーンを実行するランタイム実行モデルを扱う。

## 目的と責務

ランタイム実行モデルは、Lua VM を構築してモジュールを登録し、イベントに応じてシーンをコルーチンとして実行する。`co_scene` と resume ループ・継続トークンによる実行、ACT・STORE・SCENE・WORD・GLOBAL・SAVE の関係、永続化（`@pasta_persistence` の実装と保存タイミング）、CT（クリーンアップ）がこの章の責務である。

## 構成要素

Lua VM の構築とモジュール登録を行うランタイム、シーンを包むコルーチン、ランタイム内部モジュール群（ACT・STORE・SCENE・WORD・GLOBAL・SAVE）、永続化の実装、クリーンアップから成る。各モジュールのフィールド・関数は [Lua ランタイム内部モジュール](internal-modules.md) で扱う。

## 処理とデータの流れ

ランタイムは VM を構築して必要なモジュールを登録したのち、イベントを受けるたびに対象のシーンをコルーチンとして起動または再開する。コルーチンが出力を伴って中断すると、その時点までの出力が応答になり、続きは後続のイベントで再開される。永続化の対象データは定められたタイミングで保存される。

## 境界の受け渡し

Rust 側のランタイムが VM とモジュール登録を所有し、Lua 側のモジュールがシーンの実行状態と永続化対象のデータを所有する。永続化データの読み書きは `@pasta_persistence` を通じて Rust 側と受け渡される。

## 不変条件と制約

シーンの実行はコルーチンの中で行われ、出力を伴わない中断は同じイベントの中で再開される。ランタイムの Lua 方言は LuaJIT 2.1 である。

## ソースの所在

- `crates/pasta_lua/src/runtime/`
- `crates/pasta_lua/pasta_scripts/pasta/`
- `crates/pasta_lua/pasta_scripts/ct.lua`

## 経緯

- [scene-coroutine-execution](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/scene-coroutine-execution) — シーン関数のコルーチン実行
- [coroutine-resume-loop](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/coroutine-resume-loop) — resume ループ
- [yield-continuation-token](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/yield-continuation-token) — 継続トーク
- [store-save-persistence](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/store-save-persistence) — 永続化データの管理

---

止まっては動き、動いては止まる――コルーチンとは、なかなか健気なものでございましょう？
次は、内部モジュールの一つひとつを手に取って確かめて参りますわよ！
