# Brief: pasta-recipe-multiarch

> **ステータス**: 未着手（2026-11-14 の `/kiro-discovery` で起票。開発者の問い「`/release` の CI に、pasta-recipe.x64.zip / arm64 は含まれるか？」→ 含まれていないと答えた → 開発者「arm64 を起票して」「起票の要件に、x64 も含めた CI ビルド完成まで含めてください」）。版は開発者が決める（推奨は 0.5.3。下の「版」）。要件定義は Opus（Fable なし）。

## Problem

pasta-recipe（VS Code を使わない pasta DSL のエディタ）は、Windows x64 の `pasta-recipe.zip` 1 つしか配っていない。arm64 の Windows（Snapdragon の機械など）を使う作者は、x64 の exe をエミュレーションで動かすしかない。動くかどうかは確かめていない。開発者は、x64 版と arm64 版をどちらもリリース CI で作り、`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` として配りたい。

## Current State

2026-11-14（main `cf244e95`）に確かめた。

- **リリース CI**: `.github/workflows/release.yml` の 138〜139 行で `editors/recipe/scripts/package.ps1` を 1 回だけ呼ぶ。ランナーは `windows-latest`（x64）。配布物の検査（218〜237 行）は `release/pasta-recipe.zip` について、中身が `pasta-recipe.exe`・`THIRD_PARTY_LICENSES.txt` の 2 つ、exe の版がタグの版、30 MB 以下、を確かめる。アップロードの一覧（258 行）も `release/pasta-recipe.zip` だけ。
- **package.ps1**: `npx tauri build --no-bundle` で、target を指定せずに作る（`src-tauri/target/release/pasta-recipe.exe`）。ライセンス表は `cargo about generate ... --target x86_64-pc-windows-msvc` で、x64 に決め打ち。zip の名前は `release/pasta-recipe.zip` に決め打ち。
- **build.yml**: job `recipe`（`windows-latest`）は `x86_64-pc-windows-msvc` だけを入れて、型・テスト・ビルドを回す。aarch64 は試していない。
- **静的 CRT**: ルートの `.cargo/config.toml` は i686・x86_64 だけに `crt-static` を指定する。`package.ps1` は `editors/recipe` の下で cargo を動かすので、ルートの設定が効く。aarch64 の指定は無い（`pasta-dll-multiarch` も同じ所に足す予定）。
- **依存**: アプリの `editors/recipe/src-tauri/Cargo.lock` に `mlua`・`pasta_lua` は無い。`pasta-dll-multiarch` が心配している「aarch64 で LuaJIT がビルドできるか」は、このアプリには関係しない。C のビルドが要るのは `cc` を使う `embed-resource`（リソースコンパイラ）・`vswhom-sys` くらい。`webview2-com-sys`・`windows_aarch64_msvc` は lock に入っている。
- **名前 `pasta-recipe.zip` を使っている所**（`completed/` を除く）: `release.yml`、`package.ps1`、`.github/scripts/release/github-release.ps1`（添付 7 つの一覧）とその自己テスト `release-scripts-test.ps1`、`.claude/skills/release/scripts/release-state.ps1`・`procedure.md`（添付の数え方 7/7・6/7・4/7）、`.claude/skills/pasta-check/SKILL.md`、README、`book/src/debug/recipe.md`（固定のダウンロードのリンク）、`book/src/introduction.md`、`crates/pasta_sample_ghost/RELEASE.md`、`steering/tech.md`・`structure.md`、この roadmap。
- **出した版**: 最新のタグは v0.5.2。`pasta-recipe.zip` は 0.5.3 から添付する予定で、まだ一度も出ていない。
- **範囲外にした経緯**: `pasta-recipe` の spec は「mac・Linux・arm64・x86 の配布」を範囲外にしていた（`completed/pasta-recipe/requirements.md`）。この spec が、そのうち arm64 を引き取る。

## Desired Outcome

