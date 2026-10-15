# Research & Design Decisions: release-workflow

> 本書は、リリース CI に合わせて書き直した設計（2026-10-10）の調査と判断の記録である。手元でビルドして公開する旧設計（Stage A〜D・公開の 2 トラック・Resume・スケジュールによる再試行・マージコミット方式）の調査記録は、本書で置き換えた。旧版は git の履歴（コミット `7b0d6c99` より前）にある。

## Summary

- **Feature**: `release-workflow`
- **Discovery Scope**: Extension（完成したリリース CI の契約の上に、エージェントの手順を載せる。新しい依存は無い）
- **Key Findings**:
  - リリース CI の公開先ごとの `status`・`reason` は、job summary と job outputs にだけ書かれ、`gh` からは読めない。手元から読めるのは、step の結論と時刻（`gh run view --json jobs`）、ログ（公開の step が `status=...` の行を書く）、公開先そのものの状態である。
  - `Cargo.lock` には外部のクレート `fdeflate` が版 `0.3.7` で載っている。版の文字列の置換で `Cargo.lock` を更新すると壊す。`cargo update --workspace` は、ワークスペースの 7 クレートの `version` の行だけを変える。
  - `kiro-impl` はタスクごとに `tasks.md` を含むコミットを作る。そのままだと、版の更新のコミットに完了印が混ざり、次の実行で未完了のタスクが無くなる。実行の約束が要る。
  - リリースタグで起動した実行は、まだ 1 つも無い（v0.3.7 までは手元で公開した）。実行の特定とログの読み取りは、CI での初回のリリースが最初の実地確認になる。
  - Requirement 12 の一覧に無い古い参照が 3 つある（`kiro-complete` の SKILL.md・`RELEASE.md` の前提条件・`product.md` の現行バージョン）。

## Research Log

### 参照したスキルと規則

- `kiro-spec-design` の `rules/design-principles.md`（境界を先に決める・要件 ID は数字だけ）、`design-discovery-light.md`（既存の契約と統合点の確認）、`design-synthesis.md`（一般化・既存の採用・単純化）、`design-review-gate.md`（要件 ID の機械的な照合・境界の節・File Structure Plan）。
- 完了済みの `release-ci` の design.md（status 契約、Out of Boundary の申し送り）を、文体と契約の正として読んだ。

### リリース CI の結果を手元から読む方法

- **Context**: 要件 6.4 は、公開先ごとの結果（`published`・`skipped`・`failed`・`not-run`）と Release の URL の読み取りを求める。
- **Sources Consulted**: `.github/workflows/release.yml`、`.github/scripts/release/publish-crate.ps1`・`publish-vsix.ps1`・`github-release.ps1`、`gh run view --help`、実在の実行（`build.yml` の run 38010614540）。
- **Findings**:
  - 集約の step（`Summarize ...`）と report job は、`$GITHUB_OUTPUT` と `$GITHUB_STEP_SUMMARY` にだけ書く。ログには出さない。job summary を返す API は無い。
  - 3 つの公開スクリプトの `Complete` は、`$GITHUB_OUTPUT` があるとき `Write-Host "status=<値> [reason=<値>] [url=<値>]"` も行う。この行はログに残る。
  - スクリプトが `status` を書かずに終わった場合（認証の step の失敗・異常終了・時間切れ）、集約の step は step の結論から `failed`・`auth`／`failed`・`publish`／`not-run` を導く。この導き方は、`gh run view --json jobs` の step の結論から手元で再現できる。
  - `gh run view --json jobs` は、step ごとの `name`・`conclusion`・`startedAt`・`completedAt` を返す。所要時間は、ここから計算できる（summary に書かれる「所要 N 秒」は読めない）。
  - `gh run view --log` の 1 行は `<job 名><TAB><step 名><TAB><時刻> <内容>` の形である。
- **Implications**: 結果は「step の結論 + ログの `status=` の行」から組み立て、完了は公開先の実際の状態（定型コマンド A）で確かめる。区分を取り違えても再実行の回数が変わるだけになるよう、判定表を組む。`--failed` の再実行の後に、再実行しなかった job の step とログがどう見えるかは、実地で確かめる。

### タグで起動した実行の特定

