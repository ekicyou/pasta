# Requirements Document

## Project Description (Input)

### リリース仕様（リリース CI に合わせた書き直し版）

本仕様は、リリースのための手順を定め、「実装」（`/kiro-impl release-workflow`）を実行するたびにリリース作業を行う、**繰り返しタスク**の仕様である。本仕様は実装完了しない。新たに「実装」が指示されるたび、タスクの実行状況は初期化され、新たなリリース作業を行う。

リリースの CI 化（`release-ci`）が完了し、リリースタグ `vX.Y.Z` の push だけで、公開前の検査・配布物のビルド・crates.io（5 クレート）・VSCode Marketplace・GitHub Release への公開まで進むようになった。手元でビルドして公開し、成果物をコミットする旧手順は、そのままでは動かない。

本書き直しでは、エージェントの手順を次の 4 段へ縮める（steering `roadmap.md`「リリース手順の書き換え」）。

1. 版を決める
2. 版を上げたコミットを PR で main に入れる
3. リリースタグを push する
4. リリース CI の結果を確かめ、失敗した job を再実行する

次のものはリリース CI へ移ったので、本仕様から取り除く: 手元でのビルド、公開の 2 トラック、Resume、スケジュールによる再試行、main の CI が全部緑かの確認。マージコミット方式で統合する理由（タグが指すコミットを main から到達できるようにするため）も見直す。

あわせて、`release-ci` から申し送られた「CI での初回のリリースと、その後の後片付け」を扱う。期限は 2026-12-01（Marketplace の global PAT の廃止）より前である。

---

## Introduction

本ドキュメントは、pasta プロジェクトのリリース手順に関する要件を定義する。この手順は、LLM エージェントが開発者の指示のもとで繰り返し実行する。エージェントは版を決め、版を上げたコミットを main に入れ、リリースタグを push し、リリース CI の結果を確かめる。検査・ビルド・公開そのものはリリース CI（`.github/workflows/release.yml`）が行い、本仕様は受け持たない。

本文中の **【仮定】** は、入力だけでは決めきれず、最善の仮定で書いた箇所を示す。対応する論点は末尾の「未決事項」にまとめた。

用語:

- **リリース CI**: リリースタグの push を契機に動く GitHub Actions のワークフロー（`release-ci` の成果物）。
- **リリースタグ**: `v` の後に `X.Y.Z`（数字 3 つ）が続くタグ。例: `v0.3.8`。
- **公開先**: crates.io（`pasta_core`・`pasta_dsl`・`pasta_lua`・`pasta_shiori`・`pasta_check` の 5 クレート）・VSCode Marketplace・GitHub Release。
- **公開済み**: その版が公開先に実際に存在する状態。
- **版の更新**: リポジトリ内の版の表記を新しい版へそろえる変更（Requirement 3）。
- **作業ブランチ**: Claude Code ハーネスが供給するワークツリーの、デフォルトではないブランチ。

### 仕様の特殊性

- **繰り返し実行型**: `/kiro-impl release-workflow` が実行されるたびにタスクの状態はリセットされ、新たなリリース作業として実行される。
- **永続的未完了**: 本仕様は `completed` に移行しない。
- **パラメータ依存**: 実行のたびに、版が開発者から与えられるか、調査の結果から提案される。
- **オペレーション仕様**: 製品コードの新規作成・変更を伴わない。変えるのは版の表記だけである。

## Boundary Context

- **In scope**:
  - 版の決定（調査・提案・承認・重複の検査）。
  - 版の更新と、その整合の確認。
  - 版の更新の main への統合（PR 経由）。
  - リリースタグの作成と push。
  - リリース CI の結果の確認、失敗した job の再実行、失敗の種類ごとの対応の案内。
  - 途中で止まったリリースの再開（実際の状態からの判定）。
  - CI での初回のリリースの確認と、その後の後片付けの案内。
  - 本書き直しに伴う、文書・設定の一回限りの整合（Requirement 12）。
