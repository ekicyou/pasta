# Requirements Document

## Project Description (Input)

### リリース仕様

本仕様は、リリースのための手順を設計し、「実装」を実行するたびにリリース作業を行う、**繰り返しタスク**の仕様です。本仕様は実装完了しません。新たに「実装」が指示されるたび、タスクの実行状況は初期化され、新たな「リリース作業」を繰り返し行います。

**開発者提供の手順概要**:

1. バージョン（1.2.0など）を開発者に確認する
2. Cargo.tomlのバージョン表記を更新し、buildが通ることを確認してコミット
3. cargo publishする
4. サンプルゴーストをビルド（release.ps1の実行）してコミット
5. バージョンタグをつける
6. git push
7. gh でリリースを作る。チェンジログはgitの履歴からサルベージ。リリース時公開ファイルはpasta.dllと、hello-pasta.nar

---

## Introduction

本ドキュメントは pasta プロジェクトのリリースワークフローに関する要件を定義する。このワークフローは LLM エージェントが開発者の指示のもとで繰り返し実行するリリース作業手順であり、crates.io への公開、サンプルゴーストのビルド、GitHub Release の作成までを一貫して行う。

### 仕様の特殊性

本仕様は通常の機能仕様と異なり、以下の特性を持つ：

- **繰り返し実行型**: `/kiro-spec-impl release-workflow` が実行されるたびにタスク状況はリセットされ、新たなリリース作業として実行される
- **永続的未完了**: 本仕様は `completed` に移行しない。常に `ready_for_implementation` 状態を維持する
- **パラメータ依存**: 各実行時にバージョン番号が開発者から提供される

---

## Requirements

### Requirement 1: バージョン確認と事前検証

**Objective:** As a 開発者, I want リリース前にバージョン番号を指定し、ワークツリーが汚れていないことを保証したい, so that リリース作業が一貫した状態から開始される

#### Acceptance Criteria

1. When リリース作業が開始される and バージョン番号が指定されている, the Release Workflow shall 指定されたバージョン番号を使用する
2. When リリース作業が開始される and バージョン番号が指定されていない, the Release Workflow shall 現在の `Cargo.toml` の `[workspace.package].version` を読み取り、PATCH を +1 した値を提案バージョンとして算出する
3. When 提案バージョンが算出される, the Release Workflow shall 「vX.Y.Z から vX.Y.(Z+1) に更新します。よろしいですか？」の形式で開発者に承認を求める
4. If 開発者が提案バージョンを承認しない, the Release Workflow shall 開発者に希望するバージョン番号の入力を求める
5. When バージョン番号が提供される, the Release Workflow shall semver 形式（例: `1.2.0`）として妥当性を検証する
6. If バージョン番号が semver 形式でない, the Release Workflow shall エラーを報告し再入力を求める
7. When リリース作業が開始される, the Release Workflow shall `git status` でワークツリーがクリーンであることを確認する
8. If 未コミットの変更が存在する, the Release Workflow shall リリース作業を中止し、先にコミットまたはスタッシュするよう開発者に通知する
9. When リリース作業が開始される, the Release Workflow shall `cargo test --all` を実行し全テストが通過することを確認する
10. If テストが失敗する, the Release Workflow shall リリース作業を中止し失敗内容を報告する

### Requirement 2: Cargo.toml バージョン更新

**Objective:** As a 開発者, I want ワークスペース全体のバージョンを一括更新したい, so that 全クレートのバージョンが同期される

#### Acceptance Criteria

1. When バージョン番号が確定する, the Release Workflow shall `Cargo.toml`（ワークスペースルート）の `[workspace.package].version` フィールドを新バージョンに更新する
2. When ワークスペースバージョンが更新される, the Release Workflow shall `[workspace.dependencies]` セクション内の内部クレート参照（`pasta_core`, `pasta_lua`, `pasta_shiori`）の `version` フィールドも同じバージョンに更新する
3. When Cargo.toml が更新される, the Release Workflow shall `cargo build --workspace` を実行しビルドが成功することを確認する
4. If ビルドが失敗する, the Release Workflow shall 変更をロールバックしエラーを報告する
5. When ビルドが成功する, the Release Workflow shall バージョン更新を `chore(release): bump version to vX.Y.Z` メッセージでコミットする

### Requirement 3: crates.io 公開

**Objective:** As a 開発者, I want 依存関係の順序を考慮して全公開クレートを crates.io に公開したい, so that 下流ユーザーが最新版を利用できる

#### Acceptance Criteria

1. When バージョン更新コミットが完了する, the Release Workflow shall クレートを依存関係順（`pasta_core` → `pasta_lua` → `pasta_shiori`）に `cargo publish` する
2. When `cargo publish` を実行する, the Release Workflow shall 各クレートの公開成功を確認してから次のクレートに進む
3. If `cargo publish` が失敗する, the Release Workflow shall エラーを報告し以降の公開を中断する
4. While `pasta_sample_ghost` は `publish = false` である, the Release Workflow shall このクレートの公開をスキップする
5. When 前のクレートを公開した直後, the Release Workflow shall crates.io のインデックス更新を待つため適切な間隔（数秒〜十数秒）を空ける

### Requirement 4: サンプルゴーストビルド

