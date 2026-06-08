# Brief: pasta-source-map

## Problem
ゴースト作者・コントリビュータが VSCode で pasta ゴーストをデバッグする際、ブレークポイントを張れるのは**生成された `.lua` 上のみ**で、自分が書いた `.pasta` ソース上ではない。停止位置・コールスタック・BP もすべて `.lua` 行で提示されるため、「自分の書いた辞書のどの行で止まっているか」を頭の中で逆算する必要があり、デバッグ体験が分断されている。これは Phase 5「VSCode Lua デバッグ連携」の最終目標が未達であることを意味する。

## Current State
先行 spec `pasta-vscode-lua-debug`（2026-06-08 完了）により、**Lua レベルのデバッグは本番出荷済み**：
- Rust ホスト型 DAP バックエンド（`std::net::TcpListener` + `serde_json` 手書き DAP、`set_global_hook` + `jit.off()`）が稼働
- 生成 `.lua` 上での BP / ステップ / 変数 inspect / コルーチン inspect / VSCode attach が動作

加えて `.pasta` ソースマップの**実現可能性は確定済み**で、本番化のための「橋脚」が実装・E2E 実証されている：
- **code_gen 接合点**: `crates/pasta_lua/src/code_gen/mod.rs:49` の `writeln` 単一絞り込み点に出力行カウンタ `out_line` と `source_map: Option<&mut dyn SourceMapSink>` シームが実装済み（本番 `None` でゼロコスト）
- **マップ受け渡し IF**: producer 側 `code_gen::source_map::SourceMapSink` trait、consumer 側 `debug::source_map::{LineMap, SliceSink, resolve_lua_to_pasta}` が実装済み
- **DAP source 口**: `crates/pasta_lua/src/debug/dap.rs` の `SourceResolver` 差し替え口（既定は生成 `.lua` を返す）
- **薄い実証スライス**: 代表経路（単純 talk 1 行・`.pasta` 行2 → `.lua` 行11）について実トランスパイル由来の `LineMap` を構築し、`.lua`→`.pasta` 逆写像で `.pasta` 行 BP ヒットを実コードで E2E 実証（feature `pasta-source-map-slice`・default 無効・ゼロコスト）

**ギャップ（本仕様が埋めるもの）**：スライスは代表経路 1 本のみ。全 `generate_*` への span 引き回しは未着手、`normalize_output` の行ズレ補正は一般化されておらず、本番 `LineMap` の構築・保持・DAP `.pasta` resolver・常時提示は未実装。

## Desired Outcome
作者が VSCode で `.pasta` ファイルを開き、その行にブレークポイントを張ると、Lua デバッグ実行中に**その `.pasta` 行で停止**し、コールスタック・現在行・BP がすべて `.pasta` 座標で提示される。生成 `.lua` の存在を意識せずにデバッグできる（必要なら `.lua` レベルへ切替も可能）。Phase 5 の最終目標が達成された状態。

## Approach
先行 spec が確定した設計シームを**入力としてそのまま消費**し、実証スライスを本番品質へ拡張する：
1. **code_gen 全網羅**: 全 `generate_*` 関数群に `.pasta` の `span`（pasta_dsl AST から可用）を引き回し、`writeln` 単一点経由で `SourceMapSink::record(out_line, span)` を本番出力する
2. **行ズレ補正の一般化**: `normalize_output` による出力行のズレを算式化し、代表経路以外でも `LineMap` が正確に対応するようにする
3. **本番 LineMap**: トランスパイル時に本番マップを構築（メモリ既定）し、ランタイム/DAP バックエンドへ受け渡す。任意でディスクサイドカー出力も提供
4. **DAP `.pasta` resolver**: `SourceResolver` を `.pasta` 版で実装し、`setBreakpoints`（`.pasta` 行 → `.lua` 行群）/ `stackTrace`（`.lua` 行 → `.pasta` 行）を双方向解決。`.pasta`/`.lua` 提示モードを切替可能にする