- **Out of scope**:
  - 公開前の検査、配布物のビルド、crates.io・Marketplace への公開、GitHub Release の作成、リリースノートの生成（リリース CI が行う）。
  - リリース CI の定義（`release.yml`・`.github/scripts/release/`）と `release.ps1` の機能の変更。
  - 失敗の原因になったコードやワークフロー定義の修正そのもの（別の PR で行う。本仕様は原因を報告し、修正の後に続きを行う）。
  - 一回限りのセットアップのうち、公開先の画面で行う設定と認証情報の失効の実施（開発者が行う。本仕様は案内する）。
  - マニュアルの対象バージョン行以外の改訂と、マニュアルの公開（main への統合を契機に、マニュアル側の既存の CI が行う）。
  - プレリリースの版、pasta_lsp の独立リリース、新しいクレートの初回公開。
  - 確認ワークフロー（`release-setup-check.yml`）のログにマネージド ID の識別子が出る件の修正 **【仮定】**（秘密の値ではなく実害が無い。セットアップは済んでいる）。
- **Adjacent expectations**:
  - リリース CI: リリースタグの push で起動し、タグの形・版の一致・main からの到達を検査し、`build.yml` の検査をタグのコミットで行い、公開先ごとの結果（`published`・`skipped`・`failed`・`not-run`）と GitHub Release の URL を報告する。再実行では公開済みのものを飛ばす。本仕様はこの挙動を前提にする。
  - `crates/pasta_sample_ghost/RELEASE.md`: 人が読むリリース手順書。本仕様の手順と食い違わない。
  - `.github/release-ci-setup.md`: 一回限りのセットアップの手順書。認証の失敗の対応と、初回のリリースの後片付けの内容は、この手順書を正とする。
  - main のブランチ保護: main への変更は PR だけが通る。リリースタグの push は保護の対象外である。
  - PR の統合: PR の CI の完了を待たずに統合する運用（steering `workflow.md`）に従う。
  - 作業ブランチ: Claude Code ハーネスが供給する。
  - 席: リリースの実行中は、`release.ps1`・`release.yml` を触る変更を main に入れない（開発者の運用）。
  - 手元の `gh` は認証済みである。`CARGO_REGISTRY_TOKEN`・`VSCE_PAT` は要らない。

---

## Requirements

### Requirement 1: 版の決定

**Objective:** As a 開発者, I want リリースする版を、すでに出ている版と重ならないように確定したい, so that 取り消せない公開を誤った版で始めない

#### Acceptance Criteria

1. When リリース作業が開始され、版が指定されているとき, the Release Workflow shall 指定された版を使う。
2. When リリース作業が開始され、版が指定されていないとき, the Release Workflow shall すべての版の出どころ（`Cargo.toml`・`editors/vscode/package.json`・Git のタグ・crates.io・GitHub Releases・VSCode Marketplace）を調べ、最大の版の PATCH を 1 つ上げた値を提案する。
3. When 提案する版を算出したとき, the Release Workflow shall 出どころごとの調査結果と提案する版を開発者に示し、承認を求める。
4. If 開発者が提案する版を承認しないとき, the Release Workflow shall 希望する版の入力を求める。
5. When 版が与えられたとき, the Release Workflow shall `X.Y.Z`（数字 3 つ）の形かを検査する。
6. If 版が `X.Y.Z` の形でないとき（プレリリースの形を含む）, the Release Workflow shall エラーを示し、入力し直しを求める。
7. If 確定しようとする版が、Git のタグ・crates.io・GitHub Releases・VSCode Marketplace のいずれかにすでにあるとき, the Release Workflow shall エラーを示し、別の版の入力を求める。ただし Requirement 8 の「途中からの再開」にあたる場合を除く。
8. If 公開先への問い合わせが失敗し、版があるかどうかを確かめられないとき, the Release Workflow shall 「無い」と見なさず、確かめられなかった出どころを示して作業を止める。

### Requirement 2: 作業の開始条件

