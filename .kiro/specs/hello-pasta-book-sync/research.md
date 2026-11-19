# ギャップ分析: hello-pasta-book-sync

> 2026-11-19、`/kiro-validate-gap`。調べた基準は作業ブランチ（main `fcc3ecd0` の後）と、最新の Release v0.6.2。

## 1. まとめ

- 置き場所を作る道具（`manual.yml`・`ghost-site.mjs`・`publish-ghost-site.ps1`）はそろっている。足りないのは「重ねる」「`updates.txt` を作り直す」「新しい dll が要る辞書の検査」「消えたファイル」「見比べの部分一致」の 5 つである。
- **一番大きい見落とし: 改行コード。** nar の中の文字のファイルは CRLF である。ubuntu の checkout は LF である。そのまま重ねると、文字のファイル 19 本の md5 が全部 nar と変わる。v0.6.2 で入れた利用者は、最初の更新で 19 本を全部取り直す（要件 10.2 に反する）。
- `git -c core.autocrlf=true archive <コミット>` で取り出すと、v0.6.2 のタグで 37 本すべてが公開中の md5 と一致した（実測）。改行の問題は、取り出し方 1 つで解ける見込みが高い。
- **分け方の規則に矛盾がある。** 要件 1.4 の「そのコミットのリポジトリに無ければ nar の側」は、main で消したファイルも nar の側に入れてしまう。要件 7.1 とぶつかる。規則を「リリースのタグのツリーに無いもの」か「`.gitignore` に当たるもの」に変える必要がある。
- 新しい dll が要る辞書の検査は、最新の Release の `pasta_check`（crates.io に出ている。Lua に依存しない）を ubuntu で入れれば、Windows なしでできる。ただし今の `manual.yml` は 45 秒前後で終わる。ビルドを足すと何倍にもなる。キャッシュの設計が要る。また、`pasta_check` の検査は「読める」の近似で、`＄行事` のような実行時だけの機能は止められない。

## 2. 今あるもの（確かめた事実）

### 2.1 辞書の読み込み

- `pasta_lua` の読み込みは、`pasta.toml` の `[loader] pasta_patterns` に当たるファイルを全部読む（`crates/pasta_lua/src/loader/discovery.rs`、既定は `dic/**/*.pasta`）。
- hello-pasta の `pasta.toml` は `pasta_patterns = ["dic/*.pasta"]` である。
- そのため、利用者の手元に残った古い `dic/*.pasta` は、読まれ続ける。改名すると、新旧の 2 本が両方読まれる。要件 7 の心配は本物である。

### 2.2 `pasta_check release` の `updates.txt`

- 作り手は `crates/pasta_check/src/update_files.rs` だけである。
- 除外は、フォルダ名 `profile`・`var`（どの階層でも）、ファイル名 `updates2.dau`・`updates.txt`・`developer_options.txt`（どの階層でも）、同梱のバルーンのフォルダ。
- 行は `file,<パス>` `0x01` `<md5>` `0x01` `size=<n>` `0x01` `date=<mtime>` `0x01` と CRLF。パスの名前順。
- できた `updates.txt` を `ghost/master/updates.txt` にも写す。nar には両方が入る。
- `delete.txt` は、どこにも作らない（コードにも文書にも、作る処理が無い）。
- 段 4 で辞書を検査し、誤りなら `updates.txt` と nar を作らずに止まる。

### 2.3 nar の中身と改行コード（実測）

- 公開中の `updates.txt`（v0.6.2）の行は 40 本。
- そのうち 37 本は、リポジトリの `crates/pasta_sample_ghost/ghosts/hello-pasta/` に在る。
- 残りの 3 本は `ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt`・`ghost/master/scripts/README.md`。3 本とも `.gitignore` に書かれた生成物である。
- `scripts/README.md` は、`release.ps1` が `crates/pasta_lua/scripts/` を robocopy で写したもの。元のファイルは git に在る（別の場所）。CRLF にした v0.6.2 の元のファイルの md5 は、公開中の行と一致した。
- **改行**: リポジトリに `.gitattributes` は無い。文字のファイル 19 本は、index では LF である。リリースの `build` ジョブは `windows-latest` で checkout するので、CRLF になる。
  - 例: `dic/01-boot.pasta` は LF で md5 `95cee885…`・313 バイト、CRLF で `acb059a7…`・319 バイト。公開中の行は CRLF のほうである。
  - `git -c core.autocrlf=true archive v0.6.2` で取り出した 37 本は、公開中の md5 と全部一致した。画像（git が文字でないと見るもの）は変わらない。
- v0.6.2 の後、main では hello-pasta の 16 本が変わっている。

### 2.4 `manual.yml`

- 起動の条件（push・PR とも同じ）: `book/**`・自分・`release.yml`・スキル 3 種・`crates/*/README.md`・`README.md`・`doc/SOUL.md`・hello-pasta の `ghost/master/dic/**` と `ghost/master/calendar.csv`・`editors/vscode/syntaxes/**`。シェル・`pasta.toml`・`descript.txt`・`install.txt` は入っていない。
- 置き場所の段: 「Fetch latest hello-pasta.nar」（`releases/latest/download/hello-pasta.nar` を curl）→「Build hello-pasta update site」（`ghost-site.mjs build`）。PR でも走り、失敗は失敗のまま。
- 公開: `Upload Pages artifact` は `github.ref == 'refs/heads/main'` のときだけ。`deploy` ジョブだけが `pages: write`・`id-token: write` を持つ。全体は `contents: read`。
- concurrency: PR は PR ごと、それ以外は `pages` の 1 つの組。`cancel-in-progress: false`。
- main の直近 5 回の所要は、どれも 40〜50 秒（`gh run list` で確かめた）。
- 公開するコミットが main の最新かは、確かめていない。古い run を再実行すると、古い中身で公開しうる（要件 5.2）。

