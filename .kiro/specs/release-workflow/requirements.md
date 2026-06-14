# Requirements Document

## Project Description (Input)

### リリース仕様（cc-sdd 3.0 書き直し版）

本仕様は、リリースのための手順を設計し、「実装」（`/kiro-impl release-workflow`）を実行するたびにリリース作業を行う、**繰り返しタスク**の仕様です。本仕様は実装完了しません。新たに「実装」が指示されるたび、タスクの実行状況は初期化され、新たな「リリース作業」を繰り返し行います。

本書き直しの主眼は **並行作業性（concurrency）の見直し** です。旧仕様は全工程を単一の Sequential Pipeline として直列実行していましたが、各処理が要求する**共有リソース**（cargo ターゲットロック、git ワークツリー、ネットワーク）を分析した結果、いくつかの処理は安全に並行実行でき、また旧仕様には**偽の依存関係**（crates.io 公開 → サンプルゴーストビルド）が存在することが判明しました。本仕様ではこれらを是正します。

**リリース対象**:

| 対象                                                        | 公開先             | 備考                               |
| ----------------------------------------------------------- | ------------------ | ---------------------------------- |
| pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check | crates.io          | 依存関係順に公開（クリティカル）   |
| pasta-vscode (VSCode 拡張)                                  | VSCode Marketplace | 隔離されるが完遂必須（Req 11 のスケジュール再試行で粘る） |
| hello-pasta.nar, pasta.dll.zip                              | GitHub Release     | リリースアセット                   |
| VSIX ファイル                                               | GitHub Release     | 存在する場合のみ添付               |

**開発者提供の手順概要**:

1. バージョン（1.2.0など）を開発者に確認する
2. Cargo.toml のバージョン表記および editors/vscode/package.json を更新し、build が通ることを確認してコミット
3. cargo publish する（依存関係順）
4. VSCode 拡張をビルド・公開する（非クリティカル）
5. サンプルゴーストをビルド（release.ps1 の実行）してコミット
6. バージョンタグをつける
7. git push
8. gh でリリースを作る。チェンジログは git の履歴からサルベージ。リリース時公開ファイルは pasta.dll.zip、hello-pasta.nar、および VSIX（存在する場合）

---

## Introduction

本ドキュメントは pasta プロジェクトのリリースワークフローに関する要件を定義する。このワークフローは LLM エージェントが開発者の指示のもとで繰り返し実行するリリース作業手順であり、crates.io への公開、VSCode 拡張の Marketplace 公開、サンプルゴーストのビルド、GitHub Release の作成までを一貫して行う。

本仕様では、上記の各処理を「**何を**達成するか」（本要件書）と「**どの順序・どの並行度で**実行するか」（design.md の実行モデル）に分離して扱う。並行作業性に関する振る舞い（並行実行可否、失敗隔離、順序保証）は Requirement 8 に集約する。

### 仕様の特殊性

本仕様は通常の機能仕様と異なり、以下の特性を持つ：

- **繰り返し実行型**: `/kiro-impl release-workflow` が実行されるたびにタスク状態はリセットされ、新たなリリース作業として実行される
- **永続的未完了**: 本仕様は `completed` に移行しない。常に `ready_for_implementation` 状態を維持する
- **パラメータ依存**: 各実行時にバージョン番号が開発者から提供される
- **オペレーション仕様**: コードの新規作成・変更を伴わず、既存ツール群（cargo / git / gh / npm / release.ps1）の組み合わせで実現する
- **ワークツリー実行型**: Claude Code ハーネスが供給するワークツリー（非デフォルトの作業ブランチ）上で起動される。生成したリリースコミットとタグは、spec 完了で用いる squash-PR フローではなく、**PR のマージコミット方式（`--merge`）** で main へ統合し、コミット SHA とタグ参照の整合を保つ。main への直接 push は行わない（将来の GitHub ブランチ保護に前方互換）（Requirement 10）

## Boundary Context

