# リリースの手順（段 1〜9）

`SKILL.md` の「段」が指す、実行するコマンド・合否の基準・判定表である。コマンドは、ここに書いてあるものを、波かっこ（`{V}` など）だけ置き換えて実行する。

かっこの中の番号（2.3 など）と、各段の表の `Requirements` は、もとになった要件（`.kiro/specs/completed/release-workflow/requirements.md`）の番号である。実行には使わない。

## 実行変数

段の間で受け渡す値。コマンドの中の `{V}` などの波かっこは、この値に置き換える。文脈を失ったら、右の方法で取り直す。

| 変数 | 意味 | 取り直し方 |
|------|------|------------|
| `VREQ` | 開発者が指定した版（無いこともある） | 開発者の指示 |
| `V_MAIN` | `origin/main` の `Cargo.toml` の版 | 定型コマンド A の `main` の行 |
| `V` | 今回リリースする版（`v` を付けない） | 段 3 の手順 5。再開のときは `V_MAIN` |
| `BRANCH` | 作業ブランチ | `git rev-parse --abbrev-ref HEAD` |
| `PR` | 版の更新の PR の番号 | `gh pr list --state all --head {BRANCH} --json number,title,state` |
| `TARGET` | タグを付けるコミットの SHA | 段 6 の手順 1 |
| `RUN` | リリース CI の実行の ID | 定型コマンド A の `run` の行 |
| `ATTEMPT` | 待つ対象の試行の番号（新しい実行は 1。再実行のたびに 1 増える） | 定型コマンド A の `run` の行の `attempt` |
| `N_AUTO` | この作業の中で行った自動の再実行の回数（初期値 0） | 取り直せない。再実行（`/release`）で 0 に戻る |
| `F7_DONE` | 判定表 F7 の切り分けの再実行を行ったか（初期値 いいえ） | 同上 |

## 段 1: 開始の確認

| Field | Detail |
|-------|--------|
| Intent | リリースと関係のない変更を巻き込まない状態から始める |
| Requirements | 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 8.6 |

手順 1 と 2 は何も変えない。どちらかで不合格なら、何も変えずに止まる。

1. **ブランチ**（2.1・2.2）

   ```bash
   git rev-parse --abbrev-ref HEAD
   ```

   出力が `main` か `HEAD` なら止まり、ハーネスのワークツリーでの再実行を求める。それ以外を `BRANCH` とする。

2. **必要な操作**（2.6・2.7）。4 つとも終了コード 0 で、2 つ目の出力が `ADMIN true`・`MAINTAIN true`・`WRITE true` のどれかであること。

   ```bash
   gh auth status
   gh repo view --json viewerPermission,squashMergeAllowed -q '.viewerPermission + " " + (.squashMergeAllowed|tostring)'
   gh run list --workflow release.yml --limit 1 --json databaseId
   git ls-remote --tags origin 'refs/tags/v*'
   ```

   不合格なら、できない操作（上から順に: GitHub の認証、PR の統合の権限と squash の許可、リリース CI の閲覧、リモートのタグの読み取り）を示して止まる。

3. **作業ツリー**（2.3）。`git status --porcelain` の出力が空で、`git merge-tree` の出力と `git rev-parse` の出力が同じであること。

   ```bash
   git fetch origin main
   git status --porcelain
   git merge-tree --write-tree origin/main HEAD
   git rev-parse 'origin/main^{tree}'
   ```

   `git merge-tree` は、作業ブランチを main に取り込んだ結果のツリーを、何も変えずに計算する。main のツリーと同じなら、作業ブランチは main に無い内容を持たない（squash で統合済みのコミットは、内容が同じなので通る）。不合格なら、下の「版の更新のコミットだけが残っている場合」にあたるかを確かめる。あたらなければ、次の出力を示して止まる。

   ```bash
   git status --short
   git log --oneline --no-merges origin/main..HEAD
   git diff --stat origin/main...HEAD
   ```

   **版の更新のコミットだけが残っている場合**（8.6。段 4 の後、PR を作る前に会話が終わった場合）。`git status --porcelain` の出力が空で、次の 2 つの出力がどちらも基準どおりなら、止まらずに続ける。

   ```bash
   git log --no-merges --format=%s origin/main..HEAD
   git diff --name-only origin/main...HEAD
   ```

   - 1 つ目が 1 行だけで、`chore(release): vX.Y.Z`（`X.Y.Z` は数字 3 つ）の形である。
   - 2 つ目が、段 4 の 6 ファイル（版の表記の 5 ファイルと、見どころのファイル `doc/release-notes/v{V}.md`）だけである。

   続け方: 件名の `X.Y.Z` を `V` として開発者に示す。`VREQ` があって `V` と違うときは、止まって開発者に確かめる。手順 4（main の取り込み）を行い、段 2 を飛ばして、段 3 の手順 4（重複の検査）と、段 4 の手順 6 の (3)(4)（`cargo metadata --locked` と `verify-content.mjs`）と、定型コマンド D（`-Version {V} -Head origin/main`）を行う。どれも合格（定型コマンド D は 5 行とも `合格`）なら、段 5 から続ける。不合格なら止まる。

