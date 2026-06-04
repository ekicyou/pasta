# Research & Design Decisions: release-workflow

## Summary
- **Feature**: `release-workflow`
- **Discovery Scope**: Extension（既存ツール群の組み合わせによるオペレーション仕様）
- **Key Findings**:
  - `cargo publish` の認証は環境変数で有効であり、前提条件チェックは不要（`gh auth status` のみ確認すれば十分）
  - ルート `Cargo.toml` の5箇所のみでバージョン管理が完結する構造が確認済み
  - 既存リリース `v0.1.2` が完全な参照モデルとして利用可能（タイトル形式、アセット構成）

## Research Log

### cargo publish 認証トークン
- **Context**: gap-analysis.md で「未確認」としていた認証トークンの有無を実際に確認
- **Sources Consulted**: ローカルファイルシステム確認（`~/.cargo/credentials.toml`, `~/.cargo/credentials`）、環境変数
- **Findings**:
  - ファイルベースの credentials は存在しないが、環境変数 `CARGO_REGISTRY_TOKEN` による認証が有効
  - 過去のリリースで `cargo publish` が正常に動作していることを確認済み
  - cargo は環境変数とファイルの両方をサポートしている
- **Implications**: cargo publish の認証チェックは不要。環境変数による認証が既に有効であり、Phase 0 での前提条件確認は `gh auth status` のみで十分

### Cargo.toml バージョン更新箇所
- **Context**: gap-analysis.md で確認済みだが、設計のための正確な行番号を再確認
- **Sources Consulted**: `Cargo.toml` 直接読み取り
- **Findings**:
  - Line 9: `version = "0.1.2"` — `[workspace.package]` セクション
  - Line 47: `pasta_core = { path = "crates/pasta_core", version = "0.1.2" }`
  - Line 48: `pasta_lua = { path = "crates/pasta_lua", version = "0.1.2" }`
  - Line 49: `pasta_shiori = { path = "crates/pasta_shiori", version = "0.1.2" }`
  - 個別クレートの `Cargo.toml` は `version.workspace = true` で継承（更新不要）
- **Implications**: `replace_string_in_file` で旧バージョン文字列を新バージョンに4回置換すれば完了

### 既存リリース構造（v0.1.2）
- **Context**: GitHub Release 作成時のコマンドとパラメータの参照モデル
- **Sources Consulted**: gap-analysis.md の記録、RELEASE.md のテンプレート
- **Findings**:
  - タイトル形式: `pasta vX.Y.Z`
  - アセット: `pasta.dll` (2.59 MiB), `hello-pasta.nar` (1.29 MiB)
  - DLL パス: `target/i686-pc-windows-msvc/release/pasta.dll`
  - NAR パス: `release/hello-pasta.nar`（release.ps1 が WorkspaceRoot の release/hello-pasta.nar に出力）
- **Implications**: `gh release create` のコマンド構築時にこれらのパスとタイトル形式を使用

### チェンジログ生成パターン
- **Context**: 議題1で決定済み — `git log` + LLM 手動整形方式
- **Sources Consulted**: Conventional Commits 仕様、`git log` 出力のサンプル
- **Findings**:
  - プロジェクトのコミットメッセージは `type(scope): summary` 形式に従っている
  - 分類カテゴリ: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`
  - グループ見出し: `### ✨ Features`, `### 🐛 Bug Fixes`, `### 📝 Documentation` 等
  - `chore(spec):` や `docs(spec):` のような仕様管理コミットはリリースノートから除外が望ましい
- **Implications**: LLM がコミット履歴を読み取り、ユーザー向けに有意義なエントリのみを整形する

### release.ps1 実行フロー
- **Context**: gap-analysis.md の分析に基づく実行手順の確認
- **Sources Consulted**: gap-analysis.md（387行スクリプト、8ステップ構成）
- **Findings**:
  - 実行ディレクトリ: `crates/pasta_sample_ghost/`
  - 実行コマンド: `PowerShell -ExecutionPolicy Bypass -File release.ps1`
  - 出力: `hello-pasta.nar` + `target/i686-pc-windows-msvc/release/pasta.dll`
  - 前提: `i686-pc-windows-msvc` ターゲットがインストール済み（✅確認済み）
- **Implications**: ステップ4で `Push-Location` + `release.ps1` 実行 + `Pop-Location` の流れ

## 並行作業性の分析（cc-sdd 3.0 書き直しで追加）

### Context
旧設計は全工程を単一の Sequential Pipeline として直列実行していた。本書き直しでは、各処理が要求する共有リソースを実コードから確認し、安全に並行化できる箇所と偽の依存関係を特定した。

### Sources Consulted
- `crates/pasta_sample_ghost/release.ps1`（Step 1: `cargo build --release --target i686-pc-windows-msvc -p pasta_shiori`）
- `editors/vscode/package.json`（`prepackage` = `build:wasm` → `compile`、`build:wasm` = `powershell -File scripts/build-wasm.ps1`）
- `Cargo.toml`（workspace 構造、6箇所のバージョン参照）
- `cargo publish` の挙動（既定で検証ビルド + クリーンワークツリー要求）