- リリース CI が、1 つのタグから **`pasta-recipe.x64.zip` と `pasta-recipe.arm64.zip` の 2 つ**を作り、検査して、GitHub Release に添付する。**x64 も含めて、CI のビルドと検査と添付が全部そろったところまでが、この spec の完成**（開発者の指示）。
- どちらの zip も、中身は `pasta-recipe.exe` と、その arch のライセンス表 `THIRD_PARTY_LICENSES.txt` の 2 つ。exe の版はタグの版。
- arm64 の zip の exe は、本当に arm64 の実行ファイルである（PE ヘッダの機械の種類が ARM64）。x64 の zip の exe は x64 である。取り違えを CI が見つける。
- 手元でも `package.ps1` で同じ 2 つが作れる。
- `pasta-recipe.zip` の名前は出さない。名前を使っていた所（上の一覧）が、全部 2 つの新しい名前にそろう。
- マニュアルと README で、自分の Windows の arch に合う zip を選べる（固定のダウンロードのリンクを 2 つにする）。

## Approach

**x64 の windows のランナー 1 つで、`--target` を付けて 2 回作る（arm64 はクロスコンパイル）**（推奨。細かい所は要件・設計で決める）。

1. **package.ps1 に arch を渡す**: 引数（例 `-Arch x64|arm64`）で、`tauri build --no-bundle --target <triple>`・`cargo about --target <triple>`・zip の名前 `pasta-recipe.<arch>.zip` を切り替える。exe の置き場は `src-tauri/target/<triple>/release/` になる。フロントの `npm run build` は 1 回で済むかを設計で見る。
2. **静的 CRT**: ルートの `.cargo/config.toml` に `[target.aarch64-pc-windows-msvc]` の `crt-static` を足す（`pasta-dll-multiarch` も同じ 1 段を要る。先に入った方が足し、後の方は在るのを確かめる）。
3. **リリース CI**: `release.yml` で、rust の target に `aarch64-pc-windows-msvc` を足し、package.ps1 を 2 回呼ぶ。配布物の検査を 2 つの zip へ広げ、PE ヘッダの機械の種類（x64 = `0x8664`、ARM64 = `0xAA64`）も確かめる。30 MB の上限は zip ごと。アップロードの一覧、`github-release.ps1` の添付の一覧（7 つ → 8 つ）とその自己テスト、`release-state.ps1`・`procedure.md` の添付の数え方を合わせる。
4. **build.yml**: job `recipe` に aarch64 のビルド（少なくとも `cargo build --target aarch64-pc-windows-msvc`）を足し、arm64 で壊れたのを main で見つける。テストは x64 のまま（x64 のランナーでは arm64 の exe を動かせない）。
5. **文書**: マニュアル（`debug/recipe.md`・`introduction.md`）・README・`RELEASE.md`・steering（`tech.md`・`structure.md`）・`pasta-check` スキルの名前を直す。

退けた案:
- `windows-11-arm` のランナーで arm64 を作る（ネイティブで作れて、その場で起動も試せる）。job が 2 つに分かれ、成果物の受け渡しと Node・Rust の用意が倍になる。arm64 の起動の確かめを CI で要るかは要件で決め、要るならこの形を考え直す。
- x64 の名前を `pasta-recipe.zip` のまま残し、arm64 だけ足す。開発者の名前（`pasta-recipe.x64.zip`）と合わず、2 つが対にならない。0.5.3 より前なら古い名前を出していないので、残す理由も無い。

## Scope

