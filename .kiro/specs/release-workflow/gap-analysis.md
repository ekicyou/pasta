# Gap Analysis: release-workflow

## 1. 現状調査

### 1.1 既存リソースの棚卸し

| 要件領域 | 既存アセット | 状態 |
|----------|-------------|------|
| バージョン管理 | `Cargo.toml` (`[workspace.package].version = "0.1.2"`) | ✅ ワークスペース集中管理済み |
| クレートバージョン参照 | `[workspace.dependencies]` 内 `pasta_core`, `pasta_lua`, `pasta_shiori` | ✅ `version = "0.1.2"` で同期済み |
| 個別クレート Cargo.toml | `version.workspace = true`（全4クレート） | ✅ ワークスペース継承 |
| ゴーストビルド | `crates/pasta_sample_ghost/release.ps1`（387行） | ✅ 成熟したスクリプト |
| ゴーストビルド（bat） | `crates/pasta_sample_ghost/release.bat`（release.ps1のラッパー） | ✅ 利用可能 |
| リリース手順書 | `crates/pasta_sample_ghost/RELEASE.md` | ✅ 手動手順書あり |
| CI/CD | `.github/workflows/build.yml` | ✅ ビルド＋テスト（リリースは未自動化） |
| GitHub CLI | `gh` 認証済み（`ekicyou` アカウント） | ✅ 利用可能 |
| Rust ターゲット | `i686-pc-windows-msvc` インストール済み | ✅ 32bit ビルド可能 |
| 既存タグ | `v0.1.2` | ✅ 1件存在 |
| 既存リリース | `pasta v0.1.2`（pasta.dll + hello-pasta.nar 添付済み） | ✅ 参照モデルあり |

### 1.2 バージョン更新の対象箇所

ルート `Cargo.toml` の更新のみで全クレートに伝播する構造：

```
Cargo.toml (ワークスペースルート)
├── [workspace.package].version = "0.1.2"         ← 更新対象①
├── [workspace.dependencies].pasta_core.version    ← 更新対象②
├── [workspace.dependencies].pasta_lua.version     ← 更新対象③
└── [workspace.dependencies].pasta_shiori.version  ← 更新対象④

crates/*/Cargo.toml
└── version.workspace = true                       ← 自動継承（更新不要）
```

**重要な発見**: バージョンの更新箇所は **ルート Cargo.toml の4箇所のみ**。個別クレートの `Cargo.toml` は `version.workspace = true` で継承しているため変更不要。

### 1.3 cargo publish の依存関係順序

```
pasta_core (依存なし)          → 最初に公開
    ↓
pasta_lua (pasta_core に依存)  → 2番目に公開
    ↓
pasta_shiori (pasta_core, pasta_lua に依存) → 3番目に公開

pasta_sample_ghost (publish = false) → スキップ
```

### 1.4 release.ps1 の動作分析

`release.ps1` は8ステップで構成：
1. **Setup Phase (1-4)**: DLLビルド → ゴースト生成 → ファイルコピー → 更新ファイル生成
2. **Release Phase (5-8)**: バージョンチェック → バリデーション → .nar作成 → 手順表示

**出力ファイル**: `crates/pasta_sample_ghost/hello-pasta.nar`
**DLLパス**: `target/i686-pc-windows-msvc/release/pasta.dll`

スクリプトは `PowerShell -ExecutionPolicy Bypass -File release.ps1` で実行。

### 1.5 既存リリース（v0.1.2）の構造分析

```
gh release create v0.1.2 の構成:
├── タイトル: "pasta v0.1.2"
├── アセット:
│   ├── hello-pasta.nar (1.29 MiB)
│   └── pasta.dll (2.59 MiB)
└── ノート: Markdown形式のリリースノート
```

---

## 2. 要件とのギャップ分析

### Requirement-to-Asset Map

| 要件 | 必要な機能 | 既存アセット | ギャップ |
|------|-----------|-------------|---------|
| **Req 1-1**: バージョン確認 | ユーザー対話 | なし | **LLM対話で実現**（ツール不要） |
| **Req 1-2**: semver 検証 | 正規表現チェック | なし | **LLM判定で十分** |
| **Req 1-4**: git status チェック | `git status` コマンド | なし | **ターミナル実行で実現** |
| **Req 1-6**: テスト実行 | `cargo test --all` | CI の `build.yml` | **ターミナル実行で実現** |
| **Req 2-1/2**: Cargo.toml 更新 | テキスト編集 | `Cargo.toml`（4箇所） | **エディタツールで実現** |
| **Req 2-3**: ビルド確認 | `cargo build --workspace` | なし | **ターミナル実行で実現** |
| **Req 3-1**: cargo publish | `cargo publish -p <crate>` | なし | **ターミナル実行で実現** |
| **Req 3-5**: インデックス待機 | sleep/待機 | なし | **ターミナルで `Start-Sleep`** |
| **Req 4-1**: ゴーストビルド | `release.ps1` | ✅ 成熟スクリプト | ギャップなし |
| **Req 5-1**: タグ作成 | `git tag -a` | なし | **ターミナル実行で実現** |
| **Req 5-4**: push | `git push --tags` | なし | **ターミナル実行で実現** |
| **Req 6-1**: チェンジログ | `git log` 解析 | なし | **Missing**: ログ整形ロジック |
| **Req 6-3**: GitHub Release | `gh release create` | ✅ 既存リリースが参考 | ギャップなし |
| **Req 7**: 繰り返し実行 | spec.json の特殊処理 | なし | **設計で対応**（仕様レベル） |

