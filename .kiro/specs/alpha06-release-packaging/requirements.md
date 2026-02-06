# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **x86 配布パッケージ作成とリリースワークフロー** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01〜05 全て（最終パッケージ）
- **目的**: テスターに配布可能なアルファ版パッケージを作成し、GitHub Releases で公開
- **既存CI基盤**: `.github/workflows/build.yml`（x86/x64 マトリックスビルド、アーティファクト `pasta-dll-x86` 出力済み）

### リリース成果物

- **pasta.dll** (x86): SHIORI DLL（`pasta_shiori` クレート、`i686-pc-windows-msvc` ターゲット）
- **サンプルゴースト**: alpha04 で作成した hello-pasta ゴースト（`pasta_sample_ghost` クレート）
- **ドキュメント**: インストール手順、動作確認方法、リリースノート

---

## Requirements

### Requirement 1: パッケージ構成

**Objective:** As a テスター, I want 展開するだけで動作するパッケージがほしい, so that 簡単に試せる

#### Acceptance Criteria

1. The リリースワークフロー shall 以下のディレクトリ構成で ZIP パッケージを作成する:
   ```
   pasta-v0.1.0-alpha.1-x86/
   ├── README.md           # インストール手順
   ├── LICENSE             # MIT OR Apache-2.0 ライセンス
   ├── ghost/
   │   └── hello-pasta/    # サンプルゴースト
   │       ├── ghost/master/
   │       │   ├── pasta.dll    # SHIORI DLL (x86)
   │       │   ├── pasta.toml   # 設定ファイル
   │       │   ├── dic/         # Pasta DSL スクリプト
   │       │   └── scripts/     # Lua スクリプト
   │       └── shell/master/    # シェル画像
   └── docs/
       └── QUICKSTART.md   # 動作確認手順
   ```
2. The リリースワークフロー shall x86 (32bit, `i686-pc-windows-msvc`) アーキテクチャのみを配布対象とする
3. The ZIP ファイル名 shall `pasta-v<version>-x86.zip` 形式に従う

---

### Requirement 2: バージョン体系

**Objective:** As a 配布担当者, I want 一貫したバージョン番号を使いたい, so that リリース管理ができる

#### Acceptance Criteria

1. The リリースワークフロー shall セマンティックバージョニング（SemVer）に準拠したバージョン体系を採用する
2. The アルファ版バージョン shall `0.1.x-alpha.N` 形式とする（N = リリース番号）
3. The リリースワークフロー shall Git タグ（`v0.1.0-alpha.1` 等）からバージョン文字列を取得する
4. When タグのバージョンと `Cargo.toml` の `workspace.package.version` が不一致の場合, the リリースワークフロー shall ビルドを失敗させる

---

### Requirement 3: GitHub Releases ワークフロー

**Objective:** As a 配布担当者, I want タグ push でリリースが自動作成されてほしい, so that 手動作業を減らせる

#### Acceptance Criteria

1. The リリースワークフロー shall `.github/workflows/release.yml` として GitHub Actions ワークフローを定義する
2. When `v*` パターンに一致するタグが push された場合, the リリースワークフロー shall 自動的にリリースプロセスを開始する
3. The リリースワークフロー shall 以下のステップを順序通り実行する:
   - Rust ツールチェーンのセットアップ（`i686-pc-windows-msvc` ターゲット）
   - `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori` による x86 DLL ビルド
   - `pasta_sample_ghost` クレートによるサンプルゴースト成果物の収集
   - ドキュメント（README.md, LICENSE, QUICKSTART.md）の配置
   - ZIP パッケージの作成
   - GitHub Releases へのアップロード（タグに対応するリリースを作成）
4. The リリースワークフロー shall 既存の `build.yml` と同じ Rust キャッシュ戦略（`Swatinem/rust-cache@v2`）を使用する
5. The リリースワークフロー shall リリースページに ZIP ファイルをアセットとして添付する
6. Where タグ名に `alpha` または `beta` が含まれる場合, the リリースワークフロー shall GitHub Releases をプレリリースとしてマークする

