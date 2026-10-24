# Brief: shiori4-v3-portable

> **ステータス**: 未着手（2026-10-23 の `/kiro-discovery` で起票）。0.4.3 の最後（0.4.2 に入れる意向だったが、同じ日に 0.4.3 へ移した。開発者の決定「これは 0.4.3」）。2026-10-24 の `/kiro-discovery` で shiori4 0.1.0 に合わせた（末尾の「shiori4 0.1.0 への合わせ」。上の本文より末尾の節が新しい）。要件定義は Opus。

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
- **shiori4**（`ekicyou/shiori4`。2026-10-24 に crates.io へ 0.1.0 を公開）: 公開シンボル `shiori_get_api` が 1 つで、`#[repr(C)]` の関数テーブルを返す C ABI である。版 3（SHIORI3 互換モード）では、`request` に SHIORI/3.0 の要求の全文（UTF-8）を渡し、応答の全文を受け取る。0.1.0 から、版 3 の `load` の host は `copy_from_slice` だけを持つテーブルで、応答の `out` は常にホストが作る（SHIORI は `copy_from_slice` で写す）。0.0.2 以前の「ホストのテーブルは NULL」「確保した側が `release` で解放する」は古い（末尾の節）。

## Desired Outcome

- pasta.dll が `shiori_get_api(3)` を書き出し、shiori4 の SHIORI3 互換モードの SHIORI として読み込める。中身は今の SHIORI/3.0 の処理そのままである。
- 同じ pasta のソースから、Windows 以外（Linux・macOS）の共有ライブラリ（`.so`・`.dylib`）をビルドでき、テストが通る。
- Windows の今の入口（`load`・`loadu`・`request`・`unload`）は、そのまま残る。SSP での動きは変わらない。

## Approach

2026-10-23 の開発者の方針: pasta の shiori4 対応は 2 段に分ける。

1. **この spec（0.4.3）**: SHIORI4 のためというより、「SHIORI/3.0 を Windows 以外で動かすための別口」として、shiori4 の版 3 を出す。
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
    - SHIORI/3.0 の書き出し（`load`・`request`・`unload`）と `shiori_get_api` の両方を持つ DLL は許される（pasta の形）。ホストは `shiori_get_api` → `loadu` → `load` の順に探し、最初に見つかった口で解決する（shiori4 のコミット `b3d93ce` で確定）。`shiori_get_api` を持つ DLL は、4 も 3 も NULL なら読めなかったゴーストになり、`loadu`・`load` へは戻らない。pasta は版 4 に NULL を返すので、shiori4 を知るホストは版 3 で読む。SSP は今どおり SHIORI/3.0 の書き出しで読む。`shiori4_host` には許可する口の集合（4・3・loadu・load）を渡せるので、pasta の CI で同じ DLL を従来の口で試すときは `loadu` と `load` だけを許可する
    - 版 4 の `load` が NULL を返したら、ホストは SHIORI/3.0 として読み直さない（`shiori4_host` の `from_library`。DLL を 2 回読まないため。areka の要望）。版 3 の `load` が NULL のときに読み直すかは、知らせに書かれていない（`shiori4-v4-host` で版 4 を出すときに効く。この spec は版 4 に NULL を返すだけなので影響しない）
    - カレントディレクトリは規約で決めない。SHIORI は頼らずに `load` の dir を使う
    - ファイル名の決まり（abi-conventions の論点 12。確定）。`descript.txt` の `shiori,pasta.dll` に対して、ホストは次の順に探す
      - Windows x64: `pasta.x64.dll` → `pasta.dll`
      - Windows arm64: `pasta.arm64.dll` →（x64 のプロセスを起こせるホストだけ）`pasta.x64.dll` → `pasta.dll`
      - Linux: `libpasta.<x64|arm64>.so` → `libpasta.so`
      - macOS: `libpasta.dylib` だけ（ユニバーサルバイナリを推奨）
      - Linux・macOS は lib の付いた名前だけを探す。Rust の cdylib が作る `libpasta.so`・`libpasta.dylib` を、そのまま置ける。この spec の Windows 以外のビルドは、この名前に合わせる
    - 版 3 の `load` の失敗の理由（abi-conventions の論点 16・17。確定）: ABI では理由を返さない。NULL を返すだけで、規約は今のまま。理由は pasta 自身のログに出す。`shiori4_host` は版 3 を読む口を持ち、`load` の失敗はホストに `Err(LoadFailed)` として届く。読み込みは `shiori4_host` が持ち、`shiori4_abi` は型と定数だけを持つ
  - これで shiori4 の側の議題は、2026-10-23 にすべて決まった
  - 試験: `shiori4_host` の v3 の口と許可する口の集合（`load_ghost_with` と `EntryKind`）は、2026-10-24 の 0.1.0 に入った（0.0.2 で入る予定と書いていたもの）。試験用のホスト `shiori4-testhost` も 0.1.0 に入った。使い方は末尾の節
