# Brief: ponytail-shiori

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.4（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域（`crates/pasta_shiori`）に `/ponytail:ponytail-audit` をやり直す。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

ゴーストの利用者と作者が困る。

- ゴーストを閉じたり切り替えたりしたときに、SSP が落ちうる。
- ゴーストのフォルダが見つからないと、理由の無い 500 になる。
- コロンの後に空白の無い見出しの行（`Reference0:`）を含む事象が、黙って 204 になり、Lua に届かない。

加えて、テストだけの道具とテストデータ（約 3,900 行）が、出荷物と crates.io のパッケージに入っている。

## Current State

2026-10-19 の監査で確かめた。`cargo test -p pasta_shiori` は通り、clippy も 0 件。

1. **unload と FreeLibrary が競る**（Must fix）
   - 場所：`src/actor/thread.rs` L250-258、`src/actor/lifecycle.rs` L192-215、`src/windows.rs` L222-240。
   - `done.send(())` は、アクターのスレッドが DLL のコードを出る前に出る。その後もスレッドは、`block_on` から戻り、`_log_context` を drop し、静的に結んだ std のスレッドの後始末を走らせる。
   - `teardown_actor` は detach だけ（join しない）なので、`unload()` はすぐ戻る。SSP が FreeLibrary したときに後始末の途中なら、access violation になる。窓は狭いが、実在する。
   - もう 1 つの場合：unload をせずに FreeLibrary すると、DllMain が loader lock を持ったまま 5 秒待つ。その間、アクターの `drop(shiori)` は Lua の C モジュールを FreeLibrary しようとし（loader lock が要る）、5 秒止まる。その後、生きたスレッドの下で DLL が外れる。
2. **フォルダが無いと、理由の無い 500 になる**
   - 場所：`src/shiori.rs` L66-70。
   - `load` が、ロガーを作る前に、`last_load_error` を立てずに `Ok(false)` を返す。
   - 例：古い load（ANSI）の入口に、ANSI に無い文字を含むパスが渡ると、`?` になって存在しないパスになる。
3. **`Key:`（コロンの後に空白なし）の行で、黙って 204 になる**
   - 場所：`src/util/parsers/req_parser.pest` の `_tag = _{ ": " }`。
   - リポジトリにある SSP の記録 `crates/pasta_shiori/doc/shiori-sample.log` に、裸の `Reference0:` がある（L47 の OnInitialize、L720 の OnNotifyUserInfo、L1236 の GET OnTranslate）。
   - その要求は丸ごと解けず、`determine_method` が None を返し、ログも出ない。
4. **x64 で FFI の型が C の型と違う**
   - 場所：`src/windows.rs` L109・L129・L222・L257。
   - 仕様は `BOOL load(HGLOBAL,long)`・`HGLOBAL request(HGLOBAL, long*)`。Windows の long は 32 ビットなのに、コードは `usize`・`&mut usize` を使う。`bool` と `BOOL`（i32）も同じ類の違い。
   - x86_64 版は CI がビルドするが、配布はしない。配布する i686 版には影響しない。
5. **MAILBOX の単体テストどうしが競る**
   - 場所：`src/actor/lifecycle.rs` L246-329、`src/windows_tests.rs` L29。
   - 鍵が 2 種類あり、2 本のテストは鍵を取らない。
6. **コードページの層は、ほぼテストだけの足場**
   - 場所：`src/util/hglobal/enc.rs` の全部、`windows_api.rs` L4-72・L146-225。
   - 本番に要るのは、`multi_byte_to_wide_char(CP_ACP, MB_ERR_INVALID_CHARS, bytes)` だけ。
   - `set_len` で未初期化の Vec を作っている所は、`vec![0u16; len]` にすれば unsafe が消える。
   - roadmap の「開発者の判断が要るもの」にある `string_to_multibyte` の件は、この spec で閉じる。
7. **テストの支えが DLL に入っている**
   - 場所：`src/actor/teardown.rs` L75-98・L178-345（`ReloadProbe`・`LeakMetric`・`sample_resources` など）。
8. **ほぼ同じ写し**
   - `marshal_request`（`lifecycle.rs` L174-182 と `marshaling.rs`）
   - 204 を作る関数が 2 つ（`shiori.rs` L382-388 と `marshaling.rs` の `default_204`）
9. **出力の引数の二重の受け渡し**
   - 場所：`windows.rs` L290-320。
