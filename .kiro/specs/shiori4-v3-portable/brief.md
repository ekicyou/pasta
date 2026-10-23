# Brief: shiori4-v3-portable

> **ステータス**: 未着手（2026-10-23 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-23 開発者の意向「今やってるマイナーバージョンアップに入れちゃってもよいかな」）。要件定義は Opus。

## Problem

pasta.dll は Windows でしか動かない。

- 入口が SHIORI/3.0 の DLL の形（`load`・`request`・`unload` と HGLOBAL）だけである。
- Lua VM を動かすアクタースレッドの実行器が、Windows のメッセージ専用ウィンドウ（`wintf-winmsg-executor`）に頼っている。

Windows 以外の互換ベースウェア（areka など）は、pasta を読み込めない。

## Current State

2026-10-23（main `c360adb2`）に確かめた。

- **入口**: `crates/pasta_shiori/src/windows.rs` の `load`・`loadu`（UTF-8 の dir）・`request`・`unload`。HGLOBAL で受け渡す。
- **アクター**: `crates/pasta_shiori/src/actor/`。Lua VM はアクタースレッドに固定されている。SHIORI のスレッドは mailbox（flume）で GET を投げ、応答を待つ。NOTIFY は受け取ったらすぐ 204 を返し、処理はアクターが後から行う。unload は Stop を送り、ack を最大 5 秒待つ。スレッドは join せずに detach する。
- **Windows だけの依存**: `wintf-winmsg-executor`（通常の依存。cfg で囲っていない）、`windows-sys`（teardown のリーク検査）。`pasta_lua` は `encoding/windows.rs`・`ffi_load/win32.rs` などを `cfg(windows)` で分けている。
- **1 プロセス 1 インスタンス**: static な mailbox・ロガーの登録・デバッグの待受が 1 つずつある。
- **shiori4**（`ekicyou/shiori4`。crates.io に 0.0.1 公開済み）: 公開シンボル `shiori_get_api` が 1 つで、`#[repr(C)]` の関数テーブルを返す C ABI である。版 3（SHIORI3 互換モード）では、`request` に SHIORI/3.0 の要求の全文（UTF-8）を渡し、応答の全文を受け取る。ホストのテーブルは NULL。メモリは確保した側が `release` で解放する（pasta.dll の静的 CRT リンクと合う）。

## Desired Outcome

- pasta.dll が `shiori_get_api(3)` を書き出し、shiori4 の SHIORI3 互換モードの SHIORI として読み込める。中身は今の SHIORI/3.0 の処理そのままである。
- 同じ pasta のソースから、Windows 以外（Linux・macOS）の共有ライブラリ（`.so`・`.dylib`）をビルドでき、テストが通る。
- Windows の今の入口（`load`・`loadu`・`request`・`unload`）は、そのまま残る。SSP での動きは変わらない。

## Approach

2026-10-23 の開発者の方針: pasta の shiori4 対応は 2 段に分ける。

1. **この spec（0.4.2）**: SHIORI4 のためというより、「SHIORI/3.0 を Windows 以外で動かすための別口」として、shiori4 の版 3 を出す。
2. **`shiori4-v4-host`（0.5.0。ノベルゲームと同じリリース）**: 版 4（ホストの呼び出し・post・完了通知）に本格的に対応する。

版 3 の要求は SHIORI/3.0 の全文なので、今の解析と応答の組み立てを流用できる。新しく作るのは、`shiori_get_api` の入口と、アクターの実行器を OS に頼らない形にすることである。

## Scope

- **In**:
  - `shiori_get_api` の書き出し（版 3 だけを返す。版 4 を求められたら NULL）。`shiori4_abi` に依存してよい（std だけのクレート）
  - 版 3 の `load`（UTF-8 の dir。ホストのテーブルは NULL）・`unload`・`request`（GET と NOTIFY の全文）・`notify`（NOTIFY の全文。GET なら 400）を、今のアクターへつなぐ
  - アクタースレッドの実行器を、Windows 以外でも動く形にする（Windows でメッセージ専用ウィンドウが要る事情があるなら、それを保つか、要らなくするかを要件で決める）
  - Windows だけの依存を `cfg(windows)` に閉じ込め、Linux・macOS でビルドとテストが通るようにする
  - CI で Linux（と、できれば macOS）のビルドとテストを回す
  - 2 つ目のインスタンスの扱い（要件で決める。下の「要件で決めること」）
