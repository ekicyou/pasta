# Requirements Document

## Project Description (Input)

### pasta_check クレート（コマンドラインアプリ）の作成

1. `pasta_sample_ghost` から、`updates2.dau`/`updates.txt` ファイルの作成処理、`*.nar` の作成処理を分離して、独立したクレートにする。
2. `pasta_shiori` と同じように `pasta_lua` などを依存クレートとする。これは将来、Lua コードの単体試験などを動かせるようにするため。
3. `release.ps1` を、`pasta_check` を利用するように変更する。
4. `release.bat` をリポジトリルートに移動し、リポジトリルートからリリースバッチを起動できるようにする。`release.ps1` は移動しない。
5. `pasta_check` を crates.io に publish する。バージョンは他のクレートと同じ。
6. 「release-workflow」仕様に、`pasta_check` のリリースを含める。

#### 共通コマンドラインオプション

- `--target XXX`：処理対象のパス。すべてのゴーストファイル（DLL・スクリプト・辞書・シェル画像）が揃ったゴースト開発フォルダー。
- `--release XXX`：リリースファイルパス。ゴーストフォルダーをコピーする。
- `--nar XXX`：NAR パス。指定パス名で NAR ファイルを作成する。
- `--copy XXX`：コピーファイルパス。release フォルダーに上書きするファイルのフォルダー。複数回指定可能。

#### コマンドラインコマンド: release

1. `--release` フォルダーを一度削除し、空ディレクトリで再作成
2. `--target` の内容でファイルコピー
3. `--copy` の内容でファイルコピー（上書き）
4. `updates2.dau`/`updates.txt` ファイルの作成
5. `--nar` ファイル名で `--release` フォルダーより NAR 作成

---

## Introduction

本ドキュメントは `pasta_check` クレートの要件を定義する。`pasta_check` は、ゴーストのリリースパッケージ作成に必要な処理（更新ファイル生成・NAR パッケージング・リリースフォルダー構築）を、`pasta_sample_ghost` から分離して独立させた CLI ツールである。

現在、これらの処理は以下のように分散している：

- **更新ファイル生成** (`updates2.dau`, `updates.txt`): `pasta_sample_ghost` の `update_files.rs` に Rust コードとして実装
- **NAR パッケージング**: `release.ps1` 内の PowerShell スクリプトとして実装（ZIPをリネーム）
- **リリースフォルダー構築・ファイルコピー**: `release.ps1` 内の robocopy コマンドとして実装

これらを `pasta_check` クレートに統合することで、`release.ps1` からの Rust ランタイム呼び出しを簡素化し、将来的な Lua 単体試験サポートの基盤を確保する。

リリース成果物は開発フォルダーとは分離された `release/` ディレクトリに出力される（例: `release/hello-pasta/`, `release/hello-pasta.nar`）。開発フォルダーの内容は変更されない。`pasta_check` は将来的に `release.ps1` のリリース関連処理を全面的に吸収する方向とする。

`ghosts/hello-pasta/` はゴーストが実際にテスト動作する完全な開発フォルダーであり、辞書・設定テキストを git で直接管理する。現在 `dist-src/` に分離されているテキストファイル（辞書・設定ファイル・`install.txt` 等）はすべて `ghosts/hello-pasta/` に統合し、`dist-src/` ディレクトリは廃止する。DLL（`pasta.dll`）と生成画像（`surface*.png`）はビルドステップにより `ghosts/hello-pasta/` に配置されるビルド成果物であり、git 管理外とする。

---

## Requirements

### Requirement 1: クレート構成とワークスペース統合

**Objective:** As a 開発者, I want `pasta_check` を独立した CLI クレートとしてワークスペースに追加したい, so that リリース関連処理が再利用可能な形で分離される

#### Acceptance Criteria

1. The `pasta_check` shall `crates/pasta_check/` ディレクトリにバイナリクレートとして配置される
2. The `pasta_check` shall ワークスペースルートの `Cargo.toml` の `members` に自動的に含まれる（`crates/*` パターンによる）
3. The `pasta_check` shall `version.workspace = true` でワークスペース共通バージョン（現在 `0.1.21`）を使用する
4. The `pasta_check` shall `publish = true` として crates.io への公開を許可する
5. The `pasta_check` shall `pasta_lua` を依存クレートとして含む（将来の Lua 単体試験サポートのため）
6. The `pasta_check` shall `md5`, `encoding_rs` を依存クレートとして含む（更新ファイル生成に必要）
7. The `pasta_check` shall `edition.workspace`, `authors.workspace`, `license.workspace`, `repository.workspace` を使用する

