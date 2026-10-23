# Requirements Document

## Project Description (Input)
**誰が困っているか**: Claude などの AI にゴーストを書かせたい、ゴーストの作者。

**現状**: AI にゴーストを書かせるときに使う pasta のスキル（`pasta-ghost-authoring`・`pasta-lua-coding`・`pasta-check`）は、このリポジトリの `.claude/skills/` にしか無い。使うには、リポジトリをクローンして、3 つのフォルダを探して写す必要がある。どの版の pasta に合ったスキルなのかも分かりにくい。リリース CI が GitHub Release に添付するのは、配布物 4 つ（`pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip`・`pasta-vscode-X.Y.Z.vsix`）と詳しいリリースノートの 5 つである。

**何を変えるか**: 各版の GitHub Release に、3 つのスキルと `LICENSE` だけを詰めた `pasta-skills.zip` を 5 つ目の配布物として載せ（添付は 6 つ）、`https://github.com/ekicyou/pasta/releases/latest/download/pasta-skills.zip` で最新版を落とせるようにする。`pasta_check.zip` を足したとき（完了 spec `pasta-check-release-asset`）と同じ流れで、手元の `release.bat` とリリース CI が同じ一式を作り、リリース CI は中身を検査して、足りなければ公開しない。マニュアルに、zip の落とし方と置き場所を書く。詳細は `brief.md` を参照。

## Introduction

本仕様は、ゴーストの作者が AI にゴーストを書かせるときに使う pasta のスキル 3 つを、GitHub Release の配布物として渡せるようにする。

今、ゴーストの作者がこのスキルを使うには、リポジトリをクローンして `.claude/skills/` から 3 つのフォルダを探して写すしかない。本仕様の後は、Release のページ（または固定のリンク）から `pasta-skills.zip` を落として、ゴーストの開発フォルダの `.claude/skills/` に展開すれば、そのまま使える。Release は版ごとにあるので、使っている pasta と同じ版のスキルを選べる。

使い方の例（本仕様の後）:

```text
1. https://github.com/ekicyou/pasta/releases/latest/download/pasta-skills.zip を落とす
2. ゴーストの開発フォルダの .claude/skills/ に展開する
     .claude/skills/pasta-ghost-authoring/SKILL.md
     .claude/skills/pasta-lua-coding/SKILL.md
     .claude/skills/pasta-check/SKILL.md
     .claude/skills/LICENSE
3. そのフォルダで AI（Claude Code など）に「起動したら挨拶するシーンを書いて」などと頼む
```

zip の中身:

```text
pasta-skills.zip
├─ pasta-ghost-authoring/SKILL.md, references/…
├─ pasta-lua-coding/SKILL.md, references/…
├─ pasta-check/SKILL.md, references/…
└─ LICENSE
```

配布物を作るのは、今の作りに 1 段足した手順である。手元の `release.bat`（中で `release.ps1` を呼ぶ）とリリース CI の build は同じ `release.ps1` を使い、`pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip`・`pasta-skills.zip` の 4 つを `release/` に作る。VSIX はこれまでどおり VSCode 拡張のパッケージの手順が作る。

用語:
- **スキル**: AI のエージェント（Claude Code など）が読む手引き。フォルダ 1 つが 1 スキルで、`SKILL.md` と `references/` の下の文書から成る。本仕様で配るのは `pasta-ghost-authoring`・`pasta-lua-coding`・`pasta-check` の 3 つ（**配るスキル**）。
- **開発用のスキル**: このリポジトリの開発だけに使うスキル（`kiro-*`・`release`・`karpathy-guidelines` など）。配らない。
- **配布物**: GitHub Release に添付する、利用者向けのファイル。本仕様の後は `pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip`・`pasta-skills.zip`・`pasta-vscode-X.Y.Z.vsix` の 5 つ。
- **添付**: GitHub Release に付けるファイル。配布物 5 つと、詳しいリリースノート `release-notes-vX.Y.Z.md` の、合わせて 6 つ。
- **リリースビルド**: 配布物を作る手順。手元では `release.bat`（`release.ps1`）で、リリース CI では build の job で動く。同じ手順である。
- **リリース CI**: リリースタグ `vX.Y.Z` の push で動く GitHub Actions のワークフロー（完了 spec `release-ci`）。
- **リリースの状態の読み取り**: `release` スキルが、ある版が各公開先に出ているかを調べて表にする手順（定型コマンド A）。
- **生成ファイル**: `pasta-ghost-authoring`・`pasta-lua-coding` の `references/` のうち、マニュアルの章から `gen-skill-refs.mjs` で作るファイル。

