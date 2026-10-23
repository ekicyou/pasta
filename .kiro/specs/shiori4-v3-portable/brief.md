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
  - 試験: `shiori4_host` の v3 の口と許可する口の集合（どちらも shiori4 0.0.2 で入る予定）を dev-dependency で使えば、DLL の境界まで CI で試せる。0.0.2 が出るのは、shiori4 の abi-conventions・host-protocol を作って統合した後で、時期の目安は無い（2026-10-23 時点で、2 本はこれから要件の段）。この spec は自前の呼び出しで試す前提で進め、0.0.2 が出てから切り替える（0.0.2 を待たない。出たら shiori4 のセッションが知らせる約束で、shiori4 の roadmap にも記録済み）
- **Downstream**: `shiori4-v4-host`（この spec の入口と実行器の上に、版 4 を足す）。areka（Windows 以外で pasta を読む）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `ponytail-shiori`（隙間の spec。SHIORI/3.0 の書き出しの型（`bool`→i32 の `BOOL`、`usize`→`c_long`）を直す。この spec も `windows.rs` を触るので、先に入るほうが直してよい。2026-10-23 に shiori4 の `shiori3-legacy-host` が「BOOL は 32bit の int で 0 以外を真と読む（下位 1 バイトで読む守りは採らない）」と確定した。今の `-> bool` は EAX の上位が不定なので、失敗を真と読まれうる。推奨は、`load`・`loadu`・`unload` の戻り値を i32 の 1／0 に直すことを、この spec の In に入れること（要件で決める）。unload がアクターのスレッドが DLL を出るまで待つ・FFI の型を C に合わせる。`windows.rs`・`actor/` を共有するので、同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（`.cargo/config.toml`。動的に戻さない）
- Windows の出荷物は 1 種類のまま（`pasta.dll` に `shiori_get_api` が増えるだけ）。debug の待受は 127.0.0.1 固定のまま
- shiori4 の ABI 規約に従う（UTF-8・NUL 終端なし・`out` は確保した側の `release` で解放・パニックは `SHIORI_E_PANIC`）
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