- **In scope**: Cargo.toml / package.json のバージョン更新、crates.io 公開（5クレート）、VSCode Marketplace 公開、サンプルゴーストビルド、Git タグ・プッシュ、GitHub Release 作成、これらの**実行順序と並行スケジューリング**、ならびに**ワークツリー（非デフォルトブランチ）上での実行と、リリースコミット・タグの PR マージコミット方式（非 squash）による main 統合**
- **Out of scope**: CI/CD パイプライン統合、クロスプラットフォーム対応、認証トークンの自動設定、pasta_lsp の独立リリース管理、release.ps1 スクリプト自体の修正、**spec 完了の squash-PR 統合フロー**（リリースは別系統のため対象外）、**GitHub ブランチ保護設定そのものの構成**（本仕様は保護下でも成立する手順を定めるが、保護ルールの設定作業は対象外）
- **Adjacent expectations**: `release.ps1` は既存の成熟スクリプトとしてそのまま利用する。`gh` CLI および `cargo` / `vsce` の認証は事前に設定済みであることを前提とする。フィーチャーブランチ／ワークツリーは Claude Code ハーネスが供給する。リリースの main 統合は **PR ベース**（マージコミット方式 `--merge`、squash を行わない）で行い、将来 GitHub 側で main への直接 push を禁止しても成立させる。なお steering `workflow.md` のリリースカーブアウト改訂、`.claude/settings.json` のタグ push 許可追加・`git push origin main` 許可の縮退、および repo の merge-commit 有効化（必要時）は、**繰り返し実行されるリリース手順には含めない一回限りのセットアップ**として扱い、本 spec の設計確定後（タスク分解の前後）に**手動で実施**する（ワークツリー隔離のため別セッションへは委譲しない）。整合は Steering Gate でも確認する

---

## Requirements

### Requirement 1: バージョン確認と事前検証

**Objective:** As a 開発者, I want リリース前にバージョン番号を確定し、ワークツリーとテストの健全性を保証したい, so that リリース作業が一貫した状態から開始される

#### Acceptance Criteria

1. When リリース作業が開始される and バージョン番号が指定されている, the Release Workflow shall 指定されたバージョン番号を使用する
2. When リリース作業が開始される and バージョン番号が指定されていない, the Release Workflow shall 全バージョンソース（Cargo.toml、package.json、Git タグ、crates.io、GitHub Releases、VSCode Marketplace）を調査し、最大バージョンの PATCH を +1 した値を提案バージョンとして算出する
3. When 提案バージョンが算出される, the Release Workflow shall 全ソースの調査結果と提案バージョンを開発者に報告し承認を求める
4. If 開発者が提案バージョンを承認しない, the Release Workflow shall 開発者に希望するバージョン番号の入力を求める
5. When バージョン番号が提供される, the Release Workflow shall semver 形式（例: `1.2.0`）として妥当性を検証する
6. If バージョン番号が semver 形式でない, the Release Workflow shall エラーを報告し再入力を求める
7. When バージョン番号が確定する, the Release Workflow shall 全バージョンソースに対して重複チェックを行い、同一バージョンが既に存在する場合はエラーを報告し別のバージョン番号の入力を求める。If 当該バージョンが本ワークフローの統合済み未完了リリース（main 統合済みだが完全公開に至っていない）である, the Release Workflow shall エラーとせず Requirement 9.5 の resume モードに従って再開する
8. When リリース作業が開始される, the Release Workflow shall ワークツリーに未コミットの変更があるか確認する
9. If 未コミットの変更が存在する, the Release Workflow shall すべての変更をリリース準備コミットとしてコミットする
10. When リリース作業が開始される, the Release Workflow shall 全テストを実行し通過を確認する
11. If テストが失敗する, the Release Workflow shall リリース作業を中止し失敗内容を報告する

### Requirement 2: バージョン更新

**Objective:** As a 開発者, I want ワークスペース全体と関連プロジェクトのバージョンを一括更新したい, so that 全クレートおよび VSCode 拡張のバージョンが同期される

#### Acceptance Criteria

