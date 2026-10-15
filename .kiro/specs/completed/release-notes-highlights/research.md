# Research & Design Decisions

## Summary

- **Feature**: `release-notes-highlights`
- **Discovery Scope**: Extension（完了した `release-ci` のスクリプトと、常駐 spec `release-workflow` の手順への書き足し）
- **Key Findings**:
  - リリース CI の補助スクリプトは、役割ごとに 1 本ずつ分かれている。見どころの検査は `verify-tag.ps1` の検査の並びに 1 つ足せば済み、`release.yml` の verify の job は変えなくてよい。
  - `github-release.ps1` は、添付の名前の一覧（`$names`）だけで「そろっているか」「足りないものは何か」を判定している。詳しいリリースノートを添付のディレクトリに置き、一覧に 1 つ足せば、再実行で続けられる性質がそのまま 4 つ目にも及ぶ。
  - `release-workflow` の段と `tasks.md` のタスクは 1 対 1 で、段の番号が文書の全体から参照されている。見どころを書く段を新しい番号で足すと、参照をすべて振り直すことになる。段 4（版の更新）の中の節として足すのが、いちばん食い違いを生みにくい。
  - v0.3.8 の 57 件のうち、内部の作業は 21 件である。種類 `spec` と、スコープの短い一覧（`roadmap`・`skills` など）と `kiro-` で始まるスコープで、すべて見分けられる。

## Research Log

### リリース CI の今の作り

- **Context**: どこに何を足せば、公開の前に止まり、再実行で続けられるかを確かめる。
- **Sources Consulted**: `.github/workflows/release.yml`（verify 29〜50 行・github-release 441〜511 行）、`.github/scripts/release/verify-tag.ps1`（81 行）・`release-notes.ps1`（100 行）・`github-release.ps1`（132 行）。
- **Findings**:
  - verify の job は、タグのコミットを checkout してから `verify-tag.ps1` を呼ぶ。作業ツリーがタグのコミットなので、ファイルの有無は `Test-Path` で見られる。失敗は `$failures` に溜めて、まとめて summary に出す。
  - `release-notes.ps1` は `-Tag` のタグが在ることを求める（47 行）。前のタグは、形の合うタグのうち版で並べて `-Tag` より小さい最大のものである。件名は、種類とスコープを付けたまま載せる。
  - `github-release.ps1` は、`-NotesFile` を下書きの作成のときにだけ使う（95 行）。下書きが在るとき・公開済みのときは、本文を触らない。
  - github-release の job の集約の step は、ノートの生成の step が失敗したときも `failed`・`publish` を出す（483 行のコメント）。
- **Implications**: 見どころの検査は `verify-tag.ps1` に足す。本文と詳しいリリースノートは `release-notes.ps1` が 2 つのファイルとして書き出す。`github-release.ps1` は添付の名前を 1 つ足すだけにする。

### リリースの手順が `release` スキルへ移った（2026-10-11 追記）

- **Context**: この調査と設計を作った後に、常駐 spec `release-workflow` の手順が `release` スキルへ移り、spec は完了になった（PR #105・main のコミット 3414c77d）。
- **Sources Consulted**: `.claude/skills/release/` の `SKILL.md`・`procedure.md`・`scripts/`。`5 ファイル`・`4 行`・`3 本とも` で検索した。
- **Findings**:
  - 手順の正は、`SKILL.md`（実行の約束・止まり方・段の進め方）と `procedure.md`（段 1〜9 のコマンド・合否の基準・判定表）である。定型コマンド A〜C は `.claude/skills/release/scripts/` にある。呼び出しは `/release [X.Y.Z]` である。
  - 完了した spec（`.kiro/specs/completed/release-workflow/`）は記録で、以後は直さない。
  - 段 10・判定表の S2・F2 は無くなった。段 4 の手順 1〜8・段 5 の手順 1・段 6 の手順 1 の番号と、判定表の S7 は同じである。再開の判定（版の更新のコミットだけが残っている場合）は、段 1 の手順 4 から手順 3 に変わった。
  - 「5 ファイル」は、`SKILL.md` の実行の約束 3・段 4・段 5・「このスキルを直すとき」と、`procedure.md` の段 1 の手順 3・段 4 の Intent と手順 6〜8・段 5 の手順 1 に出る。
  - 定型コマンド C の「4 行」は、`SKILL.md` の段 5・段 6 と、`procedure.md` の段 5 の手順 5・段 6 の手順 2・段 8「タグの付け直し」の手順 2・「定型コマンドの呼び出し方」、`pretag-check.ps1` の冒頭の説明に出る。「3 本とも」は「定型コマンドの呼び出し方」に出る。
  - `SKILL.md` の段 9 に、「リリースの実行は、版の表記のほかに文書を変えない」とある。見どころのファイルを書くので、ここも直す。
  - 定型コマンド A（`release-state.ps1`）は、配布物の 3 つを名前で数える（「添付 3/3」）。4 つ目の添付があっても判定は変わらない。
  - `.claude/skills/` の下の編集は、エージェントへの操作の許可で拒まれる。`pasta-check` のスキルと同じく、開発者の許可が要る。