### Findings — 共有リソースモデル
| リソース | 種別 | 保持する処理 |
| --- | --- | --- |
| R1: cargo ターゲットロック | 排他 | `cargo build/test/run/publish`、VSCode `build:wasm` |
| R2: git ワークツリー＋index | 排他 | ファイル生成、`git add/commit/restore/tag`、`release.ps1` |
| R3: ネットワーク | 非排他 | `cargo publish` upload/index待機、`vsce publish`、`gh release create` |

### Findings — 偽の依存関係
- 旧設計は「crates.io 公開（Phase 3）→ ゴーストビルド（Phase 5）」と直列化していたが、`release.ps1` は **ローカルソースから** pasta.dll をビルドしており crates.io 公開済みクレートに依存しない。よってゴーストビルドは crates.io 公開に**非依存**であり、バージョン更新コミットにのみ依存する。

### Implications — スケジューリング決定
- R1・R2 を共有する全ローカルビルドは真の並行ができないため、1つの直列ステージ（Stage A）に集約し、ワークツリーをクリーン化してから crates.io 公開を開始する。
- crates.io 公開（Track X）・Marketplace 公開（Track Y）・チェンジログ生成（Track Z）は R2 を変更せず互いに独立するため Stage B として**並行実行可能**。特に非クリティカルな `vsce publish`（R3 のみ）を Track X の長いインデックス待機に重ねることで wall-clock を短縮する。
- 不可逆な crates.io 公開（Track X）の成功を Stage C（タグ・プッシュ）の前提とし、安全順序を保証する。
- VSCode は `build:wasm`（R1）を要するため**ビルドは Stage A**、Marketplace への **upload（R3 のみ）は Stage B Track Y** に分離する。

### Decision: Resource-Aware Staged Concurrency（採用）
- **Selected Approach**: Stage A（ローカル直列）→ Stage B（並行3トラック X∥Y∥Z）→ Stage C（タグ・プッシュ）→ Stage D（GitHub Release）
- **Rationale**: 排他リソース制約を尊重しつつ、独立したネットワークトラックを並行化して所要時間を短縮し、非クリティカル失敗を隔離する。
- **Trade-offs**: 並行トラックの完了個別検証が必要（Req 8.7）。逐次環境ではインターリーブ近似となる。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 完全インタラクティブ | LLM が各ステップを逐次実行 | 柔軟なエラー対応、結果確認、チェンジログ整形が自然 | セッション切断リスク、実行時間 | **採用** — 仕様の趣旨に最適 |
| B: ラッパースクリプト | 全工程を自動化スクリプト化 | 再現性、実行速度 | 対話要素の処理困難、保守コスト | 不採用 — 仕様の趣旨と矛盾 |
| C: ハイブリッド | 部分スクリプト化 | チェンジログ品質安定 | 境界管理の煩雑さ | 不採用 — 不要な複雑性 |

## Design Decisions

### Decision: 完全インタラクティブ実行（Option A）
- **Context**: 本仕様は LLM が繰り返しリリース作業を実行するオペレーション仕様
- **Alternatives Considered**:
  1. Option A — LLM が各ステップをターミナルで逐次実行
  2. Option B — ラッパースクリプト作成
  3. Option C — 部分スクリプト化
- **Selected Approach**: Option A — LLM が `run_in_terminal` と `replace_string_in_file` を組み合わせて実行
- **Rationale**: 仕様の趣旨（LLM による繰り返し実行）に最適。エラー時の柔軟な判断、チェンジログの知的な整形が可能
- **Trade-offs**: 実行時間は長いが、品質と柔軟性を優先
- **Follow-up**: セッション切断時の中間状態からの復旧手順を設計に含める



### Decision: チェンジログの仕様管理コミット除外
- **Context**: `docs(spec):` や `chore(spec):` のコミットはリリースノートに不要
- **Selected Approach**: LLM が Conventional Commits のプレフィックスとスコープを判定し、仕様管理（spec）スコープのコミットを除外
- **Rationale**: ユーザー向けリリースノートに内部仕様管理の変更は不要
- **Trade-offs**: LLM の判断に依存するが、コンテキスト理解力で十分対応可能

## Risks & Mitigations
- **Risk 1**: セッション切断時の中間状態 → 各ステップでコミットを行うため、`git log` で進捗を把握し再開可能
- **Risk 2**: crates.io インデックス更新遅延 → 10秒待機＋確認で対処。不足時は追加待機
- **Risk 3**: `gh` CLI 認証切れ → Phase 0 で `gh auth status` を確認し、未認証ならガイダンス提示

## References
- [Conventional Commits](https://www.conventionalcommits.org/) — コミットメッセージ分類基準
- [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) — crates.io 公開コマンド
- [gh release create](https://cli.github.com/manual/gh_release_create) — GitHub Release 作成コマンド
- gap-analysis.md — 既存アセットとギャップの詳細分析