### ギャップ詳細

#### Missing: チェンジログ生成ロジック（Req 6-1, 6-2）

**決定事項**: `git log` を LLM が手動整形する方式を採用。

**理由**: 
- 現在の開発フローは直接 main ブランチへのコミット（プルリクエストベースではない）
- `gh release create --generate-notes` はプルリクエストベースの履歴生成に最適化されているため不適切
- Conventional Commits 形式に従ったコミットメッセージを LLM が分類・整形することで、適切なチェンジログを生成可能

**実装方針**:
- `git log <前回タグ>..HEAD --oneline` でコミット履歴を取得
- LLM が Conventional Commits プレフィックス（`feat:`, `fix:`, `chore:` 等）で種別を判定
- 種別ごとにグループ化し Markdown 形式のチェンジログを生成

#### Constraint: cargo publish の認証

- `cargo publish` にはcrates.ioのAPIトークンが必要
- ローカル環境に `~/.cargo/credentials.toml` が必要
- **未確認**: 現在のローカル環境にトークンが設定されているか

#### Constraint: release.ps1 の実行環境

- PowerShell + Windows 環境限定
- `i686-pc-windows-msvc` ターゲットが必須（✅ インストール確認済み）
- `robocopy` コマンド依存（Windows標準）

---

## 3. 実装アプローチ評価

### 本仕様の特性

本仕様はコードの新規作成・変更を行わない。LLMエージェントがターミナルコマンドとエディタツールを使って一連のリリース手順を実行する「オペレーション仕様」である。

### Option A: 完全インタラクティブ実行（推奨）

**概要**: LLMエージェントが各ステップをターミナルで逐次実行し、結果を確認しながら進行。

- `git status` → ターミナル実行、出力確認
- `Cargo.toml` 編集 → エディタツール（`replace_string_in_file`）
- `cargo build` / `cargo test` → ターミナル実行
- `cargo publish` → ターミナル実行（順次）
- `release.ps1` → ターミナル実行
- `git tag` / `git push` → ターミナル実行
- チェンジログ生成 → `git log` 出力を LLM が整形
- `gh release create` → ターミナル実行

**Trade-offs**:
- ✅ 各ステップで結果確認・エラー対応が可能
- ✅ 新規ファイル作成不要、既存ツールのみで実現
- ✅ LLM の判断力でエッジケースに柔軟対応
- ❌ 実行時間が長い（各コマンドの待機）
- ❌ セッション切断時に中間状態になるリスク

### Option B: ラッパースクリプト作成

**概要**: リリース全工程を自動化するPowerShellスクリプトを新規作成。

**Trade-offs**:
- ✅ 実行時間短縮、再現性が高い
- ❌ スクリプト保守コスト
- ❌ エラー時の柔軟な対応が困難
- ❌ バージョン番号の入力など対話要素の処理が複雑
- ❌ 仕様の趣旨（LLMが実行）と矛盾

### Option C: ハイブリッド（部分スクリプト化）

**概要**: チェンジログ生成部分のみスクリプト化し、他はインタラクティブ実行。

**Trade-offs**:
- ✅ チェンジログ整形の品質安定化
- ❌ スクリプトとインタラクティブの境界管理が煩雑
- ❌ LLMがコミットログを直接読めるので不要

---

## 4. 複雑度・リスク評価

### Effort: **S**（1-3日）
- 既存のコマンドとツールの組み合わせのみ
- 新規コード作成なし
- `release.ps1` が最も重い処理だが既に成熟

### Risk: **Low**
- 全ツール（cargo, git, gh, release.ps1）が確認済みで利用可能
- v0.1.2の成功リリースが参照モデルとして存在
- ロールバック手順も明確（git reset, tag 削除）
- 唯一の不確定要素: `cargo publish` の認証トークン有無

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（完全インタラクティブ実行）

本仕様の趣旨（LLMが繰り返しリリース作業を実行）に最も適合する。

### 設計フェーズで決定すべき事項

1. **チェンジログ形式**: `git log --oneline` を LLM が Conventional Commits 形式で分類・整形するか、`gh release create --generate-notes` に委ねるか
2. **cargo publish 失敗時のリカバリ**: 途中で失敗した場合、既に公開されたクレートのバージョンを巻き戻す必要があるか（通常は yank で対応）
3. **タスクの粒度**: 各ステップを独立タスクとして定義するか、フェーズ単位でグループ化するか

### Research Needed（設計フェーズで調査）

1. `cargo publish` のローカル認証トークン有無の確認方法
2. `cargo publish` 後のインデックス反映待ち時間の最適値
3. `gh release create --generate-notes` の出力品質評価
