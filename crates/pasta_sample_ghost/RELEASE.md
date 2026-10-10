# hello-pasta リリース手順書

> このドキュメントは AI と相談しながらリリースを進めるためのガイドです。

リリースは、リリースタグ `vX.Y.Z` の push を契機にリリース CI（`.github/workflows/release.yml`）が行います。
ビルド・crates.io への公開・VSCode Marketplace への公開・GitHub Release の作成（`pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix` の添付）は、すべて CI の中で行われます。手元で公開や Release の作成をする必要はありません。

## 前提条件

- [x] リリース CI の一回限りのセットアップが済んでいる（[`.github/release-ci-setup.md`](../../.github/release-ci-setup.md)）
- [x] PR を `main` へマージでき、リリースタグを push できる（`main` へは直接 push できない）

## リリース手順

### Step 1: 版を上げたコミットを main に入れる

- 版の決定と版を上げるコミットは、`release-workflow` の手順で行います（`Cargo.toml`・`Cargo.lock`・`editors/vscode/package.json`・`editors/vscode/package-lock.json`・`book/src/introduction.md` の版をそろえて 1 コミットにする）。
- そのコミットを PR で `main` へ squash マージします。
- タグと `Cargo.toml`・`package.json` の版が一致し、タグのコミットが `main` から到達できることを、CI の verify job が検査します。

### Step 2: リリースタグを push する

版の更新を `main` へ統合した結果のコミット（squash マージでできたコミット。ふつうは `origin/main` の先頭）に、注釈付きのタグを付けて push します。先頭が別のコミットなら、下の `origin/main` の代わりに、そのコミットの SHA を指定します。

```bash
git fetch origin main
git tag -a v{VERSION} -m "Release v{VERSION}" origin/main
git push origin v{VERSION}
```

- タグの形は `vX.Y.Z`（数字 3 つ）だけです。`v1.0`・`v1.0.0-rc.1` などでは CI は起動しません。

### Step 3: Actions の結果を確かめる

GitHub の「Actions」→「Release」の実行を開きます。job は次の順に進みます。

| job | 内容 |
|-----|------|
| verify | タグの形・版の一致・`main` からの到達を検査 |
| gate | `build.yml` の検査（test・clippy・deny・luacheck・WASM）をタグのコミットで実行 |
| build | `pasta.dll.zip`・`hello-pasta.nar`・VSIX を作る |
| publish-crates | crates.io へ 5 クレートを公開 |
| publish-vsce | VSCode Marketplace へ拡張を公開 |
| github-release | リリースノートを生成し、GitHub Release `pasta vX.Y.Z` を作成 |
| report | 公開先ごとの結果と Release の URL を表にまとめる |

report job の Summary の表で、公開先ごとの結果を確かめます。

- `published`: 今回公開した
- `skipped`: 公開済みのため飛ばした
- `failed`: 失敗した（`reason` に原因の区分）
- `not-run`: 前の段の失敗で行わなかった
- 「job の summary を参照（… の結果: failure）」: その job が結果を書く前に失敗した。その job の Summary とログを見る

すべての公開先が `published` か `skipped` になり、GitHub Release の URL が出ていれば完了です。リリースノートは前のリリースタグからのコミットの件名から自動で生成されます。

### Step 4: 失敗したら、失敗した job を再実行する

実行の画面の「Re-run failed jobs」を押します。各公開先は実際の状態で公開済みかを判定するので、公開済みのものは `skipped` で飛ばされ、残りだけが公開されます。手元でコマンドを実行する必要はありません。

再実行で直らない場合は、次の「失敗したときの回復」を見てください。

## 失敗したときの回復

まず report の表と、失敗した job の Summary で原因を確かめます。

### 1. 一時的な失敗（公開先の障害・ネットワーク・タイムアウトなど）

失敗した job を再実行します（「Re-run failed jobs」）。公開済みのものは飛ばされるので、何度再実行しても二重に公開されることはありません。再実行は、初回の実行から 30 日以内・50 回までできます（GitHub の制限）。

認証の失敗（`reason` が `auth`）や、クレートが crates.io に無い（`reason` が `not-registered`）など、一回限りのセットアップに起因する失敗は、[`.github/release-ci-setup.md`](../../.github/release-ci-setup.md) に従って設定を直してから再実行します。

### 2. 再実行で直らない失敗（ワークフローの定義の不具合、タグのコミットで関門・検査が通らないなど）

再実行は、タグのコミットにあるワークフローの定義とコードで動きます。定義そのもの（`release.yml`・`.github/scripts/release/` など）の不具合や、タグのコミットで gate（test・clippy など）や verify（版の一致など）が通らないといったコードの問題は、再実行では直りません。まず直す PR を `main` へマージし、そのうえで公開の状況に応じて次のどちらかを行います。verify・gate・build で止まった（report に「どの公開先にも公開していない」と出ている）場合は、タグの付け直しにあたります。

**何も公開していない場合: タグを付け直す**

どの公開先にもまだ公開していないときに限り、同じ版のタグを、直したコミットに付け直してよいです。

- report に「どの公開先にも公開していない」と出ている（verify・gate・build で止まった）なら、この場合にあたります
- それ以外は、report の表に `published`・`skipped` が 1 つも無く、`failed` の公開先（crates.io・Marketplace）にもその版が出ていないことを確かめます

```bash
git fetch origin main
git tag -f -a v{VERSION} -m "Release v{VERSION}" origin/main
git push -f origin v{VERSION}
```

タグの push で、新しい実行が始まります。

**公開の途中の場合: 版を上げて出し直す**

1 つでも公開先にその版が出ている（`published`・`skipped` がある、または `failed` でも公開先に版が出ている）ときは、タグを付け直しません。Step 1 から、版を上げて出し直します。前の版で公開済みのものは、取り消したり上書きしたりしません。

## 手元での release.ps1（動作確認用）

手元の `release.ps1`（ワークスペースルートの `release.bat` からも呼べる）は、配布物が作れるかを確かめるためのものです。作った成果物（`release/` の中身、サンプルゴーストの `pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`scripts/` など）はコミットしません。公開に使う配布物は、CI がタグのコミットから作り直します。

```powershell
cd crates\pasta_sample_ghost
pwsh -File release.ps1
```

- `cargo-about` がインストール済みであること（`cargo install cargo-about`）。`release.ps1` が第三者ライセンス表示 `THIRD_PARTY_LICENSES.txt` を `pasta.dll` の隣に生成します
- ビルド、ゴースト生成、バリデーション、`pasta.dll.zip`・`.nar` ファイルの作成まで一括実行されます