**Objective:** As a 開発者, I want リリース作業が、リリースと関係のない変更を巻き込まない状態から始まってほしい, so that 版の更新だけが main に入り、リリースの中身が意図したとおりになる

#### Acceptance Criteria

1. When リリース作業が開始されたとき, the Release Workflow shall 作業ブランチの上で動き、main の上での実行や main への直接の push を前提にしない。
2. If 現在のブランチがデフォルトブランチであるとき, the Release Workflow shall 何も変更せずに止まり、ハーネスのワークツリーでの再実行を求める。
3. If 作業ブランチに未コミットの変更、または main に無いコミットがあるとき, the Release Workflow shall それらをリリースに含めず、内容を示して止まる **【仮定】**。
4. When 作業ブランチが main より遅れているとき, the Release Workflow shall 版の更新より前に、main の内容を作業ブランチへ履歴を書き換えずに取り込む。
5. If main の取り込みで衝突が起きたとき, the Release Workflow shall 作業を止め、開発者に解消を求める。
6. When リリース作業が開始されたとき, the Release Workflow shall リポジトリへの必要な操作（PR の作成と統合・リリースタグの push・リリース CI の結果の閲覧と再実行）ができることを確かめる。
7. If 必要な操作ができないとき, the Release Workflow shall 何も変更せずに止まり、できない操作を示す。
8. The Release Workflow shall 手元でのテスト・配布物のビルド・公開を行わず、main の CI の結果を公開の条件にしない（公開前の検査はリリース CI が行う）。

### Requirement 3: 版の更新

**Objective:** As a 開発者, I want リポジトリ内の版の表記を 1 度でそろえたい, so that リリース CI の検査を通り、クレート・拡張・マニュアルが同じ版を示す

#### Acceptance Criteria

1. When 版が確定したとき, the Release Workflow shall 次の表記をすべて同じ版に更新する。
   - `Cargo.toml` の `[workspace.package]` の `version`
   - `Cargo.toml` の `[workspace.dependencies]` にある内部クレート（`pasta_core`・`pasta_dsl`・`pasta_lua`・`pasta_shiori`・`pasta_check`）の `version`
   - `Cargo.lock` の中の、ワークスペースのクレートの版
   - `editors/vscode/package.json` の `version`
   - `editors/vscode/package-lock.json` の中の、拡張自身の版
   - マニュアルのトップページ（`book/src/introduction.md`）の対象バージョン行 `| 対象 pasta バージョン | **vX.Y.Z** |`（`v` を付ける）
2. The Release Workflow shall 版の更新で、上の表記以外を変えない（外部の依存クレート・npm パッケージの版、マニュアルの他の本文を含む）。
3. When 版の表記を更新したとき, the Release Workflow shall 次の 3 つを確かめる。(1) すべての表記が同じ版を示す。(2) `Cargo.lock` が `Cargo.toml` と食い違わない（ビルドで書き換わらない）。(3) マニュアルの内容検証（対象バージョン行と `Cargo.toml` の版の照合を含む）が通る。
4. If 確認のいずれかが失敗したとき（対象バージョン行が見つからない場合を含む）, the Release Workflow shall 版の更新を取り消し、失敗した確認を示して作業を止める。
5. When 確認がすべて通ったとき, the Release Workflow shall 版の更新だけを含む 1 つのコミットを作る。

### Requirement 4: 版の更新の main への統合

**Objective:** As a 開発者, I want 版を上げたコミットを、ほかの変更と同じ PR の流れで main に入れたい, so that main のブランチ保護のもとでリリースが成り立ち、リリースだけの特別な統合方式を覚えなくて済む

#### Acceptance Criteria

