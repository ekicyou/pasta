# Brief: pasta-debug-break-coalesce

## Problem
`.pasta` ソースレベルデバッグの利用者が、ブレークポイントで停止後に F5（Continue）を押しても、**同じ `.pasta` 行から抜け出せず再び同じ行で停止する**。1回の Continue で次の行へ進めず、何度も押す必要があり、ステップ実行の体感が壊れている。pasta ゴースト作者がデバッグでつまずく直接の原因。

## Current State
- `.pasta` ソースレベルデバッグは pasta-source-map で本番化済み（`.pasta` 行 BP・`.pasta` 座標停止・`.pasta` 粒度ステップを実 DAP-over-TCP で実証）。
- ただし Continue（F5）経路に「同一 `.pasta` 行を消化する」ロジックが欠落している。
- 根本原因の仮説（コード根拠付き）:
  - 1つの `.pasta` 行は複数の `.lua` 行へ展開される（`crates/pasta_lua/src/debug/source_map.rs:118` `lua_lines_for_pasta() -> Vec<u32>`）。
  - `.pasta` 行に張った BP は、対応する**全 `.lua` 行**へ登録される（`crates/pasta_lua/src/debug/breakpoints.rs:158` 付近、`resolve_pasta_to_lua()` の `Vec` 全要素を登録）。
  - Continue は `RunMode::Running` へ戻すだけ（`crates/pasta_lua/src/debug/session.rs:500` 付近）。次の `.lua` 行がまだ同じ `.pasta` 行を指すと `should_pause()` が即再ヒットする。
  - ステップ経路には `pasta_step_should_stop()`（`session.rs:433` 付近）で「同フレーム＋同一 `.pasta` 行は消化」する洗練があるが、**BP/Continue 経路には同等の抑制が無い**。

## Desired Outcome
- `.pasta` 行に張った BP は「`.pasta` 行を訪問するごとに1回だけ」停止する。
- BP で停止中に F5（Continue）すると、**同じ `.pasta` 行に対応する残りの `.lua` 行をすべて消化し、次の異なる `.pasta` 行（または次の BP/ステップ停止点）まで進む**。
- `SourceMode::Lua`（lua 提示・`.lua` 粒度）時は従来どおり `.lua` 行ごとに停止する（モードは直交、既存挙動を変えない）。
- 既存の Lua レベルデバッグ・OFF 経路バイト不変は無回帰。

## Approach
停止制御フローに「現在停止中の `.pasta` 行」を記録し、Continue 再開後はその `.pasta` 行を指す `.lua` 行での BP 再ヒットを、別の `.pasta` 行へ移るまで抑制する（pasta-line debounce）。
- `pasta_step_should_stop()` が持つ「同一 `.pasta` 行は消化」判定を、BP/Continue 経路にも適用するのが筋。`SourceMode::Pasta` かつ source_map ありのときのみ作用させ、`Lua` モード・map 無し時は現状の `.lua` 粒度を維持。
- 設計フェーズで、(a) `resolve_pasta_to_lua` で複数 `.lua` 行を全登録したまま停止側で集約する案 と (b) BP 登録を `.pasta` 行代表 `.lua` 行に絞る案 のトレードオフを比較する（停止精度・他経路への影響・実装局所性で判断）。
- 必ず回帰テストを追加（1つの `.pasta` 行 → 複数 `.lua` 行に展開されるケースで、Continue 1回が次の `.pasta` 行へ抜けることを実 DAP-over-TCP E2E で実証）。

## Scope
- **In**:
  - Continue（F5）経路で同一 `.pasta` 行を消化し、次の `.pasta` 行まで再ブレークを抑制するロジック追加。
  - `.pasta` 行 BP が `.pasta` 行訪問ごとに1回だけ発火することの保証。
  - 上記を実証する回帰テスト（多対1ソースマップ・Continue・E2E）。
- **Out**:
  - 提示モード（pasta/lua）の実行時トグル UX（→ pasta-debug-lua-view-toggle）。
  - ソースマップ生成（`code_gen`）の変更（マップは正しい前提。多対1は正常仕様）。
  - 条件付き BP・ログポイント等の新規 BP 種別。

## Boundary Candidates
- 停止判定の集約: `session.rs` の on_line hook 内 BP/Continue 判定と `pasta_step_should_stop()` の統合。
- BP 登録方式: `breakpoints.rs` の `.pasta` 行 → `.lua` 行群 登録（全登録 vs 代表登録）。

## Out of Boundary
- DAP の提示レゾルバ（`dap.rs`）・SourceMode 切替・VSCode 拡張 UI。
- ソースマップのデータ構造そのものの再設計。

## Upstream / Downstream
- **Upstream**: pasta-source-map（完了・`.pasta` 行 BP/座標停止/粒度ステップの本番基盤）、pasta-vscode-lua-debug（完了・DAP/停止状態機械）。
- **Downstream**: なし（独立バグ修正）。マニュアルのデバッグ章へ挙動の小追補があり得る。

## Existing Spec Touchpoints
- **Extends**: pasta-source-map の停止制御を補完（外部仕様の振る舞い改善・回帰修正）。
- **Adjacent**: pasta-debug-lua-view-toggle（同じ `debug/` だが接触ファイルが分離。`Lua` モード時は本バグが発生しないためモード直交）。

## Constraints
- LuaJIT 2.1 + mlua の `set_global_hook` / `jit.off` フックモデル内で実装。
- OFF（デバッグ無効）経路はバイト不変・ゼロコスト維持。
- 既存 Lua レベルデバッグ・既存テストスイートに無回帰。
- 検証は実 DAP-over-TCP E2E を含めること（pasta-source-map と同水準のフレッシュエビデンス）。