4. **main の取り込み**（2.4・2.5）

   ```bash
   git merge-base --is-ancestor origin/main HEAD
   ```

   終了コード 0 なら何もしない。1 なら取り込む。

   ```bash
   git merge --no-edit origin/main
   ```

   取り込みが失敗したら `git merge --abort` を実行し、衝突したファイルを示して止まる。`git rebase`・`git reset` を使わない。

## 段 2: 状態の判定

| Field | Detail |
|-------|--------|
| Intent | 実際の状態から、どの段から始めるかを決める |
| Requirements | 1.7, 1.8, 8.1, 8.2, 8.4, 8.5 |

定型コマンド A を、版を指定せずに（`origin/main` の版を調べる）実行し、出力を次の表に上から当てる。最初に当てはまった行に従う。

| # | `tag` | `集計` の公開済み | `run` | ほかの条件 | 判定 | 次の手 |
|---|-------|-------------------|-------|------------|------|--------|
| S0 | — | — | — | `不明` が 1 以上、または `main` の 2 行の版が違う | 判定できない | 止まる。`不明` の行を示す（1.8） |
| S1 | あり | 7/7 | — | `VREQ` が `V_MAIN` と同じ | 指定された版はリリース済み | エラーを示し、別の版の入力を求める（1.7）。入力されたら段 3 の手順 3 へ |
| S3 | あり | 7/7 | — | 上のどれでもない | リリース済み（8.5） | 段 3 から始める |
| S4 | あり | 6/7 以下 | `queued`・`in_progress`・`waiting`・`requested`・`pending` | — | リリース CI が実行中（8.2） | `V` = `V_MAIN` を示し、段 7 の手順 2 から |
| S5 | あり | 6/7 以下 | `completed` | — | リリース CI が未完了で終わっている（8.2） | `V` = `V_MAIN` を示し、段 7 の手順 3 から |
| S6 | あり | 6/7 以下 | 無し | — | タグはあるが、リリース CI の実行が無い | 止まる。起動していないことを示す（6.2） |
| S7 | 無し | 0/7、かつ `GitHub Release` が `無し` | — | `VREQ` が無いか `V_MAIN` と同じ | 版の更新は main にあり、タグが無い（8.1） | `V` = `V_MAIN` を示し、段 6 から |
| S8 | 無し | 0/7、かつ `GitHub Release` が `無し` | — | `VREQ` が `V_MAIN` と違う | main に未リリースの版があるのに、別の版が指定された | 止まる。`V_MAIN` を先に出すか、飛ばすかを開発者に確かめる |
| S9 | 無し | 上以外 | — | — | タグが無いのに公開済みのものがある | 止まる。公開済みの行を示し、開発者に確かめる |

- S4・S5 で `VREQ` が `V_MAIN` と違うときは、`V_MAIN` のリリースが未完了なので先に続きを行うこと、`VREQ` は始めていないことを開発者に示す。
- 判定は毎回やり直す。前回の実行の会話の記録を使わない（8.4）。
- S2 は欠番である（CI での初回のリリースの確認のための行だった。確認は 2026-10-10 に済んだ）。

## 段 3: 版の決定

| Field | Detail |
|-------|--------|
| Intent | すでに出ている版と重ならない版を確定する |
| Requirements | 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 8.3 |

1. `VREQ` があれば、それを候補にして手順 3 へ進む（1.1）。
2. `VREQ` が無ければ、段 2 で得た定型コマンド A の出力（出どころごとの `最大` と、`提案` の行）を開発者に示し、`提案` の行の版を候補にする（1.2）。承認を求めない。開発者の答えを待たずに、手順 3 へ進む（1.3）。候補にするのは `提案` の行の版（PATCH + 1）だけで、MINOR・MAJOR を自動では上げない（1.4）。`提案` の行が `出せない` のときは、`不明` の行を示して止まる（1.8）。
3. 候補が次の形かを確かめる（1.5）。`False` ならエラーを示し、入力し直しを求める（1.6）。先頭の `v` は外してから確かめる。

   ```powershell
   '{候補}' -cmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\z'
   ```