1. When バージョン番号が確定する, the Release Workflow shall `Cargo.toml`（ワークスペースルート）の `[workspace.package].version` フィールドを新バージョンに更新する
2. When ワークスペースバージョンが更新される, the Release Workflow shall `[workspace.dependencies]` セクション内の内部クレート参照（`pasta_core`, `pasta_dsl`, `pasta_lua`, `pasta_shiori`, `pasta_check`）の `version` フィールドも同じバージョンに更新する
3. When Cargo.toml が更新される, the Release Workflow shall `editors/vscode/package.json` の `version` フィールドも同じバージョンに更新する
4. When バージョン更新が完了する, the Release Workflow shall ワークスペース全体のビルドを実行しビルドが成功することを確認する
5. If ビルドが失敗する, the Release Workflow shall バージョン変更をロールバックし、エラーを報告する
6. When ビルドが成功する, the Release Workflow shall バージョン更新をコミットする

### Requirement 3: crates.io 公開

**Objective:** As a 開発者, I want 依存関係の順序を考慮して全公開クレートを crates.io に公開したい, so that 下流ユーザーが最新版を利用できる

#### Acceptance Criteria

1. When ローカルビルドが完了しワークツリーがクリーンで、かつ main 統合（タグ作成・PR マージ）が成功している, the Release Workflow shall クレートを依存関係順（`pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check`）に公開する
2. When クレートを公開する, the Release Workflow shall 各クレートの公開成功を確認してから次のクレートに進む
3. If クレートの公開が失敗する, the Release Workflow shall 段階的バックオフでリトライを試みる（待機時間を1分から1分ずつ増加し最大10分まで、最大10回リトライ）
4. If セッション内の段階的バックオフを使い切っても失敗する, the Release Workflow shall 既に公開されたクレートはそのまま残し、一時障害なら Requirement 11 のスケジュール再試行へ、非一時障害なら原因を報告する。いずれの場合もリリースを完了済みとしない（未公開クレートを残したまま完了しない）
5. While `pasta_sample_ghost` は `publish = false` である, the Release Workflow shall このクレートの公開をスキップする
6. When 前のクレートを公開した直後, the Release Workflow shall crates.io のインデックス更新を待つため待機時間を設ける

### Requirement 4: VSCode 拡張公開

**Objective:** As a 開発者, I want VSCode 拡張を Marketplace に公開し、リリースに VSIX を含めたい, so that ユーザーが最新の拡張機能を利用できる

#### Acceptance Criteria

1. When ローカルビルドステージが実行される, the Release Workflow shall VSCode 拡張のビルド（パッケージング）を実行する
2. When パッケージングが成功する, the Release Workflow shall VSIX ファイルが生成されたことを確認しパスを記録する
3. When VSIX ファイルが存在する, the Release Workflow shall VSCode Marketplace への公開を実行する
4. If Marketplace 公開が失敗する, the Release Workflow shall まずセッション内の段階的バックオフでリトライを試みる
5. If セッション内の段階的バックオフを使い切っても Marketplace 公開が失敗する, the Release Workflow shall リリースを完了とせず、Requirement 11 のスケジュール再試行に委ねて未完了として扱う（他トラックの進行は妨げない＝隔離は維持するが完遂は必須）
6. If VSCode 拡張のビルドが失敗する, the Release Workflow shall 一時障害なら Requirement 11 のスケジュール再試行へ、非一時障害（ビルドエラー等）なら未完了として原因を報告する。いずれの場合もリリースを完了済みとしない（VSIX 未生成のまま完了しない）
7. When Marketplace 公開が成功する, the Release Workflow shall 公開結果（Marketplace URL）を記録する

### Requirement 5: サンプルゴーストビルド

**Objective:** As a 開発者, I want リリースバージョンの pasta.dll を使ってサンプルゴーストをビルドしたい, so that リリースに最新の .nar ファイルを含められる

#### Acceptance Criteria

1. When バージョン更新コミットが完了する, the Release Workflow shall サンプルゴーストのビルドスクリプトを実行する
2. When ビルドスクリプトが成功する, the Release Workflow shall .nar ファイルが生成されたことを確認する
3. When ビルドスクリプトが成功する, the Release Workflow shall 32bit リリースビルドの DLL が存在することを確認する
4. If ビルドスクリプトが失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
5. When DLL の存在が確認される, the Release Workflow shall DLL を zip 圧縮する
6. When zip 圧縮が完了する, the Release Workflow shall zip ファイルの存在を確認する
7. If zip 圧縮が失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
8. When ゴーストビルドが成功する, the Release Workflow shall 変更をコミットする
9. While サンプルゴーストビルドはローカルソースから pasta.dll をビルドする, the Release Workflow shall このビルドを crates.io 公開（Requirement 3）の完了に依存させない