### 2.5 `ghost-site.mjs` と自己テスト

- `readNar`・`parseUpdates`・`checkIntegrity`・`extractNar`・`buildSite`・`verifyLive` と CLI の `build`・`verify-live`。Node 22 の標準だけ。346 行。
- `buildSite` は nar を照合して、全部を除外なしで展開し、`index.html` を足す。出力の一覧が nar＋`index.html` と同じかを最後に確かめる。
- `updates.txt` を作る処理は無い。読むだけ。
- `verifyLive` は、実 URL の `updates.txt` と nar の `updates.txt` の**バイトの一致**を、最大 30 回取り直して待つ。`--full` で各行のファイルも取って照合する。
- 自己テスト `ghost-site-test.mjs`（705 行）は、手で組んだ zip で全関数と CLI を確かめる。`manual.yml` の「Book tools self-tests」で自動で走る。

### 2.6 リリース CI の見比べ

- `release.yml` の `build` ジョブ（windows）が、作った nar を `ghost-site.mjs build` で照合する（公開前の関門）。
- `publish-ghost-site`（ubuntu、タグで checkout）が `publish-ghost-site.ps1` を呼ぶ。古いタグなら `skipped`。最新なら Release の nar を取り、`manual.yml` を main で起動して待ち、`verify-live`（バイトの一致）で見比べる。
- 結果は `status`・`reason`（`stale-tag`・`transient`・`publish`・`dispatch`・`build`・`verify`）で report の 8 行目に出る。
- `release-scripts-test.ps1` の項目 18〜28 が、偽の gh と node で、この判定を全部確かめている。見比べの中身（バイトの一致）は `ghost-site-test.mjs` の側で確かめている。
- タグのコミットが main の祖先であることは、`verify-tag.ps1` が確かめている（要件 5.1 の前提はそろう）。

### 2.7 Ubuntu で検査できるか

- Release の `pasta_check.zip` は Windows の x64 exe だけである。Linux の配布物は無い。
- `pasta_check` は、リリースごとに crates.io に出ている（`publish-crates`）。依存は `pasta_dsl`・`pasta_core`・`toml`・`serde`・`glob`・`lexopt`・`md5`・`zip` で、Lua（mlua・LuaJIT）に依存しない。
- `build.yml` の `build-unix` は ubuntu・macOS でワークスペース全体のテストを通している。`pasta_check` は Linux でビルドできる。
- ubuntu では、`cargo install pasta_check --version <最新のタグの版> --locked` で、最新の Release の版の `pasta_check` が得られる。
- 実行時まで確かめるなら、`build-unix` と同じく `pasta_shiori` を Linux でビルドし、`shiori4-testhost --entry v3` で OnBoot の 1 往復を回せる。ただし LuaJIT のビルドが入り、数分かかる。
- VSIX の WASM は LSP の解析器（`pasta_lsp`）で、CI の検査に流用するには手間が大きい。

### 2.8 手順書

- `crates/pasta_sample_ghost/RELEASE.md` に、置き場所の確認・report の読み方・回復・`delete.txt` の 1 行の注意（135 行目）がある。
- `release` スキル（`procedure.md` の 423・434・466・525 行あたり）が、置き場所の見比べを結果表の 8 行目として扱う。
- `.github/release-ci-setup.md` の 102・108〜110 行に、置き場所の権限の記述がある。
- `.claude/skills/` の編集は、分類器が拒否することがある（メモリの記録）。要件 9.2 の作業は、開発者の許可が要りうる。

### 2.9 SSP の `delete.txt`（UKADOC）

