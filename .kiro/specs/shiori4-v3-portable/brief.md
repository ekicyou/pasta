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
  - pasta のエンジンを、状態をインスタンスごとに持つ作りにする（下の「決めたこと」）
  - `shiori_get_api` の入口で、同じプロセスの 2 つ目の `load` を受けるか断るか（エンジンとは別の判断。要件で決める）
- **Out**:
  - 版 4（ホストへの同期の問い合わせ・post・OnHostComplete・Lua の API）。`shiori4-v4-host` が持つ
  - 旧来の HGLOBAL の入口で複数のインスタンスを扱うこと（1 つのまま）

## 決めたこと

- **pasta のエンジンは、状態をインスタンスごとに持ち、グローバルな状態を避ける**（2026-10-23 開発者の決定「設計としては別インスタンスを複数持てる設計を維持してください。グローバルな状態は避けたい」）。
  - これはエンジンの作りの話である。1 つのプロセスの中に、Lua VM・アクター・辞書・変数の組をいくつでも作れる作りにしておく。
  - 理由: 複数持てる作りにしておけば、グローバルな状態に頼る、原因の分かりにくい不具合を避けられる（開発者）。
  - shiori4 の入口で 2 つ目の `load` を許すかどうかは、これとは別の話である（開発者）。areka は helper のプロセスで動かすので、今は同じプロセスで 2 つ目の `load` は来ない（下の経緯）。受けるか断るかは要件で決める。
  - 例外は、SHIORI/3.0 の旧来の入口（`load`・`loadu`・`request`・`unload`）だけである。この ABI にはハンドルが無いので、その入口が使う 1 つのインスタンスの置き場は、プロセスに 1 つ持つほかない。置き場はその入口の中に閉じ込め、アクターや Lua の側にはグローバルな状態を持ち込まない。
  - 要件で決めること: 下の「プロセスに 1 つしかないもの」を、それぞれインスタンスへ移すか、移せない理由を書いて閉じ込めるか。tracing の購読者（ログの出す量の設定。`pasta_lua/src/logging/tracing_init.rs` の `FILTER_HANDLE`）とロガーの登録（`REGISTRY`）は、プロセスで共有する作りになっている。インスタンスごとに分けられるか（例: アクタースレッドごとに既定の dispatcher を持つ）を調べる。

## 経緯（2026-10-23）

- **2 つ目のインスタンス**。3 者会談の合意は「SHIORI は断ってよい（NULL＋ログ）。ただし、できれば受けるべき」。
  - プロセスに 1 つしかないもの（2026-10-23 に確かめた）: Lua VM はもともとアクタースレッドごとに 1 つ。ロガーの登録（`pasta_lua/src/logging/registry.rs` の `REGISTRY`）は load_dir ごとに振り分ける作りで、複数のゴーストを前提にしている。プロセスに 1 つしかないのは、`MAILBOX`・`ACTOR_HANDLE`（`actor/lifecycle.rs` の static）、`LOADU_INITIALIZED`（`windows.rs`）、tracing の `FILTER_HANDLE`（`logging/tracing_init.rs`）。デバッグの待受のポートは `pasta.toml` の `[debug] port` でゴーストごとに変えられる。
  - 2026-10-23 の後の知らせ: areka は SHIORI 1 つにつき helper のプロセスを 1 つ起こして動かすと決めた（areka の開発者の決定。v4 も SHIORI/3.0 も同じ）。areka のもとでは、同じプロセスで 2 つ目の `load` は起きない。
  - デバッグの待受は、ポートが塞がっていたら警告を出してデバッグなしで起動する作りが、すでにある（`pasta_lua/src/debug/enable.rs`）。2 つのプロセスや 2 つのインスタンスが同じポートを取り合っても、読み込みは失敗しない。
  - これを受けて一度は「断る」を推奨したが、開発者が上の決定（複数のインスタンスを持てる作りを保つ）をした。
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
  - 2026-10-23 の shiori4 の裁定（shiori4 のコミット `b5344f0`）:
    - SHIORI/3.0 の書き出し（`load`・`request`・`unload`）と `shiori_get_api` の両方を持つ DLL は許される（pasta の形）。ホストは先に `shiori_get_api` を見て、4 → 3 の順に試し、どちらも NULL なら SHIORI/3.0 の書き出しで読む。pasta は版 4 に NULL を返すので、shiori4 を知るホストは版 3 で読む。SSP は今どおり SHIORI/3.0 の書き出しで読む
    - 版 4 の `load` が NULL を返したら、ホストは SHIORI/3.0 として読み直さない（`shiori4_host` の `from_library`。DLL を 2 回読まないため。areka の要望）。版 3 の `load` が NULL のときに読み直すかは、知らせに書かれていない（`shiori4-v4-host` で版 4 を出すときに効く。この spec は版 4 に NULL を返すだけなので影響しない）
    - カレントディレクトリは規約で決めない。SHIORI は頼らずに `load` の dir を使う
    - ファイル名の決まり（abi-conventions の論点 12。確定）。`descript.txt` の `shiori,pasta.dll` に対して、ホストは次の順に探す
      - Windows x64: `pasta.x64.dll` → `pasta.dll`
      - Windows arm64: `pasta.arm64.dll` →（x64 のプロセスを起こせるホストだけ）`pasta.x64.dll` → `pasta.dll`
      - Linux: `libpasta.<x64|arm64>.so` → `libpasta.so`
      - macOS: `libpasta.dylib` だけ（ユニバーサルバイナリを推奨）
      - Linux・macOS は lib の付いた名前だけを探す。Rust の cdylib が作る `libpasta.so`・`libpasta.dylib` を、そのまま置ける。この spec の Windows 以外のビルドは、この名前に合わせる
  - 次の 1 つは、2026-10-23 時点で shiori4 の開発者の裁定待ちである。要件の前に結果を確かめる
    - 版 3 ではホストのテーブルが NULL なので、`load` の失敗の理由をどこに出すか
- **Downstream**: `shiori4-v4-host`（この spec の入口と実行器の上に、版 4 を足す）。areka（Windows 以外で pasta を読む）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `ponytail-shiori`（隙間の spec。unload がアクターのスレッドが DLL を出るまで待つ・FFI の型を C に合わせる。`windows.rs`・`actor/` を共有するので、同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（`.cargo/config.toml`。動的に戻さない）
- Windows の出荷物は 1 種類のまま（`pasta.dll` に `shiori_get_api` が増えるだけ）。debug の待受は 127.0.0.1 固定のまま
- shiori4 の ABI 規約に従う（UTF-8・NUL 終端なし・`out` は確保した側の `release` で解放・パニックは `SHIORI_E_PANIC`）
- 2026-10-23 の 3 者会談（pasta・areka・shiori4）の合意: `load` の dir は「絶対パス・OS の区切り・末尾に区切りあり・UTF-8」。失敗するときは理由をログに出す（版 3 はホストのログが無いので pasta のログに出す）。2 つ目のインスタンスは断ってよいが、できれば受ける（pasta の入口が受けるかは要件で決める。エンジンは複数を持てる作りにする）。「単一のスレッド」はインスタンスごとで、同じ DLL の 2 つのインスタンスは別々のスレッドから呼ばれうる