### Requirement 6: バージョンタグとプッシュ

**Objective:** As a 開発者, I want Git タグでリリースポイントを記録し、リモートに反映したい, so that リリースのトレーサビリティが確保される

#### Acceptance Criteria

1. When 全ローカルビルド・コミットが完了し、作業ブランチが main へマージ可能である, the Release Workflow shall `vX.Y.Z` 形式のアノテーションタグを作成する
2. When タグが作成される, the Release Workflow shall タグメッセージに `Release vX.Y.Z` を設定する
3. If 同名のタグが既に存在する, the Release Workflow shall エラーを報告し開発者に対応方法を確認する（既存タグの削除は自動実行しない）
4. When タグが作成される, the Release Workflow shall 作業ブランチのリリースコミットを、squash を行わず PR のマージコミット方式でデフォルトブランチ（main）へ統合する。注釈タグの push は crates.io 公開成功後に行い、リモートのタグが常に公開済みを含意するようにする（統合方式は Requirement 10 に従う）
5. If プッシュが失敗する, the Release Workflow shall エラーを報告し手動での対応を開発者に促す

### Requirement 7: GitHub Release 作成

**Objective:** As a 開発者, I want チェンジログ付きの GitHub Release を自動作成し、ビルド成果物を添付したい, so that ユーザーがリリースを容易に取得できる

#### Acceptance Criteria

1. When タグのプッシュが完了する, the Release Workflow shall 前回リリースから今回までのコミット履歴を取得する
2. When コミット履歴が取得される, the Release Workflow shall Conventional Commits 形式に基づいてコミットを種別ごとに分類・グループ化する
3. When チェンジログを整形する, the Release Workflow shall 各グループを見出し配下に箇条書きで配置する
4. When GitHub Release を作成する, the Release Workflow shall タイトルを `pasta vX.Y.Z` に設定する
5. When GitHub Release を作成する, the Release Workflow shall 整形済みチェンジログをリリースノートとして含める
6. When GitHub Release を作成する, the Release Workflow shall DLL zip ファイルおよび .nar ファイルをリリースアセットとして添付する
7. Where VSIX ファイルが存在する, the Release Workflow shall VSIX ファイルもリリースアセットとして添付する
8. If GitHub Release の作成が失敗する, the Release Workflow shall エラーを報告し手動での Release 作成手順を案内する
9. If 前回リリースタグが存在しない（初回リリース）, the Release Workflow shall 全コミット履歴をチェンジログとして使用する

### Requirement 8: 実行モデルと並行作業性

**Objective:** As a 開発者, I want リリース作業が共有リソースの制約を尊重しつつ、安全に並行化されて実行されたい, so that リリース全体の所要時間が短縮され、非クリティカルな失敗が全体を止めず、不可逆な処理の順序安全性が保たれる

> **背景**: 各処理は 3 種の共有リソースを要求する — **R1: cargo ターゲットロック**（`cargo build/publish/run`、VSCode の `build:wasm` が保持）、**R2: git ワークツリー＋index**（ファイル生成・add/commit/restore/tag が保持）、**R3: ネットワーク**（crates.io / Marketplace / GitHub、実質無制限の並行可能）。R1・R2 は単一保持の排他リソースであり、これを共有する処理は真の並行実行ができない。

#### Acceptance Criteria

