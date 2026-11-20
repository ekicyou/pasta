# Brief: pasta-dll-multiarch

> **ステータス**: 未着手（2026-10-23 の `/kiro-discovery` で起票）。版は未定（2026-10-23 開発者「版は後で決める」）。要件定義は Opus。

## Problem

pasta が配っているのは、32bit（i686）の `pasta.dll` だけである。

- areka は 64bit のベースウェアで、x64 と arm64 の Windows で動く。32bit の SHIORI は、32bit の helper プロセスを起こして読むしかない。
- arm64 の Windows では、32bit の DLL も x64 の DLL も、エミュレーションで動かすことになる。

## Current State

2026-10-23（main `c360adb2`）に確かめた。

- **ビルド**: 見本ゴーストの `crates/pasta_sample_ghost/release.ps1` が、`cargo build --release --target i686-pc-windows-msvc -p pasta_shiori` を実行する（L71）。できた DLL を `ghosts/hello-pasta/ghost/master/pasta.dll` へ写す（L97-107）。ターゲットも名前も決め打ちで、引数で変えられない。
- **静的 CRT**: ルートの `.cargo/config.toml` は、i686 と x86_64 の両方に `crt-static` を指定している。aarch64 の指定は無い。
- **配布物**: GitHub Release の `pasta.dll.zip` は、i686 の `pasta.dll` と `THIRD_PARTY_LICENSES.txt` の 2 つだけである。リリース CI が、中身がちょうどこの 2 つであることを確かめている（`release.yml` L141-144）。.nar に `ghost/master/pasta.dll` があることも確かめている（L163）。
- **aarch64**: `.github` にも、各クレートの `Cargo.toml` にも、aarch64・arm64 の言及は無い。
- **pasta_check**: `release` は DLL の名前も arch も気にせず、ゴーストのフォルダをそのまま .nar にする。`validate` は DLL を見ない。`pasta.x64.dll` などを ghost/master に置けば、そのまま .nar と updates.txt に入る。
- **自分の名前**: pasta.dll と pasta_lua は、自分のファイル名を `pasta.dll` と決めつけていない。名前を変えて置いても動く。

## Desired Outcome

- x64 版と arm64 版の pasta.dll が、32bit 版と同じリリースで出る。
- 見本ゴースト（hello-pasta）の .nar に、areka の探し方に合う名前で同梱される。
  - x64 の Windows では `pasta.x64.dll` → `pasta.dll` の順に探す。
  - arm64 の Windows では `pasta.arm64.dll` →（x64 のプロセスを起こせるホストだけ）`pasta.x64.dll` → `pasta.dll` の順に探す。
  - 接尾辞の無い `pasta.dll` は、SSP 向けの 32bit 版のまま残す。
- 作者は、自分のゴーストに同じ名前で同梱できる。手順がマニュアルにある。

## Approach

shiori4 の名前の決まり（2026-10-23 の shiori4 の開発者の裁定。abi-conventions の論点 12。areka の探し方が元）に名前を合わせる。Linux は `libpasta.<x64|arm64>.so` → `libpasta.so`、macOS は `libpasta.dylib` だけ（ユニバーサルバイナリを推奨）と決まった。この spec は Windows の arch を持つ。`pasta_check` は今のまま使える。手を入れるのは、ビルド・配布物・見本ゴースト・リリース CI・マニュアルである。

## Scope

- **In**:
  - x86_64・aarch64 の `pasta_shiori` のビルド。aarch64 にも `crt-static` を指定する（`.cargo/config.toml`）
  - `release.ps1` で 3 つの arch を作り、`pasta.dll`・`pasta.x64.dll`・`pasta.arm64.dll` の名前で見本ゴーストへ写す（手元の `release.bat` とリリース CI が同じ一式を作る）
  - GitHub Release の配布物の形（`pasta.dll.zip` に 3 つを入れるか、arch ごとに分けるか）と、リリース CI の中身の検査の更新
  - マニュアル: arch ごとの DLL の同梱のしかたと、`[loader] ffi = true` で作者が自分の DLL を読むときの注意（pasta の arch と作者の DLL の arch が合わないと読めない）
- **Out**:
  - Windows 以外の共有ライブラリ（`.so`・`.dylib`）の配布（`shiori4-v3-portable` は CI のビルドとテストまで。配るかは別に決める）
  - areka の側の探し方と helper の作り（areka が持つ。arm64 の areka は x64 と i686 の helper を同梱し、DLL の PE ヘッダで helper の arch を決める）
  - `pasta_check` の Windows 以外・arm64 への対応

## Boundary Candidates