## Boundary Context

- **In scope**:
  - `pasta-skills.zip` を作る段（zip の作成と中身の検査）を、リリースビルドに足す。
  - リリース CI の build の job で `pasta-skills.zip` を作り、検査し、後続の job へ渡す。
  - GitHub Release の添付を 6 つにする。添付の数の確かめ方（スクリプトのテスト）を整える。
  - `release` スキルのリリースの状態の読み取りと、その手順書の添付の数え方（5/5）。
  - 文書: マニュアル（`introduction.md` の配布物の並びと、スキルの受け取り方と置き場所の短い説明）、`crates/pasta_sample_ghost/RELEASE.md`・`README.md`、steering の `tech.md`・`structure.md`、ルートの `README.md` と `release.bat` の説明（2026-10-23 の要件ディスカッションで含めると決めた）。
- **Out of scope**:
  - 配るスキルの中身の書き直し（`pasta-check` の `SKILL.md` の段の並びと配布物の一覧の書き換えを除く。2026-10-23 の要件ディスカッションで、0.4.2 の zip に古い段の番号を入れないために本仕様で直すと決めた）。
  - claude.ai へのアップロード向けの、スキルごとの zip。
  - 開発用のスキルの配布。
  - VSIX へのスキルの同梱。
  - 署名。
  - `gen-skill-refs.mjs` の生成の仕組み（タグの時点の生成ファイルをそのまま詰める）。
  - `pasta_check release`（作者のゴーストの `.nar` を作る道）の変更。
  - 毎回の CI（`build.yml`）の構成と、関門（gate）が呼ぶ検査。
  - 過去の版（0.4.1 まで）の Release へ `pasta-skills.zip` を足すこと。
  - リリース CI の公開系の job の action の版の固定と Node の版（`ponytail-manual-ci` が持つ）。
- **Adjacent expectations**:
  - `pasta-check-release-asset`（完了）が作った `pasta_check.zip` の段と検査は、そのまま保つ。
  - 生成ファイルの鮮度は、これまでどおりマニュアルの CI（`manual.yml` の `gen-skill-refs.mjs --check`）が見る。本仕様はその仕組みを変えない。
  - `ponytail-manual-ci`・`ponytail-check-release`（0.4.3）と、版の決まっていない `pasta-dll-multiarch` は、本仕様の後の `release.yml`・`github-release.ps1`・`release.ps1` に合わせる。`ponytail-check-release` が `pasta-check/SKILL.md` の中身を直しても、zip はタグの時点の中身を詰めるだけなので、ぶつからない。
  - 本番の確かめ（GitHub Release に `pasta-skills.zip` が実際に載り、固定のリンクで落ちること）は 0.4.2 のリリースで行う。本仕様の中では、配布物は手元のリリースビルドで、添付の数はスクリプトのテストで確かめる。

## Requirements

### Requirement 1: pasta-skills.zip の中身

**Objective:** AI にゴーストを書かせたいゴーストの作者として、落として `.claude/skills/` に展開するだけで pasta のスキルを使いたい。そうすれば、リポジトリをクローンしてフォルダを探さずに済む。

#### Acceptance Criteria