- ゴーストのフォルダからの相対パスを、`\` 区切りで 1 行に 1 つ書く。フォルダは末尾を `\` にする。`charset,` の行は SSP だけ。
- 削除の後も `delete.txt` は消えない。書いてある間、そのファイルは消され続ける。
- `delete.txt` 自身も、ネットワーク更新で届けるファイルである。`updates.txt` に載せないと届かない（`pasta_check release` は `delete.txt` を除外しないので、ツリーに在れば載る）。

## 3. 要件ごとの対応表

| 要件 | 使えるもの | ギャップ |
| ---- | ---------- | -------- |
| 1.1 リポジトリのファイルを置く | `actions/checkout`・`git archive` | **Missing**: 重ねる処理。**Constraint**: 改行（2.3）。LF のまま置くと 19 本が nar と違う |
| 1.2 nar にだけ在るファイル | `readNar` | **Missing**: nar の側だけを選ぶ処理 |
| 1.3 両方に在れば リポジトリ | — | **Missing**。改行をそろえないと、リリースのコミットでも 19 本が「違うもの」になる |
| 1.4 分け方は「リポジトリに在るか」 | `git ls-files`・`git ls-tree` | **Constraint**: 要件 7.1 と矛盾（4.1）。手元の作業ツリーには `pasta.dll` などの無視されたファイルが在るので、作業ツリーでなく git の一覧で決める必要がある |
| 1.5 除外 | `update_files.rs` の除外表 | **Missing**: nar の `updates.txt`・`ghost/master/updates.txt` も nar の側から外す必要がある。`profile`・`var` のフォルダの除外も同じ扱いにするか |
| 1.6 リリースの間は dll が変わらない | nar の展開 | 満たす（nar の中身をそのまま使う限り） |
| 1.7 入口のページ | `buildSite` の `index.html` | 満たす |
| 2.1〜2.3 作り直しと一致 | `pasta_check release`（Rust） | **Missing**: どこで作るか（5 章の案）。`ghost/master/updates.txt` の扱いも決める |
| 2.4・2.5 公開前の照合 | `checkIntegrity` | 重ねた結果に向けて使い直す |
| 2.6 Release・nar が無いとき | 「Fetch latest hello-pasta.nar」 | 満たす |
| 2.7 PR では公開しない | `if: github.ref == 'refs/heads/main'` | 満たす |
| 3.1・3.2 起動の条件 | `paths` | **Missing**: `crates/pasta_sample_ghost/ghosts/hello-pasta/**` に広げる |
| 3.3〜3.5 | 今の起動 | 満たす（作り方を変えれば、どの起動でもそのコミットの中身になる） |
| 4.1〜4.5 新しい dll が要る辞書 | crates.io の `pasta_check`、`build-unix` の往復 | **Missing**。**Unknown**: 古い `pasta_check` がどこまで止められるか（4.3） |
| 5.1 入れた直後に古いものを返さない | `verify-tag.ps1`（タグは main の祖先） | 改行をそろえれば満たす |
| 5.2 古いコミットで公開しない | concurrency | **Missing**: main の最新との照合 |
| 5.3 nar の側を古くしない | `releases/latest` | 満たす |
| 6.1〜6.5 見比べ | `verifyLive`・`publish-ghost-site.ps1` | **Missing**: バイトの一致を、nar の側の行の一致に変える。nar の側の決め方は 1.4 と同じ規則にする |
| 7.1〜7.4 消えたファイル | — | **Missing**: `delete.txt` を作る処理。**Constraint**: 4.1 の矛盾、SSP は書いてある間消し続ける |
| 8.1〜8.4 | 今の構成 | 変えなければ満たす |
| 8.5 Windows・dll のビルドなし | — | Rust のビルド（`pasta_check`）は Windows でないので許される。ただし 8.6 とぶつかる |
| 8.6 時間 | 今は 40〜50 秒 | **Constraint**: cargo のビルドを素で足すと、数倍になる |
| 8.7 | — | 満たす（`tutorial-check` は改行を LF にそろえて比べるので、CRLF にしても影響しない） |
| 9.1〜9.5 手順書 | 2.8 の文書 | **Missing**。9.2 はスキルの編集の拒否に注意 |
| 10.1〜10.6 実機 | `ssp-ghost-test-launch` の手順（別ポートの別 SSP） | 手順はある。10.2 は改行をそろえないと通らない |

## 4. 見つけた問題

### 4.1 「リポジトリに在るか」だけでは分けられない

- 要件 1.4 の規則では、main で消したファイル（例: `dic/03-face.pasta` を消した）は「リポジトリに無い」ので、nar の側として nar から置かれる。
- 要件 7.1 は、消したファイルを置かないことを求める。2 つは同時に満たせない。
- 分け方の候補:
  - **A. リリースのタグのツリーに無いもの**を nar の側とする。タグは「どの nar のファイルがビルドで作られたか」をそのまま表す。最新のタグを `git fetch --depth=1` で取る必要がある（`CLASSIC_REF` の取り方と同じ）。ファイル名を決め打ちしない（要件 1.4 の意図を保つ）。
  - **B. `.gitignore` に当たるもの**（`git check-ignore`）を nar の側とする。今の 3 本は全部当たる。`pasta-dll-multiarch` で dll を足すときに `.gitignore` も足すのが前提になる。
  - **C. 今のコミットにもタグにも無いもの**を nar の側とする（A と同じ意味。タグを取る必要は残る）。
- 要件 1.4 の文言を、設計に入る前に直す必要がある。

### 4.2 改行コード

- 取り出し方の候補:
  - **A. `git -c core.autocrlf=true archive <コミット> -- <hello-pasta>`** で取り出す。v0.6.2 で 37 本全部の一致を確かめた（Windows の git で）。ubuntu の git でも同じになるかは、設計で 1 回確かめる。
  - **B. `.gitattributes` で hello-pasta の配下を `eol=crlf` にする。** どの OS の checkout でも CRLF になる。リリースの windows の checkout は今と同じ。ubuntu の `build.yml`・`cargo test` も CRLF の辞書を読むことになる（Windows では今もそう）。リポジトリ全体の決まりが 1 つ増える。
  - **C. 重ねるときに、文字のファイルを CRLF に直す。** 文字かどうかの判定を git と同じにするのが難しい。
- どれも採らないと、要件 10.2（`pasta.dll` 以外も、変わったファイルだけ）が通らない。

### 4.3 古い `pasta_check` で止められる範囲

- `pasta_check` の検査は、構文・参照・組み込みの名前（`validate/builtins.rs` に dll の名前を写してある）・行事の表などである。Lua を動かさない。
- 例: 13 段目の `＄行事`。`calendar.csv` を知らない古い `pasta_check` は、`＄行事` をただの変数と見て、通す見込みが高い。古い dll では値が空になるだけで、誤りにはならないかもしれない。
- 要件 4.1 の「読み込めるか」を `pasta_check` で近似するなら、止められない場合があることを手順書に書く必要がある。
- 実行時まで見るなら、最新のタグの `pasta_shiori` を Linux でビルドして OnBoot を往復させる。時間が大きく増える（2.7）。

### 4.4 nar の `updates.txt` が 2 か所にある

- nar には `updates.txt` と `ghost/master/updates.txt` がある。どちらもリポジトリに無いので、そのままだと「nar の側」として置かれる。
- 置き場所では、ルートの `updates.txt` を作り直す。`ghost/master/updates.txt` は、作り直して写すか、置かないかを決める（SSP が取りに来るのはルートだけ。`pasta_check release` は写す）。

### 4.5 `delete.txt` の性質

- `delete.txt` は書いてある間ずっと効く。後で同じ名前のファイルを足すと、足したファイルも消される（要件 7.3 の理由）。
- 「最新の Release の nar とだけ比べる」と、2 つ前の Release で入れた利用者には、1 つ前の Release で消したファイルの指示が届かない。比べる相手の範囲を決める必要がある。
- `delete.txt` は置き場所にだけ在るファイルになる。要件 6 の「リポジトリに無いファイル＝nar の側」の見比べから外す必要がある。
- `delete.txt` を作る場所: 置き場所の組み立てで作るか、リポジトリに手で置くか。

## 5. 作り方の案

### 案 A: `ghost-site.mjs` を広げる（Node だけ）

- `ghost-site.mjs build` に、リポジトリのファイルの入力（`git archive` の tar か、取り出したフォルダ）を足す。重ねる・`updates.txt` を作る・照合するを Node で行う。
- 新しい dll が要る辞書の検査は、別の段で `pasta_check validate`（最新の Release の版）を回す。
- `verifyLive` を行ごとの比べに変える。
- 良い点: 今の道具の続き。自己テストの枠がある。`manual.yml` の時間はほとんど増えない（検査の段を除く）。
- 悪い点: `updates.txt` の作り手が 2 つになる。要件 2.3 の一致テストに `pasta_check release` の出力が要る。そのテストを `manual.yml`（Node だけ）でどう回すか（Rust のテストに置く・生成済みの見本を置く・CI で両方を作って比べる）を決める必要がある。`ghost-site.mjs` は 346 行から増える。

### 案 B: 最新の Release の `pasta_check release` に作らせる

- ubuntu で、最新の Release の版の `pasta_check` を crates.io から入れる。重ねたフォルダに `pasta_check release --target <重ねた> --release <出力> --nar <捨てる>` を回す。
- 検査（要件 4）と `updates.txt` の作り直し（要件 2）が 1 回で済む。作り手は 1 つのまま（要件 2.3 が要らない）。
- `ghost-site.mjs build` は、できたフォルダ（または捨てる nar）を今のとおり照合して置く。
- 良い点: 作り方が 1 つ。nar と同じ版の作り手なので、書式のずれが起きない。
- 悪い点: cargo のビルドが入る。キャッシュ（版ごとの `actions/cache` など）が無いと、毎回 1〜2 分以上増える見込み（**Research Needed**）。`pasta_check release` は段 4 の誤りで止まるので、要件 4.2 の表示はそのまま使える。

### 案 C: 組み合わせ

- `updates.txt` は Node（案 A）で作る。検査は `pasta_check validate`（案 B の版）で回す。一致テストは `pasta_check` の Rust 側に「同じ一式なら同じ行」の見本を置く。
- 検査の段は、hello-pasta の配下か最新のタグが変わったときだけ重くなるように、キャッシュの鍵を版にする。
- 良い点: 速い経路と正しい経路を分けられる。
- 悪い点: 部品が一番多い。

### 共通の部分

- 起動の条件に `crates/pasta_sample_ghost/ghosts/hello-pasta/**` を足す（push・PR の両方）。
- 公開の前に、`github.sha` と main の最新（`git ls-remote origin refs/heads/main` など）を比べる段を置く（要件 5.2）。`contents: read` で足りる。
- 分け方の規則（4.1）と改行（4.2）は、どの案でも先に決める。
- `publish-ghost-site.ps1` の見比べを、行ごとの比べに変える。nar の側の決め方は、置き場所と同じ規則を使う（タグで checkout しているので、案 A の規則なら同じ情報が手元にある）。

## 6. 規模と危険度

- **規模: M（3〜7 日）**。既存の道具の延長だが、`manual.yml`・`ghost-site.mjs`・`release.yml` 側の見比べ・自己テスト 2 種・手順書 3 つ・実機確認と、触る所が多い。
- **危険度: Medium**。改行と分け方の規則は解き方が見えている。`delete.txt` の動きと、古い `pasta_check` で止められる範囲は、実機と検証で確かめる必要がある。置き場所は利用者がすでに見ているので、誤ると届いた辞書が巻き戻る。

## 7. 設計へ持ち越す調べもの

1. ubuntu の git で `git -c core.autocrlf=true archive` が Windows と同じ CRLF を出すか（または `.gitattributes` で済ませるか）。
2. `cargo install pasta_check --version X.Y.Z --locked` の ubuntu での所要と、キャッシュの効き方。
3. 古い版の `pasta_check validate` が、新しい機能の辞書（未知の組み込みの呼び出し・未知の `pasta.toml` のキー・`＄行事`）を誤りにするか、警告にするか、通すか。
4. SSP が `ghost/master/updates.txt` を読むか（置き場所で作り直すか、置かないかの判断）。
5. SSP の `delete.txt` の実際の動き（`\` 区切り・`charset` 行・`updates.txt` に載せる要否・消した後に同じ名前を足したとき）。ローカルの置き場所で確かめる。
6. `updates.txt` の `date=` を SSP が使うか（使わないなら、作り直しの日付は何でもよい）。

## 8. 議論に出す論点（要件ディスカッション向け）

1. 分け方の規則（4.1）: 要件 1.4 を「最新の Release のタグのツリーに無いもの」「`.gitignore` に当たるもの」のどちらに直すか。
2. 改行（4.2）: `git archive` の取り出しで CRLF にそろえるか、`.gitattributes` で決めるか。要件に「nar と同じ改行で置く」を足すか。
3. `updates.txt` の作り手（5 章）: `pasta_check release` に作らせる（案 B）か、Node で作って一致テストを置く（案 A・C）か。
4. 新しい dll が要る辞書の検査の深さ（4.3）: 最新の Release の `pasta_check` で足りるとするか、実行時の往復まで見るか。止められない場合を手順書に書くか。
5. 検査で落ちたときの扱い（要件 4.2・OPEN QUESTION 1）: 公開ごと止めると、次のリリースまでマニュアルの公開も止まる。開発者は CI を待たずにマージするので、main が赤のまま続きうる。
6. 古いコミットでの公開（要件 5.2・OPEN QUESTION 2）: 失敗にするか、成功のまま公開を飛ばすか。
7. リリース CI の見比べ（要件 6・OPEN QUESTION 3）: nar の側の行だけを比べる。リポジトリの側を比べないことでよいか。`delete.txt` を外すことを書き足すか。
8. 消えたファイル（要件 7・OPEN QUESTION 4）: `delete.txt` を出すか。比べる相手を最新の Release だけにするか、過去の Release も含めるか。手で置くか、組み立てで作るか。
9. 時間（要件 8.6・OPEN QUESTION 5）: 今は 40〜50 秒。増えてよい上限をいくつにするか。キャッシュを前提にしてよいか。
10. `ghost/master/updates.txt`（4.4）: 置き場所で作り直して写すか、置かないか。
11. 要件 9.2（`release` スキルの編集）: 分類器が拒否したときに、開発者に許可を求める流れでよいか。

## 9. 要件ディスカッションでの振り分け（2026-11-19）

- **要件へ直したもの（自明な修正）**: 8 章の 1（分け方はタグのツリーで決める）・2 のうち「nar と同じ改行で置く」（要件 1.8）・7（見比べは nar の側の行だけ）・10 のうち「nar の `updates.txt` を nar の側として置かない」、要件 5.2 の「失敗にせず飛ばす」と取りこぼさないこと（5.4）。
- **設計フェーズで決めるもの**: 改行のそろえ方（`git archive` か `.gitattributes`）・`updates.txt` の作り手（5 章の案 A〜C）・`ghost/master/updates.txt` を置くか・古いコミットの見分け方と取りこぼさない仕組み・cargo のキャッシュ・7 章の調べもの 1〜6。
- **開発者の議題にしたもの**: 新しい dll が要る辞書の検査と、落ちたときの扱い（8 章の 4・5・9）・消えたファイル（8 章の 8）。
- **議題にしないもの**: 8 章の 11（`release` スキルの編集）。手順の話で要件ではない。拒否されたら、そのときに開発者の許可を求める。

## 10. 議題の確定（2026-11-19・開発者が推奨どおり承認）

- **議題 1（新しい dll が要る辞書）**: 最新の Release の版の `pasta_check` で、重ねた中身を検査する（crates.io から入れ、版ごとにキャッシュ）。誤りなら公開ごと止める。PR でも失敗にする。キャッシュが効くときの増分は 1 分以内。Lua を動かさない検査で止められない違いは、手順書に書く（要件 4・8.6）。
- **議題 2（消えたファイル）**: `delete.txt` は出さない。`hello-pasta-network-update` の決定のまま、手順書の注意にとどめる（要件 7）。

## 11. 設計フェーズの調べもの（2026-11-19・`/kiro-spec-design`）

- 調べ方: 既存の仕組みの拡張なので、軽い調べ（`design-discovery-light`）。使ったスキル: `kiro-spec-design` の規則（設計の原則・軽い調べ・合成・レビューの関門）。サブエージェントは使っていない。

### 11.1 改行（7 章の 1）

- `git -c core.eol=lf -c core.autocrlf=true archive v0.6.2 -- crates/pasta_sample_ghost/ghosts/hello-pasta/` で取り出すと、公開中の `updates.txt` の 37 行の md5 と全部一致した（Linux の既定 `core.eol=lf` を真似た実測。git 2.56 の Windows 版）。
- 同じ取り出しを `core.autocrlf=false` にすると、19 本が合わない。
- git の文書では、`core.autocrlf=true` は「全部のファイルに `text=auto` を付け、`core.eol=crlf` にするのと同じ」である。OS の既定の改行に依らない。ubuntu の git でも同じ CRLF になる見込みが高い。ubuntu での一致は、実装の PR の run で確かめる。
- `.gitattributes` の案は採らない。リポジトリ全体の決まりが増え、ubuntu の `cargo test` の辞書も CRLF になる。取り出し方 1 つで足りる。
- 除外の 3 つの名前は、GNU `tar` の `--exclude` で外せる。`git archive` の `:(exclude,glob)**/…` の pathspec でも外せることを確かめた。

### 11.2 `updates.txt` の作り手（7 章・5 章の案）

- `pasta_check release` の引数は `--target`・`--release`・`--nar`・`--copy`（0 回以上、後勝ち）。v0.6.2 のタグと同じ引数である（`git diff v0.6.2` で `pasta_check/src` の差は無い）。
- 段 2 で `--target` を写し、段 3 で `--copy` を重ね、段 4 で `ghost/master` の辞書を検査し、段 5 で `updates.txt` を作ってルートと `ghost/master/` に置き、段 6 で nar を作る。
- `pasta.dll` が無くても動く（段 4 は Lua を動かさない）。
- v0.6.2 の取り出し（37 本）で動かすと 0.5 秒で終わった。できた `updates.txt` の 37 行は、日付の欄を除いて、公開中の行と全部同じだった（足りないのは nar の側の 3 本だけ）。
- 段 6 の nar を、今の `ghost-site.mjs build` にそのまま渡せた（40 ファイル＝37＋`updates.txt` 2 本＋`index.html`）。
- 今の main の hello-pasta も、v0.6.2 と同じ検査の規則（`pasta_dsl`・`pasta_core`・`pasta_check` の src に v0.6.2 からの差が無い）で、誤り 0 件で通った。
- crates.io の `pasta_check` は 0.6.2 まで出ている。依存は 63 個で、Lua に依存しない。
- 結論: 案 B（最新 Release の `pasta_check release` に作らせる）。`--target` にリポジトリの側、`--copy` に nar の側を渡すと、重ねる・検査する・作り直すが 1 回で済む。作り手は 1 つのままなので、要件 2.3 の一致のテストは要らない。

### 11.3 `ghost/master/updates.txt`（7 章の 4）

- UKADOC「ネットワーク更新への対応」: SSP は `homeurl` の直下の `updates2.dau`（または `updates.txt`）を取る。
- UKADOC「生成処理」: SSP 自身も、作った更新ファイルを `ghost\master` にも写す。`pasta_check release` も同じく写す。
- 置き場所の `ghost/master/updates.txt` は `updates.txt` に載らないので、利用者には届かない。今の置き場所にも在る。
- 結論: `pasta_check release` が写したもの（作り直した中身）をそのまま置く。新しい処理は要らない。

### 11.4 `date=`（7 章の 6）

- UKADOC「フィールド詳細」: `date=` は省略できる拡張の欄。「概要」は、サーバーの定義とローカルを比べて差分を取ると書き、比べるのは md5 である。`date=` を比べに使うとは書いていない。
- 結論: 日付の欄は何でもよいと仮定する。実機確認（要件 10.2）で確かめる。

### 11.5 古い run の見分け（5.2・5.4）

- `manual.yml` の concurrency は、main では 1 つの組 `pages`・`cancel-in-progress: false`。走っている run は止めず、待っている run は 1 つだけ残る（新しいものが古い待ちを取り消す）。
- だから、main の先頭までに起動の条件のファイルを変えたコミットがあれば、その run は、この run の後に走るか、すでに終わっている。
- 案 A（採った）: 組み立ての後に、最新の tag と main の先頭を見る。tag が変わった・main の先頭までに起動の条件のファイルが変わった、なら公開を飛ばし、成功で終える。起動の条件は `manual.yml` から `yq` で読む（2 か所に書かない）。`contents: read` で足りる。
- 案 B（退けた）: main の run は、いつも main の先頭を checkout する。古い run の再実行で、古い `manual.yml` の段と新しい道具が組み合わさり、失敗しうる。
- 案 C（退けた）: 飛ばした後に `manual.yml` を起動し直す。`actions: write` が要り、要件 8.3 の権限の形を崩す。
- 案 D（退けた）: 新しい run があるかを Actions の API で見る。`actions: read` が要り、再実行のときの判定が込み入る。

### 11.6 `pasta_check` のキャッシュ（8.6）

- リポジトリのワークフローは、まだ `actions/cache` を使っていない（`Swatinem/rust-cache` は使っている）。`pasta_check` の実行ファイル 1 つを残すだけなので、`actions/cache` で足りる。
- 鍵は OS と版。`cargo install --root ~/pasta-check` で入れて、そのフォルダを残す。
- リリース CI は `publish-crates` の後に Release を作る（`github-release` は `publish-crates` を待つ）。だから Release が最新になったとき、その版は crates.io に在る。

### 11.7 リリース CI の見比べ（要件 6）

- `publish-ghost-site` はタグで checkout しているので、タグのツリーは手元に在る。`git ls-tree -r --name-only <tag>:crates/pasta_sample_ghost/ghosts/hello-pasta` で、hello-pasta のフォルダからの相対の名前が取れる。
- 「nar の側」の決め方を `ghost-site.mjs` に置けば、置き場所の組み立てと見比べで同じ規則を使える。`release.yml` は変えずに済む（`pasta-dll-multiarch` と重ならない）。

## 12. 設計の判断（合成）

- **まとめた**: 「nar の側」の規則は、置き場所の組み立て（要件 1.4）と見比べ（要件 6.1）の 2 つで要る。`ghost-site.mjs` の `narOnlyFiles` 1 つにした。
- **使ったもの**: 重ねる・検査する・作り直すは `pasta_check release` の `--target`・`--copy`。照合と展開は今の `buildSite`。改行は `git archive` の `core.autocrlf`。古い run は concurrency と `git diff`。
- **作るもの**: `nar-only`（nar の側を書き出す CLI）・`listTree`・`verify-live` の比べ方・台本 `assemble-site.sh`。台本は、CI と手元（要件 10.4）で同じ手順を使うために置く。
- **削ったもの**: Node で `updates.txt` を作る処理と、その一致のテスト（案 A・C）。`delete.txt`。
- **残した危険**: リリースの直後の短い間に、nar の側が 1 つ前の Release のままであること（今の作りと同じ）。`date=` の扱い（実機で確かめる）。

## 13. 設計の議題の確定（2026-11-19・開発者が推奨どおり承認）

- **議題 1（リリースの直後の数分）**: 要件 5.1 の例外として残し、手順書に書く。`release.yml` は変えない（`pasta-dll-multiarch` と重ねない）。キャッシュの取り戻しと保存を分けて、間が延びないようにする。

## 14. 実機確認

### 14.1 SSP は `updates.txt` の日付の欄を比べに使わない（2026-11-19・タスク 1）

- 材料: v0.6.2 の Release の添付 `hello-pasta.nar`（3,670,331 バイト・sha256 `ac8fe3a6078d6774c81abd6ea4f527fa376349d18db4ee7d2ecc3ff997beefa8`）。中の `descript.txt` は `homeurl,https://ekicyou.github.io/pasta/hello-pasta/`。公開中の `https://ekicyou.github.io/pasta/hello-pasta/updates.txt`（41 行・4,161 バイト）は nar のルートの `updates.txt` とバイト一致（`updates2.dau` は 404）。
- 置き場所の写し: 公開中の `updates.txt` に載る 40 ファイルを公開サイトから取り、md5 がすべて一覧と一致することを確かめた。`updates.txt` だけ、各行の `date=…` を `date=2031-01-02T03:04:05` に置き換えた（改行 CRLF・大きさ 4,161 バイトのまま。`file,`・md5・`size=` の欄は元と同じことを `diff` で確かめた）。要求をすべて記録する小さな node:http の静的サーバーで `127.0.0.1:8761/hello-pasta/` から配信した。
- SSP: `C:\wintools\ssp` を作業ツリーの無視される `target/ssp-t1` に複製し（`ghost/`・`plugin/`・`temp/` は空）、`ssp.exe --option bootunlock,standalone --sstp-listen 9802 --ghost <複製の ghost/hello-pasta-t1>` で立てた。ゴーストは v0.6.2 の nar をそのまま展開し、`descript.txt` の `name`（`hello-pasta-t1`）と `homeurl`（`http://127.0.0.1:8761/hello-pasta/`）だけを書き換え、`02-talk.pasta` の末尾に `＊更新テスト`（`\![updatebymyself]` を出すだけ）を足した。手元のルートの `updates.txt` は公開中のもの（元の日付）と同じ。
- 更新の指示: SSTP の NOTIFY `OnPastaCallFunction`（Reference0 = `更新テスト`）。2 回行った（2 回目はゴーストを nar から作り直し、`＊OnUpdateBegin`・`＊OnUpdateReady`・`＊OnUpdateComplete`・`＊OnUpdateFailure` に Reference を返すシーンを足した。pasta は 204 の応答をログに書かないため）。
- サーバーの記録（2 回とも同じ。2 回目の抜き書き）:

  ```text
  2026-11-19T06:14:44.728Z GET /hello-pasta/updates2.dau 404 0 UA=SSP/2.9.36 (20261119-3; Windows NT 10.0.26300)
  2026-11-19T06:14:44.761Z GET /hello-pasta/updates.txt 200 4161 UA=SSP/2.9.36 (20261119-3; Windows NT 10.0.26300)
  ```

  これ以外の要求は 0 件。日付の違う 40 行のファイルを 1 本も取りに来なかった。