---

### Requirement 4: README ドキュメント（パッケージ同梱）

**Objective:** As a テスター, I want インストール方法がわかりやすく書かれていてほしい, so that 迷わず試せる

#### Acceptance Criteria

1. The リリースワークフロー shall パッケージ内 README.md に以下のセクションを含める:
   - **概要**: pasta とは何か（SHIORI DLL スクリプトエンジン）
   - **動作環境**: SSP（伺か標準ベースウェア）2.x 以上、Windows x86
   - **インストール手順**: ZIP 展開 → SSP のゴーストフォルダにコピー
   - **動作確認方法**: ゴースト切り替え → 起動確認
   - **ライセンス**: MIT OR Apache-2.0
   - **問い合わせ先**: GitHub リポジトリ URL
2. The README shall 日本語で記述する
3. The README shall パッケージのバージョン番号を明記する

---

### Requirement 5: QUICKSTART ドキュメント

**Objective:** As a 初心者, I want 最短で動作確認したい, so that pasta を評価できる

#### Acceptance Criteria

1. The リリースワークフロー shall `docs/QUICKSTART.md` をパッケージに含める
2. The QUICKSTART shall 以下のセクションを含める:
   - **前提条件**: SSP インストール済みであること
   - **手順**: 3ステップ以内で動作確認できる手順（展開→コピー→起動）
   - **期待される動作**: サンプルゴーストの起動時の表示内容
   - **トラブルシューティング**: よくある問題と対処法（DLL 読み込みエラー、文字化け等）
3. The QUICKSTART shall 日本語で記述する

---

### Requirement 6: リリースノート

**Objective:** As a ユーザー, I want 変更点を知りたい, so that 新機能を把握できる

#### Acceptance Criteria

1. The リリースワークフロー shall GitHub Releases のリリース本文にリリースノートを含める
2. The リリースノート shall 以下のセクションを含める:
   - **概要**: リリースの一言説明
   - **新機能一覧**: 実装された機能のリスト
   - **既知の問題**: 未解決の制限事項
   - **今後の予定**: 次のリリースで予定している機能
3. The リリースノート shall テンプレートファイルとして管理する
4. The リリースノート shall 日本語で記述する

---

### Requirement 7: 成果物検証

**Objective:** As a 配布担当者, I want パッケージが正しく動作することを確認したい, so that 不具合を防げる

#### Acceptance Criteria

1. The リリースワークフロー shall ZIP パッケージ作成後にサニティチェックを実行する
2. The サニティチェック shall 以下の項目を検証する:
   - `pasta.dll` が ZIP 内の正しいパスに存在すること
   - `pasta.toml` が有効な TOML 形式であること
   - シェル画像ファイル（`shell/master/` 配下）が存在すること
   - `README.md` および `LICENSE` がパッケージルートに存在すること
3. If サニティチェックのいずれかが失敗した場合, the リリースワークフロー shall ビルドをエラー終了させ、GitHub Releases へのアップロードを行わない

---

## Out of Scope

- x64 配布パッケージ（将来対応）
- 自動更新機能（ネットワークアップデート）
- インストーラー（.msi, .exe）
- 多言語ドキュメント（英語版等）
- crates.io への公開（別途管理）
- サイン・署名付きバイナリ

---

## Glossary

| 用語 | 説明 |
|------|------|
| GitHub Releases | GitHub のリリース機能（バイナリ配布・リリースノート） |
| セマンティックバージョニング (SemVer) | `MAJOR.MINOR.PATCH[-prerelease]` 形式のバージョン体系 |
| SSP | 伺か標準ベースウェア（ukagaka.github.io）。デスクトップマスコットの実行環境 |
| SHIORI DLL | SSP が読み込む対話エンジン。pasta.dll がこれに該当 |
| サニティチェック | パッケージの基本的な構成検証テスト |
| hello-pasta | `pasta_sample_ghost` クレートで生成されるサンプルゴースト |
| `i686-pc-windows-msvc` | Windows 32bit (x86) 向け Rust ターゲットトリプル |