- ビルドと静的 CRT の設定（arch の追加）
- 配布物（Release の添付・見本ゴーストの .nar・リリース CI の検査）
- 作者向けの文書

## Out of Boundary

- shiori4 の入口そのもの（`shiori4-v3-portable`）
- 名前の決まりそのもの（areka と shiori4 が持つ）

## Upstream / Downstream

- **Upstream**:
  - `shiori4-v3-portable`。areka が x64・arm64 の pasta を読むのは shiori4 の入口（`shiori_get_api`）経由なので、それが無い x64 版を出しても areka では使えない
  - shiori4 の arch ごとの名前の決まり（abi-conventions の論点 12。2026-10-23 に確定）
  - `ponytail-shiori`（隙間の spec）の「FFI の型を C に合わせる」。今の SHIORI/3.0 の書き出しは、`long` の所に `usize`・`&mut usize`、`BOOL` の所に Rust の `bool` を使っている（`crates/pasta_shiori/src/windows.rs`）。i686 では `usize` と `long` がどちらも 32bit なので表に出ないが、x64 では `usize` が 64bit になる。x64 版の `request` は、ホストの 32bit の `long` に 8 バイトを書いてしまう。x64 版を出す前に、型を `c_long`・`*mut c_long`・i32 の `BOOL` に直す必要がある（`ponytail-shiori` を先に入れるか、この spec で直す）。`bool` の戻り値も、2026-10-23 に shiori4 の `shiori3-legacy-host` の論点として挙がった（ホストは下位 1 バイトで読むのが推奨だが、正典は 32bit の BOOL）
- **Downstream**: areka での x64・arm64 の実機確認

## Existing Spec Touchpoints

- **Extends**: なし（`pasta-check-release-asset`（完了）が作った `release.ps1` の段と `pasta_check.zip` は保つ）
- **Adjacent**: `ponytail-manual-ci`・`ponytail-check-release`（隙間の spec。`release.yml`・`github-release.ps1`・`release.ps1` まわりを共有しうる。同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（新しい arch にも同じ指定をする。動的に戻さない）
- 32bit の `pasta.dll` は SSP 向けに今のまま出す（名前も中身も変えない）
- debug の待受は 127.0.0.1 固定のまま。出荷物は arch ごとに 1 種類（debug は opt-in）
- **確かめていない危険**: pasta は mlua 経由で LuaJIT を静的に組み込んでいる。`aarch64-pc-windows-msvc` で LuaJIT がビルドできるか（クロスコンパイルを含む）は、2026-10-23 時点で確かめていない。要件の前に試す。できないなら、arm64 版を落として x64 版だけにするかを開発者が決める（arm64 の areka は x64 版へ戻って読める）

## 申し送り（pasta-skills-release-asset より）

- `release.ps1` は 8 段になった（段 6 が `pasta-skills.zip` の作成と検査、版の確認が 7、大きさとまとめが 8）。arch を足すときは段 6 を保ち、段の番号と `/8` の見出し・`release.bat` の説明・文書の段の並びを合わせる。
- 配布物は 5 つ・GitHub Release の添付は 6 つになり、`release.yml` の「Verify release assets」に `pasta-skills.zip` の項目が 1 つ増えた。配布物を足すときは `release.ps1`・`release.yml`・`github-release.ps1`・`.claude/skills/release/scripts/release-state.ps1`・文書の 5 か所を合わせる。参照先: `.kiro/specs/completed/pasta-skills-release-asset/design.md` の「Revalidation Triggers」。

## shiori4 0.1.0 への合わせ（2026-10-24 の `/kiro-discovery`）

- shiori4 の 0.1.0 が 2026-10-24 に出た。名前の決まり（abi-conventions の論点 12）は、教科書の配布の決まり `doc/textbook/distribution.md` に書かれ、`shiori4_host` の `shiori_file_candidates` が同じ順を返す（`pasta.dll` に対し、x64 は `pasta.x64.dll` → `pasta.dll`、arm64 は `pasta.arm64.dll` →（x64 を起こせるホストだけ）`pasta.x64.dll` → `pasta.dll`）。上の Approach の名前は、そのまま合っている。
- 0.1.0 は 0.0.2 以前と二進互換でない（応答の `out` をホストが作る）。areka の版 4・版 3 の読み込みは 0.1.0 で作り直すので、この spec の x64・arm64 の pasta.dll も、0.1.0 の `shiori4_abi` で作った `shiori4-v3-portable` の入口が前提になる（先に要るものは変わらない）。
- 試験用のホスト `shiori4-testhost`（`shiori4_host` 0.1.0 の example）で、x64 の pasta.dll を x64 のランナーで 1 往復させられる（`shiori4-v3-portable` が `build.yml` に足す段を、arch ごとに回せる）。arm64 は、ランナーがあるときだけ。