- ゴーストに届いた更新のイベント（2 回目の `pasta.log` の抜き書き）: `OnUpdateBegin`（Reference3 `ghost`・Reference4 `script`）→ `OnUpdateComplete`（**Reference0 `none`**・Reference1 空）。`OnUpdateReady` は来なかった。`ERROR` は 0 件。
- 更新後のフォルダ: 手元のルートの `updates.txt` はサーバーのもの（日付 2031）に置き換わった。nar の一覧と md5 を照合すると、違うのは手で書き換えた `descript.txt`・`02-talk.pasta` の 2 つだけ（上書きされていない＝取っていない）。`pasta.dll` などはそのまま。
- 結論: **`No Update`（OnUpdateComplete の Reference0 が `none`）で、ファイルは 1 本も取らなかった**。SSP は `updates.txt` の行を日付の欄で比べない（md5 で比べる）。`date=` の時刻が版ごとに変わっても、中身の同じファイルは取り直されない（要件 1.6・10.2）。
- SSP 本体の更新ログの文字列（`No Update` の行）は読めなかった。SSP の MCP（HTTP の API、9821）は普段の SSP が持っていて、複製の SSP は API の口を開かなかったため。代わりに、SSP がゴーストへ送る `OnUpdateComplete` の `none` と、サーバーの要求の記録で確かめた。
- 片付け: 試験用の SSP はプロセスを指定して強制終了（WM_CLOSE・`ゴースト終了` は使っていない）。普段の SSP は動いたまま。サーバーは止め、SSP の複製は消した。
- 気づいたこと: 複製した `data/profile/app.dat` に、普段の SSP の保留中の自己更新（`SelfUpdate.Pending.ZipPath`。`C:\wintools\ssp\temp\cache\9\…zip`）が載っていたため、複製の SSP は最初の起動でその zip を使って自分を 2.9.36 に更新して終わった（2 回目の起動から普通に動いた）。その zip は元の `temp/cache` から消えた。普段の SSP の `ssp.exe` は変わっていない。次に複製するときは、`app.dat` の `SelfUpdate.Pending.*` の 2 行を消してから立てる。

