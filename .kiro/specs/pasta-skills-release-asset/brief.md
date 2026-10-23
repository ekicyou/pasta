# Brief: pasta-skills-release-asset

2026-10-23 の discovery で起票した（開発者の依頼「リリース時に、pasta 関係のスキルを zip にしたファイルを追加してください」）。同じ日に、議案を推奨どおりに承認し、0.4.2 に入れると決めた（開発者の決定）。先例は `pasta-check-release-asset`（完了。`pasta_check.zip` を 4 つ目の配布物にした）。

## Problem

ゴーストの作者が、Claude などの AI にゴーストを書かせるときに使う pasta のスキル（`pasta-ghost-authoring`・`pasta-lua-coding`・`pasta-check`）は、このリポジトリの `.claude/skills/` にしか無い。使うには、リポジトリをクローンして、3 つのフォルダを探して写す必要がある。どの版の pasta に合ったスキルなのかも分かりにくい。

## Current State

- リリース CI（`.github/workflows/release.yml`）は、タグの push で配布物 4 つ（`pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip`・`pasta-vscode-X.Y.Z.vsix`）と、詳しいリリースノートを GitHub Release に添付する（添付は 5 つ）。
- 配布物の zip は、見本ゴーストのビルドの手順 `crates/pasta_sample_ghost/release.ps1`（7 段。331 行）が作る。手元の `release.bat` とリリース CI は、同じ一式を作る。
- `release.yml` の「Verify release assets」（131〜200 行）が、配布物ごとに中身を確かめる。「Upload release-assets」（203〜214 行）が、配布物を後の job へ渡す。
- `.github/scripts/release/github-release.ps1` は、添付の 5 つの名前を決め打ちで持つ（72 行）。全部がそろわないと公開しない。`release-scripts-test.ps1` の項目 10 が、その検査を試す。
- `release` スキルの `scripts/release-state.ps1`（84 行）と `procedure.md`（477 行）が、GitHub Release の添付を「4/4」で数える。
- 3 つのスキルの `references/` は、マニュアルから `node book/tools/gen-skill-refs.mjs` で生成している。CI の `--check` が鮮度を見るので、タグの時点の中身はマニュアルと合っている。
- スキルの中身は、このリポジトリで書いた文書だけで、ほかの人のものは入っていない。リポジトリのライセンスは MIT（ルートの `LICENSE`）。
- マニュアルの `book/src/introduction.md`（38 行）の表が、リリースページの配布物を固定のリンクで並べている。

## Desired Outcome

- 各版の GitHub Release に、`pasta-skills.zip` が 5 つ目の配布物として載る（添付は 6 つ）。固定のリンク `https://github.com/ekicyou/pasta/releases/latest/download/pasta-skills.zip` で最新版を落とせる。
- zip の中身は、次の形だけになる。展開して `.claude/skills/` に置けば、そのまま使える。

  ```
  pasta-skills.zip
  ├─ pasta-ghost-authoring/SKILL.md, references/…
  ├─ pasta-lua-coding/SKILL.md, references/…
  ├─ pasta-check/SKILL.md, references/…
  └─ LICENSE
  ```

- リリース CI の配布物の検査が、3 つの `SKILL.md` と `LICENSE` があること、ほかのスキル（`kiro-*`・`release` など）が混ざっていないことを確かめる。足りなければ公開しない。
- マニュアルに、zip の落とし方と置き場所を書く。

## Approach

`pasta_check.zip` を足したときと同じ流れにする（2026-10-23 開発者の承認）。

- `release.ps1` に段を 1 つ足して、`release\pasta-skills.zip` を作り、中身を確かめる。手元の `release.bat` とリリース CI が、同じ一式を作る。
- `release.yml` の配布物の検査と upload に、`pasta-skills.zip` を足す。
- `github-release.ps1` の添付を 6 つにし、`release-scripts-test.ps1` を合わせる。
- `release` スキルの添付の数え方を 5/5 にする。
- 名前に版を付けない（`pasta.dll.zip`・`pasta_check.zip` とそろえる）。