## 申し送り（shiori4-v3-portable の要件より・2026-10-25）

- SHIORI/3.0 の書き出しの型は、`shiori4-v3-portable` で全部直ることになった（あちらの R7.3。開発者の決定）。`load`・`loadu`・`unload` の戻り値は i32 の 1／0、`load`・`loadu` の長さと `request` の長さは C の `long`。この spec の「x64 版を出す前に型を直す」は、あちらが入れば満ちる。この spec が `windows.rs` を持つ必要は無い。
- あちらは Windows 以外の配布物を出さない（CI で Linux・macOS のビルドとテストを守るだけ。Windows の版 3 の 1 往復は x64 だけ）。Windows の x64 版の `pasta.x64.dll` を出すなら、shiori4 のファイル名の決まり（`pasta.x64.dll` → `pasta.dll`）を見ること。

## 申し送り（2026-10-29 の版の組み替えより）

- ノベルゲームを 0.5.0 から 0.6.0 へ順延し、0.5.0 は「属性関係の整備」になった（開発者の決定）。この spec の推奨の版は、areka・shiori4 の版 4 の対応と揃えて 0.6.0 に変わる（版は判断待ちのまま）。

## 申し送り（ponytail-manual-ci より・2026-11-03）

- `release.yml` の第三者の action（`dtolnay/rust-toolchain`・`Swatinem/rust-cache`・`rust-lang/crates-io-auth-action`・`Azure/login`）は SHA で固定し、行末にタグのコメントを置く形になった。`rust-toolchain` は SHA で指すので、`targets:` に arm64 を足すときも `toolchain:` は足さない（その SHA の `action.yml` の既定が stable）。
- 道具（`wasm-pack`・`cargo-about`）は env の SHA-256 と `Get-FileHash` で照合してから展開する。道具を足す・版を上げるときは、その SHA-256 も env に足す。
- `build.yml` に job `manual-light`（マニュアルの軽い検査）が増えた。build の job の `upload-artifact` は v7、`release.yml` の `download-artifact` は v8。arm64 の行を足すときは、この版にそろえる。

## 申し送り（2026-11-14 の discovery「pasta-recipe-multiarch」より）

- `pasta-recipe-multiarch` も、ルートの `.cargo/config.toml` に `[target.aarch64-pc-windows-msvc]` の `crt-static` を要る。先に入った方が足し、後の方は在るのを確かめる（二重に足さない）。`release.yml` も共有するので、同じウェーブに置かない。

## 申し送り（pasta-recipe-multiarch の要件より・2026-11-14）

- **版**: 0.5.2 の次の版は 0.6.0 になった（開発者の決定。pasta-recipe の版。0.5.3 は出さない）。ノベルゲームは 0.7.0 へ送った。この spec の推奨の版は、ノベルゲーム・areka の版 4 の対応と揃えて 0.7.0 と読む（上の 2026-10-29 の申し送りの「0.6.0」も同じ。版は判断待ちのまま）。
- **静的 CRT**: `pasta-recipe-multiarch` は、arm64 の pasta-recipe の exe も静的 CRT で作る（要件 1.6）。ルートの `.cargo/config.toml` の `[target.aarch64-pc-windows-msvc]` の `crt-static` は、先に入った方が足す（前の申し送りのとおり）。あちらは exe が VC++ ランタイムの DLL を取り込まないことを CI で検査する（要件 2.11）。pasta.dll にも同じ検査を当てるなら、検査のスクリプトを共有できる。
- **添付の数**: GitHub Release の添付は 8 つになる（配布物 7 つ＋詳しいリリースノート）。`pasta-recipe.zip` は無くなり、`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` になる（0.6.0 から）。`release-state.ps1` は `n/8` で数える。この spec が配布物を足す・形を変えるときは、そこから数える。上の 2026-11-12 の「6 → 7」と `github-release.ps1` 72 行・`release-state.ps1` 84 行・`release-scripts-test.ps1` 項目 10 の並びは古くなる。
- **`release.yml` と `build.yml`**: あちらは `release.yml` の build の job の rust の `targets:` に `aarch64-pc-windows-msvc` を足し、2 つの zip の検査（中身・版・PE ヘッダの機械の種類・ライセンスの表示・大きさ・VC++ ランタイム）を足す。`build.yml` の job `recipe` は、package.ps1 で 2 つの arch を作って同じ検査を通す形になり、arm64 のランナー（`windows-11-arm`）の job も増える（Rust のテストと起動の煙試験）。この spec が `targets:` や arm64 のランナーを足すときは、あちらの形を読んでから重ねる。2026-11-12 に書いた `release.yml` の行番号（75・77・82・145〜148・182 行）は、あちらが入ると古くなる。
- **道具の取得**（2026-11-14 の設計で確定）: `release.yml` の build の job の wasm-pack・cargo-about の取得の loop と、`env` の `WASM_PACK_VERSION`・`WASM_PACK_SHA256`・`CARGO_ABOUT_VERSION`・`CARGO_ABOUT_SHA256`（2026-11-12 に書いた env の 25・26 行）は、新しい `.github/scripts/release/install-tools.ps1`（`-Name wasm-pack,cargo-about`）へ移る。`build.yml` の job `recipe` も同じスクリプトで cargo-about を取る。この spec で道具を足す・版を上げるときは、このスクリプトの固定値を直す。