1. When 版の更新のコミットを作ったとき, the Release Workflow shall PR を作り、PR 経由で main へ統合する。
2. The Release Workflow shall 版の更新を、spec の完了と同じ統合方式（squash）で main へ入れ、main の上で 1 つのコミットにする。マージコミット方式を使わない（リリースタグは統合の後に main のコミットへ付けるので、統合前のコミットを main から到達させる必要が無い）。
3. The Release Workflow shall main へ直接 push しない。
4. If PR の作成または統合が失敗したとき（衝突・統合できない状態・権限の不足など）, the Release Workflow shall リリースタグを作らず、強制 push・履歴の書き換え・統合の成功より前のブランチの削除を行わずに止まり、開発者に解消を求める。
5. When PR を統合したとき, the Release Workflow shall main の版の表記が確定した版になっていることを確かめてから、リリースタグの作成へ進む。

### Requirement 5: リリースタグの作成と push

**Objective:** As a 開発者, I want 版を上げた main のコミットにリリースタグを付けて push したい, so that リリース CI が起動し、タグがその版のソースを正しく指す

#### Acceptance Criteria

1. When 版の更新が main に入ったことを確かめたとき, the Release Workflow shall 版の更新を main へ統合した結果のコミットに、注釈付きのリリースタグ `vX.Y.Z` を、メッセージ `Release vX.Y.Z` で作る。
2. The Release Workflow shall リリースタグを、main から到達でき、かつ `Cargo.toml` と `editors/vscode/package.json` の版がタグの版と一致するコミットにだけ付ける（リリース CI が検査する条件を、push の前に満たす）。
3. If 同じ名前のタグがすでにあるとき, the Release Workflow shall エラーを示し、開発者に扱いを確かめる。既存のタグの削除・付け替えを自動では行わない（Requirement 7.6 の、承認を得た付け直しを除く）。
4. When リリースタグを作ったとき, the Release Workflow shall タグだけをリモートへ push する。
5. If リリースタグの push が失敗したとき, the Release Workflow shall 失敗の内容と、main には版の更新が入っていてタグだけが無い状態であることを示して止まる（Requirement 8 で再開できる）。

### Requirement 6: リリース CI の結果の確認

**Objective:** As a 開発者, I want タグを push した後、公開が最後まで進んだかをエージェントに見届けてほしい, so that 自分で Actions の画面を見張らなくても、公開先ごとの結果が分かる

#### Acceptance Criteria

1. When リリースタグを push したとき, the Release Workflow shall そのタグで起動したリリース CI の実行を特定する。
2. If リリース CI の起動を確かめられないとき, the Release Workflow shall 完了と報告せず、起動していないことを示して止まる。
3. While リリース CI が実行中のとき, the Release Workflow shall 実行が終わるまで結果を追い、完了と報告しない。
4. When リリース CI の実行が終わったとき, the Release Workflow shall 公開先ごと（crates.io はクレートごと）の結果（`published`・`skipped`・`failed`・`not-run`）と、GitHub Release の URL を読み取る。
5. When すべての公開先の結果が `published` か `skipped` で、GitHub Release の URL が示されているとき, the Release Workflow shall リリースを完了と判定する。
6. If 公開先の結果が示されていないとき（job が結果を書く前に失敗した場合）, the Release Workflow shall その公開先を失敗として扱い、その job の実行結果から原因を調べる。

### Requirement 7: 失敗への対応

**Objective:** As a 開発者, I want リリース CI が失敗したとき、失敗の種類に合った対応をエージェントに取ってほしい, so that 一時的な失敗は人手なしで先へ進み、人の判断が要る失敗は取り返しのつかない操作の前に止まる

#### Acceptance Criteria