1. The リリースビルド shall `pasta-skills.zip` の最上位に、`pasta-ghost-authoring/`・`pasta-lua-coding/`・`pasta-check/` の 3 つのフォルダと、`LICENSE` の 1 ファイルだけを置く。
2. The リリースビルド shall 3 つのフォルダそれぞれに、リリースするコミットの `.claude/skills/<スキル名>/` にあるファイル（`SKILL.md` と `references/` の下のファイル）を、同じ相対パス・同じ中身で、過不足なく入れる。
3. The リリースビルド shall 開発用のスキル（`kiro-*`・`release`・`karpathy-guidelines` など）と、配るスキル 3 つと `LICENSE` のほかのファイルを、`pasta-skills.zip` に入れない。
4. The リリースビルド shall `pasta-skills.zip` の `LICENSE` に、リポジトリのルートの `LICENSE`（MIT License）と同じ中身を入れる。
5. The リリースビルド shall zip の中のパスを `/` で区切って記録し、展開した先に `pasta-ghost-authoring/SKILL.md` などがフォルダの形のまま出るようにする。上に別のフォルダ（`skills/` など）を挟まない。
6. The リリースビルド shall `pasta-skills.zip` に、版を示すファイルなど、上の形のほかの物を足さない。版は、zip を添付した Release の版で示す。

### Requirement 2: 手元のリリースビルドで作って検査する

**Objective:** pasta の開発者として、手元の `release.bat` でも、リリース CI と同じ `pasta-skills.zip` を作って確かめたい。そうすれば、タグを push する前に、zip が正しい形で作れるかを手元で確かめられる。

#### Acceptance Criteria

1. When リリースビルドが成功したとき, the リリースビルド shall `release/` に `pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip` に加えて `pasta-skills.zip` を作り、終わりのまとめに `pasta-skills.zip` の場所も示す。
2. When `pasta-skills.zip` を作ったとき, the リリースビルド shall zip の中身が Requirement 1 の形（最上位の 3 つのフォルダと `LICENSE`、元のフォルダと同じファイルの一覧、`/` で区切ったパス）であることを検査する。
3. If `pasta-skills.zip` の中身が Requirement 1 の形でないとき, the リリースビルド shall 足りないファイル・余計なファイル・形の合わないパスを示して失敗する。
4. If `pasta-skills.zip` の段が失敗したとき, the リリースビルド shall 前の実行で作った `pasta-skills.zip` を `release/` に残さない。
5. While `-SkipSetup` または `-SkipDllBuild` を指定して実行しているとき, the リリースビルド shall `pasta-skills.zip` の段を飛ばさずに行い、作り直す。`pasta-skills.zip` の段だけを飛ばす指定は足さない。
6. The リリースビルド shall 手元の `release.bat`（Windows PowerShell 5.1）で走ったときも、リリース CI（PowerShell 7）で走ったときも、同じファイルの一覧と同じパスの形（`/` 区切り）の `pasta-skills.zip` を作る。
7. The リリースビルド shall `pasta-skills.zip` を作るのに、今のリリースビルドが要る道具のほかに、新しく入れる道具を要さない。
8. The リリースビルド shall 段の見出しの番号（`[1/7]` など）と、スクリプトの冒頭の説明・`release.bat` の説明を、新しい段を含めた並びに合わせる。
9. The リリースビルド shall 本仕様の前から作っていた配布物（`pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip`）の中身と、その検査を変えない。
10. The リリースビルド shall 手元とリリース CI で同じ手順を使い、リリース CI だけの別の作り方を持たない。

### Requirement 3: リリース CI で作り、検査し、渡す

**Objective:** pasta の開発者として、リリース CI が `pasta-skills.zip` をタグのソースから作り、中身を確かめてから公開してほしい。そうすれば、スキルが欠けた zip や開発用のスキルが混ざった zip のまま Release が公開されることを防げる。