10. **素通しと未使用**
    - `actor_detach_on_poison`
    - `Reply`（variant が 1 つだけの enum）
    - `_GRAMMAR` の `include_str!`
    - 文法の `sp`・`eol`・`key_sep`
    - `ShioriString` の `unsafe impl Send/Sync`
    - `MyError::Poison`
11. **実装が 1 つの `Shiori` トレイト**
    - 場所：`shiori.rs` L8-11。
12. **テストだけのパーサの写しと、未使用のテストデータ**
    - `src/util/parsers/req.rs` のテストの中のパーサ（約 110 行）
    - `src/util/parsers/test_data/logfile.txt`（3322 行・58 KB）
    - これらと `doc/` が、crates.io のパッケージに入っている（`exclude` が無い）。

## Desired Outcome

- unload が返るときには、アクターのスレッドは DLL のコードを出ている。待つのは上限つきで、「join しない・固まらない」の決定（R7.4）は保つ。
- フォルダが無いときは、その名前を 500 の理由に出す。
- `Key:` の行を受け付け、その値は `""` になる（2026-10-19 開発者の決定）。解けない要求は、警告のログを残す。
- FFI の型が、C の型と一致する。
- テストの競りが無くなる。
- 出荷物と crates.io のパッケージから、テストだけのものが無くなる。

## Approach

監査の最小の直しを採る。

- unload の道だけ（DllMain の道は除く）、ack の後に JoinHandle の生のハンドルへ `WaitForSingleObject(TEARDOWN_TIMEOUT)` をかける。
- 文法を `_tag = _{ ":" ~ " "? }` にする。
- 型は `c_long`・`*mut c_long`・i32 の BOOL にする。
- テストの鍵は、1 つの `#[cfg(test)]` の static にまとめる。
- 204 を作る関数は、`marshaling::default_204()` に寄せる。SHIORI.request が未定義のときの 204 にも `SecurityLevel: local` が付くが、これは受け入れる。
- `Cargo.toml` に `exclude = ["doc/"]` を足す。

## Scope

- **In**: 上の 1〜12 と、ギャップ分析で増えた同じ領域の指摘。マニュアル `book/src/internals/shiori.md` の該当の節（L155・unload/teardown・marshal_request・L436-437）、`crates/pasta_shiori/README.md`。
- **Out**:
  - `panic = "abort"` の方針（確定済み）
  - 静的な CRT リンク（確定済み）

## Boundary Candidates

- 片付けの順序（actor の lifecycle・thread・teardown）
- 要求の解析（pest の文法・marshal）
- FFI の型（windows.rs）
- 文字コード（hglobal）
- テストの支えの移動

## Out of Boundary

- `pasta_lua` の encoding（`ponytail-lua-loader`）
- SSP での実機の再現（任意。できれば `ssp-ghost-test-launch` の手順で確かめる）

## Upstream / Downstream

- **Upstream**: `pasta_lua` の実行時
- **Downstream**: SSP（ベースウェア）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`・`ponytail-lua-scripts`（`book/src/internals/shiori.md` の別の節を触る。後から入る側が rebase する）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。`MyError::Poison` を消してよい。README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が 1 回だけ書く。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_shiori`。i686 のビルドも通す。
- cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**: 2026-10-23 に `shiori4-v3-portable`（0.4.2）が起票された。`windows.rs`・`actor/` を共有し、0.4.2 のほうが先に入る。あちらの推奨どおりに進むと、この spec の土台が次のように変わる。
  - 4 番（FFI の型）: `load`・`loadu`・`unload` の戻り値の `bool`→i32 の `BOOL` は、あちらの In に入る見込み（2026-10-23 に shiori4 の `shiori3-legacy-host` が「BOOL は 32bit の int で 0 以外が真」と確定）。残るのは `usize`→`c_long` だけ。`pasta-dll-multiarch` が x64 版を出す前にこれが要る。
  - 1 番（unload と FreeLibrary の競り）: あちらは unload の待ちの上限を見直し、実行器の wintf を外す見込み。メッセージ専用ウィンドウの後始末は無くなるが、detach する限り競りは残る。直しはこの spec のまま。
  - 5 番（MAILBOX のテストの競り）: あちらが `MAILBOX`・`ACTOR_HANDLE` をインスタンスへ移せば、消えるか形が変わる。
  - 7・8 番（`teardown.rs` の測り・`marshal_request` の写し）: `lifecycle.rs` が書き直されるので、場所を測り直す。