1. If 公開先が一時的な原因（公開先の障害・ネットワーク・時間切れ）で失敗したとき, the Release Workflow shall 同じ実行の失敗した job を再実行し、Requirement 6 に従って結果を確かめ直す。
2. If 一時的な原因の失敗が、1 回のリリース作業の中で 3 回の再実行の後も残るとき **【仮定】**, the Release Workflow shall 自動の再実行をやめ、未完了として報告する。
3. If 失敗の原因が一回限りのセットアップにあるとき（認証の失敗・クレートが crates.io に未登録）, the Release Workflow shall 再実行せず、原因とセットアップの手順書の該当する節を示して止まる。
4. When 開発者がセットアップの設定を直したと伝えたとき, the Release Workflow shall 失敗した job の再実行から続ける。
5. If リリース CI が公開より前の段（タグの検査・関門・配布物のビルド）で失敗し、どの公開先にもその版が出ていないとき, the Release Workflow shall 原因と、何も公開していないことを報告して止まる。
6. When 公開より前の段の失敗の修正が main に入り、開発者がタグの付け直しを承認したとき, the Release Workflow shall どの公開先にもその版が無いことを確かめ直したうえで、同じ版のリリースタグを修正後の main のコミットへ付け直して push する。
7. If 再実行では直らない失敗があり、いずれかの公開先にその版がすでに出ているとき, the Release Workflow shall リリースタグを付け直さず、公開済みの公開先と未公開の公開先を示し、版を上げて出し直す必要があることを報告する。
8. The Release Workflow shall 公開済みのものを取り消したり、上書きしたりしない。
9. The Release Workflow shall 失敗の回避のために、手元から crates.io・Marketplace へ公開したり、GitHub Release を作ったりしない。リリース CI が手作業を求める場合（公開済みの Release への配布物の添付など）は、その内容を開発者に案内する。

### Requirement 8: 途中からの再開

**Objective:** As a 開発者, I want 途中で止まったリリースを、もう 1 度 `/kiro-impl release-workflow` を実行するだけで続きから進めたい, so that セッションが切れても、版を飛ばしたり二重に作業したりしない

#### Acceptance Criteria

1. When リリース作業が開始され、main の版にリリースタグが無く、どの公開先にもその版が出ておらず、版の指定が無いかその版と一致するとき, the Release Workflow shall 版の決定と版の更新を飛ばし、再開する版を開発者に示したうえで、リリースタグの作成（Requirement 5）から続ける。
2. When リリース作業が開始され、main の版のリリースタグが push 済みで、リリースが完了していないとき（リリース CI が実行中、または未完了で終わっている）, the Release Workflow shall 版の決定・版の更新・タグの作成を飛ばし、結果の確認（Requirement 6）と失敗への対応（Requirement 7）から続ける。
3. If 確定した版への版の更新の PR が、統合されないまま残っているとき, the Release Workflow shall 新しい PR を作らず、その PR を示して開発者に扱いを確かめる。
4. The Release Workflow shall 再開の位置を、前回の実行の記録ではなく、実際の状態（main の内容・リモートのタグ・公開先・リリース CI の実行）で判定する。
5. When main の版のリリースが完了しているとき, the Release Workflow shall 新しいリリースとして、版の決定（Requirement 1）から始める。

### Requirement 9: 完了の判定と報告

**Objective:** As a 開発者, I want リリースが終わったのか、途中なのかを、取り違えようのない形で知りたい, so that 一部だけ公開された状態を「完了」と思い込まない

#### Acceptance Criteria

1. The Release Workflow shall すべての公開先が公開済みになり、GitHub Release が作成されるまで、リリースを完了と報告しない。
2. When リリースが完了したとき, the Release Workflow shall 版・クレートごとの結果・Marketplace の結果・GitHub Release の URL・リリース CI の実行の URL・再実行の回数を開発者に報告する。
3. While 未完了の公開先が残ったまま作業を止めるとき, the Release Workflow shall 「未完了」として、残っている公開先・原因の種類・次に誰が何をするか・再開の方法を報告する。
4. When 公開より前の段で止めたとき, the Release Workflow shall どの公開先にも公開していないことを報告に含める。

### Requirement 10: 繰り返し実行の仕様特性

**Objective:** As a 開発者, I want この仕様を何度でも実行してリリースを行いたい, so that 毎回のリリースで同じ手順が保証される

#### Acceptance Criteria

1. The Release Workflow shall `/kiro-impl release-workflow` が実行されるたびに、タスクの状態を初期化する（すべてのタスクを未完了に戻す）。
2. The Release Workflow shall 本仕様を完了済みにせず、`completed/` へ移さない。
3. The Release Workflow shall 各実行を、前回の実行が残したタスクの完了印に依存しない作業として行う（途中からの再開は Requirement 8.4 の実際の状態だけに基づく）。

