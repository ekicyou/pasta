# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **GitHub Actions による x86/x64 DLL ビルド CI** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: なし（独立実装可能）
- **目的**: Windows 向け pasta.dll の自動ビルド・テスト環境を確立

### ビルドターゲット

- **x86 (32bit)**: `i686-pc-windows-msvc` - 伺かベースウェア（SSP等）の標準アーキテクチャ
- **x64 (64bit)**: `x86_64-pc-windows-msvc` - 開発・テスト用

---

## Requirements

### Requirement 1: ビルドワークフロー

**Objective:** As a 開発者, I want push/PR 時に自動ビルドが走ってほしい, so that ビルド破損を早期検知できる

#### Acceptance Criteria

1. The alpha05-build-ci shall GitHub Actions ワークフローを `.github/workflows/` に定義する
2. The ワークフロー shall 以下のトリガーで実行される:
   - `push` to `main` ブランチ
   - `pull_request` to `main` ブランチ
3. The ワークフロー shall x86 と x64 の両アーキテクチャでビルドする

---

### Requirement 2: Rust ツールチェーン設定

**Objective:** As a 開発者, I want 適切な Rust ツールチェーンでビルドしたい, so that 正しいターゲットで DLL が生成される

#### Acceptance Criteria

1. The alpha05-build-ci shall `stable-x86_64-pc-windows-msvc` ツールチェーンをインストールする
2. The alpha05-build-ci shall `i686-pc-windows-msvc` ターゲットを追加インストールする
3. The ビルドコマンド shall 以下を使用する:
   - x86: `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori`
   - x64: `cargo build --release --target x86_64-pc-windows-msvc -p pasta_shiori`

---

### Requirement 3: テスト実行

**Objective:** As a 開発者, I want CI でテストが実行されてほしい, so that リグレッションを検知できる

#### Acceptance Criteria

1. The alpha05-build-ci shall ビルド後に `cargo test` を実行する
2. The テスト shall x64 環境で実行する（x86 テストはオプション）
3. The テスト失敗時 shall ワークフローが失敗ステータスとなる

---

### Requirement 4: アーティファクト保存

**Objective:** As a 配布担当者, I want ビルド成果物をダウンロードしたい, so that 手動配布やテストに使える

#### Acceptance Criteria

1. The alpha05-build-ci shall ビルド成果物を GitHub Actions アーティファクトとして保存する
2. The 保存対象 shall 以下を含む:
   - `target/i686-pc-windows-msvc/release/pasta.dll` (x86)
   - `target/x86_64-pc-windows-msvc/release/pasta.dll` (x64)
3. The アーティファクト保持期間 shall 7 日間とする

---

### Requirement 5: キャッシュ設定

**Objective:** As a 開発者, I want ビルド時間を短縮したい, so that CI フィードバックが早くなる

#### Acceptance Criteria

1. The alpha05-build-ci shall Cargo キャッシュを設定する
2. The キャッシュ shall `~/.cargo/registry` と `target/` を対象とする
3. The キャッシュキー shall `Cargo.lock` のハッシュを含む

---

### Requirement 6: マトリックスビルド

**Objective:** As a 開発者, I want ビルド設定を拡張しやすくしたい, so that 将来的なターゲット追加が容易になる

#### Acceptance Criteria

1. The alpha05-build-ci shall GitHub Actions の matrix 機能を使用する
2. The matrix shall `target: [i686-pc-windows-msvc, x86_64-pc-windows-msvc]` を定義する
3. The matrix shall 将来的な追加（Linux, macOS 等）を考慮した構造とする

---

## Out of Scope

- リリースパッケージ作成（alpha06 で実装）
- Linux/macOS 向けビルド
- クロスコンパイル環境構築
- nightly Rust サポート

---

## Glossary

| 用語 | 説明 |
|------|------|
| i686-pc-windows-msvc | Windows 32bit ターゲット |
| x86_64-pc-windows-msvc | Windows 64bit ターゲット |
| pasta.dll | pasta_shiori クレートから生成される SHIORI DLL |
| アーティファクト | GitHub Actions で保存されるビルド成果物 |