- **Implications**: 直す相手を、スキルの `SKILL.md`・`procedure.md`・`scripts/` に替える。完了した spec に要件（Requirement 13）を足す案は、やめる。スキルの編集が拒まれたら、リリース CI の側だけを main へ入れない（要件 6.7）。下の「`release-workflow` の『5 ファイルだけ』に効いている場所」と、以降の決定の中の `release-workflow` の文書の行番号は、移る前の記録である。

### `release-workflow` の「5 ファイルだけ」に効いている場所

- **Context**: 版の更新のコミットに入るファイルが 1 つ増える。食い違いを残さないために、効いている場所をすべて洗う（`brief.md` 議題 8）。
- **Sources Consulted**: `.kiro/specs/release-workflow/` の `requirements.md`・`design.md`・`tasks.md`・`scripts/`。`5 ファイル`・`4 行` で検索した。
- **Findings**:
  - `design.md`: 19・39・278・311・392・460・508・510・524・535 行（Goals・Boundary・段の表・実行の約束 4・段 1 の手順 4・段 4 の Intent と手順 6〜8・段 5 の手順 1）。
  - `tasks.md`: 14・83・84・85・89・93・153 行（実行の約束 4・タスク 4・5・11）。
  - `requirements.md`: 3.2・3.5 が版の更新の範囲を決めている。2.3・8.6 は「版の更新のコミットだけ」と書いていて、用語「版の更新」の定義を広げれば、文面は変えなくてよい。
  - 補助スクリプト: `pretag-check.ps1`（定型コマンド C。4 行）だけが、タグを付ける条件を見ている。`release-state.ps1`（A）と `ci-result.ps1`（B）は、ファイルの数を見ていない。
  - 定型コマンド C の「4 行とも合格」は、`design.md` の 571・589・730・996 行と、`tasks.md` の 98・104 行に出てくる。
  - `research.md` 154 行にも「5 ファイル」があるが、過去の調査の記録なので変えない。
- **Implications**: 文書の直しは、上の行をすべて対象にする。直した後に `5 ファイルだけ`・`4 行とも` を検索して、残りが無いことを確かめる。

### 内部の作業の見分け方

- **Context**: 詳しいリリースノートから内部の作業を除く決まりを、機械で当てられる形にする（`brief.md` 議題 5）。
- **Sources Consulted**: `git log v0.3.7..v0.3.8 --no-merges --format=%s`（57 件）。
- **Findings**:
  - 種類 `spec` が 9 件（`spec(discovery)`・`spec(release-ci)`）。
  - スコープ `roadmap` 3 件・`release-workflow` 2 件・`skills` 2 件・`kiro-discussion`・`kiro-requirements-discussion`・`kiro-skills`・`cc-sdd` が各 1 件。
  - `chore(release): v0.3.8` が 1 件。
  - 合わせて 21 件。残る 36 件は、利用者に届く変更である（`feat(release-ci)` はリリースの仕組みの追加で、利用者にも関係するので残す）。
  - main の最近のコミットには `docs(discovery)` もある。
- **Implications**: 決まりは 3 つにする。(1) 種類が `spec`。(2) スコープが `spec`・`discovery`・`roadmap`・`steering`・`skills`・`cc-sdd`・`release-workflow` のどれか、または `kiro-` で始まる。(3) 種類が `chore` でスコープが `release`（版の更新のコミット）。

### 破壊的変更の印

- **Context**: 接頭辞を外すと、`!` の印が消える（要件 3.5）。
- **Findings**: v0.3.8 の破壊的変更 3 件（#65・#69・#70）は、件名の本文にも「（破壊的変更）」と書いている。ただし、これは書き方の決まりではない。
- **Implications**: 行ごとに印を付けると、今の件名では「破壊的変更」が 2 度出る。専用の見出しに集めれば、印を件名に足さずに済み、読む人は 1 か所で確かめられる。

### 公開済みの Release の編集