4. 定型コマンド A を `-Version {候補}` で実行し、次のすべてを満たすことを確かめる。
   - `集計` の `不明` が 0（満たさなければ、`不明` の行を示して止まる。1.8）
   - `tag` が `無し`、`集計` の公開済みが 0/7、`GitHub Release` が `無し`（満たさなければ、すでにある出どころを示し、別の版の入力を求めて手順 3 へ戻る。1.7）
   - `PR` が `無し`（`未統合` の行があれば、新しい PR を作らず、その PR を示して開発者に扱いを確かめる。8.3）
5. 候補を `V` に確定し、開発者に示す。

## 段 4: 版の更新

| Field | Detail |
|-------|--------|
| Intent | 版の表記の 5 ファイルをそろえ、見どころを書き、1 つのコミットにする |
| Requirements | 3.1, 3.2, 3.3, 3.4, 3.5 |

`V_OLD` は更新の前の版（= `V_MAIN`）である。

1. **事前の確認**（まだ何も変えない）。出力が上から `6`・`1`・`1` で、4 つ目が終了コード 0 であること。違えば、何も変えずに止まる。

   ```bash
   grep -cF 'version = "{V_OLD}"' Cargo.toml
   grep -cF '| 対象 pasta バージョン | **v{V_OLD}** |' book/src/introduction.md
   grep -cF '"version": "{V_OLD}"' editors/vscode/package.json
   cargo metadata --locked --format-version 1 > /dev/null
   ```

   4 つ目が失敗するのは、版の更新より前から main の `Cargo.lock` が `Cargo.toml` と食い違っている場合である。別の PR で直す必要があることを報告する。

2. `Cargo.toml`: `version = "{V_OLD}"` を `version = "{V}"` に、Edit の全置換で置き換える（6 か所）。
3. `book/src/introduction.md`: `| 対象 pasta バージョン | **v{V_OLD}** |` を `| 対象 pasta バージョン | **v{V}** |` に置き換える（1 か所）。
4. `editors/vscode/package.json` と `package-lock.json`:

   ```bash
   npm --prefix editors/vscode version {V} --no-git-tag-version
   ```

5. `Cargo.lock`:

   ```bash
   cargo update --workspace
   ```

   ワークスペースのクレートの版だけを書き換える。外部のクレートの版は動かない。`Cargo.lock` を文字列の置換で直さない（外部のクレートに同じ版の文字列がある）。

**見どころを書く**（手順 5 の後、手順 6 の前に行う）。リリースの本文になる見どころのファイル `doc/release-notes/v{V}.md` を書く。書き方の決まりは `doc/release-notes/README.md` である。

- **H1 材料の一覧**。出力の最後の行が `**Full Changelog**: https://github.com/ekicyou/pasta/compare/v{V_OLD}...v{V}` であること。違えば、前のタグを読めていないので止まる。

  ```powershell
  & "$PWD/.github/scripts/release/release-notes.ps1" -Tag v{V} -Repo ekicyou/pasta -Head origin/main
  ```

- **H2 見どころのファイルを書く**。`doc/release-notes/README.md` の「書き方」に従い、H1 の一覧から見どころを選んで、`doc/release-notes/v{V}.md` を Write で作る。「動きが変わる変更」の見出しの下の項目は、すべて動きが変わる点として載せる。一覧の件名だけで足りないときは、`gh pr view {番号} --json title,body` と、マニュアル（`book/src/`）を読んでよい。読むだけである。
- **H3 確認**。定型コマンド D を `-Version {V} -Head origin/main` で実行する。5 行とも `合格` であること。合格なら、手順 6 へ進む。
- **H4 不合格のとき**。`不合格` の行の値（材料に無い番号・載っていない番号・見つかった spec 名など）を直して、H3 をやり直す。直すのは 3 回までである。3 回直しても `合格` にならなければ、手順 7（取り消し）を行い、`不合格` の行を示して止まる。H3 が合格するまで、手順 8（コミット）を行わない。