## Scope
- **In**:
  - 全 `generate_*`（element_gen / scope_gen 等）への span 引き回しと本番ソースマップ出力
  - `normalize_output` 行ズレ補正の一般化（代表経路以外の正確性）
  - 本番 `LineMap` の構築・**メモリ内保持（既定）＋任意のディスクサイドカー出力**
  - DAP `.pasta` `SourceResolver` 実装、`setBreakpoints` / `stackTrace` の `.pasta`↔`.lua` 双方向解決
  - **`.pasta` 常時提示（既定）＋ `.pasta`/`.lua` 提示モード切替**（launch.json 等の設定で）
  - 実証スライス feature gate（`pasta-source-map-slice`）の本番化・整理
  - 多対多マッピング規則・`currentline` 複数バイトコード端ケースの確定
- **Out**:
  - Lua レベルのデバッグ基盤そのもの（DAP/transport/hook/inspect/session）— 先行 spec で出荷済み・改修は本仕様の波及最小限
  - `.pasta` の**編集時**ラウンドトリップ（フォーマッタ・逆生成）— デバッグ時マッピングに限定
  - `.lua` 以外の生成ターゲット
  - SHIORI 以外の pasta ホスト向け固有調整（デバッグ基盤の汎用性は先行 spec が担保済み）

## Boundary Candidates
- **producer 境界**: code_gen 内の span 引き回し・`SourceMapSink` 本番出力（pasta_lua / code_gen サブモジュール内で完結）
- **マップ表現境界**: `LineMap` の本番構築・保持・（任意）シリアライズ
- **consumer/DAP 境界**: `.pasta` `SourceResolver` と DAP `setBreakpoints`/`stackTrace` の双方向解決・提示モード切替
- **VSCode 拡張境界**: launch.json の提示モード設定・`.pasta` ソース解決（既存 `editors/vscode` への薄い追加）

## Out of Boundary
- DAP プロトコル本体・transport・hook・inspect・session（先行 spec 所有）
- pasta_dsl の AST/span 生成そのもの（既に span は可用・本仕様は消費側）
- 生成 `.lua` のセマンティクス変更

## Upstream / Downstream
- **Upstream**: `pasta-vscode-lua-debug`（完了・Lua デバッグ基盤と全設計シームを提供）、`pasta-lua-debug-feasibility`（GO+ 判定済み）
- **Downstream**: なし（Phase 5 の最終 spec。完成で Phase 5 完結）。将来 `pasta-runtime-internals-doc` 等のドキュメント整備が参照する可能性あり

## Existing Spec Touchpoints
- **Extends**: `pasta-vscode-lua-debug`（完了）が遺した `code_gen::source_map` / `debug::source_map` / `debug::dap::SourceResolver` シームと feature `pasta-source-map-slice` を本番化する
- **Adjacent**: `pasta-user-manual` / `pasta-manual-syntax-highlight`（VSCode TextMate 文法・マニュアルとは別境界・直接の重複なし）

## Constraints
- 組込 LuaJIT（mlua vendored 静的リンク）の範囲内。外部 C モジュール（.dll）に非依存の「プレーン実装」方針を維持
- 既存 DAP バックエンド（`std::net` + `serde_json` 手書き）との互換を維持し、依存追加を最小化
- ソースマップ producer は本番デフォルトでゼロコスト（feature OFF 時は出力バイト不変）を維持
- LuaJIT のラインフック（`set_global_hook` EVERY_LINE）が撃つ `.lua` 行を、本番 `LineMap` で `.pasta` 行へ正確に逆写像できること
- **検証負荷の注意**: 「メモリ＋任意ディスク出力」「`.pasta`/`.lua` 切替」の両柔軟性採用により、保持方式×提示モードの組合せ検証面が増える。tasks フェーズでマトリクスを明示すること
