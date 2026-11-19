# hello-pasta リリース手順書

> このドキュメントは AI と相談しながらリリースを進めるためのガイドです。

リリースは、リリースタグ `vX.Y.Z` の push を契機にリリース CI（`.github/workflows/release.yml`）が行います。
ビルド・crates.io への公開・VSCode Marketplace への公開・GitHub Release の作成（`pasta.dll.zip`・`hello-pasta.nar`・`pasta-starter.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip`・`pasta-skills.zip`・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` の添付）は、すべて CI の中で行われます。手元で公開や Release の作成をする必要はありません。

## 前提条件

- [x] リリース CI の一回限りのセットアップが済んでいる（[`.github/release-ci-setup.md`](../../.github/release-ci-setup.md)）
- [x] PR を `main` へマージでき、リリースタグを push できる（`main` へは直接 push できない）

### リリース前の確認

hello-pasta の置き場所（GitHub Pages）を足した変更を入れた後の最初のリリースの前に、次を確かめます。

- 置き場所の変更を `main` へ入れた直後の公開について、実 URL での確認が通っている。`--full` は `updates.txt` の各行のファイルを取り、`.dll`・`.pasta`・`.png` などが変換されずに返ることを md5・大きさで確かめます。`{VERSION}` と `{WORK}` の用意は、下の「手元で組み立てる」と同じです

  ```bash
  node book/tools/ghost-site/ghost-site.mjs verify-live --nar {WORK}/hello-pasta.nar --tree v{VERSION}:crates/pasta_sample_ghost/ghosts/hello-pasta --base https://ekicyou.github.io/pasta/hello-pasta/ --full
  ```

- 通っていないときは、リリースを出さず、直す PR を先に `main` へ入れる

### 利用者に更新が届く条件

`homeurl` を持つ版の `hello-pasta.nar` を入れた利用者から、更新が効きます。それより前の版を入れている利用者には届きません。この版を入れ直してもらうまで、更新は届かないものとして扱います。

## リリース手順

### Step 1: 版を上げたコミットを main に入れる

- 版の決定と版を上げるコミットは、`release` スキル（`/release`）の手順で行います（`Cargo.toml`・`Cargo.lock`・`editors/vscode/package.json`・`editors/vscode/package-lock.json`・`book/src/introduction.md` の版をそろえ、その版の見どころのファイル `doc/release-notes/vX.Y.Z.md` を書いて 1 コミットにする）。見どころの書き方は `doc/release-notes/README.md` にあります。
- そのコミットを PR で `main` へ squash マージします。
- タグと `Cargo.toml`・`package.json` の版が一致し、タグのコミットが `main` から到達できることを、CI の verify job が検査します。
- タグを打つ前に、タグのコミットにその版の見どころのファイルが要ります。無いとき、または中身が空白と改行だけのときは、verify job で止まり、どこにも公開しません。

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
| verify | タグの形・版の一致・`main` からの到達・見どころのファイルの有無を検査 |
| gate | `build.yml` の検査（test・clippy・deny・luacheck・WASM）をタグのコミットで実行 |
| build | 配る pasta のスキルに入れたマニュアルの写しが最新かを照合して、`pasta.dll.zip`・`hello-pasta.nar`・`pasta-starter.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip`・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` を作る |
| publish-crates | crates.io へ 5 クレートを公開 |
| publish-vsce | VSCode Marketplace へ拡張を公開 |
| github-release | 詳しいリリースノートを作り、見どころを本文とする GitHub Release `pasta vX.Y.Z` を作成して、配布物 8 つと詳しいリリースノートの 9 つを添付 |
| publish-ghost-site | 最新のタグのときだけ、Actions の `Manual`（`manual.yml`）を `main` で起動して待ち、公開後の実 URL の `updates.txt` を Release の nar の側の行と見比べる（`github-release` が成功したときだけ走る） |
| report | 公開先ごとの結果と Release の URL を表にまとめる |