1. When リリース作業をスケジュールする, the Release Workflow shall 各処理を要求リソース（R1 cargo ロック / R2 ワークツリー / R3 ネットワーク）で分類し、排他リソースを共有する処理を直列化する
2. While ワークツリーを変更するローカルビルド（バージョン更新ビルド、サンプルゴーストビルド、VSCode 拡張パッケージング）が R1・R2 を共有する, the Release Workflow shall これら全ローカルビルドとコミットを完了しワークツリーをクリーン化してから main 統合（タグ作成・PR マージ）および crates.io 公開（R3 を要し R2 のクリーン状態を前提とする）を開始する
3. Where crates.io 公開・Marketplace 公開・チェンジログ生成は互いに独立しワークツリーを変更しない, the Release Workflow shall これらを並行（concurrent）に実行してよい
4. If Marketplace 公開が失敗する, the Release Workflow shall 他の処理（crates.io 公開、タグ・プッシュ、GitHub Release）の進行を妨げず継続する（失敗隔離）。ただし Marketplace 公開はリリース完遂の必須要素であり、未完了のまま全体を完了済みとせず Requirement 11 のスケジュール再試行で完遂する
5. While main 統合（タグ作成・PR マージ）は revert で可逆だが crates.io 公開は不可逆である, the Release Workflow shall 「main 統合 → crates.io 公開 → GitHub Release 作成」の順で実行し、不可逆な crates.io 公開を可逆な main 統合の後段に置く。If main 統合が失敗する, the Release Workflow shall crates.io 公開および GitHub Release を実行しない。If main 統合の成功後に crates.io 公開が失敗する, the Release Workflow shall 統合済み main 状態（コミット・タグ）を保持したまま公開をリトライまたは中断して報告し、GitHub Release は crates.io 公開成功まで作成しない（安全順序保証。統合方式は Requirement 10 に従う）
6. The Release Workflow shall 独立した処理を不要に直列化しない（偽の依存関係の排除）。特にサンプルゴーストビルドを crates.io 公開の後段に配置しない
7. When 並行実行する処理のいずれかがバックグラウンドで進行する, the Release Workflow shall 各並行トラックの完了・失敗を個別に検証し、結果をサマリーに反映する

### Requirement 9: 繰り返し実行の仕様特性

**Objective:** As a 開発者, I want この仕様を何度でも再実行してリリース作業を行いたい, so that 毎回のリリースで同じ品質の手順が保証される

#### Acceptance Criteria

1. The Release Workflow shall `/kiro-impl release-workflow` が実行されるたびにタスク状態を初期化（全タスクを未完了に戻す）する
2. The Release Workflow shall spec.json の `phase` を `completed` に変更しない（常に `ready_for_implementation` を維持する）
3. The Release Workflow shall 各実行が前回の実行状態に依存しない独立した作業として動作する
4. When リリース作業が完了する（全ターゲット完遂時）, the Release Workflow shall 実行結果のサマリー（バージョン、公開クレート、Release URL、Marketplace 公開結果、各並行トラックの成否）を開発者に報告する。While 未完了ターゲットが残る, the Release Workflow shall 「未完了（再試行待ち）」として残作業とスケジュール状態を報告する（完了済みと報告しない）
5. When 再実行時に main の現行バージョンが完全公開（全公開クレートが crates.io に存在・タグ push 済み・GitHub Release 作成済み）に至っていないことを検出する, the Release Workflow shall バージョン再決定・バージョン更新・main 統合をスキップし、未完了の crates.io 公開・Marketplace 公開・タグ push・GitHub Release 作成を冪等に再開する（resume モード）

### Requirement 10: ワークツリー実行と PR ベース main 統合

**Objective:** As a 開発者, I want リリースワークフローを Claude Code のワークツリー（非デフォルトの作業ブランチ）上で起動し、生成されたリリースコミットとタグを PR 経由で main に統合したい, so that ハーネスのワークツリー隔離環境でリリースを実行でき、将来 main への直接 push を禁止してもリリースが成立し、かつタグと公開物の参照整合性が保たれる

> **背景**: Claude Code ハーネスはリリース作業を非デフォルトのワークツリーブランチ上で起動する。一方、リリースは複数のコミット（prepare / bump / ghost build）と、特定コミットを指す注釈タグ `vX.Y.Z` を生成する。これらを spec 完了用の squash-PR（`--squash`）や rebase でマージするとコミット SHA が書き換わり、タグが main から到達不能な孤児コミットを指し（`git describe`・Release のコミットリンク・チェンジログ compare URL が破綻）、不可逆な crates.io 公開内容のアンカーも失われる。本要件はこれを **PR のマージコミット方式（`--merge`）** で回避し、将来の GitHub ブランチ保護（main 直接 push 禁止）にも前方互換とする。

