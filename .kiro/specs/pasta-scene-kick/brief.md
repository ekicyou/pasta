# Brief: pasta-scene-kick

## Problem
作者が「**このシーンを今すぐ再生して観たい**」（デバッグ／オーサリング）を満たす手段がない。Phase 5 のデバッグ位置ブレークでは代替できない——求められているのは任意シーンの再生キックである。

## Current State
`pasta-actor-runtime` 完了後を前提とする:
- 宿主非依存コア ＋ アクタースレッド（`wintf_winmsg_executor`）＋ CH marshaling（GET/NOTIFY/drop→204）が存在。
- 出力は `pasta_shiori` アダプタが presentation event stream をさくらスクリプトへ描画。
- debug backend は DAP-over-TCP で VSCode と接続済み（`crates/pasta_lua/src/debug/`、`editors/vscode/`）。
- ライブ SSP がプレビューを兼ねられる構造（別プレビュー画面は不要）。

## Desired Outcome
- **talk FIFO**（`pasta_shiori`）＋ **OnSecondChange drain**。
- **抑制**: SSP `Status: talking`（権威は SSP）で gate。会話中は通常 FIFO を消費しない（割り込まない）。
- **即時フラグ**: `talking` でも問答無用で FIFO 消費 → スクリプト応答 → **preempt**（中断側の前 `co_scene` は閉じる・自動復帰しない。デバッグキックは礼儀正しいキューではない）。
- **非即時キック**: 会話中は FIFO で待ち、アイドル（`talking` なし）で吐く。
- **キック源**: VSCode 拡張のコマンド/ボタン。transport は既存 debug DAP チャネルを再利用・一般化（custom request 例 `playScene`）。
- **キックは executor スレッドで非同期実行・レンダリング** → FIFO へ積む（GET ブロックは短く保つ）。OnSecondChange の GET は FIFO を drain して返すだけ。
- **ライブ SSP がプレビューを兼ねる**（作者は本物のゴーストの反応を観る）。
- **debug backend をアクターのクライアントとして吸収**（キックとデバッグを単一制御面に）。

## Approach
- 採用: FIFO ＋ `Status`-gated drain ＋ 即時 preempt。OnSecondChange の pull 機会を逆手に取り、**pull 契約を守ったまま ≤1秒でライブ SSP へ届く**。SSTP/`\![raise]` 不要。
- 却下: 別プレビュー画面（ライブ SSP で足りる）、SSTP push 出力（pull 衝突・別境界）、即時トークの退避→復帰（preempt-and-abort を採用）。

## Scope
- **In**: talk FIFO、`Status: talking` gate、即時 preempt（前 `co_scene` close）、非即時はアイドルで吐く、VSCode キックコマンド、debug DAP チャネル一般化（`playScene` custom request）、debug backend のアクタークライアント化、executor 非同期実行 → FIFO。
- **Out**: SSTP/`\![raise]` ライブ出力（`pasta-sstp-live-output`・別境界）、`*.pasta` 編集ウィンドウ（`pasta-authoring-window`・別境界）、`pasta_novel` adapter、即時トークの退避→復帰セマンティクス。

## Boundary Candidates
- talk FIFO ＋ drain 制御（`Status` gate ＋即時 preempt）
- キック transport（debug DAP 一般化）
- VSCode UI（コマンド/ボタン）

## Out of Boundary
- ライブ SSP 以外の出力先
- 礼儀正しいキュー復帰セマンティクス（デバッグキックは preempt-and-abort）

## Upstream / Downstream
- **Upstream**: `pasta-actor-runtime`（アクター/CH/presentation event stream の土台）、`pasta-vscode-lua-debug`（DAP チャネル）
- **Downstream**: `pasta-authoring-window`（将来・`*.pasta` ウィンドウからのキック）、`pasta-sstp-live-output`（将来）

## Existing Spec Touchpoints
- **Extends**: `pasta-vscode-lua-debug`（DAP チャネルに `playScene` を追加し、debug backend をアクタークライアント化）
- **Adjacent**: `pasta-debug-lua-view-toggle`（DAP custom request ＋ VSCode コマンドの実装前例）

## Constraints
- SHIORI/3.0 `Status` ヘッダ（`talking` 等）準拠
- OnSecondChange 配信レイテンシ ≤1秒（実 SSP の tick 周期依存）
- GET ブロックは短く（キックは executor で非同期実行）
- `editors/vscode` 拡張との整合
- 外部 SHIORI の通常会話挙動は不変（キックは追加経路）
