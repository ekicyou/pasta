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