- **Context**: 要件 6.1・6.2。タグを付け直すと、同じタグ名の実行が 2 つになる。
- **Findings**: `gh run list --workflow release.yml --json` は `headBranch`・`headSha`・`attempt`・`status`・`conclusion` を返す。タグの push で起動した実行の `headBranch` はタグ名になる（GitHub の仕様。本リポジトリでは未確認）。`gh run list --workflow release.yml` は、今は 0 件を返す。
- **Implications**: `headBranch` がタグ名で、`headSha` がタグのコミットと一致するもののうち、最も新しいものを採る。サーバー側の `--branch` の絞り込みに頼らず、手元で絞る。

### `Cargo.lock` と `package-lock.json` の更新

- **Context**: 要件 3.1・3.2。外部の依存の版を動かさずに、ワークスペースの版だけを更新する。
- **Findings**（開発機の写しで確かめた。cargo 1.99・npm 12）:
  - `cargo update --workspace` の後の差分は、`version = "0.3.7"` → `version = "0.3.8"` の 7 行だけ（`pasta_check`・`pasta_core`・`pasta_dsl`・`pasta_lsp`・`pasta_lua`・`pasta_sample_ghost`・`pasta_shiori`）。`--offline` の有無で結果は同じ。
  - cargo は `Cargo.lock` を LF で書き直す。作業ツリーは CRLF（`core.autocrlf=true`）だが、`git diff --numstat` は `7 7 Cargo.lock` になる。
  - `cargo metadata --locked --format-version 1` は、食い違いがあると終了コード 101（`cannot update the lock file ... because --locked was passed`）、無ければ 0。コンパイルしないので、`NoDefaultCurrentDirectoryInExePath` の影響を受けない。
  - `npm --prefix editors/vscode version 0.3.8 --no-git-tag-version` は、`package.json` の 5 行目と、`package-lock.json` の 3 行目・9 行目だけを変える。
- **Implications**: `Cargo.lock` は `cargo update --workspace`、npm の 2 ファイルは `npm version` で更新する。確認は `git diff --numstat` の行数と、`Cargo.lock` の変更行の形で行う。

### マニュアルの内容検証

- **Findings**: `node book/tools/verify-content.mjs` は、`book/src/introduction.md` の対象バージョン行と `Cargo.toml` の最初の `version = ` の行を照合する（検証 `F-version`）。ビルドも `npm ci` も要らず、作業ツリーを変えない。今の main で `RESULT: OK`。
- **Implications**: 要件 3.3 の (3) は、このコマンドの終了コードで判定する。

### main に無い内容の見分け方

- **Context**: 要件 2.3。squash で統合した後の作業ブランチは、コミットとしては main に無いが、内容は main にある。コミットの有無（`git log origin/main..HEAD`）で判定すると、同じワークツリーでの再開（8.1）が止まってしまう。
- **Findings**: `git merge-tree --write-tree origin/main HEAD` は、取り込んだ結果のツリーの ID を、作業ツリーも ref も変えずに返す（git 2.38 以上）。`origin/main^{tree}` と同じなら、作業ブランチは main に無い内容を持たない。要件を書いたブランチ（main に無い 1 コミットを持つ）で、違う ID になることを確かめた。
- **Implications**: 段 1 は内容で判定する。

### 公開先の照会

- **Findings**: crates.io API は User-Agent が無いと 403。`/api/v1/crates/<crate>` の `crate.max_version` と、`/api/v1/crates/<crate>/<version>` の 200・404 が使える。`vsce show ekicyou.pasta-vscode --json` の `versions[].version` が使える（開発機の vsce は 4.0.0。`NODE_OPTIONS=--dns-result-order=ipv4first` で ECONNRESET を避ける）。`gh release view vX.Y.Z --json isDraft,url,assets` は、無いとき `release not found` を返す。
- **Implications**: リリース CI のスクリプトと同じ読み方（200・404 だけを判定に使い、ほかは不明）にそろえる。

### 現在の状態（2026-10-10）

- `origin/main` の版は 0.3.7。タグ `v0.3.7` があり、5 クレート・Marketplace・GitHub Release（添付 3/3）がすべて公開済み。リリース CI の実行は無い。
- リポジトリは `squashMergeAllowed: true`、手元の `gh` の権限は `ADMIN`。
- 定型コマンド A は、`0.3.7` で「公開済み 7/7・提案 0.3.8」、`0.3.8` で「公開済み 0/7」を返した。

