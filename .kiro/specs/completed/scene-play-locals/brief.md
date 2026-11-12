# Brief: scene-play-locals

> **ステータス**: 未着手（2026-11-12 の `/kiro-discovery` で起票。`pasta-recipe` の依頼「シーン呼び出し時にローカル変数を設定して渡せる機能」の、実行時の側を分けた）。版は 0.5.3（開発者の承認）。要件定義は Opus（Fable なし）。

## Problem

シーンの中でローカル変数（`＄名前`）の値によって分かれる話を確かめるには、今は、その変数に値を入れる前置きのシーンを別に書くか、辞書を書き換えて鳴らすしかない。開発の道具（`pasta-recipe`、のちに VS Code 拡張も）から、ローカル変数を入れた状態でシーンを鳴らしたい。

## Current State

2026-11-12（main `5c27b869`）に確かめた。

- **`OnPastaScene`・`OnPastaSceneAt`**: `pasta_scripts/pasta/shiori/event/scene_play.lua` 38〜90 行。Reference0・Reference1 は、グローバルシーンとローカルシーンの名前か、ファイルと行。どちらも `scene.lua` の `co_exec_exact` か `co_exec_registered` を呼ぶ。この 2 つは引数を取らない（284〜303 行）。デバッグが有効で、SecurityLevel が external でないときだけ効く。
- **`OnPastaCallFunction`**: Reference1 以降を、位置で並べた文字列の引数として `handler(act, ...)` へ渡す。シーンの関数は `function SCENE.x(act, ...)` で `local args = {...}` になり（`code_gen/scope_gen.rs` 210〜250 行）、DSL からは `args[n]` で読む。ローカル変数（`var.x`）に前もって値を入れることはできない。
- **書式の方針**: pasta のシリアライズは TOML（2026-10-24 の方針）。

## Desired Outcome

- `OnPastaScene`・`OnPastaSceneAt` に、ローカル変数の名前と値を付けて送ると、そのシーンはその変数に値が入った状態で始まる。
- 付けなければ、今と同じに振る舞う。
- 書き方の誤り（TOML として読めない・変数の名前にならない・扱えない型）は、今の `OnPastaSceneAt` の誤りと同じ形（バルーンの短い文とログ）で知らせ、シーンは鳴らさない。

## Approach

**今の Reference の次に、ローカル変数を TOML のインライン表で 1 つ足す**（discovery で開発者が承認した推奨案）。

```text
OnPastaSceneAt  Reference0=C:\…\ghost\master\dic\02-talk.pasta  Reference1=12  Reference2={ "名前" = "ぱすた", "回数" = 3 }
OnPastaScene    Reference0=挨拶  Reference1=朝  Reference2={ "天気" = "雨" }
→ シーンの最初の行の時点で ＄名前・＄回数（＄天気）に値が入っている
```

- 値の型は TOML のまま、文字列・数値・真偽にする。表と配列を許すかは要件で決める。
- 入れた値がどこまで届くかは要件で決める。たとえば、呼んだシーンの中から `＞` で呼んだ先へ届くか、ローカルシーンへ届くか。今のローカル変数の決まりに合わせる。
- `OnPastaScene` で、ローカルシーンの名前を省く書き方と Reference2 との並びは、要件で決める。

## Scope

- **In**:
  - `scene_play.lua`: Reference2 を TOML として読み（`@toml`）、検査し、シーンを始めるときにローカル変数へ入れる。
  - `scene.lua`（`co_exec_exact`・`co_exec_registered`）か、生成するシーンの関数の入口: ローカル変数を受け取る道。どちらで受けるかは設計で決める。生成するコード（`pasta_lua` の code_gen）に手を入れるなら、その範囲もここ。
  - マニュアル: `book/src/lua/shiori-events.md` の `OnPastaScene`・`OnPastaSceneAt` の節に Reference2 を足し、誤りの表を足す。スキルの `references/` を作り直す。
  - テスト: Lua の仕様のテスト（入れた値が読める・付けないと今のまま・誤りの 3 種）。
- **Out**:
  - 開発の道具の画面（`pasta-recipe-dev-actions` が持つ）。
  - VS Code 拡張から Reference2 を送ること。
  - `OnPastaCallFunction` の引数の形の変更。
  - グローバル変数（`＄＊`）を同じ口で入れること（`OnPastaSetSaveVariable` がある）。

## Boundary Candidates

- イベントの受け口（Reference2 の読み取りと検査）と、シーンを始めるときにローカル変数を入れる道。

## Out of Boundary

- `OnPastaSceneAt` の行からシーンを引く規則（シーンマップ）。
- ローカル変数の名前の決まりそのもの。

## Upstream / Downstream

- **Upstream**: `debug-mcp-runtime`（completed。`OnPastaScene`・`OnPastaSceneAt`）、`toml-serialization`（completed。`@toml`）。
- **Downstream**: `pasta-recipe-dev-actions`（ローカル変数を入れて鳴らす画面）。

## Existing Spec Touchpoints

- **Extends**: なし（`debug-mcp-runtime` は completed）。
- **Adjacent**: `cue-command-runtime`・`novel-*`（`scene.lua` を触る。同じウェーブに置かない）。

## Constraints

- Reference2 が無いときの振る舞いを変えない。今の MCP の利用者（VS Code 拡張・ghost_dev の MCP の使い方）が壊れないこと。
- デバッグが有効で、SecurityLevel が external でないときだけ効く（今の 2 つのイベントと同じ）。
- LuaJIT 2.1 で動くこと。