退けた案: スキルごとに 3 つの zip を作る（claude.ai へのアップロードの形に近いが、添付が 3 つ増える）。

## Scope

- **In**:
  - `release.ps1` の新しい段（`pasta-skills.zip` の作成と中身の検査）
  - `release.yml` の配布物の検査と upload
  - `github-release.ps1` の添付の名前と数、`release-scripts-test.ps1`
  - `release` スキルの添付の数え方（`scripts/release-state.ps1`・`procedure.md`・`SKILL.md` の言い方）
  - 文書: マニュアル（`introduction.md` の配布物の表と、スキルの受け取り方と置き場所の短い説明）、`crates/pasta_sample_ghost/RELEASE.md`・`README.md`、`tech.md`・`structure.md` の配布物の行
- **Out**:
  - スキルの中身の書き直し
  - claude.ai 向けのスキルごとの zip
  - 開発用のスキル（`kiro-*`・`release`・`karpathy-guidelines`）の配布
  - VSIX へのスキルの同梱
  - 署名

## Boundary Candidates

- zip を作る段（`release.ps1`）と、それを確かめて公開する側（`release.yml`・`github-release.ps1`）
- 添付の数を数える `release` スキル

## Out of Boundary

- `gen-skill-refs` の生成の仕組み（タグの時点の生成物をそのまま詰める）
- `pasta_check release`（作者のゴーストの .nar を作る道）

## Upstream / Downstream

- **Upstream**: `pasta-check-release-asset`（完了。`release.ps1` の段 3 と `pasta_check.zip` の検査の形）、`gen-skill-refs` の鮮度の検査
- **Downstream**: 0.4.3 の `ponytail-manual-ci`・`ponytail-check-release`、版の決まっていない `pasta-dll-multiarch` が、この spec の後の `release.yml`・`github-release.ps1`・`release.ps1` に合わせる

## Existing Spec Touchpoints

- **Extends**: なし（`pasta-check-release-asset` が作った段と検査は保つ）
- **Adjacent**: `ponytail-manual-ci`（`release.yml`・`github-release.ps1`）、`ponytail-check-release`（`pasta-check/SKILL.md` の中身を直す。zip はタグの時点のものを詰めるだけなので、ぶつからない）、`pasta-dll-multiarch`（`release.ps1`・配布物の検査）

## Constraints

- リリースの席（`release.yml`・`release.ps1`・`github-release.ps1`）は、1 つのウェーブに 1 本だけ。
- zip の中のパスは `/` で区切り、展開した先に `pasta-ghost-authoring/` などのフォルダがそのまま出る形にする（Windows の `Compress-Archive` の癖に注意する）。
- 手元の `release.bat` は Windows PowerShell 5.1 でも走る。
- `.claude/skills/release/` の編集は、分類器が Self-Modification として止める。その場面になったら、回避策を探さずに、入った分と残りを示して開発者の許可を求める。

## 2026-10-23 discovery の測定（main 26b1f5cb）

- **触るファイル**: `crates/pasta_sample_ghost/release.ps1`（331 行）、`.github/workflows/release.yml`（589 行）、`.github/scripts/release/github-release.ps1`（132 行）・`release-scripts-test.ps1`（256 行）、`.claude/skills/release/`（`SKILL.md`・`procedure.md`・`scripts/release-state.ps1`）、`book/src/introduction.md`、`crates/pasta_sample_ghost/RELEASE.md`・`README.md`、`tech.md`・`structure.md`。1,000 行に近いものは無い。
- **規模**: 6〜8 タスク。
- **先に要るもの**: なし。
- **種別**: 機能（開発者の依頼）。
- **要件定義のモデル**: Opus。後戻りしにくい土台の決定を含まない。
- **議題の候補**: zip の検査の厳しさ（3 つのフォルダの全部のファイルを突き合わせるか、`SKILL.md` と `LICENSE` だけを見るか）、マニュアルのどこに受け取り方を書くか。