### 一回限りの整合の対象の現物

- `.claude/settings.json`: `permissions.allow` に `Bash(cargo publish:*)`・`PowerShell(vsce publish:*)`・`Bash(gh release create:*)` がある。`autoMode.allow` の 2 つ目の文が、マージコミット方式と手元からの公開を許可している。
- `.kiro/steering/workflow.md`: 「CI を待たない」の段落の最後の文（main の CI 全緑・`release-workflow` Task 1.1）と、「release タグ公開のカーブアウト」の段落（`gh pr merge --merge`）。
- 「名前の表」: `publish-vsix.ps1` の 102 行目、`release.yml` の 320 行目と 412 行目。手順書の見出しは「12. 名前の対応表」。
- 要件 12 の一覧に無い古い参照: `.claude/skills/kiro-complete/SKILL.md` の 321 行目（「main の CI 全緑」は `release-workflow` が課す）、`RELEASE.md` の前提条件（`main` へ push）、`product.md` の「現行バージョン v0.2.4」。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 状態を読んで判定表で進む直列の手順 | 進み具合を持たず、毎回 main・タグ・公開先・実行を読み直す | 会話が切れても再開できる。記録と実際がずれない | 照会のコマンドが長くなる | 採用。要件 8.4・10.3 がこの形を求める |
| 進み具合を spec のファイルに書く | 段ごとに完了を記録し、続きを記録から決める | 照会が少ない | 記録をコミットすると版の更新に混ざる。記録と実際がずれる | 不採用（8.4 に反する） |
| 手順をスクリプトやワークフローにまとめる | 版の更新からタグまでを 1 つのスクリプトにする | 実行が 1 行になる | 開発者の承認・判断の点がスクリプトの中に入る。新しい部品が増える | 不採用（要件に無い） |

## Design Decisions

### Decision: 結果は step の結論とログから組み立て、完了は公開先の状態で確かめる

- **Context**: job summary と job outputs を `gh` から読めない。
- **Alternatives Considered**:
  1. ログの `status=` の行だけで判定する — 行が無い場合（認証の失敗・異常終了）を扱えない
  2. 公開先の状態だけで判定する — `published` と `skipped` を区別できず、`reason` が分からない
  3. `release.yml` の report job に、ログへの出力を足す — `release-ci` の持ち場の変更になる
- **Selected Approach**: step の結論（最新の状態）を先に見て、`status`・`reason` の値をログの行で補う。完了は、実行の結論 `success` と、定型コマンド A の 7/7 の両方で判定する。
- **Rationale**: リリース CI を変えずに、要件 6.4〜6.6 と 9.1 を満たせる。取り違えの影響が再実行の回数に限られる。
- **Trade-offs**: ログの形（step 名・`status=` の行）に依存する。Revalidation Triggers に入れた。
- **Follow-up**: CI での初回のリリースで、設計の「確かめること」の 1〜4 を確かめる。

### Decision: `Cargo.lock` は `cargo update --workspace` で更新し、`build.yml` に `--locked` を足さない

- **Context**: 要件の未決事項 4。
- **Alternatives Considered**:
  1. `build.yml` の cargo に `--locked` を足す — 食い違いを PR の CI と関門で検出できるが、ふだんの PR の CI の挙動が変わる
  2. 足さず、版の更新の段で検出する
- **Selected Approach**: 2。段 4 の手順 1（更新の前）と手順 6（更新の後）で `cargo metadata --locked` を実行する。
- **Rationale**: 版の更新での更新し忘れも、main に前からある食い違いも、PR を作る前に検出できる。リリース CI の build の段の検査は、最後の守りとして残る。
- **Trade-offs**: リリース以外の PR が `Cargo.lock` を更新し忘れても、次のリリースの段 4 まで見つからない。

### Decision: タグは squash でできたコミットに付ける

- **Context**: 要件 5.1。今の `RELEASE.md` は `origin/main` の先頭に付けると書いている。
- **Selected Approach**: `gh pr view --json mergeCommit` の SHA に付ける。再開のときは、件名 `chore(release): vX.Y.Z` で探す。
- **Rationale**: 統合の直後に別の PR が入っても、その変更がリリースに混ざらない。
- **Trade-offs**: 作業ツリーがタグのコミットと同じとは限らないので、`verify-tag.ps1` をそのまま手元では使えない。同じ条件を `git show <SHA>:<path>` で確かめる（定型コマンド C）。