6. **確認**（3.2・3.3）。次の 5 つをすべて満たすこと。

   ```bash
   git diff --numstat
   git diff -U0 -- Cargo.lock | grep -E '^[+-][^+-]' | sort | uniq -c
   cargo metadata --locked --format-version 1 > /dev/null
   node book/tools/verify-content.mjs
   git status --porcelain
   ```

   | # | 確認 | 合格の基準 |
   |---|------|------------|
   | (1) | すべての表記が同じ版 | 1 つ目の出力が次の 5 行だけ: `N N Cargo.lock`・`6 6 Cargo.toml`・`1 1 book/src/introduction.md`・`2 2 editors/vscode/package-lock.json`・`1 1 editors/vscode/package.json`（`N` は 1 以上。今は 7） |
   | (2) | `Cargo.lock` の変更が版の行だけ | 2 つ目の出力が 2 行だけ: `N +version = "{V}"` と `N -version = "{V_OLD}"`（`N` は (1) と同じ） |
   | (3) | `Cargo.lock` が `Cargo.toml` と食い違わない | `cargo metadata --locked` が終了コード 0 |
   | (4) | マニュアルの内容検証 | `verify-content.mjs` が終了コード 0（`RESULT: OK`） |
   | (5) | 他のファイルを変えていない | `git status --porcelain` に出るのが、版の表記の 5 ファイルと、`?? doc/release-notes/v{V}.md` だけ |

7. **不合格のとき**（3.4）。6 ファイルを元に戻し、不合格の確認を示して止まる。

   ```bash
   git restore -- Cargo.toml Cargo.lock editors/vscode/package.json editors/vscode/package-lock.json book/src/introduction.md
   git clean -f -- doc/release-notes/v{V}.md
   ```

8. **コミット**（3.5）

   ```bash
   git add Cargo.toml Cargo.lock editors/vscode/package.json editors/vscode/package-lock.json book/src/introduction.md doc/release-notes/v{V}.md
   git commit -m "chore(release): v{V}"
   git show --stat --format=%s HEAD
   ```

   最後の出力が、件名 `chore(release): v{V}` と 6 ファイルであること。その後、`doc/release-notes/v{V}.md` の全文を開発者に示す。承認を求めない。開発者の答えを待たずに、段 5 へ進む。

**`build.yml` に `--locked` を足さない理由**: 版の更新での更新し忘れは手順 6 の (3) が、main に前からある食い違いは手順 1 が、PR を作る前に検出する。リリース CI の build の段の検査（`git diff --exit-code -- Cargo.lock`）は、そのまま最後の守りとして残る。`build.yml` を変えると、ふだんの PR の CI の挙動まで変わる。

## 段 5: main への統合

| Field | Detail |
|-------|--------|
| Intent | 版の更新を、ほかの変更と同じ PR の流れ（squash）で main に入れる |
| Requirements | 4.1, 4.2, 4.3, 4.4, 4.5 |

1. PR に入る変更が、段 4 の 6 ファイルだけであることを確かめてから、作業ブランチを push する（main ではない）。1 つ目の出力が 6 ファイルだけでなければ、push せずに止まる（ほかのファイルがコミットに混ざっている）。

   ```bash
   git diff --name-only origin/main...HEAD
   git push -u origin HEAD
   ```

2. PR を作る。

   ```bash
   gh pr create --base main --head {BRANCH} --title "chore(release): v{V}" --body "v{V} へ版を更新する。統合の後にリリースタグ v{V} を push し、リリース CI が公開する。"
   gh pr view {BRANCH} --json number -q .number
   ```

   2 つ目の出力を `PR` とする。

3. PR の CI を待たずに、squash で統合する。

   ```bash
   gh pr merge {PR} --squash --delete-branch --subject "chore(release): v{V}" --body "v{V} へ版を更新する。"
   gh pr view {PR} --json state,mergeCommit -q '.state + " " + (.mergeCommit.oid // "")'
   ```

   統合の成否は、2 つ目の出力だけで判定する。`MERGED <SHA>` なら成功で、`<SHA>` を `TARGET` とする。`gh pr merge` の出すローカルブランチの削除の警告（`'main' already used by worktree` など）は、失敗ではない。

   2 つ目のコマンドを実行できなかったとき（出力が得られなかったとき）だけ、代わりに次で判定する。2 つ目の出力が 1 行なら統合は成功で、その SHA を `TARGET` とする。空なら、統合を確かめられないので止まる。

   ```bash
   git fetch origin main
   git log origin/main -1 --format=%H -F --grep="chore(release): v{V}"
   ```

   squash でできたコミットの件名は `chore(release): v{V}` だけになり、PR の番号は付かない（v0.3.8 で確かめた）。段 6 の手順 1 の検索も、この件名に合う。

4. **失敗のとき**（4.4）。手順 1〜3 のどれかが失敗した、または手順 3 の出力が `MERGED` で始まらない（代わりの判定では、出力が空の）ときは、止まる。リリースタグを作らない。`git push --force`・履歴の書き換え・ブランチの削除を行わない。PR の URL と失敗の内容を示し、開発者に解消を求める。