- **In**:
  - `editors/recipe/scripts/package.ps1` の arch の切り替え（triple・ライセンス表・zip の名前・自己検査）。
  - ルートの `.cargo/config.toml` の aarch64 の `crt-static`。
  - `.github/workflows/release.yml`（2 つのビルド・配布物の検査・PE ヘッダの検査・アップロード・Cargo.lock の不変検査）。
  - `.github/workflows/build.yml` の job `recipe` の aarch64 のビルド。
  - `.github/scripts/release/github-release.ps1` と `release-scripts-test.ps1`（添付 8 つ）。
  - `.claude/skills/release/scripts/release-state.ps1`・`procedure.md`（添付の数え方。0.5.3 からは 8/8）、`.claude/skills/pasta-check/SKILL.md`。
  - マニュアル・README・`crates/pasta_sample_ghost/RELEASE.md`・steering の名前と、arch の選び方。
  - x64 も含めて、リリース CI で 2 つの zip が作られ検査を通ることの確かめ（`workflow_dispatch` か、それに当たる回し方。タグを打って本当のリリースをするのは `release` スキル）。
- **Out**:
  - x86（32bit）・mac・Linux の pasta-recipe。
  - インストーラ・コード署名・自動更新。
  - arm64 の exe の実機での起動の確かめを CI で行うこと（要件で要否を決める。決めるまでは範囲外）。
  - pasta.dll の arch（`pasta-dll-multiarch` が持つ）。
  - アプリの機能の変更。

## Boundary Candidates

- ビルドの段（package.ps1・`.cargo/config.toml`）と、CI の段（release.yml・build.yml の組み込みと検査）。
- 配布物の名前の付け替え（スクリプト・スキル・文書の一斉の書き換え）。

## Out of Boundary

- `pasta-dll-multiarch` の pasta.dll・見本ゴーストの .nar・`release.ps1`。
- pasta-recipe の画面と Rust のコマンド（`pasta-recipe`・`pasta-recipe-dev-actions` が作ったもの）。

## Upstream / Downstream

- **Upstream**: `pasta-recipe`（`package.ps1`・`release.yml` の組み込み。入った）。`pasta-recipe-dev-actions`（入った）。
- **Downstream**: 0.5.3 のリリース（`release` スキル）。0.5.3 に入れるなら、リリースのウェーブの前に入れる。

## Existing Spec Touchpoints

- **Extends**: なし（`pasta-recipe` は `completed/` にあり、範囲外にしていた arm64 をこの spec が引き取る）。
- **Adjacent**: `pasta-dll-multiarch`（`.cargo/config.toml` の aarch64 の行と `release.yml` を共有する。同じウェーブに置かない）。`vsce-4-upgrade` は 2026-11-14 に入った（`release.yml` の vsce まわりを変えたので、着手の前に今の `release.yml` を読み直す）。

## Constraints

- **版**: 開発者が決める。推奨は 0.5.3（リリースの前に入れれば、`pasta-recipe.zip` の名前は一度も出ない。古い名前の互換も、`release-state.ps1` の過去の版の数え方の追加も要らない）。0.5.3 より後にするなら、0.5.3 が出す `pasta-recipe.zip` から名前を変えることになり、固定のダウンロードのリンクの互換と、添付の数え方の版ごとの読み替えを要件で決める。
- **静的 CRT**: pasta の配布物は VC++ ランタイムに頼らない（ルートの `.cargo/config.toml` の方針）。arm64 版も同じ。
- **x64 のランナーでは arm64 の exe を動かせない**: 検査は中身・版・PE ヘッダ・大きさまで。
- **`.claude/skills/` の編集**: `release-state.ps1`・`procedure.md`・`SKILL.md` の書き換えは、分類器が Self-Modification として拒むことがある。拒まれたら回避策を探さず、入った分と残りを示して開発者の許可を求める。
- **手元のビルド**: `cargo build` の前に `NoDefaultCurrentDirectoryInExePath` を外す（このアプリは LuaJIT を持たないが、ルートのクレートを触るときは要る）。
- **確かめていない危険**: `windows-latest` の MSVC に ARM64 のビルドツールがあり、`embed-resource` のリソースコンパイラと Tauri 2.12 の `--target aarch64-pc-windows-msvc` がクロスで通るかは、2026-11-14 時点で試していない。要件の前か、設計の最初に `workflow_dispatch` で試す。