### 14.2 `assemble-site.sh` で組んだ置き場所と公開中の v0.6.2 の一致（2026-11-19・タスク 2.3）

- 道具: `pasta_check` は `cargo install pasta_check --version 0.6.2 --locked --root <スクラッチ>` で入れた crates.io の 0.6.2 を `PASTA_CHECK` で渡した（手元の開発版は使っていない）。nar は 14.1 と同じ v0.6.2 の `hello-pasta.nar`、タグは手元の `refs/tags/v0.6.2`。
- タグのコミット: `assemble-site.sh 97db0132（v0.6.2） v0.6.2 <v0.6.2 の nar> <出力先>` は終了コード 0。nar の側は `ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt`・`ghost/master/scripts/README.md` の 3 本。`pasta_check release` は 37＋3 本を重ね、辞書 13 個で誤り 0 件、`updates.txt` 40 行を作った。
  - できた `updates.txt`（41 行）は、公開中の v0.6.2 の `updates.txt` と、各行の日付の欄（4 つ目の欄）を空にして `diff` すると全部の行で一致した（行の順・パス・md5・`size=`・CRLF とも）。違うのは日付の欄だけ（作り直した側は `2026-11-15T11:16:37` と、作業ツリーの時刻の `2026-11-19T06:28:25`）。
  - 置き場所の `ghost/master/updates.txt` はルートの `updates.txt` とバイト一致。`index.html` も置かれた。