## 申し送り（pasta-recipe-multiarch の実装より・2026-11-14）

- **静的 CRT**: ルートの `.cargo/config.toml` の `[target.aarch64-pc-windows-msvc]` の `crt-static` は、pasta-recipe-multiarch が入れた（8e588187）。この spec は在るのを確かめるだけで、足し直さない。
- **`release.yml`**: build の job の rust の `targets:` に `aarch64-pc-windows-msvc` が入り、rust-cache の key は `i686-x86_64-aarch64-pc-windows-msvc` になった。道具の取得は `install-tools.ps1 -Name wasm-pack,cargo-about` の 1 行。
- **VC++ ランタイムの検査**: `.github/scripts/release/verify-recipe-zips.ps1` は、`dumpbin /dependents` で `vcruntime*`・`msvcp*`・`vccorlib*`・`concrt*`・`msvcr<数字>*` を数える（UCRT は数えない）。`-DumpbinCommand` で偽の道具を渡せる。pasta.dll に同じ検査を当てるなら、この判定を参考にできる。

## 申し送り（manual-download-page より・2026-11-14）

- **ダウンロードのページ**: マニュアルに `book/src/download.md` ができた。「やりたいこと → 取るファイル」の表（6 行）と配布物ごとの節（サンプルゴースト・SHIORI の本体・VS Code を使わずに書くアプリ・VS Code の拡張・検査の CLI・AI 向けのスキル・過去の版）で、表の右の列は最新版の固定リンク（`https://github.com/ekicyou/pasta/releases/latest/download/<名前>`）である。`pasta.dll.zip` を arch ごとに分ける・名前を変えるなら、表の行と「SHIORI の本体」の節を同じ変更で直す。ほかの章・README の入手の案内は、このページの節へのリンクに寄せてある（`getting-started/prerequisites.md`・`getting-started/setup.md` が `#shiori-の本体`・`#サンプルゴースト` を指す）。見出しを変えるとアンカーが変わるので、指している章も同じコミットで直す。
- **固定リンクの名前の検査**: `book/tools/link-check.mjs` の `checkReleaseAssetLinks` が、マニュアルの本文と読者に届く文書（クレートの README・ルートの README・`doc/SOUL.md`・配る 3 スキル）の固定リンクの名前を、`release.yml` の 1 行 1 つの `release/<名前>` の行（`${{` を含む行は除く）と照らす。添付の名前を変えて固定リンクを直し忘れると、マニュアルの CI が `release-asset` で止まる。`manual.yml` の起動条件に `release.yml` が入ったので、`release.yml` だけの変更でも走る。添付の行の形（1 行 1 つの `release/<名前>`）を崩すときは、この読み取りも合わせる（`shortcut:` のコメントがある）。参照先: `.kiro/specs/completed/manual-download-page/design.md` の ReleaseAssetLinkCheck。

## 申し送り（2026-11-19 manual-reader-first から）

入門用のスターター `pasta-starter.nar` を配布物に足した。GitHub Release の添付は 9 つ（配布物 8 つ＋詳しいリリースノート）になった。この brief の「添付の数」と、添付の数を書いた所は、ここから数え直す（`release.yml`・`github-release.ps1`・`release` スキルの `release-state.ps1` と `procedure.md` は 9 つに直してある）。スターターは hello-pasta の nar と同じ `pasta.dll` を同梱するので、arch 別の DLL を足すときは `crates/pasta_sample_ghost/release.ps1` のスターターの組み立てと `release.yml` のスターターの検査（中身の完全一致）も合わせる。

## 2026-11-20 棚卸の再測定（main 58aba27c）

