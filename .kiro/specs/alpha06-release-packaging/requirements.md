# Requirements Document

## Introduction

本仕様は pasta のサンプルゴースト「hello-pasta」をリリース配布物として提供するワークフローを定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01〜05 全て（リリース基盤）
- **目的**: hello-pasta ゴースト（`.nar` 形式）を GitHub Releases で公開し、テスターが SSP で直接インストール可能な状態を実現
- **既存基盤**: `setup.bat`（ゴースト配布物生成）、GitHub CLI（リリース公開）

### リリース成果物

- **hello-pasta.nar**: サンプルゴースト配布ファイル（`.nar` = `.zip` 形式）
  - pasta.dll (x86)、pasta.toml、pasta DSL スクリプト、Lua ランタイム、シェル画像を含む
- **GitHub Releases**: リリース本文にドキュメント（インストール手順、動作確認方法）を記載

---

## Requirements

### Requirement 1: ゴースト配布物（.nar）の構成

**Objective:** As a テスター, I want SSP にそのまま展開できるゴースト配布ファイルがほしい, so that インストール作業を簡略化したい

#### Acceptance Criteria

1. The リリースプロセス shall 以下のディレクトリ構成で `.nar` ファイルを生成する:
   ```
   hello-pasta.nar（内部構成）
   ├── ghost/master/
   │   ├── pasta.dll    # SHIORI DLL (x86, i686-pc-windows-msvc)
   │   ├── pasta.toml   # 設定ファイル
   │   ├── descript.txt # ゴースト設定（自動生成）
   │   ├── dic/         # Pasta DSL スクリプト
   │   │   ├── boot.pasta
   │   │   ├── talk.pasta
   │   │   ├── click.pasta
   │   │   └── actors.pasta
   │   └── scripts/     # Lua ランタイム（pasta_lua）
   ├── shell/master/
   │   ├── descript.txt # シェル設定（自動生成）
   │   ├── surfaces.txt # サーフェス定義
   │   └── surface*.png # ピクトグラム画像（18ファイル）
   ├── install.txt      # インストール情報（自動生成）
   ├── updates.txt      # 更新情報（自動生成）
   └── updates2.dau     # 更新情報バイナリ（自動生成）
   ```
2. The リリースプロセス shall `setup.bat` 実行により自動生成される `ghosts/hello-pasta/` ディレクトリをそのまま ZIP 圧縮し、`.nar` 拡張子に変更する
3. The `.nar` ファイル名 shall `hello-pasta.nar` とする

---

### Requirement 2: バージョン管理

**Objective:** As a 配布担当者, I want バージョン管理を単純にしたい, so that ビルド・リリース手順が明確である

#### Acceptance Criteria

1. The ゴースト shall Cargo.toml の `workspace.package.version` に従うバージョンを使用する
2. The リリースタグ shall `v<version>` 形式とする（例：`v0.1.1`）
3. The タグのバージョン shall Cargo.toml の `workspace.package.version` と一致していること（リリース前に確認）

---

### Requirement 3: ゴースト配布物の生成プロセス

**Objective:** As a 配布担当者, I want ゴースト配布物を確実に生成したい, so that 品質保証ができる

#### Acceptance Criteria

1. The ゴースト生成プロセス shall 以下のステップで構成される:
   - `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori` による x86 DLL ビルド
   - `cargo run -p pasta_sample_ghost` によるゴーストファイル生成（dic/, shell/, 設定ファイル）
   - `cargo run -p pasta_sample_ghost -- --finalize` による更新ファイル生成（updates.txt, updates2.dau）
   - DLL コピー：`target/i686-pc-windows-msvc/release/pasta.dll` → `ghosts/hello-pasta/ghost/master/pasta.dll`
   - Lua ランタイムコピー：`crates/pasta_lua/scripts` → `ghosts/hello-pasta/ghost/master/scripts`
2. The ステップ実行順序 shall `setup.bat` に準拠する
3. The 生成されたゴースト shall `ghosts/hello-pasta/` ディレクトリに配置される

---

### Requirement 4: GitHub Releases への公開

**Objective:** As a テスター, I want ゴースト配布ファイルを GitHub から入手したい, so that 最新版をダウンロードできる

#### Acceptance Criteria