- main の先頭: このブランチと origin/main の分かれ目 `fcc3ecd0`（＝ その時点の origin/main の先頭、#288）で組み立てると終了コード 0。md5 が v0.6.2 と違う行は 16 本で、`git diff --name-only v0.6.2 fcc3ecd0 -- crates/pasta_sample_ghost/ghosts/hello-pasta/` の 16 本（`ghost/master/descript.txt`・`dic/01-boot.pasta`〜`dic/13-calendar.pasta`・`pasta.toml`・`shell/master/descript.txt`）とちょうど同じ。増えた行・消えた行は無い。`pasta.dll` の行は md5 `e91d580fef3a33a269aef17a121ae7fd`・`size=3670016` のまま。
- 失敗の確かめ（どれも終了コード 1）: 引数の数の誤り、無いタグ（`nar-only` が git の文を添えて失敗）、無いコミット（`git archive` の失敗）、空でない出力先（`build` が失敗）、`PASTA_CHECK` が無いコマンド。一時のフォルダは毎回消えた。

### 14.3 古い run の見分けの段の 4 つの枝（2026-11-19・タスク 3.2）

- 作り: `manual.yml` の「Check freshness」の段の `run:` の本文を、そのまま一時のファイルに書き出して `bash` で動かした（本文に `${{ }}` は無く、`TAG`・`GITHUB_SHA`・`GITHUB_REPOSITORY`・`GH_TOKEN`・`GITHUB_OUTPUT`・`GITHUB_STEP_SUMMARY` は環境変数で与えた）。
- 一時の git リポジトリ: 元のリポジトリ（`manual.yml` の写し・`book/x.md`・`other.txt` を持つコミット A）を作り、`file://` から `--depth 1` で clone した作業ツリー（CI の浅い checkout と同じ形）で動かした。`GITHUB_SHA` は A。main を進めるときは元のリポジトリにコミットを足した。
- 偽の道具: `PATH` の先頭に、`gh api repos/o/r/releases/latest --jq .tag_name` に `FAKE_LATEST` を返す偽の `gh` を置いた。手元に `yq` が無いため、`yq '.on.push.paths[]' <file>` に `on.push.paths` の行を awk で返す偽の `yq` を置いた（本物の `yq` の式は、ブランチの run の「Show trigger paths (yq)」の段で確かめる）。
- 結果（すべて期待どおり）:

  | 場合 | 終了コード | `publish` | notice・summary |
  |------|-----------|-----------|-----------------|
  | 最新の tag が v0.0.1 から v0.0.2 に変わった | 0 | `false` | 「最新の Release が v0.0.1 から v0.0.2 に変わった…」 |
  | main の先頭が A のまま | 0 | `true` | なし |
  | main が進み、`other.txt` だけ変わった（差なし） | 0 | `true` | なし |
  | さらに `crates/a/b/README.md` を足した（`crates/*/README.md` の `*` は `/` をまたがないので差なし） | 0 | `true` | なし |
  | さらに `book/x.md` を変えた（差あり） | 0 | `false` | 「main が <先頭> まで進み、起動の条件のファイルが変わった…」 |
  | 偽の `gh` が失敗する | 1 | なし | なし（ジョブの失敗） |
  | `origin` を無い場所に替えて `git fetch` が失敗する | 128 | なし | なし（ジョブの失敗） |

