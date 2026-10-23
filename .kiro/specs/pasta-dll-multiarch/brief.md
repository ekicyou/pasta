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
  - arm64 の Windows では `pasta.arm64.dll` → `pasta.x64.dll` → `pasta.dll` の順に探す。
  - 接尾辞の無い `pasta.dll` は、SSP 向けの 32bit 版のまま残す。
- 作者は、自分のゴーストに同じ名前で同梱できる。手順がマニュアルにある。

## Approach

areka の探し方（2026-10-23 の areka の開発者の決定。shiori4 の正本に置く提案中）に名前を合わせる。`pasta_check` は今のまま使える。手を入れるのは、ビルド・配布物・見本ゴースト・リリース CI・マニュアルである。

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
  - shiori4 で決まる arch ごとの名前の決まり（areka の論点 12。2026-10-23 時点で裁定待ち）
- **Downstream**: areka での x64・arm64 の実機確認

## Existing Spec Touchpoints

- **Extends**: なし（`pasta-check-release-asset`（完了）が作った `release.ps1` の段と `pasta_check.zip` は保つ）
- **Adjacent**: `ponytail-manual-ci`・`ponytail-check-release`（隙間の spec。`release.yml`・`github-release.ps1`・`release.ps1` まわりを共有しうる。同じウェーブに置かない）

## Constraints

- pasta.dll は静的 CRT リンクを保つ（新しい arch にも同じ指定をする。動的に戻さない）
- 32bit の `pasta.dll` は SSP 向けに今のまま出す（名前も中身も変えない）
- debug の待受は 127.0.0.1 固定のまま。出荷物は arch ごとに 1 種類（debug は opt-in）
- **確かめていない危険**: pasta は mlua 経由で LuaJIT を静的に組み込んでいる。`aarch64-pc-windows-msvc` で LuaJIT がビルドできるか（クロスコンパイルを含む）は、2026-10-23 時点で確かめていない。要件の前に試す。できないなら、arm64 版を落として x64 版だけにするかを開発者が決める（arm64 の areka は x64 版へ戻って読める）