report job の Summary の表で、公開先ごとの結果を確かめます。

- `published`: 今回公開した
- `skipped`: 公開済みのため飛ばした
- `failed`: 失敗した（`reason` に原因の区分）
- `not-run`: 前の段の失敗で行わなかった
- 置き場所（`GitHub Pages hello-pasta`）の行だけ、ラベルが違う。`published` は「今回更新した」、`skipped` は「最新ではないタグのため飛ばした」（`reason` は `stale-tag`。古いタグの再実行で、何も起こしていない）
- 「job の summary を参照（… の結果: failure）」: その job が結果を書く前に失敗した。その job の Summary とログを見る

置き場所の行が `failed` のときの次の手は、`reason` ごとに次のとおりです。

| reason | 意味 | 次の手 |
|--------|------|--------|
| `dispatch` | `Manual` を起動できない、または起動した run を特定できない | 権限（`actions: write`）と API の応答をログで確かめ、直したら再実行 |
| `build` | `Manual` の run が失敗した（置き場所の検査・照合で止まったことが多い） | その run のログを読み、下の「検査で止まったとき」に従って直す |
| `verify` | 実 URL の `updates.txt` の nar の側の行が Release の nar と違う、または取れない | 配信の遅れなら失敗した job を再実行。続くなら Pages の公開状況を確かめる |
| `transient` | 最新の判定・nar の取得・run の待ちの時間切れ | 失敗した job を再実行 |
| `publish` | スクリプトの異常終了（最新の Release のタグが `vX.Y.Z` の形でない、`homeurl` が無い、など） | ログを読む。再実行で直らないことがある |

すべての公開先が `published` か `skipped` になり、GitHub Release の URL が出ていれば完了です。Release の本文は、見どころのファイルの中身に、詳しいリリースノートへのリンクと件数の行を足したものです。詳しいリリースノート（`release-notes-vX.Y.Z.md`）は、前のリリースタグからのコミットの件名から作られ、添付されます。

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

### 3. hello-pasta の置き場所（GitHub Pages）の失敗

置き場所は、Release を作った後の最後の段です。ここが `failed` でも、crates.io・Marketplace・GitHub Release の公開は済んでいます。版を上げて出し直す必要はありません。

- 失敗した job（`publish-ghost-site`）を再実行します。最新ではないタグなら `skipped` で飛ばされます。
- 再実行で直らないときは、Actions の `Manual` を `main` で手動実行します。`Manual` はその時点の `main` と最新の Release の nar から置き場所を作り直して公開します。
- 初回の実リリースで `dispatch` になったときは、権限（`actions: write`）と、`manual.yml` が `main` にあることを確かめます。
- リリースの実行中は、`release.yml` を `main` へ入れません。起動した `Manual` が `main` の定義で動くためです。

**`latest` の nar が壊れていたとき**

`build` ジョブは、公開前に nar を一度展開して照合するので、壊れた nar は Release になりません。それでも壊れた nar が `latest` になったときは、置き場所が `Manual` の照合で止まります。公開済みの Release は差し替えられないので、直す PR を `main` へ入れて、新しい版を切り、出し直します。

## hello-pasta の更新の置き場所

### 作り方

置き場所（`https://ekicyou.github.io/pasta/hello-pasta/`）は、`main` のリポジトリの hello-pasta のファイルに、最新の Release の `hello-pasta.nar` にだけ在るファイル（`pasta.dll` など、最新の Release のタグのツリーに無いもの）を重ねて作ります。重ねた一式を最新の Release と同じ版の `pasta_check release` で検査し、`updates.txt` を作り直して照合してから公開します。

置き場所が出るきっかけは 2 つです。

- `main` への push（hello-pasta の配下の変更を含む）。辞書の変更は、リリースを待たずに利用者へ届きます
- リリース。リリース CI が `Manual` を `main` で起動し、nar の側を新しい Release のものに入れ替えます