- **触るファイル**: 前のとおり `crates/pasta_shiori/**`（`Cargo.toml` の `exclude` を含む）、`book/src/internals/shiori.md`、`crates/pasta_shiori/README.md`。`crates/pasta_shiori/src` は 2026-10-19 から変わっていない（最後は #166）。1,000 行に近いものは無い（いちばん大きいのは `shiori_lifecycle_tests.rs` の 596 行）。
- **規模**: 14〜18。あちらが 4 番の一部と 5 番を持っていけば、12〜15。
- **先に要るもの**: 機能の上では無い。`shiori4-v3-portable` の後に入る（`windows.rs`・`actor/` を共有。同じウェーブに置かない）。ほかの重なり: `shiori4-v4-host`（`actor/`）、`pasta-dll-multiarch`（4 番の `c_long` をどちらが直すか）。`ponytail-lua-loader`・`ponytail-lua-scripts` とは `internals/shiori.md` の別の節（後から入る側が合わせる）。
- **種別**: バグ（ゴーストを閉じると SSP が落ちうる・`Key:` の行が黙って 204）。監査の指摘より。
- **要件定義のモデル**: Opus（前のまま）。
- **分割の案**: 要らない。
- **見つけた穴・古くなった記述**: Current State の行番号のうち、`lifecycle.rs` の 2 つは監査の日にもずれていた。`teardown_actor` は 150〜182 行（「L192-215」は `kick_into_mailbox`）、`marshal_request` は 124〜148 行（「L174-182」ではない）。`thread.rs` 250〜258 行・`shiori.rs` 66〜70 行・`windows.rs` 109・129・222・257 行・`req_parser.pest` の `_tag`（44 行）・`shiori-sample.log` 47 行の裸の `Reference0:` は合っている。ギャップ分析は `shiori4-v3-portable` が入った後の main で測り直す。Existing Spec Touchpoints の Adjacent に `shiori4-v3-portable`・`shiori4-v4-host`・`pasta-dll-multiarch` が無い。

## 2026-10-24 棚卸の再測定（main c20ad104）

- **前提の変化**
  - `crates/pasta_shiori` に入ったのは PR #187 だけ。ctor を 0.2 → 1.0 にし（`Cargo.toml` 50 行の dev 依存）、テスト 14 ファイル（`src/shiori_request_tests.rs` と `tests/` の 13）の 1 行を `#[ctor::ctor(unsafe)]` に直した。本体のソースは変わっていない。新しいテストに `PASTA_DEBUG` を中和する守りを足すときは、この新しい書き方にする。
  - アクタースレッドのメッセージループ wintf-winmsg-executor は 0.0.5 になった（ルートの `Cargo.toml` 48 行。pasta は `block_on` しか使わず、振る舞いは同じ）。`shiori4-v3-portable` が外す見込みは前のまま。
  - 2026-10-24 に、この spec は 0.4.4（`shiori4-v3-portable` の後）になった。10-23 の再測定の行番号（`windows.rs` 109・129・222・257 行、`shiori.rs` 66〜70 行、`thread.rs` 250〜258 行、`lifecycle.rs` の `marshal_request` 124 行・`teardown_actor` 150 行、`req_parser.pest` 44 行の `_tag`）は今も合っている。
- **触るファイル**: 前のとおり `crates/pasta_shiori/**`（`Cargo.toml` の `exclude` を含む）、`book/src/internals/shiori.md`、`crates/pasta_shiori/README.md`。1,000 行に近いものは無い（いちばん大きいのは `tests/call_execution_correctness_e2e_test.rs` の 687 行。10-23 の「`shiori_lifecycle_tests.rs` の 596 行」は `src` の中だけの話）。
- **規模**: 14〜18（`shiori4-v3-portable` が 4 番の `BOOL` と 5 番を持っていけば 12〜15）のまま。
- **先に要るもの**
  - 働きの上では無い。`shiori4-v3-portable` が入った後の main でギャップ分析をやり直す（前のまま）。
  - 0.4.3 の 6 本との重なり: `shiori4-v3-portable`（`windows.rs`・`actor/{lifecycle,thread,teardown,mod}.rs`・`pasta_shiori` の `Cargo.toml`・`README.md`・`tests/actor_*`。`internals/shiori.md` は節が別）、`mlua-0-12-upgrade`（`mlua::` を引く `src/error.rs`・`src/shiori.rs`・`src/lua_request.rs` とテスト 4 つ。`shiori.rs` は 2 番と 8 番、`error.rs` は 10 番の `MyError::Poison` の場所）。どちらも 0.4.3 に先に入り、こちらが合わせる。`toml-serialization`・`mlua-stdlib-removal`・`recommend-menu-scenes`・`ponytail-check-release` とは重ならない。
  - ほか: `shiori4-v4-host`（`actor/`）、`pasta-dll-multiarch`（4 番の `c_long` をどちらが直すか）。