- `git diff` に要るのは両方のコミットのツリーだけなので、浅い checkout に main の先頭を `--depth=1` で足すだけで比べられた。

### 14.4 ブランチの CI・検査で止まること・所要の時間（2026-11-19・タスク 5.1）

- ブランチの run（`gh workflow run manual.yml --ref <ブランチ>`）。どれも緑で、公開はされなかった（「Check freshness」「Upload Pages artifact」は skipped、`deploy-manual` は skipped）。

  | run | コミット | キャッシュ | build-manual の所要 | 主な段 |
  |-----|---------|-----------|--------------------|--------|
  | 44473058112 | 3.1 | 外れ | 71 秒（06:33:46→06:34:57） | Install・Save pasta_check が走った。置き場所の段は成功 |
  | 44473670154 | 3.2 | 当たり | 40 秒（06:38:46→06:39:26） | Install・Save は skipped。「Show trigger paths (yq)」で `yq v4.54.3` が在った |
  | 44476014020 | bb202a3a（この時点の先頭） | 当たり | 35 秒（06:58:05→06:58:40） | 同上。Resolve latest Release→Fetch→Restore→Build hello-pasta update site→yq の 5 段で 3 秒 |

- 前の作りの基準: 直前の main の run 5 本（44460339770・44453981500・44452775421・44447683401・44441227657）の build-manual は 28・33・30・31・27 秒（中央値 30 秒）。前の作りの置き場所の段（nar の取得と展開）は 1 秒。
- 増分: キャッシュが当たった run で **+5〜+10 秒**（35・40 秒 対 30 秒）。1 分以内を満たす（要件 8.6）。外れた run は +41 秒（`cargo install pasta_check` の分）。
- 検査で止まること（手元。0.6.2 の `pasta_check`・v0.6.2 の nar・タグ `v0.6.2`）。一時の変更は scratchpad の使い捨てのワークツリー（detached）でコミットし、`assemble-site.sh <そのコミット> v0.6.2 <nar> <出力先>` に渡した:
  - `pasta.toml` の `[loader]` に未知のキーを足した場合: 0.6.2 は誤りにしなかった（誤り 0 件で終了コード 0）。この書き方は止める例にならないため、構文の誤りに替えた。
  - `dic/02-talk.pasta` の末尾に `＊＊＊` の行を足した場合: 段 4 で止まり、ファイルと理由を出して終了コード 1。出力先は作られなかった（要件 4.2・4.4）。

    ```text
    [4/6] Validating dictionaries...
    ghost/master/dic/02-talk.pasta:12:2: 誤り: 構文エラー:   --> 12:2 | 12 | ＊＊＊ |  ^--- | = expected id or attr
    検査した辞書ファイル 13 個: 誤り 1 件、注意 0 件
    Error: dictionary validation found 1 error(s); update files and NAR were not created
    ```

  - 注意だけの変更（無いシーンへの Call `＞存在しないシーン` を足した）: 段 4 で `注意: Call の飛び先「存在しないシーン」が辞書のシーンにありません…`（誤り 0 件、注意 1 件）を出して続け、終了コード 0（要件 4.5）。`updates.txt`（41 行）・`index.html`・`ghost/`・`shell/` が置かれた。
  - 片付け: 使い捨てのワークツリーと出力先を消した（`git worktree list` に残っていない。作業ツリーは clean）。
- `git diff origin/main -- .github/workflows/release.yml crates/pasta_check/` は空（要件 8.1・8.2）。
