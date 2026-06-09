# Brief: pasta-debug-lua-view-toggle

## Problem
pasta ゴースト作者が `.pasta` 行にブレークを張ってデバッグするとき、停止位置の**生成 `.lua` コードを確認したい**場面がある（トランスパイル結果の挙動確認・不具合切り分け）。現状はデバッグセッション中に提示を `.pasta`⇔`.lua` へ切り替える手段が利用者に露出しておらず、`.pasta` 提示に固定されている体感になっている。

## Current State
- 提示モード切替の**内部基盤は既存**:
  - `crates/pasta_lua/src/debug/mod.rs` の `SourceMode { Pasta, Lua }`（既定 `Pasta`）。
  - `crates/pasta_lua/src/debug/dap.rs` の `default_source_resolver()`（`.lua` 提示）/ `pasta_source_resolver()`（`.pasta` 提示・`resolve_lua_to_pasta()` 経由）。
  - `crates/pasta_lua/src/debug/wiring.rs` の `SharedSourceMode`・`attach_pasta_resolver()`、attach 引数 `sourcePresentation` による初期指定。
- 不足:
  - **デバッグ中の実行時トグルが無い**（attach 時の固定指定のみ。セッション開始後に切替不可）。
  - VSCode 拡張側に切替コマンド/ボタン等の UI が未露出（`editors/vscode/`）。
  - 「`.pasta` 行に BP を張ったまま停止 → `.lua` 表示」のワークフローが E2E で未検証。

## Desired Outcome
- デバッグセッション中に、**VSCode のコマンド/ボタンから `.pasta`⇔`.lua` の提示を即切替**できる。
- `.pasta` 行に張った BP はそのまま有効で、停止時の**スタックトレース・source 応答が選択中モードに応じて `.pasta` か `.lua` を提示**する。
- 切替は停止中・実行中いずれでも次の停止提示へ反映される（DAP カスタムリクエストで状態更新）。
- attach 引数 `sourcePresentation` による初期モード指定との整合（初期値 → 実行時トグルで上書き）。
- OFF 経路・既存挙動に無回帰。

## Approach
内部の `SharedSourceMode` を実行時に更新する DAP カスタムリクエスト（例: `pasta/setSourcePresentation` または reverse 不要の custom request）を追加し、VSCode 拡張にトグルコマンド（コマンドパレット＋デバッグツールバー/ボタン）を実装して接続する。
- バックエンド: カスタムリクエスト受信 → `SharedSourceMode.set()` → 以後の `stackTrace`/`source` 応答が新モードのレゾルバを使う。必要なら `invalidated`/`stopped` 系イベントで現在フレーム提示を再描画させる。
- フロントエンド（`editors/vscode/`）: `package.json` の `contributes.commands` ＋ `menus`（debug toolbar）でトグルを公開、`customRequest` を発行。`launch.json` の `sourcePresentation` を初期値として尊重。
- ステップ粒度との関係を設計で明確化: `Lua` モードでは `.lua` 粒度、`Pasta` モードでは `.pasta` 粒度（既存 `effective_mode()` 連動）。実行時切替時の進行中ステップの扱いを定義。

## Scope
- **In**:
  - 提示モードを実行時に更新する DAP カスタムリクエスト（バックエンド）。
  - VSCode 拡張のトグルコマンド/UI（コマンドパレット＋デバッグツールバー）。
  - 切替が `stackTrace`/`source` 提示へ即反映される接続と再描画。
  - 「`.pasta` BP のまま `.lua` 表示」を含む実 DAP-over-TCP E2E 検証。
  - マニュアル デバッグ章（`book/src/debug/source-level.md` の「提示モード切替」）の実行時トグル手順への更新。
- **Out**:
  - 同一 `.pasta` 行の再ブレーク抑制（→ pasta-debug-break-coalesce）。
  - 提示レゾルバ/ソースマップそのものの再設計（既存 `resolve_lua_to_pasta` を流用）。
  - `.lua` 提示時の独自シンタックス装飾等の表示拡張。

## Boundary Candidates
- バックエンド制御: `wiring.rs`（`SharedSourceMode` 更新口）＋ `dap.rs`（カスタムリクエスト処理・レゾルバ差し替え）。
- フロントエンド UX: `editors/vscode/`（コマンド・メニュー・customRequest 発行）。
- ドキュメント: `book/src/debug/source-level.md`。

## Out of Boundary
- 停止制御フロー（`session.rs`/`breakpoints.rs`）の Continue/ステップ判定（pasta-debug-break-coalesce の領域）。
- ソースマップ生成（`code_gen`）。

## Upstream / Downstream
- **Upstream**: pasta-source-map（完了・提示モード切替基盤と双方向マップ）、pasta-vscode-lua-debug（完了・DAP バックエンド/VSCode attach）。
- **Downstream**: なし（独立 UX 拡張）。マニュアル デバッグ章を更新。

## Existing Spec Touchpoints
- **Extends**: pasta-source-map の「提示モード `.pasta`/`.lua` 切替可能」を、attach 時固定から**実行時トグル＋VSCode UI 露出**へ前進させる。
- **Adjacent**: pasta-debug-break-coalesce（同じ `debug/` だが接触ファイル分離・依存なし）。pasta-manual-debugging（デバッグ章のドキュメント本体）。

## Constraints
- DAP 最小サブセット手書き実装（`serde_json`）の枠内でカスタムリクエストを追加。
- 静的リンク LuaJIT・トランスポートは Rust 側の既存方針を踏襲（外部 C モジュール非依存）。
- OFF（デバッグ無効）経路はバイト不変・ゼロコスト維持。
- VSCode 拡張のビルド（VSIX）は wasm-pack を要する既存パイプラインに従う。
- ドキュメントは mdBook を権威とし、`book/src/debug/` を更新（DEBUGGING.md は再掲しない）。