#### Acceptance Criteria

1. When リリース作業が開始される, the Release Workflow shall ハーネスが供給する現在の作業ブランチ（ワークツリーブランチ）上で動作し、main ブランチ上での実行や main への直接 push を前提条件としない
2. While リリースコミット（prepare / bump / ghost build）が作業ブランチ上に作成される, the Release Workflow shall これらを作業ブランチに保持し、main への統合を統合フェーズ（全ローカルビルド完了後・crates.io 公開前）でのみ行う
3. When 作業ブランチのコミットを main へ統合する, the Release Workflow shall PR を作成し、マージコミット方式（`--squash` でも `--rebase` でもない）でマージして、各リリースコミットの SHA を保持したまま main から到達可能にする
4. The Release Workflow shall spec 完了で用いる squash-PR フロー（`--squash`）を使用せず、main への直接 push も行わない
5. When 注釈タグを作成する, the Release Workflow shall タグが統合後の main から到達可能なコミット（リリース HEAD コミット）を指すことを保証し、タグ参照の push は crates.io 公開成功後（GitHub Release 作成の直前）に行う
6. When main への統合（タグ作成・PR マージ）が完了する, the Release Workflow shall その成功を確認してから不可逆な crates.io 公開を開始する
7. If main への統合（PR の作成またはマージ）が失敗する（コンフリクト・mergeable でない・権限不足等）, the Release Workflow shall crates.io 公開を実行せず、force push・リモート履歴の書き換え・マージ成功前のブランチ削除を行わずに中断し、開発者に解消を求める
8. If main 統合の成功後に crates.io 公開が失敗する, the Release Workflow shall 統合済みの main 状態（コミット反映済み・タグはローカル保持）を維持したまま、セッション内バックオフ → Requirement 11 のスケジュール再試行で未公開分を完遂まで再試行する（既公開クレートは残す）
9. When ローカルビルドの前に作業ブランチが main より遅れている（main が先行している）ことを検出する, the Release Workflow shall main を作業ブランチへ非破壊マージで取り込み、更新後のツリー上でビルドと公開を行う。If 取り込みでコンフリクトが生じる, the Release Workflow shall リリース作業を中止し開発者に解消を求める

### Requirement 11: 完遂保証とスケジュール永続リトライ

**Objective:** As a 開発者, I want 相手側サーバーのビジー等で失敗しやすい手順（特に Marketplace 公開）を、時間がかかってもスケジュール再試行で完遂まで自動的に粘ってほしい, so that 中途半端な状態でリリースが「完了」することが決して起きない

> **背景**: リリースの各公開先（crates.io / VSCode Marketplace / GitHub）は相手側サーバーのビジー・レート制限・一時的ネットワーク障害で失敗し得る。基本方針は「時間はいくらでもかかってよいが、中途半端な状態での完了は避ける」。よって有限回で打ち切って一部未公開のまま完了する従来方針を改め、**全ターゲット完遂まで（短期バックオフ→スケジュール再試行の二段で）粘る**。

#### Acceptance Criteria