### Requirement 11: CI での初回のリリースの確認と後片付け

**Objective:** As a メンテナー, I want リリース CI での初回のリリースで、認証の配線と再実行の冪等性を実地で確かめ、手元に残った長期の認証情報の経路を閉じたい, so that 2026-12-01 の global PAT の廃止の後もリリースを続けられ、古い認証情報が残らない

#### Acceptance Criteria

1. While CI での初回のリリースの確認が済んだ記録が無いとき, when リリースが完了したとき, the Release Workflow shall 同じ実行のすべての job を再実行し、すべての公開先の結果が `skipped` になることを確かめる。
2. While CI での初回のリリースの確認が済んだ記録が無いとき, when リリースが完了したとき, the Release Workflow shall クレートごとの公開の所要時間を、認証の有効期限（30 分）に対する余裕とあわせて報告する。
3. When 初回のリリースの確認が通ったとき, the Release Workflow shall セットアップの手順書が定める必須の後片付け（5 クレートの「Trusted Publishing のみ」の有効化・`CARGO_REGISTRY_TOKEN` の失効・`VSCE_PAT` の失効）を開発者に案内する。自分では行わない。
4. The Release Workflow shall `VSCE_PAT` の失効を、Marketplace の結果が今回の実行で `published` になったことを確かめた後にだけ案内する。
5. When 開発者が後片付けの完了を伝えたとき, the Release Workflow shall 初回のリリースの確認と後片付けが済んだことをリポジトリに記録し、以後の実行で本要件の手順を行わない。
6. If 初回のリリースで、2 つ目以降のクレートの認証だけが失敗したとき, the Release Workflow shall セットアップの手順書の「auth の取り直しが拒否されたとき」にあたることを示し、Requirement 7.7 に従って報告する。

### Requirement 12: 手順の書き換えに伴う一回限りの整合

**Objective:** As a 開発者・エージェント, I want リリースに関わる文書と設定が、書き換えた手順と一致していてほしい, so that 古い手順（手元での公開・マージコミット方式・main の CI の確認）に従って誤った操作をしない

本要件は、本書き直しと同じ流れの中で 1 度だけ行う。繰り返し実行するリリースのタスクには含めない（行う場所と順は設計で決める）。

#### Acceptance Criteria

1. The repository shall エージェントへの操作の許可（`.claude/settings.json`）を、書き換えた手順が使う操作（PR の作成と統合・リリースタグの push・リリース CI の結果の閲覧と再実行）に合わせ、手元からの公開（crates.io・Marketplace・GitHub Release の作成）の許可とその説明を外す。
2. The 文書 shall steering `workflow.md` の「取り消せない公開の前に main の CI 全緑を確かめる」の記述を、公開前の検査をリリース CI が行う形に改める。
3. The 文書 shall steering `workflow.md` のリリースの例外の記述を、統合は squash で行い、リリースタグの push だけが直接 push の禁止の対象外である形に改める。
4. The 文書 shall `crates/pasta_sample_ghost/RELEASE.md` の版の更新の一覧に `Cargo.lock` と `editors/vscode/package-lock.json` を加え、タグを付けるコミットの説明を Requirement 5 に合わせる。
5. The repository shall 認証の失敗の案内文が指す手順書の見出しを、実際の見出し「名前の対応表」に合わせる（`.github/scripts/release/publish-vsix.ps1` の 1 か所と `release.yml` の 2 か所。案内文だけを変え、挙動を変えない）。
6. The 文書 shall steering の `product.md`・`roadmap.md` の本仕様の説明と、「リリース手順の書き換え」の進み具合を、書き換えた手順に合わせる。
7. The repository shall 本要件の変更の後も、`cargo test --all` と clippy が通る状態を保つ。

---

## 旧仕様（手元でビルド・公開する版）からの変更点

