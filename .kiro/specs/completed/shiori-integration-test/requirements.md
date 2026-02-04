# Requirements Document

## Project Description (Input)
pasta_shioriの PastaShiori::load / requestのインテグレーションテストを、pasta_sample_ghost のゴースト定義を活用して行う。

### テスト環境構築とload

1. テンポラリディレクトリに`hello-pasta/ghost/master`をコピーする
2. PastaShioriを作成し、PastaShiori::load を適切な引数で実施。
3. エラーが発生せず、loadが完了すること。


### request
OnBoot reqを投げて、期待値のresが返ってくることを確認する。

#### 入力
OnBoot リクエスト（SHIORI/3.0 プロトコル形式）

```
GET SHIORI/3.0
Charset: UTF-8
Sender: SSP
SecurityLevel: local
ID: OnBoot
Reference0: マスターシェル

```

**注意**: 改行は CRLF (`\r\n`)、末尾は CRLFCRLF (`\r\n\r\n`) で終端する。

#### 期待値（レスポンス）
以下をさくらスクリプトに変換した200応答。

＊OnBoot
　女の子：＠通常　起動したよ～。
　男の子：＠通常　さあ、始めようか。

### pasta_sample_ghostの修正項目
＊OnBootが2つトークあるが、テストが不安定になるため、OnBootトークは1つだけに変更する。残すのは単語辞書呼び出し（＠起動挨拶）を含まない方とし、ランダム要素を完全に排除する。ビルド後、`setup.bat`を実行してゴーストを作り直すこと。

### 追加要件: トークウェイト設定
pasta.tomlに`[talk]`セクションでウェイト設定を追加し、テストで期待値を確認できるようにする。

## Introduction

本仕様は、pasta_shiori クレートの `PastaShiori::load` および `request` メソッドのインテグレーションテストを定義する。テスト対象として pasta_sample_ghost の hello-pasta ゴースト定義を活用し、実際のゴースト動作を検証する。これにより、SHIORI プロトコル経由でのパスタスクリプト実行パイプライン全体の動作を保証する。

## Requirements

### Requirement 1: pasta_sample_ghost の OnBoot シーン修正
**Objective:** As a テスト開発者, I want OnBootシーンが決定的に動作する, so that インテグレーションテストが安定的に実行できる

#### Acceptance Criteria
1. The boot.pasta shall OnBootシーン定義を1つのみ含む（ランダム選択による不安定性を排除）
2. When OnBootシーンが修正される, the 開発者 shall `cargo build -p pasta_sample_ghost` でビルドを実行する
3. When ビルドが完了する, the 開発者 shall `setup.bat` を実行してゴースト定義を再生成する

### Requirement 2: pasta.toml へのトークウェイト設定追加
**Objective:** As a ゴースト開発者, I want トークウェイト設定がpasta.tomlで管理できる, so that キャラクターの会話テンポをカスタマイズできる

#### Acceptance Criteria
1. The pasta.toml shall `[talk]`セクションを含む
2. When `[talk]`セクションが定義される, the pasta.toml shall `script_wait_normal`（デフォルト: 50ms）を設定可能とする
3. When `[talk]`セクションが定義される, the pasta.toml shall `script_wait_period`（デフォルト: 1000ms）を設定可能とする
4. When `[talk]`セクションが定義される, the pasta.toml shall `script_wait_comma`（デフォルト: 500ms）を設定可能とする
5. When `[talk]`セクションが定義される, the pasta.toml shall `script_wait_strong`（デフォルト: 500ms）を設定可能とする
6. When `[talk]`セクションが定義される, the pasta.toml shall `script_wait_leader`（デフォルト: 200ms）を設定可能とする

### Requirement 3: テスト環境のセットアップ
**Objective:** As a テスト実行者, I want テスト環境が自動的に構築される, so that 手動セットアップなしでテストを実行できる

#### Acceptance Criteria
1. When テストが開始される, the テストコード shall テンポラリディレクトリを作成する
2. When テンポラリディレクトリが作成される, the テストコード shall `hello-pasta/ghost/master`の内容をコピーする
3. The テストコード shall コピー元として `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master` を使用する

### Requirement 4: PastaShiori::load の検証
**Objective:** As a SHIORI 統合テスト開発者, I want load処理が正常に完了することを確認したい, so that ゴースト初期化が正しく動作することを保証できる

#### Acceptance Criteria
1. When `PastaShiori::load`が適切な引数で呼び出される, the PastaShiori shall `Ok(true)`を返す
2. When load処理が完了する, the PastaShiori shall ランタイムを正常に初期化する
3. If load_dirが存在しない, the PastaShiori shall `Ok(false)`を返す

### Requirement 5: OnBoot リクエストの検証
**Objective:** As a SHIORI 統合テスト開発者, I want OnBootイベントへの応答を検証したい, so that シーン実行パイプラインが正しく動作することを保証できる

#### Acceptance Criteria
1. When 完全なOnBootリクエスト（Charset, Sender, SecurityLevel, ID, Reference0ヘッダー付き）が送信される, the PastaShiori shall 200応答を返す
2. When OnBootリクエストが処理される, the レスポンス shall さくらスクリプト形式の Value ヘッダーを含む
3. When さくらスクリプトが生成される, the レスポンス shall スポット切り替えタグ（`\0`, `\1`）を含む
4. When さくらスクリプトが生成される, the レスポンス shall 表情変更タグ（`\s[通常]`）を含む
5. When さくらスクリプトが生成される, the レスポンス shall ウェイトタグ（`\_w[ms]`）を含む
6. When OnBootシーンが実行される, the レスポンス shall 「起動したよ～。」「さあ、始めようか。」のテキストを含む

### Requirement 6: テストファイルの配置
**Objective:** As a プロジェクト保守者, I want テストが適切な場所に配置される, so that テスト構造が一貫している

#### Acceptance Criteria
1. The テストファイル shall `crates/pasta_shiori/tests/`ディレクトリに配置される
2. The テストファイル shall `shiori_sample_ghost_test.rs`という名前で作成される
3. The テストコード shall 既存の`common`モジュールを活用する
