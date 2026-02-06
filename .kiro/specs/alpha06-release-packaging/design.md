# Design Document

## Overview

**Purpose**: hello-pasta ゴーストを `.nar` 形式で GitHub Releases に公開するための手動リリースワークフローを提供する。

**Users**: 配布担当者（開発者本人）がリリース作業を実施し、テスターが GitHub Releases からダウンロードする。

### Goals
- `setup.bat` 生成済みのゴーストを `.nar` に変換する手順の確立
- `gh release create` による GitHub Releases 公開手順の確立
- リリース前の配布物検証手順の確立

### Non-Goals
- GitHub Actions による自動リリース
- x64 版リリース
- SSP ネットワーク更新（自動更新）
- crates.io 公開

## Architecture

### Architecture Pattern & Boundary Map

```mermaid
graph LR
    subgraph Existing
        SetupBat[setup.bat]
        GhostDir[ghosts/hello-pasta/]
    end
    subgraph New
        ReleaseScript[release.ps1]
    end
    subgraph External
        GhRelease[gh release create]
        GitHubReleases[GitHub Releases]
    end

    SetupBat --> GhostDir
    GhostDir --> ReleaseScript
    ReleaseScript -->|hello-pasta.nar| GhRelease
    GhRelease --> GitHubReleases
```

**Architecture Integration**:
- **Selected pattern**: リニアパイプライン（setup.bat → release.ps1 → gh release create）
- **Domain boundaries**: ゴースト生成（setup.bat）とリリースパッケージング（release.ps1）を分離
- **Existing patterns preserved**: `setup.bat` の 4 ステップワークフローに変更なし
- **New components rationale**: `release.ps1` のみ追加。ZIP → .nar 変換とリリース公開を担当
- **Steering compliance**: 既存のスクリプト配置規約（`crates/pasta_sample_ghost/` 配下）に従う

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| CLI | PowerShell 5.1+ | release.ps1 実行環境 | Windows 標準搭載 |
| CLI | GitHub CLI 2.x | リリース公開 | `gh release create` |
| Tool | robocopy | profile/ 除外コピー | Windows 標準搭載 |
| Tool | Compress-Archive | ZIP 圧縮 | PowerShell 標準コマンドレット |

## System Flows

### リリースワークフロー全体

```mermaid
sequenceDiagram
    participant Dev as 配布担当者
    participant SetupBat as setup.bat
    participant ReleasePs1 as release.ps1
    participant GH as gh CLI
    participant GitHub as GitHub Releases

    Dev->>SetupBat: 1. ダブルクリック実行
    SetupBat->>SetupBat: DLL ビルド + ゴースト生成 + コピー + finalize
    SetupBat-->>Dev: Setup Complete!

    Dev->>ReleasePs1: 2. 実行
    ReleasePs1->>ReleasePs1: バージョン確認（Cargo.toml）
    ReleasePs1->>ReleasePs1: 配布物検証（ファイル存在チェック）
    ReleasePs1->>ReleasePs1: profile/ 除外 → 一時ディレクトリ
    ReleasePs1->>ReleasePs1: ZIP 圧縮 → .nar リネーム
    ReleasePs1-->>Dev: hello-pasta.nar 生成完了

    Dev->>GH: 3. gh release create v0.1.1 hello-pasta.nar
    GH->>GitHub: リリース作成 + アセットアップロード
    GitHub-->>Dev: リリース公開完了
```

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1 | ZIP 圧縮 → .nar 変換 | release.ps1 | PackageNar | リリースワークフロー Step 2 |
| 1.2 | .nar 内部構成 | release.ps1（検証） | ValidateGhost | リリースワークフロー Step 2 |
| 1.3 | setup.bat 後の手動操作 | — | — | リリースワークフロー Step 1→2 |
| 2.1 | gh release create 実行 | release.ps1 | PublishRelease | リリースワークフロー Step 3 |
| 2.2 | v\<version\> タグ形式 | release.ps1 | GetVersion | — |
| 2.3 | Cargo.toml バージョン一致 | release.ps1 | GetVersion | — |
| 2.4 | .nar アセット添付 | release.ps1 | PublishRelease | リリースワークフロー Step 3 |
| 2.5 | リリース本文 | リリースノート本文 | — | — |
| 3.1–3.4 | リリースノート + インストール手順 | リリースノート本文 | — | — |
| 4.1 | ファイル存在チェック | release.ps1 | ValidateGhost | リリースワークフロー Step 2 |
| 4.2 | 検証失敗時の中断 | release.ps1 | ValidateGhost | — |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|------------|--------|--------------|-----------------|-----------|
| release.ps1 | パッケージング | ゴースト検証・ZIP 圧縮・.nar 生成・リリース公開 | 1.1, 1.2, 2.1–2.4, 4.1, 4.2 | setup.bat 生成物 (P0), gh CLI (P0) | Batch |
| リリースノート本文 | ドキュメント | GitHub Releases 本文テンプレート | 2.5, 3.1–3.4 | — | — |

