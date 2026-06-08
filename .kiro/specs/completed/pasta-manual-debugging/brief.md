# Brief: pasta-manual-debugging

## Problem
Phase 5 で VSCode Lua デバッグ連携（`pasta-vscode-lua-debug` + `pasta-source-map`、いずれも 2026-06-08 完了）が `.pasta` ソースレベルまで本番化されたが、**利用者マニュアル（mdBook サイト・GitHub Pages 公開）にデバッグの使い方が一切載っていない**。ゴースト作者は、せっかく実装されたデバッグ機能の有効化方法・VSCode 接続手順・`.pasta` ソースレベルでの操作方法を知る手段がない。

## Current State
- **デバッグ機能本体**: 完成・出荷済み。`.pasta` 行ブレークポイント／`.pasta` 座標での停止・コールスタック提示／`.pasta` 粒度ステップ（コルーチン跨ぎ含む）／提示モード切替（`.pasta`/`.lua`）／DAP-over-TCP（既定 `127.0.0.1:9276`）で attach。有効化は `pasta.toml [debug]` または環境変数 `PASTA_DEBUG` / `PASTA_DEBUG_PORT`。無効時は本番ゼロコスト・サンドボックス維持。
- **既存資産 `DEBUGGING.md`（ルート）**: 運用ガイドとして有用（全体像・有効化・「ブレーク中はホスト応答が止まる」構造的制約・緩和策）。ただし **内容が陳腐化**: 「`.pasta` ソースレベルデバッグは実験的・将来の別仕様」と記述しているが、その別仕様（`pasta-source-map`）は既に完了・本番化済み。
- **mdBook マニュアル（`book/src/`）**: SUMMARY.md は「はじめに／入門・チュートリアル／Pasta DSL 文法／Lua API・コーディング／リファレンス」のみ。**デバッグ章が存在しない**。
- **SSOT 状況**: `doc/spec/`（ch01–12）は Pasta DSL 文法の正準仕様で、`drift-check`（`book/tools/drift-check.mjs` + `book/manual-sources.toml`）が文法章 → `doc/spec` の追従を機械検証。**デバッグに関する `doc/spec` 文書は存在しない**ため、デバッグ章は競合する `doc/spec` 権威を持たず、mdBook を権威にできる（ch08/ch12 同様、ドリフト追跡対象外）。

## Desired Outcome
- mdBook 利用者マニュアルに **デバッグ章**（新セクション）が追加され、GitHub Pages の公開サイトから検索・閲覧できる。
- ゴースト作者が、有効化 → VSCode 設定（`launch.json`）→ attach → `.pasta` ソースレベルでの BP/ステップ/変数 inspect/提示モード切替、までを章だけで完遂できる。
- 「ブレーク中はホスト（SHIORI/SSP）応答が止まる」構造的制約と SSP タイムアウト回避の運用緩和策が、最新の本番挙動に整合した形で説明されている。
- **デバッグ内容の権威が mdBook に一本化**され、ルート `DEBUGGING.md` は陳腐化した重複ではなく、マニュアルへのリダイレクト（薄い誘導）になる。

## Approach
ユーザー決定（推奨案・完全網羅）に基づく:
1. mdBook に新しいデバッグ章（`book/src/debugging/` 配下、SUMMARY.md に新セクション追加）を作成。`.pasta` ソースレベルまで完全網羅（有効化／VSCode `launch.json`／attach 手順／`.pasta` BP・ステップ・変数 inspect・コルーチン inspect・提示モード切替／構造的制約と緩和策）。
2. 既存 `DEBUGGING.md` の内容を章へ移植し、**最新の本番挙動へ更新**（`.pasta` ソースレベルは「実験的・将来」ではなく「本番提供」へ）。
3. ルート `DEBUGGING.md` はマニュアル該当章への薄いリダイレクト／誘導に置換（情報源を一本化し二重管理を防ぐ）。
4. デバッグ章は `doc/spec` 由来を持たないため `manual-sources.toml` のドリフト追跡には登録しない（mdBook 自体を権威とする）。

## Scope
- **In**:
  - mdBook デバッグ章の新規作成（SUMMARY.md への章追加含む）
  - `.pasta` ソースレベルまでの完全な利用手順（有効化・`launch.json`・attach・BP/ステップ/変数 inspect/コルーチン inspect/提示モード切替）
  - 構造的制約（ブレーク中ホスト応答停止）と SSP タイムアウト緩和策の最新版記述
  - `DEBUGGING.md` のマニュアルへの統合（移植＋最新化）と、ルートファイルのリダイレクト化
  - mdBook ビルド健全性確認（既存 `book/tools` の検証スクリプトとの整合）
- **Out**:
  - デバッグ機能そのものの実装変更（完成済み・本仕様は文書のみ）
  - 文法・Lua API を含む **マニュアル全体の SSOT/権威化の再編**（doc/spec vs mdBook の権威整理）→ 別仕様（下記 Downstream）
  - ランタイム内部設計の解説（既存将来仕様 `pasta-runtime-internals-doc` の領域）
  - 構造的制約の根本解決（ホスト非同期化アーキテクチャ）

## Boundary Candidates
- マニュアル章コンテンツ（執筆・SUMMARY.md 統合・検索可能性）
- `DEBUGGING.md` の移植・最新化・リダイレクト化（情報源一本化）
- mdBook ビルド／検証整合（drift 対象外の明示・既存ツールとの非回帰）

## Out of Boundary
- デバッグバックエンド（Rust/DAP）コードの変更
- マニュアル全体の権威化再編（別仕様）
- ランタイム内部設計解説（別将来仕様）

## Upstream / Downstream
- **Upstream**:
  - `pasta-vscode-lua-debug`（完了）— Lua レベルデバッグ・有効化・DAP attach の本番挙動
  - `pasta-source-map`（完了）— `.pasta` ソースレベル BP/停止/コールスタック/提示モード切替の本番挙動
  - `pasta-user-manual`（完了）— mdBook サイト基盤・SUMMARY.md 構造・`book/tools` 検証
  - 既存 `DEBUGGING.md` — 章コンテンツの主要シード
- **Downstream**:
  - **将来仕様（提案）**: マニュアル全体の SSOT/権威化整理 — 文法・Lua 含め「mdBook に書く項目は mdBook を権威にする」方針の確立と、`doc/spec` との並行管理の解消。ユーザー指摘（2026-06-08）により本仕様外・別仕様として申し送り。

## Existing Spec Touchpoints
- **Extends**: 実質的に `pasta-user-manual`（完了・閉鎖済み）のマニュアル領域への新章追加だが、独立した完結デリバラブルのため新規単一仕様として切り出す
- **Adjacent**:
  - `pasta-manual-syntax-highlight`（`.pasta` コードブロックのハイライト — デバッグ章のコード例にも適用される）
  - `pasta-runtime-internals-doc`（将来仕様・読者層が異なる別境界・重複回避）

## Constraints
- マニュアルは mdBook（サーバー不要の静的 HTML+JS）。追加エコシステム依存を持ち込まない。
- spec.json `language = ja`。マニュアル本文・spec ドキュメントは日本語で執筆。
- 既存 `book/tools` の検証（drift-check / static / search 等）を壊さない。デバッグ章は doc/spec 由来なしのため drift マッピング非対象。
- 記述する本番挙動は完了済み仕様（`pasta-vscode-lua-debug` / `pasta-source-map`）の実装に整合させる（陳腐化記述の再発防止）。