5. **main の確認**（4.5）。`git fetch origin main` の後、定型コマンド C を `-Version {V} -Target {TARGET}` で実行する。5 行とも `合格` であること。不合格なら止まる。

## 段 6: リリースタグの作成と push

| Field | Detail |
|-------|--------|
| Intent | 版を上げた main のコミットに注釈付きのタグを付け、タグだけを push する |
| Requirements | 5.1, 5.2, 5.3, 5.4, 5.5 |

1. **`TARGET` の決定**（5.1）。段 5 から来たときは、段 5 の手順 3 の `TARGET` を使う。段 2 の S7 から来たとき（再開）は、次で求める。

   ```bash
   git fetch origin main
   git log origin/main -1 --format=%H -F --grep="chore(release): v{V}"
   ```

   出力が 1 行ならそれを `TARGET` とする。空なら止まり、版の更新のコミットが見つからないことと、`git log origin/main -5 --oneline -- Cargo.toml` の出力を示して、開発者にコミットを確かめる。

   **S7 で見どころが無いとき**。S7 から来て、手順 2 で定型コマンド C の 5 行目（見どころ）だけが `不合格` のときは、`git rev-parse origin/main` の出力を `TARGET` にして、定型コマンド C をやり直す。5 行とも `合格` なら、その `TARGET` で続ける（見どころのファイルが、後の PR で main に入っている）。なお `不合格` なら、リリースタグを作らずに止まり、`doc/release-notes/v{V}.md` を PR で main に入れることを開発者に求める。

2. **タグを付ける前の検査**（5.2）。定型コマンド C を `-Version {V} -Target {TARGET}` で実行する。5 行とも `合格` であること。不合格なら、タグを作らずに止まる。

3. **同名のタグ**（5.3）。2 つとも出力が空であること。

   ```bash
   git ls-remote --tags origin refs/tags/v{V}
   git tag -l v{V}
   ```

   - リモートにある（1 つ目が空でない）: エラーを示して止まり、開発者に扱いを確かめる。削除・付け替えをしない。
   - ローカルにだけある: `git rev-parse 'v{V}^{commit}'` が `TARGET` と同じなら、手順 4 を飛ばして手順 5 へ進む（前回の push の失敗の続き）。違えば、止まって開発者に確かめる。

4. **タグの作成**

   ```bash
   git tag -a v{V} -m "Release v{V}" {TARGET}
   ```

5. **タグだけを push**（5.4）

   ```bash
   git push origin v{V}
   git ls-remote --tags origin refs/tags/v{V}
   ```

   2 つ目の出力が空でなければ成功である。push が失敗したら（5.5）、失敗の内容と「main には版の更新が入っていて、タグだけが無い」ことを示して止まる。再実行すると、段 2 の S7 から続く。

## 段 7: リリース CI の結果の確認

| Field | Detail |
|-------|--------|
| Intent | タグで起動したリリース CI を最後まで追い、公開先ごとの結果を読む |
| Requirements | 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 9.1 |