### Requirement 2: CLI インターフェース

**Objective:** As a 開発者, I want 共通のコマンドラインオプションで柔軟にリリース操作を指示したい, so that 異なるゴーストや出力先に対して統一された操作で作業できる

#### Acceptance Criteria

1. The `pasta_check` shall `release` サブコマンドを提供する
2. When `release` サブコマンドが実行される, the `pasta_check` shall `--target` オプションで、すべてのゴーストファイル（DLL・スクリプト・辞書・シェル画像）が揃った開発フォルダーパスを受け取る
3. When `release` サブコマンドが実行される, the `pasta_check` shall `--release` オプションでリリースフォルダーの出力先パスを受け取る
4. When `release` サブコマンドが実行される, the `pasta_check` shall `--nar` オプションで生成する NAR ファイルのパスを受け取る
5. The `pasta_check` shall `--copy` オプションを 0 回以上の複数指定可能とし、指定された順序で各フォルダーの内容をリリースフォルダーに上書きコピーする
6. If 必須オプション（`--target`, `--release`, `--nar`）のいずれかが未指定, the `pasta_check` shall エラーメッセージと使用方法を表示して終了する

### Requirement 3: release サブコマンドの実行フロー

**Objective:** As a 開発者, I want `release` コマンド一発でリリースフォルダー構築から NAR 作成まで完了したい, so that リリース作業の手動操作が最小化される

#### Acceptance Criteria

1. When `release` サブコマンドが実行される, the `pasta_check` shall まず `--release` フォルダーが存在すれば削除し、空ディレクトリとして再作成する
2. When リリースフォルダーが準備される, the `pasta_check` shall `--target` フォルダーの内容を `--release` フォルダーに再帰的にコピーする
3. Where `--copy` オプションが 1 回以上指定される, the `pasta_check` shall 指定された順序で各 `--copy` フォルダーの内容を `--release` フォルダーに上書きコピーする
4. When ファイルコピーが完了する, the `pasta_check` shall `--release` フォルダー内に `updates2.dau` および `updates.txt` を SSP 仕様に準拠して生成する
5. When 更新ファイルの生成が完了する, the `pasta_check` shall `--release` フォルダーの内容から `--nar` で指定されたパスに NAR ファイルを作成する
6. When 各ステップが実行される, the `pasta_check` shall 進捗メッセージを標準出力に表示する
7. If いずれかのステップで IO エラーが発生する, the `pasta_check` shall エラー内容を報告してゼロ以外の終了コードで終了する
8. The `pasta_check` shall `--target` フォルダーの内容を変更せず、すべてのリリース成果物を `--release` フォルダーおよび `--nar` 指定パスに出力する

### Requirement 4: 更新ファイル生成（updates2.dau / updates.txt）

**Objective:** As a 開発者, I want SSP 仕様準拠の更新ファイルを自動生成したい, so that ゴーストのネットワーク更新に必要なファイルが正確に作成される

#### Acceptance Criteria

1. The `pasta_check` shall `updates2.dau` を Shift_JIS エンコーディング、CRLF 改行、SOH 区切りフォーマットで生成する
2. The `pasta_check` shall `updates.txt` を Shift_JIS エンコーディング、CRLF 改行、カンマ区切りフォーマットで生成する
3. The `pasta_check` shall ファイルの MD5 ハッシュとサイズを各エントリに含める
4. The `pasta_check` shall `profile/`, `var/` ディレクトリ、`updates2.dau`, `updates.txt`, `developer_options.txt` を除外する
5. The `pasta_check` shall ファイルエントリをパスのアルファベット順でソートする
6. If Shift_JIS にエンコードできない文字が含まれる, the `pasta_check` shall UTF-8 のまま書き込む（SSP の UTF-8 サポートによるフォールバック）

### Requirement 5: NAR ファイル作成