- **Downstream**: `shiori4-v4-host`（この spec の入口と実行器の上に、版 4 を足す）。areka（Windows 以外で pasta を読む）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `ponytail-shiori`（隙間の spec。SHIORI/3.0 の書き出しの型（`bool`→i32 の `BOOL`、`usize`→`c_long`）を直す。この spec も `windows.rs` を触るので、先に入るほうが直してよい。2026-10-23 に shiori4 の `shiori3-legacy-host` が「BOOL は 32bit の int で 0 以外を真と読む（下位 1 バイトで読む守りは採らない）」と確定した。今の `-> bool` は EAX の上位が不定なので、失敗を真と読まれうる。推奨は、`load`・`loadu`・`unload` の戻り値を i32 の 1／0 に直すことを、この spec の In に入れること（要件で決める）。unload がアクターのスレッドが DLL を出るまで待つ・FFI の型を C に合わせる。`windows.rs`・`actor/` を共有するので、同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（`.cargo/config.toml`。動的に戻さない）
- Windows の出荷物は 1 種類のまま（`pasta.dll` に `shiori_get_api` が増えるだけ）。debug の待受は 127.0.0.1 固定のまま
- shiori4 の ABI 規約に従う（UTF-8・NUL 終端なし・`out` は常にホストが作り、SHIORI は `host.copy_from_slice` で写す（0.1.0 から。ABI 規約 5・6）・パニックは `SHIORI_E_PANIC`）
- 2026-10-23 の 3 者会談（pasta・areka・shiori4）の合意: `load` の dir は「絶対パス・OS の区切り・末尾に区切りあり・UTF-8」。失敗するときは理由をログに出す（版 3 はホストのログが無いので pasta のログに出す）。2 つ目のインスタンスは断ってよいが、できれば受ける（pasta の入口が受けるかは要件で決める。エンジンは複数を持てる作りにする）。「単一のスレッド」はインスタンスごとで、同じ DLL の 2 つのインスタンスは別々のスレッドから呼ばれうる

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**: 起票の後に入った 4 本（external-control-events・reg-handler-chain・chain-talk-interrupt・talk-grapheme-clusters）は `crates/pasta_shiori` を触っていない（最後の変更は #166）。Current State はそのまま合っている。crates.io の `shiori4`・`shiori4_abi`・`shiori4_host` は 0.0.1 だけ（0.0.2 はまだ）。`shiori4_abi` は依存が 0 で MIT（`deny.toml` の許可に入る）、rust-version 1.85。版 3 の定数・`ShioriApi`（`load` がインスタンスのポインタを返す）・`Shiori` トレイト（`SUPPORTS_V3`）・書き出しのマクロ `export_shiori!` が 0.0.1 に既にある。手書きせずにこれを使える。
- **実行器**: アクターの中の `await` は `thread.rs` 173 行の `rx.recv_async()` の 1 つだけで、pasta のどこにもウィンドウのメッセージを使う所が無い。メッセージ専用ウィンドウが要る事情は見つからない。推奨は、全部の OS で `rx.recv()` の素のループにして `wintf-winmsg-executor` を外すこと（依存が減る）。
- **すでに囲ってあるもの**: `lib.rs` の `mod windows`、`util/hglobal`、`windows-sys`（両クレートとも `cfg(windows)` の依存）、`pasta_lua` の `encoding`・`ffi_load`、`teardown.rs` の測り（`cfg(not(windows))` の代わりあり）。残る Windows だけの依存は `wintf-winmsg-executor` だけ。
- **プロセスに 1 つのもの（追加）**: 上の一覧に無いものとして、キックの投函口 `kick_sink`（`lifecycle.rs` 184〜227 行）が `static MAILBOX` を読む。インスタンスへ移すなら、送り先を閉じ込めた投函口にする。`init_tracing_with_reload` は load のたびに呼ばれ（`shiori.rs` 89 行）、`FILTER_HANDLE` は 1 つなので、2 つ目のインスタンスの `[logging]` の量が 1 つ目にも効く。アクタースレッドごとの `tracing::dispatcher::set_default` が候補。`REQUEST_SEQ`・`UNIQUE_COUNTER` は数を振るだけで害は無い。
- **触るファイル**: `pasta_shiori` の `Cargo.toml`・`lib.rs`・`windows.rs`（335 行）・`actor/{lifecycle,thread,teardown,mod}.rs`・新しい版 3 の入口（新規）・テスト（新規 1〜2 と、wintf を使う `tests/actor_{marshaling,teardown,thread_vm}_test.rs`）・`README.md`。ルートの `Cargo.toml`・`Cargo.lock`（`shiori4_abi` を足し、wintf を外す）。`.github/workflows/build.yml`（149 行。Linux の job は今 1 つも cargo を回していない）。インスタンスに移すなら `pasta_lua/src/logging/{tracing_init,registry}.rs`。マニュアル `internals/shiori.md`（42〜74 行の図と表）、`tech.md` 63 行（wintf）。1,000 行に近いものは無い。
- **規模**: 14〜18 のまま。いちばんの危険は、Linux で初めて回すワークスペースのテスト。`"C:\\…"` の文字列を持つファイルが `pasta_lua/src/debug/*_tests.rs`・`loader/discovery_tests.rs`・`loader/context.rs`・`pasta_check/src/balloon.rs` などに 10 ほどある（さくらスクリプトの `\` もまじる）。
- **先に要るもの**: 機能の上では無い（shiori4 0.0.1 で始められる）。ファイルの重なり: `recommend-menu-scenes` とは重ならない（あちらは `pasta_core` の別名・`pasta_lua/src/loader/config/`・`pasta_check/src/validate/config.rs`・Lua。こちらは `loader/config/` を触らない約束にする）。隙間の spec では `ponytail-shiori`（`windows.rs`・`actor/`）、`ponytail-manual-ci`（`build.yml`）、`ponytail-core-registry`・`ponytail-editor-tooling`（`Cargo.lock` の席）、Linux のテストを囲うなら `ponytail-lua-debug`（`debug/**`）・`ponytail-lua-loader`（`loader/**`）。どれも 0.4.2 の後なので、こちらが先に入り、あちらが合わせる。
- **種別**: 機能（Windows 以外で読める別口。開発者の意向で 0.4.2 に入れた）。
- **要件定義のモデル**: Opus。2 つ目の `load` は「受ける／断る（NULL＋ログ）」の 2 つに絞れていて、エンジンを複数持てる作りにする決定と 3 者会談の「できれば受ける」から、推奨（受ける）が立つ。インスタンスの状態はエンジンの内側の作りで、DSL・保存の形・公開の Lua の取り決めではない（Fable の (i) に当たらない）。ABI は shiori4 の正本が決めている。
- **分割の案**: 今は分けない。20 を超えたら、`pasta-portable-build`（実行器・cfg の囲い・Linux と macOS の CI まで）と、この spec（`shiori_get_api`・インスタンスの状態・2 つ目の load）に分ける。
- **見つけた穴・古くなった記述**: Constraints の「パニックは `SHIORI_E_PANIC`」は、リリースの `panic = "abort"`（ルートの `Cargo.toml` 85 行）の下では効かない（`catch_unwind` が受けられない。今の `windows.rs` も同じ）。要件に書く。Upstream の shiori4 のコミット `b5344f0`・`b3d93ce` は、shiori4 の main からは辿れない（squash で PR #7・`2d8178d` に入った）。引くなら PR #7 を指す。Linux・macOS では静的 CRT の指定は当てはまらない（Windows だけの決まりと書く）。`about.toml` の `targets` に Linux を足すかは、配布しないなら要らない。

## 2026-10-24 棚卸の再測定（main c20ad104）

- **前提の変化**
  - 前回（main `82176f19`）の後、`pasta_shiori` に入ったのは依存の更新（PR #187）だけ。`wintf-winmsg-executor` を 0.0.3 から 0.0.5 に上げた（ウィンドウの種類の口と既定のカーソルの追加。pasta は `block_on` だけを使うので当たらない）。試験のファイルは、環境変数を中和する関数の印が 1 行ずつ替わっただけ。
  - アクターの中の待ちは今も `actor/thread.rs` 173 行の `rx.recv_async()` の 1 つで、`wintf` を外す推奨はそのまま。
  - 2026-10-24 の shiori4 からの知らせ（`shiori4-v4-host` の brief の末尾）: pasta は `shiori4::Ghost` でなく `shiori4_abi::Shiori` を直に使う。この spec は今の SHIORI/3.0 の全文の形のまま入れ、内の表現の作り直しは `shiori4-v4-host` が持つ。この spec の範囲は変わらない。
- **触るファイル**: 前回のまま（`pasta_shiori` の `Cargo.toml`・`lib.rs`（26）・`windows.rs`（335）・`actor/` の `lifecycle.rs`（329）・`thread.rs`（272）・`teardown.rs`（345）・`mod.rs`（42）、版 3 の入口（新規）、試験、`README.md`。ルートの `Cargo.toml`・`Cargo.lock`。`.github/workflows/build.yml`（149）。インスタンスに移すなら `pasta_lua/src/logging/`。マニュアル `internals/shiori.md`。`tech.md` 63 行）。2 つ目のインスタンスでログの量を分けるなら、`pasta_shiori/src/shiori.rs` 89 行（`init_tracing_with_reload` の呼び出し）も触る。
- **規模**: 14〜18 のまま。
- **先に要るもの**
  - 働きの上では無い（`shiori4_abi` 0.0.1 に版 3 の型と書き出しのマクロがある）。
  - ファイルの順: `mlua-0-12-upgrade` と `Cargo.toml`・`Cargo.lock`・`pasta_shiori/src/shiori.rs`（`mlua::` を参照）を共有するので、その後。ほかの 0.4.3 の 4 本とは重ならない。
  - これ以外に「まだ実施できない」理由は、リポジトリに書かれていない（2026-10-23 の開発者の決定）。始める時期は開発者が決める。
- **種別**: 機能（Windows 以外で読める別口。開発者の意向）。
- **要件定義のモデル**: Opus（前回のまま。2 つ目の `load` は 2 つに絞れて推奨がある）。
- **分割の案**: 前回のまま（20 を超えたら `pasta-portable-build` と分ける）。
- **見つけた穴・古くなった記述**
  - 冒頭のステータス（「0.4.2 に入れる」）と Approach の「この spec（0.4.2）」は古い。今は 0.4.3 の最後。
  - `crates/pasta_shiori/README.md` 92 行の `wintf-winmsg-executor` が 0.0.3、112 行の `ctor` が 0.2 のまま（PR #187 で 0.0.5・1.0）。この spec で `wintf` を外すなら、92 行は行ごと消す。
  - crates.io の `shiori4` 0.0.2 が出たかは、今回は確かめていない。

## 2026-10-24 棚卸の再測定（main `1fb7ed38`）

- **前提の変化**: 前回（main `c20ad104`）の後に入った 4 本のうち、`pasta_shiori` に触れたのは `recommend-menu-scenes` の別名の試験（`tests/scene_alias_ontalk_e2e_test.rs`・`tests/fixtures/scene_alias_ontalk/pasta.toml`）と `README.md` の版の表だけ。`actor/`・`windows.rs`・`lib.rs`・`shiori.rs`・`pasta_lua/src/logging/`・`build.yml` は変わっていない。
- **触るファイル**: 前回のまま。
- **規模**: 14〜18 のまま。分割は不要。
- **先に要るもの**
  - 働きの上では無い。
  - ファイルの順: 記録にある始められない理由は、今も `mlua-0-12-upgrade` とルートの `Cargo.toml`・`Cargo.lock`・`pasta_shiori/src/shiori.rs`（3 行 `use pasta_lua::mlua::{Function, Table}`）を共有することだけ。`mlua-0-12-upgrade` は今すぐ始められるようになったので、その次のウェーブで始められる。同じウェーブには置けない（`Cargo.lock` の席と `shiori.rs`）。
- **種別**: 機能（Windows 以外で読める別口）。
- **要件定義のモデル**: Opus（前回のまま）。
- **見つけた穴・古くなった記述**
  - 前回の穴の `README.md` 92 行（`wintf-winmsg-executor` 0.0.5）・112 行（`ctor` 1.0）は直った。`wintf` を外すなら 92 行を行ごと消すのは、そのまま残る。
  - 冒頭のステータス（「0.4.2 に入れる」）は、まだ古い。

## 申し送り（mlua-0-12-upgrade より）

- 2026-10-24 に mlua 0.12.2 が main に入った（`Cargo.lock` で動いたのは mlua・mlua-sys・luajit-src・lua-src の 4 つだけ）。ファイルの順で待っていた理由は満ちた。
- 0.12 で移った名前は `mlua::Debug` → `mlua::debug::Debug`、`mlua::String` → `mlua::LuaString` の 2 つで、`pasta_shiori` の `use pasta_lua::mlua::{Function, Table}` は当たらず、`pasta_shiori` のコードは変えていない。新しく mlua の型を書くときは 0.12 の名前で書く（`.kiro/specs/completed/mlua-0-12-upgrade/design.md` の Revalidation Triggers）。
- x86 の pasta.dll のセーブの整数は、x64 と同じ決まり（`persistence.rs` の `lua_int`）で書く。Linux・macOS の 32 ビットの target を足すなら、この決まりと `target_pointer_width` で期待を分けたテストがそのまま効く。

## shiori4 0.1.0 への合わせ（2026-10-24 の `/kiro-discovery`）

shiori4 の 0.1.0 が 2026-10-24 に出た（crates.io の `shiori4_abi`・`shiori4`・`shiori4_host`、GitHub のタグ `v0.1.0`）。shiori4 の PR や教科書には「0.0.3」と書いた所があるが、出た版は 0.1.0 である（shiori4 のセッションからの知らせ）。この spec は 0.1.0 を使う。知らせの正本は shiori4 の `.kiro/specs/completed/abi-textbook/research.md` の「二進互換の申し送り」、移り方は教科書の `doc/textbook/migration.md`。

### 二進互換でない変更（shiori4 #11）

- 応答の `ShioriBytes`（`request` の `out`）は、常にホストが作る。SHIORI は `ShioriHostApi` の末尾に足された `copy_from_slice` で写す。`release` は常にホストの関数で、unload の後でも、どのスレッドからでも呼べる。
- 版 3 の `load` にも、`copy_from_slice` だけを持つテーブル（`abi_version` は 3。`request`・`post`・`log` は NULL）が渡る。SHIORI はこの欄があるかだけを確かめ、無ければ `load` で NULL を返す。
- Rust の `ShioriHostApi` の `request`・`post`・`log` の欄は `Option` になった（二進の形は C と同じ）。
- 0.0.2 以前のホストでは、新しい SHIORI は読めない（`load` が NULL）。SHIORI もホストも 0.1.0 で作り直す。

### pasta への効き目

- **写しのコードは書かない**。`copy_from_slice` があるかの確かめと写しは、`shiori4_abi` の `ShioriApi::new` が作るテーブルの関数（`shiori_load`・`shiori_request`）が行う。pasta は `shiori4_abi::Shiori` を実装し（`SUPPORTS_V3 = true`。`load` には `ShioriLoadMode::V3` が来る）、応答を `ShioriResponse` で返すだけでよい。`Option` になった 3 つの欄も、版 3 では触らない。
- **書き出しのマクロ `export_shiori!` は使えない**。このマクロは、版 4 を求められると `SUPPORTS_V3` にかかわらず版 4 のテーブルを返す（`shiori4_abi` 0.1.0 の `src/lib.rs` 1003〜1018 行）。この spec は版 4 に NULL を返すので、`shiori_get_api` を自分で書き出す。`ShioriApi::new::<_, { SHIORI_ABI_VERSION_3 }>()` の static を 1 つ持ち、3 のときだけそれを返し、ほかは NULL にする（10 行ほど）。版 4 を足す `shiori4-v4-host` で、`export_shiori!` に替えてよい。
- **版 4 に NULL を返すのは要る**。版 4 の `load` が NULL を返すと、ホストは版 3 へ読み直さない（Upstream の 2026-10-23 の裁定）。版 4 のテーブルを返して `load` で断る形にすると、shiori4 を知るホストで読めなくなる。
- **依存**: `shiori4_abi = "0.1"`（依存 0・MIT・rust-version 1.85 のまま）。試験に `shiori4_host` を使うなら dev-dependency で `0.1`。
- **areka**: 0.1.0 の `shiori4_host` で作り直す（shiori4 から areka への知らせ）。pasta の試験も 0.1.0 のホストだけで行い、0.0.2 以前のホストで試さない。

### 試験

- **試験用のホスト `shiori4-testhost`**（`shiori4_host` 0.1.0 の example。教科書の `doc/textbook/testhost.md`）で、ベースウェア無しに 1 往復させられる。
  - 入れ方: `cargo install shiori4_host --version 0.1.0 --example shiori4-testhost`
  - 呼び方: `shiori4-testhost [--entry v3] [-H "名前: 値"]... <ライブラリ> <ゴーストのディレクトリ（絶対パス）> [ID]`（ID を省くと `OnBoot`）
  - 終了の状態: 0＝2xx・1＝2xx 以外（版 3 で開始行が読めないときも）・2＝ABI エラー・3＝読み込みの失敗・4＝従来の口だけが見つかった・64＝引数の誤り
  - shiori4 自身の CI が 3 OS でこれを回している（shiori4 の `.github/workflows/build.yml` の job `textbook`）。
- **この spec の CI**: Windows・Linux（できれば macOS）で、見本ゴーストを `--entry v3` で読み、終了の状態 0 を確かめる段を `build.yml` に足す。`--entry` を省いても、pasta.dll は `shiori_get_api` を持つので版 3 で解決される（版 4 は NULL）。どの ID を送るか（`OnBoot` で台本が返るか）は要件で決める。
- **Rust の試験**: dev-dependency の `shiori4_host` の `load_ghost_with(.., &[EntryKind::V3])` で、DLL の境界まで試せる。試験用のホストと Rust の試験のどちらを使うか（両方か）は、設計で決める。
- **従来の口（`loadu`・`load`）は、shiori4 0.1.0 では試せない**。試験用のホストも `shiori4_host` の `load_ghost_with` も、従来の口は見つけるだけで呼ばない（`LoadedGhost::Legacy`。pasta.dll は `shiori_get_api` を持つので、そもそもそちらで解決される）。従来の口を CI で試せるのは、shiori4 の `shiori3-legacy-host`（x86 の Windows だけ。まだ出ていない）が出てから。それまでは今の pasta の試験（`windows_tests.rs`・`shiori_lifecycle_tests.rs` など）で守る。

### 今の入口（`windows.rs`）をどうするか

2026-10-24 の discovery で決めた（shiori4 のセッションから「移すか、移行をどう段に分けるかも discovery で決める」と頼まれた。開発者は要件のディスカッションで覆してよい）。

- **shiori4 のクレートへは移さない。HGLOBAL の入口は pasta に残す。**
  - shiori4 0.1.0 には、SHIORI の側で従来の口（`loadu`・`load`・`request`・`unload`）を書き出すものが無い。`export_ghost!`・`export_shiori!` が書き出すのは `shiori_get_api` だけで、`shiori3-legacy-host` はホストの側（従来の DLL を読む口）である。
  - shiori4 の位置づけでも、版 3 は「Windows 以外のための、HGLOBAL に頼らない SHIORI/3.0 の口」で、従来の DLL を置き換えない（教科書の移行の手引き「v3 互換モードと旧来の DLL」）。SSP は従来の口でしか読まないので、この口は無くならない。
  - 従来の口には pasta の事情が多い（`loadu` で済んだら `load` は何もしない・`DllMain` の FreeLibrary での片付け・ANSI のパスのデコード・loader lock を避けて load 起点でスレッドを起こす）。
- **段の分け方**:
  1. **この spec（0.4.3）**: 2 つの口を並べ、同じ芯につなぐ。芯は「インスタンス 1 つ＝アクター 1 つ」の型（名前は設計で決める）で、`load`（UTF-8 の dir）・`request`（SHIORI/3.0 の全文）・`unload` を持つ。版 3 の口は、それを `shiori4_abi::Shiori` の実装で包み、インスタンスごとに持つ。HGLOBAL の口は、芯を 1 つだけ置く場所（プロセスに 1 つ）を `windows.rs` に閉じ込め、HGLOBAL の受け渡しとパスのデコードだけを受け持つ。今の `actor/lifecycle.rs` の static（`MAILBOX`・`ACTOR_HANDLE`）は、芯の中か、HGLOBAL の口の置き場へ移す（上の「決めたこと」）。
  2. **`shiori4-v4-host`（0.5.0）**: 同じ `Shiori` の実装に版 4 を足す（`shiori_get_api` が 4 にもテーブルを返す）。中の表現を「メソッド＋ヘッダ部分」「ステータス＋ヘッダ部分」に寄せ、開始行と `Charset` の読み書きを、従来の口と版 3 の口だけが行う（あちらの brief の 2026-10-24 の節）。そのとき、2 つの口の変換に `shiori4::v3::{parse_request, response}` を使えるかを測る。使えば、変換は shiori4 のクレートに寄り、HGLOBAL の受け渡しだけが pasta に残る。`shiori4::v3::parse_request` は SHIORI/2.x の要求を 400 にするので、従来の口で 2.x を受け続けるかも、そこで決める。
  3. **その後（予定しない）**: 従来の口を消すことは無い（SSP のため）。HGLOBAL の受け渡しまで shiori4 のクレートへ移すのは、shiori4 が SHIORI の側で従来の口を書き出す道を持ったときだけ考える。pasta から shiori4 へは頼まない（あちらの範囲の判断）。

### 先に要るもの・規模

- **先に要るもの**: 働きの上でも、ファイルの順でも無い（`mlua-0-12-upgrade` は 2026-10-24 に入った。shiori4 0.1.0 で試験の口も揃った）。roadmap の「開発者の判断が要るもの」にあった「shiori4 の側の準備など、ほかに待つもの」は無い。
- **規模**: 14〜18 のまま。増えるのは、自前の `shiori_get_api`（小さい）と、CI の試験用のホストの段（1 つ）。