### パッケージング

#### release.ps1

| Field | Detail |
|-------|--------|
| Intent | setup.bat 生成済みゴーストの検証・.nar 変換・GitHub Releases 公開 |
| Requirements | 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 4.1, 4.2 |

**Responsibilities & Constraints**
- `ghosts/hello-pasta/` の検証（必須ファイル存在確認）
- `profile/` 除外付き ZIP 圧縮 → `.nar` リネーム
- `Cargo.toml` からバージョン読み取り → タグ名生成
- `gh release create` の実行（ドライラン対応）

**Dependencies**
- Inbound: `ghosts/hello-pasta/` — setup.bat 生成物 (P0)
- External: `gh` CLI — リリース公開 (P0)
- External: `Compress-Archive` — ZIP 圧縮 (P0)
- External: `robocopy` — profile 除外コピー (P0)

**Contracts**: Batch [x]

##### Batch / Job Contract

- **Trigger**: 配布担当者が手動実行（`.\release.ps1`）
- **Input / validation**:
  - `ghosts/hello-pasta/` が存在すること
  - 必須ファイルの存在確認（4.1 のチェックリスト）
  - `Cargo.toml` の `workspace.package.version` が読み取り可能であること
- **Output / destination**:
  - `hello-pasta.nar` — `crates/pasta_sample_ghost/` に生成
  - GitHub Releases に公開（`--dry-run` オプションで公開スキップ可能）
- **Idempotency & recovery**:
  - 既存の `hello-pasta.nar` は上書き
  - 一時ディレクトリは処理完了後にクリーンアップ
  - リリース公開失敗時は `.nar` ファイルが残るため、`gh release create` のみ再実行可能

**Implementation Notes**
- **Integration**: `setup.bat` 実行完了後に `release.ps1` を実行するリニア手順
- **Validation**: ファイル存在チェック（`Test-Path`）。TOML パースは不要（存在確認のみ）
- **Risks**: `Compress-Archive` の ZIP 形式が SSP と非互換の可能性（初回検証で確認）

### ドキュメント

#### リリースノート本文

| Field | Detail |
|-------|--------|
| Intent | GitHub Releases 本文に記載するリリース情報とインストール手順 |
| Requirements | 2.5, 3.1, 3.2, 3.3, 3.4 |

**Responsibilities & Constraints**
- リリースごとにバージョン・概要・コンポーネント情報を記載
- インストール手順（ドラッグ＆ドロップ / 手動展開）を記載
- 動作確認方法と問題報告先を記載
- `gh release create --notes-file` で参照するテキストファイルとして `release.ps1` 内で生成

**Implementation Notes**
- `release.ps1` 内でヒアドキュメントとしてリリースノート本文を生成し、一時ファイルに書き出す
- バージョン番号は `Cargo.toml` から動的に挿入
- インストール手順・動作確認方法は固定テキスト（毎回同じ内容）

## Data Models

該当なし（ファイルシステム操作のみ）。

## Error Handling

### Error Strategy
- `release.ps1` はエラー発生時に即時中断（`$ErrorActionPreference = 'Stop'`）
- 各ステップでエラーメッセージを表示し、原因と対処法を提示

### Error Categories and Responses

| エラー | 原因 | 対処 |
|--------|------|------|
| 配布物不完全 | `setup.bat` 未実行 or 失敗 | `setup.bat` を再実行 |
| Cargo.toml 読み取り失敗 | ワークスペースルートで実行していない | 正しいディレクトリで実行 |
| `gh` CLI 未認証 | GitHub CLI 未ログイン | `gh auth login` を実行 |
| ZIP 圧縮失敗 | ディスク容量不足 or 権限不足 | ディスク確認 / 管理者権限で実行 |
| リリース公開失敗 | ネットワークエラー or タグ重複 | ネットワーク確認 / タグ名変更 |

## Testing Strategy

### 手動テスト
- `setup.bat` 実行後に `release.ps1` を実行し、`hello-pasta.nar` が生成されることを確認
- `.nar` ファイルを SSP にドラッグ＆ドロップしてインストール・動作確認
- `--dry-run` オプションで GitHub Releases 公開をスキップした状態でのパッケージング確認

### 検証チェックリスト
- `hello-pasta.nar` 内に `profile/` が含まれていないこと
- `.nar` のファイルサイズが妥当であること（数 MB 程度）
- SSP でのインストール成功 → ゴースト切り替え → 会話動作確認
