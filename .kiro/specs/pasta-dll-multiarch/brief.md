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

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**: 起票の後に入った 4 本は、ビルド・配布物・`release.ps1`・`release.yml` を触っていない。Current State の行番号（`release.ps1` 71 行・97〜107 行、`release.yml` 141〜144 行・163 行）は今も合っている。aarch64・arm64 の言及が無いのも同じ。名前の決まり（x64 は `pasta.x64.dll`、arm64 は `pasta.arm64.dll`）は shiori4 の main（PR #7）に入った。
- **触るファイル**: `.cargo/config.toml`（aarch64 の `crt-static`）、`crates/pasta_sample_ghost/release.ps1`（331 行）、`.github/workflows/release.yml`（589 行。75 行の targets・80 行の cache の key・141〜163 行の中身の検査・208 行の添付）、`.github/workflows/build.yml`（arm64 のビルドを足すなら。今の matrix は i686 と x86_64 だけ）、`about.toml`（17 行の `targets` に aarch64 が無い。ライセンスの一覧を arch ごとに作るなら要る）、`.gitignore`（40 行は `ghost/master/pasta.dll` だけを外す。`pasta.x64.dll`・`pasta.arm64.dll` も足す）、`crates/pasta_sample_ghost/tests/`（`pasta.dll` を見るテスト 4 つ）、マニュアル（`getting-started/13-nar.md`・`reference/startup.md`・`internals/shiori.md` など、`pasta.dll` に触れる頁が 12）。FFI の型を直すならこの spec が `pasta_shiori/src/windows.rs` も持つ。
- **規模**: 9〜12。
- **先に要るもの**: `shiori4-v3-portable`（areka は `shiori_get_api` で読む）。x64 版を出す前に、SHIORI/3.0 の書き出しの型（`usize`→`c_long`、`bool`→i32 の `BOOL`。`windows.rs` 109・129・222・257 行）を直すこと。`shiori4-v3-portable` が `BOOL` を直すなら、残りは `c_long` だけ。そこも同じ spec か `ponytail-shiori` で先に直すのが推奨（この spec が `windows.rs` を持たずに済む）。ファイルの重なり: `ponytail-manual-ci`（`release.yml`・`build.yml`）、`ponytail-check-release`（`pasta_sample_ghost/tests/**`）、`shiori4-v3-portable`（`build.yml`）、`windows.rs` を直すなら `ponytail-shiori`。
- **種別**: 機能（areka の 64bit・arm64 向けの配布物）。
- **要件定義のモデル**: Opus。決めることは配布物の形（zip を 1 つにするか分けるか）と arm64 を出すかで、どれも後から変えられる。
- **分割の案**: 要らない。
- **見つけた穴・古くなった記述**: aarch64 で LuaJIT（mlua の vendored）がビルドできるかは、まだ試していない（前のまま。要件の前に試す）。CI で arm64 を作るには、arm64 の Windows の runner か、x64 からのクロスビルドが要る。`embed-resource` の VERSIONINFO も arm64 で通るかを一緒に試す。`.cargo/config.toml` の頭の説明（9 行「出荷ターゲットは x86・x64 の両方」）は、x64 の `pasta_check.exe` のことで、x64 の pasta.dll は配っていない。arch を足すときに書き直す。

## 申し送り（pasta-skills-release-asset より）

- `release.ps1` は 8 段になった（段 6 が `pasta-skills.zip` の作成と検査、版の確認が 7、大きさとまとめが 8）。arch を足すときは段 6 を保ち、段の番号と `/8` の見出し・`release.bat` の説明・文書の段の並びを合わせる。
- 配布物は 5 つ・GitHub Release の添付は 6 つになり、`release.yml` の「Verify release assets」に `pasta-skills.zip` の項目が 1 つ増えた。配布物を足すときは `release.ps1`・`release.yml`・`github-release.ps1`・`.claude/skills/release/scripts/release-state.ps1`・文書の 5 か所を合わせる。参照先: `.kiro/specs/completed/pasta-skills-release-asset/design.md` の「Revalidation Triggers」。