#### Acceptance Criteria

1. When リリースタグが push され、関門の検査が通ったとき, the リリース CI shall build の job で、タグのコミットのソースから `pasta-skills.zip` を、ほかの配布物と同じリリースビルドで作る。
2. When build の job が配布物を検査するとき, the リリース CI shall `pasta-skills.zip` が在り、3 つの `SKILL.md`（`pasta-ghost-authoring/SKILL.md`・`pasta-lua-coding/SKILL.md`・`pasta-check/SKILL.md`）と `LICENSE` があり、最上位にほかのフォルダやファイル（開発用のスキルなど）が無いことを検査する。
3. When build の job が配布物を検査するとき, the リリース CI shall `pasta-skills.zip` の中のファイルの一覧が、タグのコミットの配るスキル 3 つのフォルダのファイルの一覧と過不足なく一致し、パスが `/` で区切られていることを検査する。
4. When build の job が配布物を作り始める前に, the リリース CI shall 生成ファイルが、タグのコミットのマニュアルから生成した結果と一致することを照合する。生成ファイルを配る以上、マージで CI を待たない運用でもタグの時点の鮮度を保証するため。
5. If `pasta-skills.zip` が無いか、中身が合わないか、生成ファイルの照合が合わないとき, the リリース CI shall 何が合わないかを示して build の job を失敗させ、どの公開先（crates.io・Marketplace・GitHub Release）へも公開しない。
6. When build の job の検査が通ったとき, the リリース CI shall `pasta-skills.zip` を、ほかの配布物と同じ受け渡し（artifact `release-assets`）に入れて後続の job へ渡す。
7. The リリース CI shall 毎回の CI（`build.yml`）の構成と、関門（gate）が呼ぶ検査を変えない。

### Requirement 4: GitHub Release に 6 つを添付する

**Objective:** AI にゴーストを書かせたいゴーストの作者として、GitHub Release に `pasta-skills.zip` が載っていて、固定のリンクで最新版を落とせてほしい。そうすれば、版を気にせずに最新のスキルを入れられ、古い版の pasta を使うときはその版の Release から同じ版のスキルを選べる。

#### Acceptance Criteria

1. When GitHub Release を作るとき, the リリース CI shall 配布物 5 つ（`pasta.dll.zip`・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip`・`pasta-skills.zip`）と詳しいリリースノート `release-notes-vX.Y.Z.md` の 6 つを添付する。
2. The リリース CI shall `pasta-skills.zip` をこの名前（版を含まない名前）で添付し、`https://github.com/ekicyou/pasta/releases/latest/download/pasta-skills.zip` で最新版を落とせるようにする。
3. If 添付する 6 つのうち 1 つでも手元に無いとき, the リリース CI shall Release を作らず・公開せずに失敗し、無いファイルの名前を示す。
4. When 6 つの添付がすべてそろったとき, the リリース CI shall Release を公開する。6 つがそろう前の Release を公開状態にしない。
5. When 失敗した job を再実行したとき, the リリース CI shall 今と同じ規則（公開済みで 6 つ添付済みなら飛ばす・下書きなら足りない添付だけを付け直す・公開済みの添付は消さない）で続ける。
6. The リリース CI shall 添付の数を 6 つとする判定を、手元で、ネットワークと gh を使わずに確かめられるテストを持つ。テストは少なくとも、`pasta-skills.zip` だけが無いときに Release を作らずに失敗し、その名前を示すことを確かめる。`pasta_check.zip` だけが無いときの今のテストは、6 つの前提に合わせて保つ。テストは毎回の CI でも走る。

### Requirement 5: リリースの状態の読み取りと手順書

**Objective:** リリースを進めるエージェントとして、`release` スキルのリリースの状態の読み取りが、5 つの配布物を数えてほしい。そうすれば、`pasta-skills.zip` が欠けた Release を「公開済み」と取り違えずに済む。

#### Acceptance Criteria