1. **実行の特定**（6.1・6.2）。タグの名前とタグのコミットの両方が一致する実行を、最大 3 分待って探す。

   ```powershell
   $V = '{V}'; $sha = git rev-parse "v$V^{commit}"
   foreach ($i in 1..18) {
       $run = @((gh run list --workflow release.yml --limit 30 --json databaseId,headBranch,headSha,status,attempt,url,createdAt) -join "`n" | ConvertFrom-Json |
           Where-Object { $_.headBranch -eq "v$V" -and $_.headSha -eq $sha } | Sort-Object createdAt -Descending)
       if ($run.Count) { break }
       Start-Sleep -Seconds 10
   }
   if ($run.Count) { "run | $($run[0].databaseId) | $($run[0].status) | attempt $($run[0].attempt) | $($run[0].url)" } else { 'run | 無し' }
   ```

   `run | 無し` なら、完了と報告せず、リリース CI が起動していないことを示して止まる。それ以外は、ID を `RUN` とする。

2. **待機**（6.3）。次のコマンドを、ツールのタイムアウトを 10 分にして実行する。`{ATTEMPT}` は、待つ対象の試行の番号（最初は 1。再実行のたびに 1 増える）。出力が `completed` になるまで繰り返す。20 回（約 3 時間）繰り返しても `completed` にならなければ、「リリース CI が実行中」として止まる。

   ```powershell
   $RUN = '{RUN}'; $ATTEMPT = {ATTEMPT}
   foreach ($i in 1..17) {
       $s = (gh run view $RUN --json status,attempt) -join "`n" | ConvertFrom-Json
       if ($s.attempt -ge $ATTEMPT -and $s.status -eq 'completed') { break }
       Start-Sleep -Seconds 30
   }
   "run | $RUN | $(if ($s.attempt -ge $ATTEMPT) { $s.status } else { 'waiting' }) | attempt $($s.attempt)"
   ```

3. **読み取り**（6.4・6.6）。定型コマンド B を `-Run {RUN}` で実行し、下の「結果表の作り方」で、7 つの公開先（5 クレート・Marketplace・GitHub Release）の `status`・`reason` と、GitHub Release の URL（`url=` の値）を決める。

4. **完了の判定**（6.5・9.1）。次のすべてを満たせば完了で、段 9 へ進む。
   - 定型コマンド B の `run` の行の結論が `success`
   - 結果表の 7 つがすべて `published` か `skipped`
   - 定型コマンド A（`-Version {V}`）の `集計` が、公開済み 7/7・`不明` 0（`GitHub Release` の行の URL を、報告の URL とする）

   1 つ目と 2 つ目を満たすのに 3 つ目が 7/7 でないときは、1 分後に定型コマンド A を 1 度だけ実行し直す。それでも 7/7 でなければ、未完了として止まる。1 つ目か 2 つ目を満たさないときは、段 8 へ進む。

### 結果表の作り方

公開先ごとに、定型コマンド B の `step` の行（step の結論）と `line` の行（ログの `status=` の行）を、次の対応で見る。

| 公開先 | 認証の step | 公開の step |
|--------|-------------|-------------|
| crates.io `pasta_core`・`pasta_dsl`・`pasta_lua`・`pasta_shiori`・`pasta_check` | `Auth crates.io (<クレート>)` | `Publish <クレート>` |
| Marketplace `ekicyou.pasta-vscode` | `Azure login (OIDC)` | `Publish VSIX to Marketplace` |
| GitHub Release | なし | `Create GitHub Release` |

規則は上から当て、最初に当てはまったものを使う。リリース CI の集約の step（`Summarize ...`）と同じ読み方である。

| # | 条件 | `status` | `reason` |
|---|------|----------|----------|
| 1 | 公開の step の結論が `success` | その step の `line` のうち、attempt が最大のものの値（`published` か `skipped`）。`line` が無ければ「`published` か `skipped`（区別できない）」 | なし |
| 2 | 認証の step の結論が `failure` | `failed` | `auth` |
| 3 | 公開の step の結論が `failure`・`cancelled` | `failed` | その step の `line` のうち、attempt が最大のものの `reason=` の値。`line` が無ければ `publish` |
| 4 | 公開の step の結論が `skipped`、または `step` の行が無い | `not-run`。ただし、その job の結論が `failure`・`cancelled`・`timed_out` で、その job のどの公開先も `failed` でないときは、その job の先頭の公開先を `failed` にする（6.6） | 読み替えたものは `不明` |

- 「区別できない」は、完了の判定では `published` か `skipped` として扱う。報告には「区別できない」と書く。
- `not-run` は、上流の失敗の結果である。それ自体は対応の対象にしない。

## 段 8: 失敗への対応

| Field | Detail |
|-------|--------|
| Intent | 失敗の種類ごとに、再実行・案内・停止を判定表で決める |
| Requirements | 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9 |

段 7 の結果表と、定型コマンド B の `job` の行を、次の表に上から当てる。最初に当てはまった行に従う。「手順書」は `.github/release-ci-setup.md` である。

| # | 条件 | 区分 | 次の手 |
|---|------|------|--------|
| F0 | 実行の結論が `cancelled` | 取り消された | 再実行しない。取り消されていることを示して止まる |
| F1 | job `verify`・job `build`・名前が `gate` で始まる job のどれかが `failure`・`cancelled`・`timed_out` | 公開前の段の失敗（7.5） | 再実行しない。定型コマンド A で 0/7 を確かめ、失敗した job と step の名前、失敗のログの末尾、「どの公開先にも公開していない」を報告して止まる（9.4）。0/7 でないときは、「公開していない」と書かず、公開済みの行を示して止まる。続きは「公開前の段の失敗の続き」 |
| F3 | どれかが `failed`・`auth` | セットアップ（7.3） | 再実行しない。crates.io なら手順書 8 節と 12 節、Marketplace なら 3 節・7 節・12 節を示して止まる。続きは「セットアップを直した後の続き」 |
| F4 | どれかが `failed`・`not-registered` | セットアップ（7.3） | 再実行しない。手順書 11 節「新しいクレートの初回公開」を示して止まる。続きは F3 と同じ |
| F5 | GitHub Release が `failed`・`immutable` | 手作業（7.9） | 再実行しない。公開済みの Release に足りない配布物（定型コマンド A の `添付不足` の行）を、実行の artifact `release-assets` から手で添付する必要があることを案内して止まる |
| F6 | どれかが `failed`・`transient` | 一時的な失敗（7.1） | `N_AUTO` が 3 未満なら、失敗した job を再実行する。3 なら、自動の再実行をやめ、未完了として止まる（7.2） |
| F7 | どれかが `failed`・`publish`、または `failed`・`不明` | 切り分け | `F7_DONE` が「いいえ」で `N_AUTO` が 3 未満なら、`F7_DONE` を「はい」にして、失敗した job を 1 度だけ再実行する。そうでなければ「再実行では直らない失敗」 |

F2 は欠番である（CI での初回のリリースだけの行だった）。

**失敗した job の再実行**（F6・F7）

```bash
gh run rerun {RUN} --failed
```

終了コード 0 なら `N_AUTO` を 1 増やし、`{ATTEMPT}` を 1 増やして、段 7 の手順 2 へ戻る。再実行は、公開済みのものを飛ばす。

**失敗のログの末尾**（F1・F7 の報告に付ける）

```bash
gh run view {RUN} --log-failed | tail -n 60
```

**再実行では直らない失敗**（F7 の 2 度目）。定型コマンド A（`-Version {V}`）を実行する。

- 公開済みが 0/7 で、`GitHub Release` が `無し`: F1 と同じ報告をして止まる（何も公開していない。修正の後にタグを付け直せる）。
- それ以外: タグを付け直さない。公開済みの公開先と未公開の公開先を示し、版を上げて出し直す必要があることを報告して止まる（7.7）。

**セットアップを直した後の続き**（7.4）。開発者が設定を直したと伝えたら、同じ会話なら上の「失敗した job の再実行」から続ける。新しい会話なら、`/release` の再実行で段 2 の S5 に入り、同じ失敗が残っていれば、また F3・F4 で止まる。そのときは、開発者の「直した」という指示を受けて、失敗した job を再実行する。

**公開前の段の失敗の続き**（7.5・7.6）。開発者が次のどちらかを指示する。

- 「そのまま再実行」: 上の「失敗した job の再実行」を行う（`N_AUTO` は増やさない）。
- 「修正を main に入れた。タグを付け直してよい」: 下の「タグの付け直し」を行う。

**タグの付け直し**（7.6）。開発者の承認が明示されているときだけ行う。

1. 定型コマンド A（`-Version {V}`）で、公開済みが 0/7・`不明` が 0・`GitHub Release` が `無し` であることを確かめる。満たさなければ、付け直さずに止まる（7.7）。
2. `git fetch origin main` の後、`git rev-parse origin/main` の出力を `TARGET` とする。定型コマンド C を `-Version {V} -Target {TARGET}` で実行し、5 行とも `合格` であることを確かめる。
3. タグを付け直して push する。この 2 つは許可の一覧に入れない。実行のたびに、開発者が許可の確認に答える。

   ```bash
   git tag -f -a v{V} -m "Release v{V}" {TARGET}
   git push -f origin v{V}
   ```

4. 新しい実行が起動する。`{ATTEMPT}` を 1 に戻し、段 7 の手順 1 から続ける。

## 段 9: 完了の報告

| Field | Detail |
|-------|--------|
| Intent | 完了を、取り違えようのない形で報告する |
| Requirements | 9.1, 9.2 |

段 7 の手順 4 を満たしたときだけ、次の形で報告する。`<...>` は結果表と定型コマンド B の出力から埋める。

```
リリース完了: v{V}
- crates.io: pasta_core <status> / pasta_dsl <status> / pasta_lua <status> / pasta_shiori <status> / pasta_check <status>
- VSCode Marketplace（ekicyou.pasta-vscode）: <status>
- GitHub Release: <URL>
- リリース CI の実行: <URL>
- 再実行の回数: <最後の attempt から 1 を引いた数>
- 手順との食い違い: <なし / 手順どおりに動かなかった箇所>
```

## 定型コマンド A の出力（状態の照会）

1 行に 1 つの事実を、` | ` で区切って出す。値の語は固定である。

| 行 | 形 | 値 |
|----|----|----|
| `fetch` | `fetch \| origin/main \| <結果>` | `成功`・`不明` |
| `main` | `main \| Cargo.toml \| <版>` と `main \| package.json \| <版>` | 版・`不明` |
| `tag` | `tag \| 最大 <版> \| v{V} <状態>` | `あり <コミットの SHA>`・`無し`。照会できなければ `tag \| 不明` |
| `crates.io` | `crates.io \| <クレート> \| 最大 <版> \| {V} <状態>`（5 行） | `あり`・`無し`・`不明（HTTP <コード>）` |
| `Marketplace` | `Marketplace \| ekicyou.pasta-vscode \| 最大 <版> \| {V} <状態>` | `あり`・`無し`。照会できなければ `不明` |
| `GitHub Release` | `GitHub Release \| 最大 <版> \| v{V} <状態>` | `あり（公開・添付 3/3） <URL>`・`下書き（添付 n/3）`・`添付不足（公開・添付 n/3） <URL>`・`無し`・`不明` |
| `run` | `run \| <ID> \| <status> \| <conclusion> \| attempt <n> \| <URL>` | 無ければ `run \| 無し`。照会できなければ `run \| 不明` |
| `PR` | `PR \| 未統合 \| #<番号> \| <ブランチ> \| <URL>` | 無ければ `PR \| 無し` |
| `集計` | `集計 \| {V} の公開済み <n>/7 \| 不明 <m>` | 7 = 5 クレート + Marketplace + GitHub Release |
| `提案` | `提案 \| <版>（すべての出どころの最大 <版> の PATCH + 1）` | `不明` があれば `提案 \| 出せない（不明がある）` |