**Objective:** As a 開発者, I want ゴースト配布用の NAR ファイルを Rust で生成したい, so that PowerShell の `Compress-Archive` への依存がなくなる

#### Acceptance Criteria

1. The `pasta_check` shall リリースフォルダーの内容を ZIP 形式で圧縮し、`.nar` 拡張子で出力する
2. The `pasta_check` shall NAR 作成時に `profile/` ディレクトリを除外する
3. The `pasta_check` shall NAR ファイルのサイズを完了メッセージに含める
4. If NAR ファイルの出力先ディレクトリが存在しない, the `pasta_check` shall 親ディレクトリを再帰的に作成する
5. If 同名の NAR ファイルが既に存在する, the `pasta_check` shall 上書きする

### Requirement 6: pasta_sample_ghost からの処理分離

**Objective:** As a 開発者, I want `pasta_sample_ghost` から更新ファイル生成処理を分離したい, so that `pasta_sample_ghost` が画像生成に専念でき、関心の分離が実現される

#### Acceptance Criteria

1. When `pasta_check` が完成する, the `pasta_sample_ghost` shall `update_files.rs` モジュールを削除する
2. When `pasta_check` が完成する, the `pasta_sample_ghost` shall `--finalize` オプションを削除する
3. The `pasta_sample_ghost` shall 画像生成（`surface*.png`, `surfaces.txt`）機能のみを保持する
4. The `pasta_sample_ghost` shall `md5` および `encoding_rs` への依存を削除する（更新ファイル生成が不要になるため）
5. When `pasta_check` が完成する, the Release Workflow shall `crates/pasta_sample_ghost/hello-pasta.nar` ファイルを削除する

### Requirement 7: release.ps1 の簡素化

**Objective:** As a 開発者, I want `pasta_check` にリリース処理を委譲して `release.ps1` を簡素化したい, so that リリーススクリプトの複雑性が低減され、将来的な `release.ps1` の廃止への道筋がつく

#### Acceptance Criteria

1. When リリースフローが実行される, the `release.ps1` shall 更新ファイル生成（旧 Step 5）、NAR 作成（旧 Step 8）を `pasta_check release` コマンドに委譲する
2. The `release.ps1` shall `pasta_check release` コマンドの正常終了をもってリリース成果物の整合性を保証し、別途バリデーションステップ（旧 Step 7）は設けない
3. The `release.ps1` shall DLL ビルド（Step 1）、画像生成（Step 2）、DLL/scripts の `ghosts/hello-pasta/` へのコピー（Step 3）の実行後に `pasta_check release` を呼び出す構成とする
4. The `release.ps1` shall 現在の配置場所（`crates/pasta_sample_ghost/release.ps1`）に留まる
5. When `pasta_check` が完成する, the Release Workflow shall `dist-src/` ディレクトリを削除し、その内容（辞書・設定テキスト・`install.txt`）を `ghosts/hello-pasta/` に直接統合する

### Requirement 8: release.bat のリポジトリルート移動

**Objective:** As a 開発者, I want `release.bat` をリポジトリルートから起動できるようにしたい, so that リリース操作が直感的に開始できる

#### Acceptance Criteria

1. The `release.bat` shall リポジトリルート（`pasta/release.bat`）に配置される
2. The `release.bat` shall `crates/pasta_sample_ghost/release.ps1` を呼び出す（パスを適切に解決する）
3. When 既存の `crates/pasta_sample_ghost/release.bat` が存在する, the Release Workflow shall 旧ファイルを削除する
4. The `release.bat` shall 既存のオプション（`-SkipSetup`, `-SkipDllBuild`）をそのまま `release.ps1` にパススルーする

### Requirement 9: crates.io 公開設定

**Objective:** As a 開発者, I want `pasta_check` を crates.io に公開したい, so that 他の開発者もゴーストリリースツールを利用できる

#### Acceptance Criteria

1. The `pasta_check` shall `Cargo.toml` で `publish = true`（明示的、またはデフォルト）として設定される
2. The `pasta_check` shall 他のワークスペースクレートと同じバージョン番号を使用する
3. The `pasta_check` shall `description` フィールドに CLI ツールの説明を含める
4. When `release-workflow` 仕様が実行される, the Release Workflow shall `pasta_check` を `cargo publish` 対象クレートに含める
