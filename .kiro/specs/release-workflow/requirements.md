# Requirements Document

## Project Description (Input)

### リリース仕様

本仕様は、リリースのための手順を設計し、「実装」（`/kiro-impl release-workflow`）を実行するたびにリリース作業を行う、**繰り返しタスク**の仕様です。本仕様は実装完了しません。新たに「実装」が指示されるたび、タスクの実行状況は初期化され、新たな「リリース作業」を繰り返し行います。

**リリース対象**:

| 対象                                                        | 公開先             | 備考                               |
| ----------------------------------------------------------- | ------------------ | ---------------------------------- |
| pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check | crates.io          | 依存関係順に公開                   |
| pasta-vscode (VSCode 拡張)                                  | VSCode Marketplace | 非クリティカル（失敗時も後続継続） |
| hello-pasta.nar, pasta.dll.zip                              | GitHub Release     | リリースアセット                   |
| VSIX ファイル                                               | GitHub Release     | 存在する場合のみ添付               |

**開発者提供の手順概要（更新版）**:

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

### 仕様の特殊性

本仕様は通常の機能仕様と異なり、以下の特性を持つ：

- **繰り返し実行型**: `/kiro-impl release-workflow` が実行されるたびにタスク状態はリセットされ、新たなリリース作業として実行される
- **永続的未完了**: 本仕様は `completed` に移行しない。常に `ready_for_implementation` 状態を維持する
- **パラメータ依存**: 各実行時にバージョン番号が開発者から提供される

## Boundary Context

- **In scope**: Cargo.toml / package.json のバージョン更新、crates.io 公開（5クレート）、VSCode Marketplace 公開、サンプルゴーストビルド、Git タグ・プッシュ、GitHub Release 作成
- **Out of scope**: CI/CD パイプライン統合、クロスプラットフォーム対応、認証トークンの自動設定、pasta_lsp の独立リリース管理
- **Adjacent expectations**: `release.ps1` は既存の成熟スクリプトとしてそのまま利用する。`gh` CLI および `cargo` の認証は事前に設定済みであることを前提とする

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
7. When バージョン番号が確定する, the Release Workflow shall 全バージョンソースに対して重複チェックを行い、同一バージョンが既に存在する場合はエラーを報告し別のバージョン番号の入力を求める
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

1. When バージョン更新コミットが完了する, the Release Workflow shall クレートを依存関係順（`pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check`）に公開する
2. When クレートを公開する, the Release Workflow shall 各クレートの公開成功を確認してから次のクレートに進む
3. If クレートの公開が失敗する, the Release Workflow shall 段階的バックオフでリトライを試みる（待機時間を1分から1分ずつ増加し最大10分まで、最大10回リトライ）
4. If 最大リトライ後も失敗する, the Release Workflow shall エラーを報告し、以降の公開を中断し、既に公開されたクレートはそのまま残し、開発者の指示を待つ
5. While `pasta_sample_ghost` は `publish = false` である, the Release Workflow shall このクレートの公開をスキップする
6. When 前のクレートを公開した直後, the Release Workflow shall crates.io のインデックス更新を待つため待機時間を設ける

### Requirement 4: VSCode 拡張公開

**Objective:** As a 開発者, I want VSCode 拡張を Marketplace に公開し、リリースに VSIX を含めたい, so that ユーザーが最新の拡張機能を利用できる

#### Acceptance Criteria

1. When crates.io 公開が完了する, the Release Workflow shall VSCode 拡張のビルド（パッケージング）を実行する
2. When パッケージングが成功する, the Release Workflow shall VSIX ファイルが生成されたことを確認する
3. When VSIX ファイルが存在する, the Release Workflow shall VSCode Marketplace への公開を実行する
4. If Marketplace 公開が失敗する, the Release Workflow shall 段階的バックオフでリトライを試みる
5. If 最大リトライ後も公開が失敗する, the Release Workflow shall 警告を記録し後続のフェーズへ継続する（非クリティカル）
6. If VSCode 拡張のビルドが失敗する, the Release Workflow shall 警告を記録し後続のフェーズへ継続する（非クリティカル）
7. When Marketplace 公開が成功する, the Release Workflow shall 公開結果（Marketplace URL）を記録する

