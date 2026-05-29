# Requirements Document

## Introduction
pasta_checkはゴーストリリース用CLIツールであり、ファイルコピー・NARアーカイブ作成・更新ファイル（updates.txt）生成を担う。本監査では、ファイルI/Oパス操作の安全性検証、アーカイブ処理の脆弱性調査、MD5ハッシュ使用の適切性評価、デッドコード除去、冗長表現削減を実施する。外部振る舞い（CLI出力・生成ファイル形式・NAR互換性）は一切変更しない。

## Boundary Context
- **In scope**: pasta_check/src/ 全5ファイル（main.rs, release.rs, copy.rs, nar.rs, update_files.rs）の脆弱性調査、パストラバーサル検証、シンボリックリンク追跡検証、デッドコード除去、冗長表現削減
- **Out of scope**: CLIインターフェースの変更（引数追加・出力フォーマット変更）、NARフォーマット仕様の変更、新サブコマンド追加、updates.txtフォーマット仕様変更
- **Adjacent expectations**: release-workflow specが本ツールを利用するリリース手順を定義しており、CLI引数・出力形式・生成物（NAR、updates.txt）の互換性が維持される前提で依存している

## Requirements

### Requirement 1: パストラバーサル安全性
**Objective:** As a ゴースト開発者, I want ファイルパス操作がパストラバーサル攻撃に対して安全であること, so that 悪意あるパスを含むゴーストデータを処理しても意図しないファイルへのアクセスが発生しない

#### Acceptance Criteria
1. When pasta_checkがディレクトリを再帰走査するとき, the pasta_check shall 生成される相対パスがルートディレクトリの外を指さないことを保証する
2. If `strip_prefix`によるパス正規化が失敗した場合, the pasta_check shall エラーを返して処理を中断する
3. When NARアーカイブにエントリを追加するとき, the pasta_check shall ZIPエントリ名に`..`コンポーネントが含まれないことを保証する
4. When ファイルコピー操作を実行するとき, the pasta_check shall コピー先パスが指定されたコピー先ディレクトリの外を指さないことを保証する

### Requirement 2: シンボリックリンク安全性
**Objective:** As a ゴースト開発者, I want シンボリックリンクを含むディレクトリを処理する際に安全であること, so that シンボリックリンクを通じた意図しないファイルアクセスが防止される

#### Acceptance Criteria
1. When ファイル走査中にシンボリックリンクを検出したとき, the pasta_check shall シンボリックリンクを追跡せずスキップする、またはリンク先がルートディレクトリ内であることを検証する
2. When NARアーカイブ作成中にシンボリックリンクを検出したとき, the pasta_check shall シンボリックリンクをアーカイブに含めない、またはリンク先の実ファイルの内容のみを安全に含める

### Requirement 3: MD5ハッシュ使用の適切性
**Objective:** As a ゴースト開発者, I want MD5ハッシュの使用が用途に対して適切であること, so that セキュリティ上の懸念なくファイル変更検出に利用できる

#### Acceptance Criteria
1. The pasta_check shall MD5ハッシュをファイル変更検出（updates.txt生成）の用途にのみ使用し、認証・署名・暗号学的用途には使用しない
2. The pasta_check shall MD5の使用箇所にその用途（ファイル整合性チェック、非暗号学的用途）をコードコメントとして明記する

### Requirement 4: デッドコード除去
**Objective:** As a メンテナー, I want 使われていないコードが除去されること, so that コードベースの保守性が向上する

#### Acceptance Criteria
1. When `#[allow(dead_code)]`アトリビュートが付与された関数が存在するとき, the pasta_check shall その関数が本当に不要であれば除去し、必要であれば`#[allow(dead_code)]`を外して適切に使用する
2. The pasta_check shall 未使用のインポート、変数、定数が存在しないことを保証する

### Requirement 5: 冗長表現削減
**Objective:** As a メンテナー, I want 冗長なコード表現が簡潔になること, so that コードの可読性と保守性が向上する

#### Acceptance Criteria
1. The pasta_check shall 繰り返しのエラー変換パターン（`map_err`による`io::Error`変換など）を簡潔な共通パターンに置き換える
2. The pasta_check shall 不要な中間変数やボイラープレートコードを削減する
3. The pasta_check shall Rustイディオムに沿った簡潔な表現を使用する

### Requirement 6: 外部振る舞いの不変性
**Objective:** As a ゴースト開発者, I want 監査による内部変更が外部振る舞いに影響しないこと, so that 既存のリリースワークフローが中断なく動作し続ける

#### Acceptance Criteria
1. The pasta_check shall 既存の全テスト（ユニットテスト・統合テスト）をパスする
2. The pasta_check shall CLI引数の解析結果が変更前と同一であることを保証する
3. The pasta_check shall 生成されるupdates.txtの形式と内容が変更前と同一であることを保証する
4. The pasta_check shall 生成されるNARアーカイブの内容（エントリ名・ファイル内容）が変更前と同一であることを保証する
5. The pasta_check shall 標準出力・標準エラー出力のメッセージ形式が変更前と同一であることを保証する
