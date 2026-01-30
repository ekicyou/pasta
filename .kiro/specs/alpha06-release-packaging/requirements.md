# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **x86 配布パッケージ作成とリリースワークフロー** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01-05 全て（最終パッケージ）
- **目的**: テスターに配布可能なアルファ版パッケージを作成し、GitHub Releases で公開

### リリース成果物

- **pasta.dll** (x86): SHIORI DLL
- **サンプルゴースト**: alpha04 で作成したゴースト
- **ドキュメント**: インストール手順、動作確認方法

---

## Requirements

### Requirement 1: パッケージ構成

**Objective:** As a テスター, I want 展開するだけで動作するパッケージがほしい, so that 簡単に試せる

#### Acceptance Criteria

1. The alpha06-release-packaging shall 以下の構成で ZIP パッケージを作成する:
   ```
   pasta-0.1.0-alpha.1-x86/
   ├── README.md           # インストール手順
   ├── LICENSE             # ライセンス
   ├── ghost/
   │   └── pasta-sample/   # サンプルゴースト
   │       ├── ghost/master/
   │       │   ├── pasta.dll    # SHIORI DLL
   │       │   ├── pasta.toml
   │       │   ├── dic/
   │       │   └── scripts/
   │       └── shell/master/
   └── docs/
       └── QUICKSTART.md   # 動作確認手順
   ```
2. The パッケージ shall x86 (32bit) アーキテクチャのみを対象とする

---

### Requirement 2: バージョン体系

**Objective:** As a 配布担当者, I want 一貫したバージョン番号を使いたい, so that リリース管理ができる

#### Acceptance Criteria

1. The alpha06-release-packaging shall セマンティックバージョニングを採用する
2. The アルファ版バージョン shall `0.1.0-alpha.N` 形式とする（N = リリース番号）
3. The バージョン shall `Cargo.toml` と一致させる

---

### Requirement 3: GitHub Releases ワークフロー

**Objective:** As a 配布担当者, I want タグ push でリリースが自動作成されてほしい, so that 手動作業を減らせる

#### Acceptance Criteria

1. The alpha06-release-packaging shall GitHub Actions でリリースワークフローを定義する
2. The ワークフロー shall 以下のトリガーで実行される:
   - `push` of tags matching `v*` (例: `v0.1.0-alpha.1`)
3. The ワークフロー shall 以下のステップを実行する:
   - x86 DLL ビルド
   - サンプルゴースト収集
   - ZIP パッケージ作成
   - GitHub Releases へアップロード

---

### Requirement 4: README ドキュメント

**Objective:** As a テスター, I want インストール方法がわかりやすく書かれていてほしい, so that 迷わず試せる

#### Acceptance Criteria

1. The alpha06-release-packaging shall パッケージ内 README.md に以下を含める:
   - 動作環境（SSP 2.x 以上）
   - インストール手順（ZIP 展開 → SSP のゴーストフォルダにコピー）
   - 動作確認方法（ゴースト切り替え → 起動確認）
2. The README shall 日本語で記述する

---

### Requirement 5: QUICKSTART ドキュメント

**Objective:** As a 初心者, I want 最短で動作確認したい, so that pasta を評価できる

#### Acceptance Criteria

1. The alpha06-release-packaging shall `docs/QUICKSTART.md` を提供する
2. The QUICKSTART shall 以下を含める:
   - 前提条件（SSP インストール済み）
   - 3ステップでの動作確認手順
   - トラブルシューティング（よくある問題）

---

### Requirement 6: リリースノート

**Objective:** As a ユーザー, I want 変更点を知りたい, so that 新機能を把握できる

#### Acceptance Criteria

1. The alpha06-release-packaging shall GitHub Releases にリリースノートを含める
2. The リリースノート shall 以下を含める:
   - 新機能一覧
   - 既知の問題
   - 今後の予定

---

### Requirement 7: 成果物検証

**Objective:** As a 配布担当者, I want パッケージが正しく動作することを確認したい, so that 不具合を防げる

#### Acceptance Criteria

1. The alpha06-release-packaging shall パッケージ作成後にサニティチェックを実行する
2. The チェック shall 以下を検証する:
   - pasta.dll が存在する
   - pasta.toml が有効な TOML 形式である
   - シェル画像が存在する

---

## Out of Scope

- x64 配布パッケージ（将来対応）
- 自動更新機能
- インストーラー（.msi, .exe）
- 多言語ドキュメント

---

## Glossary

| 用語 | 説明 |
|------|------|
| GitHub Releases | GitHub のリリース機能（バイナリ配布） |
| セマンティックバージョニング | MAJOR.MINOR.PATCH[-prerelease] 形式 |
| SSP | 伺か標準ベースウェア（ukagaka.github.io） |
| サニティチェック | 基本的な動作確認テスト |