- **前提の変化**: `pasta-recipe-multiarch` が入った。ルートの `.cargo/config.toml` に `[target.aarch64-pc-windows-msvc]` の `crt-static` が在る（20〜21 行。11 行の説明も 3 つの arch になった）。`release.yml`（683 行）の build の job は、73 行の `targets:` に 3 つ、78 行の cache の key も 3 つ。PE ヘッダの arch の検査は `.github/scripts/release/verify-recipe-zips.ps1`（108 行。22 行の `$Machines`・`Get-PeMachine`）に在り、pasta.dll にも使える。`build.yml`（411 行）に arm64 のランナー（`windows-11-arm`）の job が 2 本増えた（225 行の `recipe-arm64-test`・263 行の `recipe-arm64-smoke`）。`manual-reader-first` がスターター `pasta-starter.nar` を足した（配布物 8 つ・添付 9 つ）。`hello-pasta-book-sync` が、ネットワーク更新を「main のファイル＋最新 Release の nar にだけ在るファイル」で組むようにした（`book/tools/ghost-site/ghost-site.mjs` の `narOnlyFiles`。git が追わない名前を nar から拾う）。
- **残る仕事**: `release.ps1`（`crates/pasta_sample_ghost/`。475 行）の 32bit のビルド（78 行）の隣に x64・arm64 の `pasta_shiori` のビルドを足し、`pasta.x64.dll`・`pasta.arm64.dll` を hello-pasta とスターターへ写す（DLL の写しは 104 行、スターターの組み立ては 253 行から、`pasta.dll.zip` は 314 行の段 5）。`release.yml` の検査（`pasta.dll.zip` の中身 126〜129 行・.nar の必須 163 行・スターターの完全一致 173 行）と、pasta.dll の PE の arch の検査を足す。`about.toml` 17 行の targets に aarch64 が無い（pasta.dll のライセンス表示は `release.ps1` 126 行の `cargo about generate` がこれを読む）。`.gitignore` 40 行は `pasta.dll` だけなので、2 つの名前を足す。ネットワーク更新は、git が追わない名前を nar から拾うので、手を入れずに載る見込み（`ghost-site-test.mjs` で確かめる）。マニュアル（`download.md` の表と「SHIORI の本体」の節、`[loader] ffi` の arch の注意）。
- **触るファイル**: `release.ps1`・`release.bat`（32 行。段の説明）・`release.yml`・`about.toml`（28）・`.gitignore`・pasta.dll の arch の検査（`verify-recipe-zips.ps1` を共通にするか写すか）・`release-scripts-test.ps1`（522）・`build.yml`（arm64 で `pasta_shiori` を試すなら）・`pasta_sample_ghost/tests/` の `pasta.dll` を見る所（`common/mod.rs` 37 行・`integration_test.rs`・`starter_test.rs`・`tutorial_stages_test.rs`）・マニュアル。zip を arch ごとに分けるなら、`github-release.ps1` 72 行・`release-state.ps1` 84 行・`link-check.mjs` の `checkReleaseAssetLinks` も。1,000 行に近いものは無い。
- **規模**: 9〜11（静的 CRT と `targets:` は済んだ。PE の検査は使い回せる）。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `shiori4-v4-host`（`build.yml`）、`hello-novel-sample`（`tests/common/mod.rs`）。単独の席: `release.yml`・`release.ps1`（リリースの実行中は main へ入れない）。前回の `pasta-recipe`・`vsce-4-upgrade` は入った。
- **フェーズ**: ノベルではない。areka の x64・arm64 向けだが、SSP の 32bit 版は変えないので、areka を待たずに入れられる（areka の実機の確認は後）。
- **種別**: 機能（areka の 64bit・arm64 向けの配布物。areka の相談から）。
- **要件定義のモデル**: Opus（前のまま）。
- **分割の案**: 要らない。
- **見つけた穴・古くなった記述**: Current State の行番号、「aarch64・arm64 の言及は無い」「`.cargo/config.toml` に aarch64 の指定は無い」は古い。配布物の数の申し送り（5・6・7・8）は、配布物 8 つ・添付 9 つから数える。前回の「`.cargo/config.toml` 11 行の説明の誤り」は直っていた。前回から残るもの: aarch64 の LuaJIT（mlua）と `embed-resource` の VERSIONINFO は、要件の前に試す（pasta-recipe は LuaJIT を持たないので、確かめたことにならない）。CI の arm64 の pasta.dll を、x64 からのクロス（`pasta-recipe-multiarch` と同じ）で作るか、arm64 のランナーで作るかを決める。
