# Brief: pasta-actor-runtime

## Problem
エンジンが SHIORI スレッドに束縛され、自前の時計を持たない。これが SHIORI(pull) とノベルゲーム(push/常駐) の整合を阻み、任意シーンキックの土台を欠く。`unsafe impl Send` ＋ Mutex ハックも温存している。さくらスクリプト描画という **SHIORI 固有の出力形式がエンジンコアに焼き込まれている**ため、宿主差し替えができない。

## Current State
- Lua VM（`!Send`）は `PastaLuaRuntime` 保持・`unsafe impl Send`＋`Arc<Mutex>` で SHIORI スレッド束縛（`crates/pasta_shiori/src/windows.rs`, `shiori.rs`）。
- さくらスクリプト処理が `pasta_lua`（エンジンコア）内に存在（tech.md: `pasta_lua` の SakuraScript）。
- 出力・継続はすべてホスト tick 駆動。
- `pasta-actor-feasibility` が GO 判定済みであることを前提に着手。

## Desired Outcome
- **宿主非依存エンジンコア**（`pasta_lua`、`!Send`・executor 非依存）が **presentation event stream**（talk ライン／アクター切替／wait／choice 等の宿主非依存マーカー）を出力する契約を確立。設計哲学「UI 独立性: Wait/Sync はマーカーのみ」を出力全体へ徹底。
- **アクタースレッド**（`pasta_shiori` が `wintf_winmsg_executor` で所有）が VM を pin し、全 VM アクセスを**単一直列キュー**で処理（データ競合ゼロ・順序保存）。
- **SHIORI marshaling**: SHIORI event を CH 送受信。GET＝応答 tx 付き（受信側が応答義務）、NOTIFY＝義務なし（即 204）、**drop→204 ガード**、GET ブロックは短く、エンジンは yield して block-wait しない。
- **さくらスクリプト描画を `pasta_shiori`（アダプタ）へ移設**。コア↔アダプタ境界＝presentation event stream に確定。
- **外部 SHIORI 挙動はバイト不変**（純内部リファクタ）。`unsafe impl Send` ハック解消。

## Approach
- 採用: 宿主非依存コア ＋ presentation event stream ＋ 差し替えアダプタ。executor 選択は**アダプタ層の決定**（SHIORI=`wintf_winmsg_executor`）でコア純度を保つ。
- 理由: SHIORI(pull) とノベルゲーム(push/常駐) を一コアで支えるにはコアが自前の時計を持つしかない。
- 進め方: リファクタは安全かつ可逆に — 特性化テスト先行＋1 抽出=1 検証=1 コミットの revert 可能な小ステップ。検証は速度より優先。

## Scope
- **In**: presentation event stream 契約定義、さくらスクリプト描画の `pasta_shiori` 移設、アクタースレッド＋VM pin、CH marshaling（GET/NOTIFY/drop→204）、単一直列キュー、reload teardown 本番化、`unsafe impl Send` 解消、全既存テスト回帰不変。
- **Out**: talk FIFO / `Status: talking` gate / 即時 preempt / キック transport（`pasta-scene-kick`）、SSTP 出力、`*.pasta` ウィンドウ、`pasta_novel` adapter、トーク/応答セマンティクス変更（非同期トーク等）。

## Boundary Candidates
- コア↔アダプタの presentation event stream 縫い目
- さくらスクリプト描画の移設
- アクタースレッド / executor / VM pin / teardown
- CH marshaling（GET/NOTIFY/drop→204）

## Out of Boundary
- 新しいユーザー可視挙動（本 spec は挙動バイト不変）
- キック FIFO とその UI（次 spec）

## Upstream / Downstream
- **Upstream**: `pasta-actor-feasibility`（GO 判定）
- **Downstream**: `pasta-scene-kick`（FIFO/キックの土台）、将来 `pasta_novel` adapter・`pasta-authoring-window`

## Existing Spec Touchpoints
- **Extends**: なし（新境界。ただし `pasta_lua` / `pasta_shiori` の中核を再編）
- **Adjacent**: `oversized-file-decomposition`（巨大ファイル分解・全ファイル<600行の知見）、`audit-workspace-patterns`（クレート横断パターン）、`shiori-async-talk`（`CALLBACK`/yield 基盤）

## Constraints
- SHIORI/3.0 準拠・**外部挙動バイト不変**
- LuaJIT 2.1 コルーチンモデル内
- `wintf_winmsg_executor`（`block_on`/`spawn_local`/`MessageLoop`、Send 不要）はアダプタ層に閉じ込める
- 既存 yield/resume（`STORE.co_scene`、`resume_until_valid`、`CALLBACK`）互換維持
- リファクタは特性化テスト先行・revert 可能な小ステップ・1 抽出=1 検証=1 コミット
