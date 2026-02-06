# Research & Design Decisions: release-workflow

## Summary
- **Feature**: `release-workflow`
- **Discovery Scope**: Extension（既存ツール群の組み合わせによるオペレーション仕様）
- **Key Findings**:
  - `cargo publish` の認証トークン（`~/.cargo/credentials.toml`）が未設定。リリース実行前にトークン設定が必須前提条件となる
  - ルート `Cargo.toml` の4箇所のみでバージョン管理が完結する構造が確認済み
  - 既存リリース `v0.1.2` が完全な参照モデルとして利用可能（タイトル形式、アセット構成）

## Research Log

### cargo publish 認証トークン
- **Context**: gap-analysis.md で「未確認」としていた認証トークンの有無を実際に確認
- **Sources Consulted**: ローカルファイルシステム確認（`~/.cargo/credentials.toml`, `~/.cargo/credentials`）
- **Findings**:
  - `credentials.toml` も `credentials` も存在しない
  - `cargo login` で事前にトークン設定が必要
  - トークンは [crates.io](https://crates.io/) のアカウント設定から取得
- **Implications**: 設計のステップ0（前提条件確認）に `cargo login` のガイダンスを含める必要がある。ただし `cargo login` は一度実行すれば永続化されるため、2回目以降のリリースでは不要

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
  - NAR パス: `crates/pasta_sample_ghost/hello-pasta.nar`
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

### Decision: cargo publish 認証は前提条件チェックで対処
- **Context**: 認証トークンが未設定であることが判明
- **Alternatives Considered**:
  1. リリースフロー内で `cargo login` を自動実行
  2. 前提条件として事前チェックし、未設定ならガイダンスを提示
- **Selected Approach**: 前提条件チェック方式
- **Rationale**: トークンは一度設定すれば永続化される。フロー内での自動 login はセキュリティ上好ましくない
- **Trade-offs**: 初回のみ手動対応が必要
- **Follow-up**: 実装時に `cargo login --help` のガイダンスを含める

### Decision: チェンジログの仕様管理コミット除外
- **Context**: `docs(spec):` や `chore(spec):` のコミットはリリースノートに不要
- **Selected Approach**: LLM が Conventional Commits のプレフィックスとスコープを判定し、仕様管理（spec）スコープのコミットを除外
- **Rationale**: ユーザー向けリリースノートに内部仕様管理の変更は不要
- **Trade-offs**: LLM の判断に依存するが、コンテキスト理解力で十分対応可能

## Risks & Mitigations
- **Risk 1**: `cargo publish` 認証トークン未設定 → 前提条件チェックで初回実行時にガイダンス提供
- **Risk 2**: セッション切断時の中間状態 → 各ステップでコミットを行うため、`git log` で進捗を把握し再開可能
- **Risk 3**: crates.io インデックス更新遅延 → 10秒待機＋確認で対処。不足時は追加待機

## References
- [Conventional Commits](https://www.conventionalcommits.org/) — コミットメッセージ分類基準
- [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html) — crates.io 公開コマンド
- [gh release create](https://cli.github.com/manual/gh_release_create) — GitHub Release 作成コマンド
- gap-analysis.md — 既存アセットとギャップの詳細分析