1. The Release Workflow shall すべてのリリースターゲット（全公開クレートの crates.io 公開、VSCode Marketplace 公開、タグ push、GitHub Release 作成）が成功するまで、リリースを「完了」と報告しない（完遂保証 / no half-done）
2. When 外部サービス通信が一時障害（サーバービジー・レート制限・ネットワーク等）で失敗する, the Release Workflow shall まずセッション内で段階的バックオフによる短期リトライを行う
3. If 短期バックオフの一巡で未完了ターゲットが残る, the Release Workflow shall 同一セッション内で ScheduleWakeup により次回再試行時刻まで待機し、再開して未完了分の続行を全ターゲット完遂まで繰り返す（セッションを開いている限り継続。完遂前にセッションが終了した場合は手動再実行が Requirement 9.5 の resume モードで続行する）
4. When 待機から再開する, the Release Workflow shall 各ターゲットの実状態を確認して未完了分のみを冪等に再試行し、全ターゲット完遂で待機ループを終了する
5. While 未完了ターゲットが残る, the Release Workflow shall リリースを「未完了（再試行待ち）」状態として報告し、完了済みと誤認させない
6. If 失敗が非一時的（認証無効・権限不足・ビルドエラー等、リトライで解消しない種別）である, the Release Workflow shall リリースを完了とせず、原因と必要な対応を開発者に報告し、対応後に resume で完遂できる状態を保つ
7. The Release Workflow shall リトライ回数・累計時間に固定上限を設けず、全ターゲット完遂または開発者の明示的中止まで再試行を継続する（所要時間は完了条件としない）。While 自律継続は同一セッション／ScheduleWakeup ループの寿命（約7日）に律速される, the Release Workflow shall それを超える場合は手動再実行の resume モード（Requirement 9.5）で継続できる状態を保つ
8. While 第2段スケジュール再試行が継続中である, the Release Workflow shall 一定回数または一定経過ごとに開発者へ進捗（失敗継続中である旨・累計試行回数・最終エラー・障害分類）を通知し、試行履歴を記録する（無限リトライを可観測にし「完遂待ち」と「実質詰み」を判別可能にする）

---

## 旧仕様（直列 Pipeline 版）からの変更点

1. **実行モデルの再設計（Req 8 新設）**: 全工程を Sequential Pipeline として直列実行していた旧設計を、共有リソース（R1 cargo ロック / R2 ワークツリー / R3 ネットワーク）に基づく **リソース認識型ステージ並行モデル** へ書き直し。並行実行可否・失敗隔離・順序安全性を要件として明文化
2. **偽の依存関係の排除（Req 5.9, Req 8.6）**: 旧設計はサンプルゴーストビルドを crates.io 公開の後段（Phase 3 → Phase 5）に置いていたが、`release.ps1` はローカルソースから pasta.dll をビルドしており crates.io 公開に依存しない。この偽の依存を排除し、ゴーストビルドをバージョン更新直後のローカルビルドステージへ移動
3. **公開トラックの並行化（Req 8.3）**: crates.io 公開・Marketplace 公開・チェンジログ生成は互いに独立しワークツリーを変更しないため、並行実行を許可。特に非クリティカルな Marketplace 公開を crates.io 公開のネットワーク待機に重ねることで wall-clock を短縮
4. **VSCode ビルドと公開の分離（Req 4.1–4.2 vs 4.3–4.7）**: `build:wasm` は cargo（R1）を要するためローカルビルドステージで実施し、Marketplace への upload（R3 のみ）は並行公開ステージへ分離
5. **安全順序保証の明文化（Req 8.5）**: 不可逆な crates.io 公開をタグ・プッシュ／GitHub Release より前に完了させ、crates.io 公開中断時はタグ・プッシュを行わないことを要件化
6. **cc-sdd 3.0 タスク注釈の採用**: tasks.md に `(P)` 並行マーカー、`_Depends:_`、`_Boundary:_` を導入し、並行実行可能なタスクと依存関係を明示
7. **番号体系**: 旧 Req 8（繰り返し実行）→ Req 9 に繰り下げ（Req 8 を実行モデルに割当）
8. **ワークツリー実行と PR ベース main 統合の追加（Req 10 新設）**: Claude Code ハーネスのワークツリー（非デフォルトブランチ）上での起動を前提とし、リリースコミット・タグを **PR のマージコミット方式（`--merge`）** で main へ統合する要件を新設。spec 完了用の squash-PR や直接 push を排除し、コミット SHA とタグの参照整合性を保ちつつ将来の GitHub ブランチ保護に前方互換とする。これに伴い Req 6.4（タグ・プッシュ）の統合方式を更新し、Boundary Context に統合方針を明記。さらに**安全順序を「main 統合 → crates.io 公開 → GitHub Release」へ反転**し、不可逆な crates.io 公開を可逆な main 統合の後段に配置（Req 8.5・8.2・6.1・3.1・10 AC6–8 を更新。旧仕様は crates.io 公開を先行させていた）
