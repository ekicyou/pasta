# Brief: debug-startup-logging

## Problem
pasta ゴースト作者がデバッグモードでゴーストを起動しても、`pasta.log` には「デバッグが有効化された」「DAP バックエンドが待ち受けを開始した」ことを示すログが一切出ない。そのため、VSCode から attach する前に「そもそもデバッグモードで起動できているのか」を確認する手段がログ上に存在せず、接続失敗時の切り分けが難しい。

## Current State
- デバッグバックエンド（`crates/pasta_lua/src/debug/`）の `enable()`・`transport`・`wiring`・`session`・`hook` には **情報ログ（`info!`）が一切ない**（ソース確認済み・2026-06-08）。
- debug モジュールのログ出力は **警告 2 件のみ**:
  - `debug/mod.rs:98` `tracing::warn!` — `sourcePresentation`/`PASTA_DEBUG_SOURCE_MODE` に不正値が来たときのフォールバック警告。
  - `debug/source_map.rs:523` `tracing::warn!` — サイドカー書込失敗時。
- 正常にデバッグモードで起動・`127.0.0.1:9276` で待ち受け開始しても `pasta.log` は無言。
- `enable()` は `cfg.enabled == false` のとき即 `Ok(None)` を返し、ポートも開かない（ゼロコスト無効パス）。待ち受け開始は `Transport::start(cfg.listen)` 後に `local_addr` を取得している（`debug/mod.rs:638-639`）。
- `pasta_lua` は既に `tracing` 0.1 / `tracing-subscriber` / `tracing-appender` を依存に持ち、`pasta.log` へのロギング基盤が存在する（tech.md）。

## Desired Outcome
- デバッグを有効化してゴーストを起動すると、`pasta.log` に **デバッグ有効化＋待ち受け開始（バインドアドレス `host:port`）を示す `info!` ログ**が出力される。
- 利用者が attach 前に「デバッグモードで起動できている／どのポートで待ち受けているか」をログで確認できる。
- デバッグ**無効時は従来どおり完全に無言・ゼロコスト**（ログも出さない）を維持する。

## Approach
- `enable()` の待ち受け開始箇所（`Transport::start` 成功後・`local_addr` 取得後）に、実際にバインドしたアドレスを含む `tracing::info!` を 1 箇所追加する。
- 望ましくは「デバッグ有効化を検知した」段と「待ち受け開始（実バインドアドレス）」段を簡潔に出す（詳細はレベル感を設計で確定）。
- 既存の `tracing` 基盤を利用（新依存なし）。出力レベルは `info`。秘密情報は出力しない（出すのは loopback アドレスとポートのみ）。
- 無効パス（`!cfg.enabled` の早期 return）には一切手を入れず、ゼロコスト・無言を厳守。

## Scope
- **In**:
  - `enable()`（`crates/pasta_lua/src/debug/mod.rs`）への有効化／待ち受け開始の `info!` ログ追加。
  - 実バインドアドレス（`local_addr`）をログに含める。
  - 無効時は無言・ゼロコストの維持（非回帰）。
- **Out**:
  - クライアント attach/切断ごとの逐次ログ（必要なら将来検討。本仕様は起動確認に限定）。
  - ログフォーマット基盤・出力先（`pasta.log`）の変更（既存 tracing 基盤をそのまま使う）。
  - デバッグの機能挙動（BP/ステップ/変数/提示モード/サイドカー）の変更。
  - ドキュメント（マニュアル）への確認手順追記は別途 `pasta-manual-debugging` 側の小追補で対応（下記 Downstream）。

## Boundary Candidates
- 起動ログ出力（`enable()` 内の有効化・待ち受け開始箇所）。
- 無効パスのゼロコスト維持（早期 return に触れない）。

## Out of Boundary
- attach/切断の逐次ログ、ログ基盤・出力先の変更、デバッグ機能挙動の変更。

## Upstream / Downstream
- **Upstream**: `pasta-vscode-lua-debug`（完了）— `enable()`/transport の本番実装。`pasta-source-map`（完了）。本仕様はこの観測性ギャップを埋める派生。
- **Downstream**: `pasta-manual-debugging`（実装済み・GO）— 本起動ログが入ったら、デバッグ章（`book/src/debug/vscode-setup.md` または `index.md`）に「`pasta.log` でデバッグ起動を確認する」手順を小追補できる。現状マニュアルは「ログは出ない・`Get-NetTCPConnection -LocalPort 9276` でポート確認」と書く想定だったが、本仕様完了後は「ログ＋ポート確認」に更新可能。

## Existing Spec Touchpoints
- **Extends**: 実質的に `pasta-vscode-lua-debug`（完了・閉鎖済み）のデバッグバックエンド領域への観測性追加だが、独立した小デリバラブルのため新規単一仕様として切り出す。
- **Adjacent**: `pasta-manual-debugging`（ドキュメント・本仕様完了後に確認手順を追補）。

## Constraints
- 既存 `tracing` 基盤を使用（新依存なし）。
- デバッグ無効時のゼロコスト・無言・サンドボックス維持を厳守（R5 系の既存保証を壊さない）。
- 出力はループバックのバインドアドレス＋ポートのみ。秘密情報を出さない。
- `cargo test --all` 緑・既存デバッグ挙動の無回帰。LuaJIT ビルドは環境変数 `NoDefaultCurrentDirectoryInExePath` を外して実行する点に留意。