1. When リリースの状態を読み取るとき, the リリースの状態の読み取り shall GitHub Release の添付のうち、配布物 5 つ（`pasta-skills.zip` を含む）がそろっているかを数え、`n/5` の形で示す。
2. The リリースの状態の読み取り shall 配布物 5 つがすべてそろって公開された Release だけを「あり（公開・添付 5/5）」として公開済みに数え、下書きと添付不足は数えない。
3. The `release` スキルの手順書 shall 状態の表の読み方（`あり（公開・添付 5/5）`・`下書き（添付 n/5）`・`添付不足（公開・添付 n/5）`）と、`pasta_check.zip`・`pasta-skills.zip` を添付していない 0.4.1 までの版の出方（`添付不足（公開・添付 3/5）`）を、5 つの配布物に合わせる。
4. The `release` スキル shall 添付不足のときに `release-assets` から手で添付する案内と、スキルを直すときの見どころ（配布物の名前・個数）の言い方を、5 つの配布物に合わせる。
5. The `release` スキル shall 公開先の数（crates.io の 5 クレート・Marketplace・GitHub Release の合わせて 7 つ）を変えない。

### Requirement 6: マニュアルにスキルの受け取り方と置き場所を書く

**Objective:** AI にゴーストを書かせたいゴーストの作者として、マニュアルを読めば、pasta のスキルをどこから落としてどこに置けばよいかが分かってほしい。そうすれば、リポジトリの中を探さずに、AI に pasta の書き方を教えられる。

#### Acceptance Criteria

1. The 利用者マニュアル shall はじめに（`introduction.md`）の関連リンクの配布物の並びに、`pasta-skills.zip` を固定のリンク（`releases/latest/download/pasta-skills.zip`）で加える。
2. The 利用者マニュアル shall `pasta-skills.zip` について、中身（AI にゴーストを書かせるときに使う pasta のスキル 3 つ）、落とし方（固定のリンクと、版ごとのリリースページ）、置き場所（ゴーストの開発フォルダの `.claude/skills/` に展開する）を、`introduction.md` の関連リンクの後の短い節に書く。章は足さない。
3. The 利用者マニュアル shall 使っている pasta と同じ版の `pasta-skills.zip` を使うことと、版を上げるときは 3 つのフォルダを新しい版のもので置き換えること（上書きで展開すると、新しい版で消えたファイルが残るため）を書く。
4. The 利用者マニュアル shall claude.ai へのアップロードの手順や、スキルごとに zip を作り直す手順など、配布物の形で用意していない使い方の代わりの書き方を載せない。
5. The 利用者マニュアル shall マニュアルの検査（リンクの検証・章の数の検査・台詞部品の検査など、`manual.yml` が行うもの）を通る状態に保つ。

### Requirement 7: 配布物の一覧を書いた文書

**Objective:** pasta の開発者とリリースを進めるエージェントとして、配布物の一覧とリリースビルドの段の並びを書いた文書が、`pasta-skills.zip` を足した後の姿と合っていてほしい。そうすれば、古い一覧を見て配布物の数や段の番号を取り違えずに済む。

#### Acceptance Criteria

1. The 文書 shall 配布物の一覧を書いた所（`crates/pasta_sample_ghost/RELEASE.md`・`crates/pasta_sample_ghost/README.md`・steering の `tech.md`・`structure.md`・ルートの `README.md`）に `pasta-skills.zip` を加え、配布物の数（5 つ）・添付の数（6 つ）・リリースビルドの段の並びの記述を合わせる。
2. The 文書 shall `pasta-check` スキルの `SKILL.md` のうち、hello-pasta のリリースビルドの段の並びと配布物の一覧を書いた所を、新しい段と `pasta-skills.zip` に合わせる。
3. The 文書 shall リリース CI のワークフローの冒頭の説明（作る配布物の数）を、5 つの配布物に合わせる。