**Objective:** As a 開発者, I want リリースバージョンの pasta.dll を使ってサンプルゴーストをビルドしたい, so that リリースに最新の .nar ファイルを含められる

#### Acceptance Criteria

1. When crates.io 公開が完了する, the Release Workflow shall `release.ps1` を `crates/pasta_sample_ghost/` ディレクトリで実行する
2. When `release.ps1` が成功する, the Release Workflow shall `hello-pasta.nar` が生成されたことを確認する
3. When `release.ps1` が成功する, the Release Workflow shall 32bit リリースビルドの `pasta.dll` が `target/i686-pc-windows-msvc/release/pasta.dll` に存在することを確認する
4. If `release.ps1` が失敗する, the Release Workflow shall エラーを報告しリリース作業を中断する
5. When ゴーストビルドが成功する, the Release Workflow shall 変更を `chore(release): build hello-pasta vX.Y.Z` メッセージでコミットする

### Requirement 5: バージョンタグとプッシュ

**Objective:** As a 開発者, I want Git タグでリリースポイントを記録し、リモートに反映したい, so that リリースのトレーサビリティが確保される

#### Acceptance Criteria

1. When ゴーストビルドのコミットが完了する, the Release Workflow shall `vX.Y.Z` 形式のアノテーションタグを作成する
2. When タグが作成される, the Release Workflow shall タグメッセージに `Release vX.Y.Z` を設定する
3. If 同名のタグが既に存在する, the Release Workflow shall エラーを報告し開発者に対応方法を確認する（既存タグの削除は自動実行しない）
4. When タグが作成される, the Release Workflow shall `git push origin main --tags` でコミットとタグの両方をリモートにプッシュする
5. If プッシュが失敗する, the Release Workflow shall エラーを報告し手動での対応を開発者に促す

### Requirement 6: GitHub Release 作成

**Objective:** As a 開発者, I want チェンジログ付きの GitHub Release を自動作成し、ビルド成果物を添付したい, so that ユーザーがリリースを容易に取得できる

#### Acceptance Criteria

1. When タグのプッシュが完了する, the Release Workflow shall `git log <前回タグ>..HEAD --oneline` コマンドで前回リリースから今回までのコミット履歴を取得する
2. When コミット履歴が取得される, the Release Workflow shall Conventional Commits 形式（`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:` 等）に基づいてコミットを種別ごとに分類・グループ化する
3. When チェンジログを整形する, the Release Workflow shall 各グループを見出し（例: `### Features`, `### Bug Fixes`, `### Documentation`）配下に箇条書きで配置する
4. When GitHub Release を作成する, the Release Workflow shall `gh release create vX.Y.Z` コマンドを使用する
5. When GitHub Release を作成する, the Release Workflow shall タイトルを `pasta vX.Y.Z` に設定する
6. When GitHub Release を作成する, the Release Workflow shall 整形済みチェンジログをリリースノートとして含める
7. When GitHub Release を作成する, the Release Workflow shall 以下の2ファイルをリリースアセットとして添付する:
   - `target/i686-pc-windows-msvc/release/pasta.dll`
   - `crates/pasta_sample_ghost/hello-pasta.nar`
8. If `gh` コマンドが失敗する, the Release Workflow shall エラーを報告し手動での Release 作成手順を案内する
9. If 前回リリースタグが存在しない（初回リリース）, the Release Workflow shall 全コミット履歴（`git log --oneline`）をチェンジログとして使用する

### Requirement 7: 繰り返し実行の仕様特性

**Objective:** As a 開発者, I want この仕様を何度でも再実行してリリース作業を行いたい, so that 毎回のリリースで同じ品質の手順が保証される

#### Acceptance Criteria

1. The Release Workflow shall `/kiro-spec-impl release-workflow` が実行されるたびにタスク状態を初期化（全タスクを未完了に戻す）する
2. The Release Workflow shall spec.json の `phase` を `completed` に変更しない（常に `ready_for_implementation` を維持する）
3. The Release Workflow shall 各実行が前回の実行状態に依存しない独立した作業として動作する
4. When リリース作業が完了する, the Release Workflow shall 実行結果のサマリー（バージョン、公開クレート、Release URL）を開発者に報告する

---

## 開発者提供手順からの調整事項

### 追加した要件

1. **事前検証（Req 1）**: ワークツリーのクリーンチェック・テスト実行を追加。汚れた状態からのリリースを防止。
2. **内部クレート参照の同期（Req 2-2）**: `[workspace.dependencies]` セクション内の `pasta_core`, `pasta_lua`, `pasta_shiori` のバージョンも更新が必要。
3. **公開順序と待機（Req 3）**: crates.io は依存関係順に公開する必要あり。インデックス更新の待機も必要。
4. **タグ競合処理（Req 5-3）**: 既存タグとの衝突時に安全に停止。workflow.md の「危険な Git 操作の禁止」ポリシーに準拠。
5. **繰り返し実行特性（Req 7）**: 本仕様の特殊な運用モデルを明示的に要件化。

### 手順の並び替え

開発者提供の手順では「タグ → push → Release」の順だが、これは適切。`cargo publish` の後にゴーストビルドを行う順序も、公開済みクレートの整合性を確認してからビルドする意味で妥当。
