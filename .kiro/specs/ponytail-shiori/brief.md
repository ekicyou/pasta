# Brief: ponytail-shiori

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

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