1. **手順を 4 段へ縮めた**: 版の決定 → 版の更新の統合 → リリースタグの push → リリース CI の結果の確認と再実行。旧 Requirement 3（crates.io 公開）・4（VSCode 拡張公開）・5（サンプルゴーストビルド）・7（GitHub Release 作成）はリリース CI へ移り、本仕様から外した。
2. **実行モデルと並行作業性（旧 Requirement 8）を廃止**: 手元のビルドと公開の 2 トラックが無くなり、共有リソースの調停が要らなくなった。
3. **統合方式をマージコミットから squash へ**: 旧仕様は、統合の前に作ったタグが指すコミットを main から到達させるために、マージコミット方式を必須にしていた。新しい手順はタグを統合の後に main のコミットへ付けるので、理由が無くなった（旧 Requirement 10 → Requirement 4・5）。
4. **順序の反転**: 旧仕様はタグの push を crates.io の公開の後に置いた。新しい手順ではタグの push が公開の入口である。
5. **完遂保証とスケジュール再試行（旧 Requirement 11）を縮小**: 公開の再試行と冪等性はリリース CI が持つ。エージェントは失敗した job の再実行と、未完了の報告だけを受け持つ（Requirement 7・9）。「一部だけ公開された状態を完了と報告しない」は引き継ぐ。
6. **Resume を「途中からの再開」へ縮小**: 公開の再開はリリース CI の再実行が担う。エージェントは、版の更新・タグ・リリース CI のどこまで進んだかを実際の状態で見分けるだけになった（旧 Requirement 9.5 → Requirement 8）。
7. **事前検証の縮小**: 手元での全テストの実行（旧 1.10）と main の CI が全部緑かの確認は、リリース CI の関門に置き換わった（Requirement 2.8）。未コミットの変更を「リリース準備コミット」にする挙動（旧 1.9）は、止まって示す挙動に改めた（Requirement 2.3）。
8. **版の更新の対象を追加**: `Cargo.lock`（`release-ci` で追跡を始めた。更新しないと配布物のビルドの段で検査に落ちる）と `editors/vscode/package-lock.json`（v0.3.7 では追いかけのコミットで直した）を加えた。手元でのビルドによる確認は、整合の確認に置き換えた（Requirement 3）。
9. **CI での初回のリリースを追加**: `release-ci` からの申し送り（Requirement 11）。
10. **一回限りの整合を要件にした**: 旧仕様は Boundary Context の注記で扱っていた（Requirement 12）。

## 未決事項

本文の **【仮定】** に対応する。

1. **未コミットの変更・main に無いコミットの扱い**（2.3）: 旧仕様はすべてを「リリース準備コミット」にしてリリースに含めた。仮定: 含めずに止まる（squash で版の更新のコミットに混ざり、リリースノートから見えなくなるため）。
2. **一時的な失敗の自動の再実行の回数**（7.2）: 仮定: 1 回のリリース作業の中で 3 回まで。それを超えたら未完了として報告し、開発者の指示か再度の実行（Requirement 8）で続ける。
3. **確認ワークフローのログに出る識別子**（Out of scope）: 仮定: 直さない。
4. **→ 設計へ**: `build.yml` に `--locked` を足すか（版の更新で `Cargo.lock` を更新し忘れた場合を、関門の段で検出するか、今のまま配布物のビルドの段で検出するか）。
5. **→ 設計へ**: 実行するモデルの前提（開発者の方針 2026-10-10）。初回（CI での初回のリリース）は Opus で実行し、実行が安定したら Sonnet で実行する。設計とタスクは、実行するコマンドと判定の表（リリース CI の結果 → 次の手）を明記し、モデルの推論に任せる箇所を残さない粒度で書く。
6. **→ 設計へ**: リリース CI の実行が終わるのを待つ方法、初回のリリースの確認が済んだ記録の置き場所、Requirement 12 を行う場所と順、古い `research.md`・`gap-analysis.md` の扱い。