- **Out**:
  - 版 4（ホストへの同期の問い合わせ・post・OnHostComplete・Lua の API）。`shiori4-v4-host` が持つ
  - 旧来の HGLOBAL の入口で複数のインスタンスを扱うこと（1 つのまま）

## 要件で決めること

- **2 つ目のインスタンス**。areka は 1 つのプロセスで複数のゴーストを動かす。2 体が同じ pasta.dll を指すと、`load` が 2 回呼ばれる。3 者会談の合意は「SHIORI は断ってよい（NULL＋ログ）。ただし、できれば受けるべき」。
  - 2026-10-23 に確かめた pasta の実情: Lua VM はもともとアクタースレッドごとに 1 つ。ロガーの登録（`pasta_lua/src/logging/registry.rs`）は load_dir ごとに振り分ける作りで、複数のゴーストを前提にしている。プロセスに 1 つしかないのは、`MAILBOX`・`ACTOR_HANDLE`（`actor/lifecycle.rs` の static）、`LOADU_INITIALIZED`（`windows.rs`）、デバッグの待受（ポート固定 9276）の 3 つ。
  - 推奨: `shiori_get_api` の入口では複数のインスタンスを受ける。新しい入口は inst を返すので、static をインスタンスへ移す好機である。デバッグの待受は、2 つ目以降のインスタンスでは開かない。`shiori4-v4-host` にこの作業を回すと、0.5.0 まで areka で 2 体の pasta ゴーストを同じプロセスに置けない。
- **unload の待ち時間**。3 者会談の合意で、「unload は長く止まらないこと。ホストは OS の終了の猶予の中で各インスタンスの unload を呼ぶことがある」と注意が入る（数字は書かない）。pasta は ack を最大 5 秒待つ。インスタンスが複数あると、待ちが重なる。上限を見直すかを決める。
  - Windows 以外の配布物をリリースに載せるか（要件で決める。推奨は「この spec では載せない。CI のビルドとテストで守るだけ」）
  - ゴーストの配布（nar）や `pasta_check` の Windows 以外への対応

## Boundary Candidates

- 入口の層（`shiori_get_api` とテーブル。HGLOBAL の入口と並べる）
- アクターの実行器（OS ごとの差を閉じ込める所）
- ビルドと CI（cfg の囲い・Windows 以外のジョブ）

## Out of Boundary

- SHIORI/3.0 の要求の解析と、Lua に渡す `req` の形（変えない）
- `pasta_lua` の Lua の振る舞い

## Upstream / Downstream

- **Upstream**: shiori4（`shiori4_abi` の版 3 の取り決め。0.0.1 で公開済みなので、shiori4 の Phase 2 の spec を待たずに始められる）
- **Downstream**: `shiori4-v4-host`（この spec の入口と実行器の上に、版 4 を足す）。areka（Windows 以外で pasta を読む）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `ponytail-shiori`（隙間の spec。unload がアクターのスレッドが DLL を出るまで待つ・FFI の型を C に合わせる。`windows.rs`・`actor/` を共有するので、同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（`.cargo/config.toml`。動的に戻さない）
- Windows の出荷物は 1 種類のまま（`pasta.dll` に `shiori_get_api` が増えるだけ）。debug の待受は 127.0.0.1 固定のまま
- shiori4 の ABI 規約に従う（UTF-8・NUL 終端なし・`out` は確保した側の `release` で解放・パニックは `SHIORI_E_PANIC`）
- 2026-10-23 の 3 者会談（pasta・areka・shiori4）の合意: `load` の dir は「絶対パス・OS の区切り・末尾に区切りあり・UTF-8」。失敗するときは理由をログに出す（版 3 はホストのログが無いので pasta のログに出す）。2 つ目のインスタンスは断ってよいが、できれば受ける。「単一のスレッド」はインスタンスごとで、同じ DLL の 2 つのインスタンスは別々のスレッドから呼ばれうる