### Decision: 定型コマンドを本書に置き、補助スクリプトを足さない

- **Context**: 照会（約 100 行）と結果の読み取り（約 20 行）は、判定を含む定型の処理である。
- **Alternatives Considered**: spec の下か `.github/scripts/release/` にスクリプトとして置く。
- **Selected Approach**: 設計の Supporting References に置く。設計の時点で、本書の中の 3 つのブロックをそのまま取り出して実行し、動作を確かめた。
- **Trade-offs**: 設計書が長くなる（約 1100 行のうち約 140 行）。実行のたびに写す。Open Questions 2 に送った。

### Decision: `kiro-impl` との食い違いは「実行の約束」で埋める

- **Context**: `kiro-impl` は、タスクごとのサブエージェント・レビュー・`tasks.md` を含むコミット・最後の `/kiro-validate-impl` を行う。
- **Selected Approach**: 設計に約束を置き、タスク生成が `tasks.md` の冒頭に写す。完了印は作業ツリーの中だけで付け、開始時に戻す。
- **Rationale**: スキルを変えずに済む（スキルの編集は拒否されやすく、cc-sdd の更新で消える）。
- **Trade-offs**: 約束が守られることに頼る。Open Questions 1 に送った。

### Decision: 初回の記録は spec の下のファイルにする

- **Context**: 要件 11.5 と未決事項 6。
- **Selected Approach**: `.kiro/specs/release-workflow/first-ci-release.md` が `origin/main` にあることを記録とする。
- **Rationale**: 有無を 1 つのコマンド（`git ls-tree`）で判定できる。`spec.json` は kiro のコマンドが書き換えるので、独自の項目を置かない。

### Decision: Requirement 12 は書き直しと同じ PR で行う

- **Context**: `/kiro-impl` を使うとリリースが始まる。`tasks.md` は実行のたびに初期化される。
- **Selected Approach**: 設計に変更の一覧を置き、タスクの承認の後、開発者の直接の指示で行う。済んだことは `roadmap.md` のチェックで記録する。

### 統合（Synthesis）の結果

- **一般化**: 版の重複の検査・再開の判定・タグの付け直しの前の確認・完了の確認は、同じ「公開先とタグと実行の状態を読む」問題である。定型コマンド A の 1 つにまとめた。
- **既存の採用**: `verify-tag.ps1` の検査の条件、`verify-content.mjs`、`cargo update --workspace`、`npm version`、`gh run rerun --failed` を使う。新しい依存は無い。
- **単純化**: 進み具合のファイル・スケジュールによる再試行・補助スクリプト・公開の並行トラックを持たない。

## Risks & Mitigations

- リリース CI のログの形（step 名・`status=` の行）が変わると、結果を読めなくなる — Revalidation Triggers に入れた。読めないときは「区別できない」「不明」に倒れ、判定表は止まる側に進む。
- タグで起動した実行の `headBranch` がタグ名でない場合、実行を特定できない — CI での初回のリリースで確かめる。特定できなければ、段 7 が「起動していない」として止まる（公開は CI の中で進む）。
- `tasks.md` の完了印がコミットに混ざる — `git add` に 5 ファイルのパスを並べる。段 4 の手順 8 が、コミットの中身を確かめる。
- 設計書の中のコマンドを写し間違える — 出力の形（行の先頭の語）を固定し、判定表が読む値を限った。Open Questions 2。
- `.claude/settings.json` の編集が拒否される — 回避せず、変更を開発者に示す（一回限りの整合）。

## References

- `.github/workflows/release.yml`・`.github/scripts/release/*.ps1` — リリース CI の定義と status 契約の実装
- `.kiro/specs/completed/release-ci/design.md` — status 契約・Out of Boundary の申し送り
- `.github/release-ci-setup.md` — 一回限りのセットアップ（9 節: 初回のリリース、10 節: 必須の後片付け、11 節: 新しいクレート、12 節: 名前の対応表）
- `crates/pasta_sample_ghost/RELEASE.md` — 人が読むリリース手順書
- `.kiro/steering/workflow.md` — PR の squash マージ・CI を待たない運用
- `.kiro/steering/roadmap.md` 「リリース手順の書き換え（`release-ci` の後）」 — 申し送りの一覧