1. The リリース公開 shall GitHub CLI（`gh release create`）を使用して実行される
2. The 実行コマンド shall 以下の形式：
   ```
   gh release create <tag> hello-pasta.nar --notes-file RELEASE_NOTES.md
   ```
3. The GitHub Releases ページ shall `.nar` ファイルをアセットとして添付する
4. The リリース本文 shall リリースノートテンプレートの内容を含める

---

### Requirement 5: リリースノート

**Objective:** As a ユーザー, I want リリース内容を理解したい, so that pasta の機能を把握できる

#### Acceptance Criteria

1. The リリースノートテンプレート shall `.kiro/templates/release_notes.md` として管理される
2. The テンプレート shall 以下のセクションを含める:
   - **バージョン**: 公開バージョン（例：0.1.1）
   - **リリース日**: リリース日付
   - **概要**: リリースの一言説明
   - **含まれるコンポーネント**: pasta.dll, hello-pasta ゴースト、各バージョン情報
   - **必要環境**: SSP 2.x 以上、Windows x86
   - **インストール方法**: `.nar` ファイルを SSP のゴーストフォルダにコピー
   - **動作確認方法**: ゴースト切り替え手順
   - **既知の問題**: 制限事項、推奨環境
   - **フィードバック**: GitHub Issues への誘導
3. The テンプレート shall 日本語で記述する

---

### Requirement 6: インストール・動作確認ドキュメント

**Objective:** As a 初心者, I want インストール手順を明確に理解したい, so that スムーズにセットアップできる

#### Acceptance Criteria

1. The インストール手順 shall GitHub Releases の説明欄に含められる
2. The 手順 shall 以下を記載：
   - `.nar` ファイルの入手先（GitHub Releases）
   - SSP のゴーストフォルダパス（`%ProgramFiles%\SSP\ghost\` または `~\SSP\ghost\` 等）
   - ファイルコピー手順（ドラッグ＆ドロップ、またはコマンドライン）
3. The 動作確認手順 shall 以下を記載：
   - SSP の「ゴースト選択」画面で hello-pasta を選択
   - ゴースト起動・キャラクター表示の確認
   - トラブルシューティング（DLL 読み込みエラー、文字化け等）

---

### Requirement 7: ゴースト配布物の検証

**Objective:** As a 配布担当者, I want 配布前に品質確認したい, so that ユーザーへの不具合配布を防げる

#### Acceptance Criteria

1. The サニティチェック shall リリース前に `ghosts/hello-pasta/` に対して実行される
2. The チェック項目 shall 以下を含める:
   - `ghost/master/pasta.dll` が存在し、ファイルサイズが 0 でないこと
   - `ghost/master/pasta.toml` が有効な TOML 形式であること
   - `ghost/master/dic/` 配下に `.pasta` ファイルが 4 ファイル存在すること
   - `ghost/master/scripts/` 配下に Lua ランタイムファイルが存在すること
   - `shell/master/` 配下に `surface*.png` ファイルが存在すること
   - `install.txt`, `updates.txt`, `updates2.dau` が存在すること
3. If いずれかのチェックが失敗した場合, the リリースプロセス shall 中断し、エラー通知を出す

---

## Out of Scope

- x64 版リリース（将来対応）
- 自動更新機能（SSP ネイティブの更新チェック）
- インストーラー（.msi, .exe）
- 多言語ドキュメント
- 署名付きバイナリ
- crates.io への公開（別途仕様）
- GitHub Actions による完全自動化（手動実行が基本）

---

## Glossary

| 用語 | 説明 |
|------|------|
| `.nar` ファイル | ZIP ファイルを `.nar` 拡張子に変更したもの。伺か ゴースト配布形式 |
| GitHub Releases | GitHub のリリース機能。バイナリ配布・変更履歴公開に使用 |
| GitHub CLI (`gh`) | GitHub をコマンドラインから操作するツール。`gh release create` でリリース公開可能 |
| SSP | 伺か標準ベースウェア。デスクトップマスコット実行環境 |
| SHIORI DLL | SSP が読み込む対話エンジン DLL。pasta.dll がこれに該当 |
| hello-pasta | `pasta_sample_ghost` クレートで生成されるサンプルゴースト |
| セマンティックバージョニング | `MAJOR.MINOR.PATCH` 形式のバージョン体系 |