- **Context**: v0.3.8 のページを直せるか（要件 9）。
- **Sources Consulted**: `gh release view v0.3.8 --json assets,name,isDraft`、`brief.md` の測定。
- **Findings**: v0.3.8 は公開済みで、添付は 3 つ。Immutable Releases は無効なので、本文の差し替えと添付の追加ができる。エージェントへの操作の許可（`.claude/settings.json`）には、`gh release` の系統が無い。
- **Implications**: 直しは 1 回きりなので、許可の設定を足さない。拒まれたら、コマンドを開発者に示して止まる（要件 9.9）。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 既存のスクリプトに足す | `verify-tag.ps1` に検査を 1 つ、`release-notes.ps1` に整形と本文の組み立て、`github-release.ps1` に添付の名前を 1 つ | 新しいファイルが少ない。job の並びを変えない。再実行の性質をそのまま引き継ぐ | `release-notes.ps1` の役割が「一覧」から「一覧と本文」に広がる | 採用 |
| 見どころ用の新しいスクリプトと job | 検査・本文の組み立てを別のスクリプトにし、verify の後に job を足す | 役割が細かく分かれる | job と結果の報告の表が増える。`release-workflow` の結果の読み方（7 つの公開先）に波及する | 採らない |
| 手順の文書に新しい段を足す | `release-workflow` に「段 5 見どころ」を足し、以降を振り直す | 段の役割が 1 つずつになる | 段の番号への参照（判定表 S0〜S9・F0〜F7・`tasks.md`）をすべて振り直す。振り直しの漏れは、リリースの誤操作になる | 採らない。段 4 の中の節にする |

## Design Decisions

### Decision: 本文の組み立てを `release-notes.ps1` に置く

- **Context**: 本文は「見どころのファイルの内容 ＋ 詳しいリリースノートへのリンクと件数」である（要件 4.1・4.2）。件数は、一覧を作る側だけが知っている。
- **Alternatives Considered**:
  1. `github-release.ps1` が組み立てる — 件数を受け渡す引数か出力が要る。
  2. `release.yml` の step の中で組み立てる — 手元（v0.3.8 の直し）で同じ組み立てを再現できない。
- **Selected Approach**: `release-notes.ps1` に `-BodyFile` を足す。指定されたら、`doc/release-notes/{Tag}.md` を読み、末尾にリンクと件数を足して書き出す。
- **Rationale**: 件数・タグ・リポジトリの名前を、すでに持っている。CI と手元で同じ 1 本を使える。検査（要件 8）が 1 本を動かせば、本文の形まで確かめられる。
- **Trade-offs**: スクリプトの名前（release-notes）は、役割の広がりに対して狭い。名前は変えない（`release.yml`・文書・`structure.md` の参照を増やさない）。
- **Follow-up**: 検査で、本文が見どころで始まり、リンクで終わることを確かめる。

### Decision: 破壊的変更を専用の見出しに集める

- **Context**: 要件 3.5。
- **Alternatives Considered**:
  1. 行の頭に印（【破壊的変更】）を付ける — 今の件名では、同じ言葉が 2 度出る。
  2. 件名に「破壊的変更」が無いときだけ印を付ける — 特別扱いが増える。
- **Selected Approach**: 見出し「動きが変わる変更」を先頭に置き、`!` の付いた件名を、種類の見出しの代わりにそこへ入れる。
- **Rationale**: 読む人が、上げる前に確かめることが 1 か所にまとまる。`highlights-check.ps1` が、この見出しの下の PR の番号を「必ず載せる番号」として読める。
- **Trade-offs**: 破壊的変更は、新機能か不具合の修正かが分からなくなる。件名を読めば分かるので、足りる。

### Decision: 見どころを書く段を、`release-workflow` の段 4 の中の節にする

- **Context**: 要件 5・6。段とタスクは 1 対 1 で、番号が文書の全体から参照されている。
- **Alternatives Considered**:
  1. 新しい段として足し、以降を振り直す。
  2. 段 4 の手順の途中に手順を足し、手順の番号を振り直す — 「段 4 の手順 6 の (3)(4)」「手順 8」への参照が、段 1 と `tasks.md` にある。
- **Selected Approach**: 段 4 に「見どころを書く」の節（手順 H1〜H4）を足す。手順 5 の後・手順 6 の前に行う。手順 6〜8 は番号を変えず、見どころのファイルを対象に足す。
- **Rationale**: 既存の番号への参照が、すべて生きたままになる。段 8 にも名前の付いた節（「失敗した job の再実行」「タグの付け直し」）があり、同じ書き方である。
- **Trade-offs**: 段 4 が長くなる（Intent が「版の更新と見どころ」に広がる）。
- **Follow-up**: 直した後、`5 ファイルだけ`・`4 行とも`・`3 本とも` を検索して、古い記述が残っていないことを確かめる。

### Decision: 手順の側の確認を、定型コマンド D（スクリプト）にする

- **Context**: 要件 5.3 の 5 つの確認を、モデルの推論に任せない（`release-workflow` の方針）。
- **Alternatives Considered**:
  1. `grep` を 5 つ並べる — PR の番号の突き合わせ（材料の一覧との集合の比較）が 1 行で書けない。
  2. `verify-tag.ps1` に全部を足す — リリース CI の検査の範囲（在ること・空でないことまで）を超える（要件 2.5）。