### Requirement 5: サンプルゴーストビルド

**Objective:** As a 開発者, I want リリースバージョンの pasta.dll を使ってサンプルゴーストをビルドしたい, so that リリースに最新の .nar ファイルを含められる

#### Acceptance Criteria

1. When VSCode 拡張公開フェーズが完了する（成功・失敗問わず）, the Release Workflow shall サンプルゴーストのビルドスクリプトを実行する
2. When ビルドスクリプトが成功する, the Release Workflow shall .nar ファイルが生成されたことを確認する
3. When ビルドスクリプトが成功する, the Release Workflow shall 32bit リリースビルドの DLL が存在することを確認する
4. If ビルドスクリプトが失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
5. When DLL の存在が確認される, the Release Workflow shall DLL を zip 圧縮する
6. When zip 圧縮が完了する, the Release Workflow shall zip ファイルの存在を確認する
7. If zip 圧縮が失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
8. When ゴーストビルドが成功する, the Release Workflow shall 変更をコミットする

### Requirement 6: バージョンタグとプッシュ

**Objective:** As a 開発者, I want Git タグでリリースポイントを記録し、リモートに反映したい, so that リリースのトレーサビリティが確保される

#### Acceptance Criteria

1. When ゴーストビルドのコミットが完了する, the Release Workflow shall `vX.Y.Z` 形式のアノテーションタグを作成する
2. When タグが作成される, the Release Workflow shall タグメッセージに `Release vX.Y.Z` を設定する
3. If 同名のタグが既に存在する, the Release Workflow shall エラーを報告し開発者に対応方法を確認する（既存タグの削除は自動実行しない）
4. When タグが作成される, the Release Workflow shall コミットとタグの両方をリモートにプッシュする
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

### Requirement 8: 繰り返し実行の仕様特性

**Objective:** As a 開発者, I want この仕様を何度でも再実行してリリース作業を行いたい, so that 毎回のリリースで同じ品質の手順が保証される

#### Acceptance Criteria

1. The Release Workflow shall `/kiro-impl release-workflow` が実行されるたびにタスク状態を初期化（全タスクを未完了に戻す）する
2. The Release Workflow shall spec.json の `phase` を `completed` に変更しない（常に `ready_for_implementation` を維持する）
3. The Release Workflow shall 各実行が前回の実行状態に依存しない独立した作業として動作する
4. When リリース作業が完了する, the Release Workflow shall 実行結果のサマリー（バージョン、公開クレート、Release URL、Marketplace 公開結果）を開発者に報告する

---

## 旧仕様からの変更点

1. **VSCode 拡張公開（Req 4）**: design.md / tasks.md で VSX.1–VSX.6 として参照されていたが、requirements.md に正式な要件として未定義だった。本更新で Requirement 4 として正式化
2. **package.json バージョン更新（Req 2.3）**: Cargo.toml と同期して editors/vscode/package.json のバージョンも更新する要件を追加
3. **全バージョンソース調査（Req 1.2, 1.7）**: 単一ソースではなく全バージョンソースを調査し重複を防止する要件を明確化
4. **コマンド参照**: `/kiro-spec-impl` → `/kiro-impl` に更新
5. **番号体系**: 旧 Req 4（サンプルゴースト）→ Req 5、旧 Req 5（タグ）→ Req 6、旧 Req 6（GitHub Release）→ Req 7、旧 Req 7（繰り返し）→ Req 8 に繰り下げ
6. **実装詳細の除去**: 具体的なコマンド文字列・ファイルパス・コミットメッセージ形式を要件から除去し、設計フェーズに委譲
7. **Boundary Context 追加**: In scope / Out of scope / Adjacent expectations を明示
8. **EARS 準拠の強化**: 重複番号（旧 Req 3 の「6」が2つ）、旧 Req 3.7 の循環参照を解消