- **種別**: バグ（ゴーストを閉じると SSP が落ちうる・`Key:` の行が黙って 204）。監査の指摘より。
- **要件定義のモデル**: Opus（前のまま）。
- **分割の案**: 要らない。
- **見つけた穴・古くなった記述**: Existing Spec Touchpoints の Adjacent に `shiori4-v3-portable`・`shiori4-v4-host`・`pasta-dll-multiarch`・`mlua-0-12-upgrade` が無い（前の 3 つは 10-23 の再測定でも指摘済み）。Current State の `lifecycle.rs` の 2 つの行番号の誤りは 10-23 の再測定のとおり。

## 申し送り（shiori4-v3-portable の要件より・2026-10-25）

`shiori4-v3-portable` の要件のディスカッションで、開発者が次を決めた（あちらの `requirements.md` の末尾の「要件のディスカッションで決めたこと」）。あちらが先に入るので、この spec のギャップ分析は、あちらが入った後の main で測り直すこと。

- **1 番（unload と FreeLibrary の競り）は、あちらで閉じる**。あちらの R4.2・4.4・7.6: 後始末の完了が上限（5 秒）の中で届けば、アクタースレッドの終わりを待ってから `unload` を返す。上限を過ぎたら今どおり手放してログに出す。版 3 と旧来の入口の両方に効く（芯の型の片付けに 1 度書く）。
  - **`DllMain` の中の片付け（unload をせずに FreeLibrary されたとき）は、あちらの Out**（loader lock の中ではスレッドの終わりを待てない）。この spec の 1 番の「もう 1 つの場合」（DllMain が loader lock を持ったまま 5 秒待ち、アクターの `drop(shiori)` が Lua の C モジュールの FreeLibrary で止まる）は、こちらに残る。
  - こちらの Approach の「ack の後に `WaitForSingleObject(TEARDOWN_TIMEOUT)`」は、あちらの作りに置き換わる（2026-10-25 確定）: `ActorInstance::shutdown(JoinPolicy)` が ack を 5 秒まで待ち、届けば `JoinHandle::join`、届かなければ手放して警告。`unload` は `JoinAfterAck`、`DllMain` の FreeLibrary の detach は `lifecycle::teardown_actor_detached()`（`JoinPolicy::Detach`）。`TeardownReport` に `joined` が足される。
  - あちらはディレクトリの占有（`REGISTRY` の世代の札 `ClaimToken`）も足す。旧来の口の再読み込みは、前の片付けが打ち切りに終わっても占有を引き継いで読める（今の振る舞いを保つ）。
- **4 番（FFI の型）は、あちらで全部閉じる**。あちらの R7.3: `load`・`loadu`・`unload` の戻り値は i32 の 1／0、`load`・`loadu` の長さと `request` の長さ（受けと返し）は C の `long`。10-23 の再測定の「残るのは `usize`→`c_long` だけ」も無くなる。
- **5 番（MAILBOX の単体テストの競り）は、こちらに残る**（2026-10-25 にあちらの設計で確定）。あちらは `MAILBOX`・`ACTOR_HANDLE`・`REQUEST_SEQ` を消し、芯の型 `ActorInstance` へ移すが、旧来の入口の置き場 `static LEGACY: ArcSwapOption<ActorInstance>` と窓口の自由関数（`spawn_actor`・`marshal_request`・`teardown_actor`・`kick_sink`、新しく `teardown_actor_detached`）を `actor/lifecycle.rs` に全部の OS で残す（案 C）。`lifecycle` を呼ぶ試験は `LEGACY` を共有し続けるので、鍵の 2 種類と鍵を取らない 2 本の競りは形を変えて残る。あちらの新しい試験（`ActorInstance` を直に持つもの）は置き場を使わない。
- **7・8 番（`teardown.rs` の測り・`marshal_request` の写し）**: `lifecycle.rs`・`thread.rs` が書き直され、実行器の `wintf-winmsg-executor` も外れる（あちらの R8）。場所と形を測り直すこと。
- **2 番（フォルダが無いと理由の無い 500）**: あちらの版 3 の `load` は、ディレクトリが無いときに NULL を返す（R2.4。ログを置けないので理由は出さなくてよい）。旧来の入口の 500 の理由は、こちらに残る。
- **3 番・6 番・9〜12 番**: あちらは触らない。
- **規模**: 1 番の主な部分と 4 番が抜けるので、10-23 の見積もり（12〜15）より小さくなる見込み。