「公開済み」に数えるのは、crates.io と Marketplace の `あり`、GitHub Release の `あり（公開・添付 3/3）` だけである。下書きと添付不足は数えない。

## 定型コマンド B の出力（リリース CI の結果）

| 行 | 形 |
|----|----|
| `run` | `run \| <ID> \| <status> \| <conclusion> \| attempt <n> \| <URL>` |
| `job` | `job \| <job 名> \| <結論>`（`gate` の中の job は `gate / <名前>`） |
| `step` | `step \| <job 名> \| <step 名> \| <結論> \| <所要> 秒`（`Publish pasta_*` には ` \| 期限まで <秒> 秒` が付く） |
| `line` | `line \| attempt <n> \| <job 名> \| <step 名> \| status=<値> [reason=<値>] [url=<値>]` |

## 定型コマンドの呼び出し方

定型コマンドは、`.claude/skills/release/scripts/` のスクリプトである。4 本とも読むだけで、何も変えない。リポジトリのルートを作業ディレクトリにして、PowerShell ツールで次の 1 行をそのまま実行する（波かっこだけを置き換える）。スクリプトの中身を写して実行しない。

| 定型コマンド | 呼び出し | 出力の形 |
|--------------|----------|----------|
| A: 状態の照会 | `& "$PWD/.claude/skills/release/scripts/release-state.ps1" -Version {V}` | 上の「定型コマンド A の出力」 |
| A（`origin/main` の版を調べる） | `& "$PWD/.claude/skills/release/scripts/release-state.ps1"` | 同上 |
| B: リリース CI の結果の読み取り | `& "$PWD/.claude/skills/release/scripts/ci-result.ps1" -Run {RUN}` | 上の「定型コマンド B の出力」 |
| C: タグを付ける前の検査 | `& "$PWD/.claude/skills/release/scripts/pretag-check.ps1" -Version {V} -Target {TARGET}` | 5 行（タグの形・`Cargo.toml` の版・`package.json` の版・main からの到達・見どころ）。それぞれ `合格` か `不合格` |
| D: 見どころの確認 | `& "$PWD/.claude/skills/release/scripts/highlights-check.ps1" -Version {V} -Head origin/main` | 5 行（見どころのファイル・Claudia の出典リンク・PR の番号・動きが変わる点・spec 名）。それぞれ `合格` か `不合格` |

- A は、crates.io の照会に User-Agent を付け、Marketplace の照会（`vsce show`）で IPv4 を優先する。200・404 以外の応答と、コマンドの失敗を `不明` にする。
- B は、実行が `completed` になってから使う。
- C は、`.github/scripts/release/verify-tag.ps1` と同じ 5 つの条件を、タグを付けるコミットの中身（`git show`）で確かめる。先に `git fetch origin main` を済ませる。
- D は、作業ディレクトリの `doc/release-notes/v{V}.md` を、`.github/scripts/release/release-notes.ps1` の出力（材料）と `.kiro/specs/` のフォルダの名前に照らして確かめる。タグの在る版を確かめるときは、`-Head origin/main` を省く。
- 段 7 の手順 1・2 の待機のコマンドは短いので、段 7 に置いたままにする。