- **Selected Approach**: `.claude/skills/release/scripts/highlights-check.ps1` を足す（設計のときは `.kiro/specs/release-workflow/scripts/` だった。手順がスキルへ移ったので、置き場所を替えた）。定型コマンド A〜C と同じ置き場所・同じ呼び出し方・同じ出力の形（`名前 | 値 | 合格／不合格`）にする。
- **Rationale**: 既存の定型コマンドの型に乗る。v0.3.8 の直しでも同じコマンドで確かめられる。
- **Trade-offs**: spec 名の検査は、`.kiro/specs/` のフォルダ名との完全な一致（前後が英数字・`_`・`-` でない）で行う。spec 名と同じ綴りの言葉を、見どころに書けない。今ある 180 余りの名前はすべて `-` か `_` を含むので、ふつうの言葉には当たらない。

### Decision: 検査を `build.yml` でも動かす

- **Context**: 要件 8 は、1 つのコマンドで動く検査を求める。CI で動かすかは設計に任された。
- **Alternatives Considered**:
  1. 手元だけ — 動かし忘れる。壊れたスクリプトが、リリースのときに初めて見つかる。
  2. `release.yml` の verify の job で動かす — 見つかるのがタグの push の後になる。
- **Selected Approach**: `build.yml` に job を 1 つ足す（ubuntu・pwsh・1 step）。
- **Rationale**: PR の時点で見つかる。`build.yml` はリリース CI の関門でもあるので、公開の前にも必ず動く。既存の job を変えないので、ふだんの PR の挙動は変わらない（job が 1 つ増えるだけである）。
- **Trade-offs**: `build.yml` を触る spec が増える。

### Decision: 見どころを示した後に承認を求めない（確定）

- **Context**: 要件 5.7。`requirements.md` の未決事項 6。
- **Selected Approach**: 全文を開発者に示して、先へ進む。
- **Rationale**: 版を自動で決めるときに承認を求めない決まり（`release-workflow` の要件 1.3）と、リリースを自律で完遂させる方針に合わせた。
- **確定**: 2026-10-11、開発者が「全て推奨で」と答えた。
- **Follow-up**: 後で承認を待つ形に変えるなら、`procedure.md` の段 4 の手順 8 の後の 1 文と、`SKILL.md` の段 4 の箇条を変えるだけで済む。

### Synthesis（一般化・既存の採用・単純化）

- **一般化**: 「リリース CI の本文」「タグを打つ前の材料の一覧」「v0.3.8 の直し」は、どれも「ある範囲のコミットから一覧を作り、見どころと合わせる」の変形である。`release-notes.ps1` に `-Head`（範囲の終わり）と `-BodyFile` を足した 1 本で、3 つとも賄う。
- **既存の採用**: GitHub が自動で作るリリースノート（`--generate-notes`）は、PR の題名を並べるので spec 名が載る。採らない（areka との違いは合わせない決定）。添付の判定・再実行・結果の報告は、`github-release.ps1` の今の仕組みをそのまま使う。
- **単純化**: 見どころのファイルの中に、版・件数・リンクを書かせない（件数とリンクは CI が足す）。見どころの検査のための新しい job・新しい結果の区分を作らない。`release-state.ps1`・`ci-result.ps1` は変えない。`.claude/settings.json` に許可を足さない。

## Risks & Mitigations

- この spec が main に入った後は、見どころのファイルの無いリリースタグが verify で止まる — `release-workflow` の文書と補助スクリプトを同じ PR で直す。リリースの実行中は main に入れない（席）。
- `release-workflow` の文書の直しに漏れがあると、エージェントが見どころのあるコミットを異常として止める（または見どころを落とす） — 効いている行を上の調査で洗い、直した後に検索で確かめる。漏れても、リリース CI の verify と定型コマンド C が、公開の前に止める。
- 見どころの文章が、動きが変わる点を取り違えて書く — 定型コマンド D が、破壊的変更の PR の番号がすべて載っていることを確かめる。内容の正しさは、開発者に全文を示すことと、PR に残ることで補う。
- v0.3.8 の直しで、公開済みのページを壊す — 直す前の本文をリポジトリに残す。添付を先に足し、本文は最後に差し替える。配布物・タグ・題名を触るコマンドを使わない。
- スキルの編集が拒まれる — 回避せず、改める文面を開発者に示す（要件 7.5）。

## References

- `.kiro/specs/release-notes-highlights/brief.md` — discovery の決定と測定
- `.kiro/specs/completed/release-ci/` — リリース CI の要件（6.2・6.4・6.6〜6.9）
- `.kiro/specs/release-workflow/design.md` — 段 1〜10・定型コマンド A〜C
- `book/AUTHORING.md` 第 1 節 — Claudia の口調の定義
- <https://github.com/ponapalt/claudia> — Claudia の出典