リリースの直後の数分の間（Release が最新になってから、リリース CI の起動した `Manual` が公開を終えるまで）は、nar の側が 1 つ前の Release のままです。この間に新しい nar を入れて更新確認をした利用者は古い `pasta.dll` を受け取り、次の更新確認で新しいものに戻ります。

### 検査で止まったとき

`Manual` の「Build hello-pasta update site」の段で止まり、ログに `pasta_check release` の報告（ファイル・行・理由）か、照合の `NG <パス>: …` が出ます。どちらのときも公開しません。

辞書が、まだリリースしていない `pasta.dll` の機能を使っていて止まったときは、次のどちらかで直します。

- リリースを出す。リリース CI の起動する `Manual` が、新しい版の `pasta_check` で通して公開します
- 急ぐときは、辞書を戻す PR を `main` へ入れる

この検査は Lua を動かしません。読み込める辞書で、実行の結果だけが変わる違い（新しい日時変数の値が空になるなど）は止められません。

### 見比べの読み方

リリース CI の `publish-ghost-site` は、実 URL の `updates.txt` のうち nar の側の行だけを、Release の nar の `updates.txt` の行と md5・大きさで比べます。リポジトリの側の行と日付の欄は比べません。

- 一致: `OK updates.txt: <n> 回目で nar の側の <k> 行が一致`。報告の表は `published`
- 合わない・取れない: `NG <パス>: md5 が違う（nar: …、実 URL: …）` などが出て、報告の表は `failed`（`reason` は `verify`）

### ファイルを消す・改名するとき

`main` でファイルを消す・改名すると、置き場所からはなくなります。SSP の更新は、`updates.txt` が前回と違うファイルだけを取り、古いファイルを消しません。利用者の手元には古いファイルが残り、pasta が読み続けることがあります。`delete.txt` を使うかは別に検討します。

### 手元で組み立てる

リポジトリのルートで、最新の Release の版 `{VERSION}` と、リポジトリの外の作業フォルダ `{WORK}` を決めて実行します。`{WORK}/site` は無いか空にしておきます。

```bash
git fetch origin tag v{VERSION}
gh release download v{VERSION} --pattern hello-pasta.nar --dir {WORK}
cargo install pasta_check --version {VERSION} --locked --root {WORK}/pasta-check
PASTA_CHECK={WORK}/pasta-check/bin/pasta_check bash book/tools/ghost-site/assemble-site.sh HEAD v{VERSION} {WORK}/hello-pasta.nar {WORK}/site
```

- 引数は、組み立てるコミット・最新の Release のタグ・その nar・出力先の順です
- `pasta_check` が PATH に在れば、`PASTA_CHECK` は要りません
- 止まったときは、上の「検査で止まったとき」と同じ報告が出ます

## 手元での release.ps1（動作確認用）

手元の `release.ps1`（同じフォルダの `release.bat` からも呼べる）は、配布物が作れるかを確かめるためのものです。作った成果物（`release/` の中身、サンプルゴーストの `pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`scripts/` など）はコミットしません。公開に使う配布物は、CI がタグのコミットから作り直します。

```powershell
cd crates\pasta_sample_ghost
pwsh -File release.ps1
```

- `cargo-about` がインストール済みであること（`cargo install cargo-about`）。`release.ps1` が第三者ライセンス表示 `THIRD_PARTY_LICENSES.txt` を `pasta.dll` の隣に生成します
- `pasta.dll` のビルド、DLL と `scripts/` のコピー、`pasta_check.zip` の作成（64 ビット版の `pasta_check.exe` とそのライセンス表示）と中身の検査、その `pasta_check.exe` での `pasta_check release`（検査と `.nar` の作成）、`pasta.dll.zip` の作成、`pasta-skills.zip` の作成（pasta のスキル 3 つと `LICENSE`）と中身の検査、版の確認、`.nar` の大きさの検査までの 8 段を一括で実行します
