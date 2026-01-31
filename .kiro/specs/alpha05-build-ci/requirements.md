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

### 成功基準

- mainブランチへのpush/PRで自動ビルドが実行される
- x86/x64両アーキテクチャでDLLがビルドされる
- 全テストがパスする
- ビルド成果物がダウンロード可能

---

## Requirements

### Requirement 1: ビルドワークフロー定義

**Objective:** As a 開発者, I want push/PR 時に自動ビルドが走ってほしい, so that ビルド破損を早期検知できる

#### Acceptance Criteria

1. The CI shall ワークフロー定義ファイルを `.github/workflows/build.yml` に作成する
2. When mainブランチへのpushが発生した時, the CI shall 自動的にビルドジョブを開始する
3. When mainブランチへのPRが作成/更新された時, the CI shall 自動的にビルドジョブを開始する
4. The ワークフロー shall `windows-latest` ランナーを使用する
5. The ジョブ名 shall ビルドターゲットを識別可能な形式（例: `build-i686-pc-windows-msvc`）とする

---

### Requirement 2: Rust ツールチェーン設定

**Objective:** As a 開発者, I want 適切な Rust ツールチェーンでビルドしたい, so that 正しいターゲットで DLL が生成される

#### Acceptance Criteria

1. The CI shall `dtolnay/rust-toolchain` アクションを使用してRustをインストールする
2. The CI shall `stable` ツールチェーンを指定する
3. The CI shall ビルドターゲットに応じて以下のターゲットを追加する:
   - `i686-pc-windows-msvc`（x86ビルド用）
   - `x86_64-pc-windows-msvc`（x64ビルド用）
4. The ツールチェーンバージョン shall GitHub Actions のログで確認可能である（dtolnay/rust-toolchain が自動出力）

---

### Requirement 3: DLLビルド実行

**Objective:** As a 開発者, I want pasta.dll が正しくビルドされてほしい, so that SHIORI DLLとして配布できる

#### Acceptance Criteria

1. The CI shall `pasta_shiori` クレートをreleaseモードでビルドする
2. When x86ビルドを実行する時, the CI shall `--target i686-pc-windows-msvc` を指定する
3. When x64ビルドを実行する時, the CI shall `--target x86_64-pc-windows-msvc` を指定する
4. The ビルドコマンド shall 以下の形式を使用する:
   ```
   cargo build --release --target <target> -p pasta_shiori
   ```
5. If ビルドエラーが発生した時, the CI shall 非ゼロ終了コードでジョブを失敗させる

---

### Requirement 4: テスト実行

**Objective:** As a 開発者, I want CI でテストが実行されてほしい, so that リグレッションを検知できる

#### Acceptance Criteria

1. The CI shall ビルド成功後に `cargo test --all` を実行する
2. The テスト shall x64環境（デフォルトターゲット）で実行する
3. If テストが失敗した時, the CI shall 非ゼロ終了コードでジョブを失敗させる
4. The テスト結果 shall GitHub Actions のログに出力される
5. Where x86テストが必要な場合, the CI shall `--target i686-pc-windows-msvc` でテストを実行可能とする（オプション）

---

### Requirement 5: アーティファクト保存

**Objective:** As a 配布担当者, I want ビルド成果物をダウンロードしたい, so that 手動配布やテストに使える

#### Acceptance Criteria

1. The CI shall `actions/upload-artifact` を使用してビルド成果物を保存する
2. The アーティファクト shall 以下のファイルを含む:
   - `target/i686-pc-windows-msvc/release/pasta.dll`（x86）
   - `target/x86_64-pc-windows-msvc/release/pasta.dll`（x64）
3. The アーティファクト名 shall ターゲットアーキテクチャを識別可能とする（例: `pasta-dll-x86`, `pasta-dll-x64`）
4. The アーティファクト保持期間 shall 7日間とする
5. When ビルドが成功した時, the CI shall アーティファクトをアップロードする

---

### Requirement 6: ビルドキャッシュ

**Objective:** As a 開発者, I want ビルド時間を短縮したい, so that CI フィードバックが早くなる

#### Acceptance Criteria

1. The CI shall `Swatinem/rust-cache` アクションを使用してCargoキャッシュを設定する
2. The キャッシュ shall 以下を対象とする:
   - `~/.cargo/registry/`（クレートレジストリ）
   - `~/.cargo/git/`（gitソース）
   - `target/`（ビルド成果物）
3. The キャッシュキー shall `Cargo.lock` のハッシュを含む
4. When キャッシュがヒットした時, the CI shall 依存クレートの再ダウンロードをスキップする
5. The キャッシュ shall ビルドターゲットごとに分離する

---

### Requirement 7: マトリックスビルド構成

**Objective:** As a 開発者, I want ビルド設定を拡張しやすくしたい, so that 将来的なターゲット追加が容易になる

#### Acceptance Criteria

1. The CI shall GitHub Actions の `strategy.matrix` 機能を使用する
2. The matrix shall 以下の変数を定義する:
   - `target: [i686-pc-windows-msvc, x86_64-pc-windows-msvc]`
3. The matrix shall `fail-fast: false` を設定し、一方のビルド失敗が他方に影響しないようにする
4. The ワークフロー shall matrix変数 `${{ matrix.target }}` を各ステップで参照する
5. Where 将来的なターゲット追加が必要な場合, the matrix shall 設定値の追加のみで対応可能な構造とする

---

### Requirement 8: ワークフロー品質

**Objective:** As a 開発者, I want CI設定が保守しやすいものであってほしい, so that 長期的な運用が容易になる

#### Acceptance Criteria

1. The ワークフロー shall 各ステップに明確な `name` を設定する
2. The ワークフロー shall 重要な設定値（保持期間等）を変数またはコメントで説明する
3. The ワークフロー shall YAML構文エラーがない状態で作成する
4. If 手動実行が必要な場合, the CI shall `workflow_dispatch` トリガーをサポートする（オプション）

---

## Out of Scope

- **ビルド成果物のリポジトリコミット** - pasta.dll は .gitignore 対象、必要時に Artifacts からダウンロード
- リリースパッケージ作成（alpha06 で実装）
- Linux/macOS 向けビルド（将来的なmatrix拡張で対応可能）
- クロスコンパイル環境構築
- nightly Rust サポート
- コード署名
- 自動リリース作成

---

## Glossary

| 用語 | 説明 |
|------|------|
| i686-pc-windows-msvc | Windows 32bit ターゲット（MSVC ABIを使用） |
| x86_64-pc-windows-msvc | Windows 64bit ターゲット（MSVC ABIを使用） |
| pasta.dll | pasta_shiori クレートから生成される SHIORI DLL |
| アーティファクト | GitHub Actions で保存されるビルド成果物 |
| matrix | 複数の構成を並列でテストするGitHub Actionsの機能 |
| rust-cache | Cargoビルドキャッシュ用のGitHub Action |
| dtolnay/rust-toolchain | Rustツールチェーンインストール用のGitHub Action |

---

## References

- [GitHub Actions ドキュメント](https://docs.github.com/ja/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
