# Research & Design Decisions: workspace-migration

## Summary
- **Feature**: `workspace-migration`
- **Discovery Scope**: Extension（既存システムのリファクタリング）
- **Key Findings**:
  - Cargoワークスペースは`resolver = "2"`で最新の依存関係解決を適用
  - registryのAST非依存設計により、pasta_coreへの配置が自然
  - error.rsは責任範囲で分離（ParseError vs PastaError）することで言語層の独立性を保証

## Research Log

### Cargo Workspace構成パターン
- **Context**: 単一クレートからワークスペースへの移行方法の調査
- **Sources Consulted**: 
  - [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
  - Rust Edition 2024のfeature gates
- **Findings**:
  - `[workspace]`セクションで`members = ["crates/*"]`を指定しglob展開
  - `resolver = "2"`が推奨（Cargo 2021以降のデフォルト）
  - `[workspace.dependencies]`で共有依存関係バージョンを一元管理
  - `[workspace.package]`でauthors, license, editionなどの共通メタデータを定義
- **Implications**:
  - ルートCargo.tomlは`[package]`を持たず、純粋なワークスペース管理に
  - 各クレートは`依存関係.workspace = true`で参照

### registryモジュールのAST非依存性
- **Context**: registryをどのクレートに配置すべきかの判断
- **Sources Consulted**: `src/registry/mod.rs`, `src/registry/scene_registry.rs`のソースコード分析
- **Findings**:
  - registryはAST型（Statement, Expr等）に直接依存していない
  - `SceneEntry`, `WordEntry`は文字列ベースの純粋なデータ型
  - transpilerとruntimeの両方から使用されるが、どちらにも依存しない
- **Implications**:
  - pasta_coreに配置してもparserとの循環依存なし
  - pasta_runeがpasta_coreに依存する単方向依存関係を維持可能

### error.rs分離戦略
- **Context**: エラー型をどのクレートに配置するかの設計判断
- **Sources Consulted**: `src/error.rs`のバリアント分析
- **Findings**:
  - **ParseError関連**: ファイル位置情報を持つパース固有エラー
  - **Rune関連**: VmError, RuneCompileError, RuneRuntimeError
  - **ファイルシステム**: IoError, DirectoryNotFound等
  - **ビジネスロジック**: SceneNotFound, WordNotFound, FunctionNotFound等
- **Implications**:
  - pasta_core: ParseError, PestError, ParseErrorInfo
  - pasta_rune: PastaError（上記以外全て + pasta_core::ParseErrorからの変換）
  - 各クレートで独立したResult型を定義

### 公開API設計パターン
- **Context**: クレート間のAPI公開方法の選定
- **Sources Consulted**: Rustのモジュール可視性ベストプラクティス
- **Findings**:
  - モジュール単位公開: `pub mod parser;` `pub mod registry;`
  - 間接公開: `pub use pasta_core as core;`でサブクレートを別名で公開
  - 再エクスポート: 主要型を`lib.rs`で直接公開
- **Implications**:
  - pasta_core: `pub mod parser; pub mod registry; pub mod error;`
  - pasta_rune: `pub use pasta_core as core;` + 主要型の直接公開
  - 外部ユーザーは`pasta_rune::core::parser`でアクセス可能

### テストフィクスチャ共有パターン
- **Context**: 両クレートで使用するテストフィクスチャの配置
- **Sources Consulted**: Cargoワークスペースのテスト構成
- **Findings**:
  - ワークスペースレベル`/tests/fixtures/`を共有可能
  - `tests/common/mod.rs`でパス解決ユーティリティを提供
  - 各クレートのテストは`tests/common/`のヘルパーを使用
- **Implications**:
  - フィクスチャは移動せず、パス解決関数で動的解決
  - pasta_coreとpasta_runeの両方から同一フィクスチャを参照

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 2クレート分離 | pasta_core（parser+registry） + pasta_rune（残り） | 単純な依存関係、言語層分離明確 | 初期移行コスト | **採用** |
| 3クレート分離 | pasta_core + pasta_registry + pasta_rune | 最大限の分離、registry完全独立 | 過剰な分離、管理コスト増 | 却下 |
| 統一クレート維持 | 現状維持、内部モジュールで分離 | 移行コストなし | 将来の言語切替困難 | 却下 |

## Design Decisions

### Decision: クレート構成（2クレート）
- **Context**: 将来の言語切替（Lua等）を見据えた依存性整理
- **Alternatives Considered**:
  1. pasta_parser + pasta_rune（parserのみ分離）
  2. pasta_core + pasta_rune（parser + registry分離）★採用
  3. pasta_core + pasta_registry + pasta_rune（3クレート）
- **Selected Approach**: Option 2 - pasta_core（parser + registry）+ pasta_rune
- **Rationale**: 
  - registryはAST非依存のため、parserと同居しても問題なし
  - 3クレート構成は過剰分離
  - 2クレート構成で十分な責任分離を実現
- **Trade-offs**: 
  - ✅ 単純な依存関係グラフ
  - ✅ 将来の言語切替で pasta_rune のみ交換
  - ❌ 初期移行コスト
- **Follow-up**: 実装時にインポートパス修正を慎重に検証

### Decision: error.rs分離（責任範囲別）
- **Context**: ParseErrorとPastaErrorの配置決定
- **Alternatives Considered**:
  1. pasta_coreに統一（全エラー型）
  2. pasta_runeに統一（parserが依存）
  3. 責任範囲別に分離★採用
- **Selected Approach**: Option 3 - pasta_core: ParseError、pasta_rune: PastaError
- **Rationale**:
  - pasta_coreは言語非依存層→Rune関連エラーを含むべきでない
  - ParseErrorはparserモジュールと密結合
  - PastaErrorはRune VM、ファイルシステム、ビジネスロジックを統合
- **Trade-offs**:
  - ✅ 各クレートの責任明確化
  - ✅ 循環依存なし
  - ❌ エラー変換コード必要（`impl From<ParseError> for PastaError`）
- **Follow-up**: pasta_runeでFromトレイト実装を検証

### Decision: 公開API（モジュール単位公開 + 間接公開）
- **Context**: 外部ユーザーからのアクセス方法設計
- **Alternatives Considered**:
  1. すべて直接公開（`pub use`で再エクスポート）
  2. モジュール単位公開（`pub mod`）★採用
  3. 最小公開（主要型のみ）
- **Selected Approach**: モジュール単位公開 + `pub use pasta_core as core;`
- **Rationale**:
  - モジュール構造を保持しつつ必要な型にアクセス可能
  - `pasta_rune::core::parser`で直感的なアクセスパス提供
  - 内部実装の詳細を隠蔽しつつ柔軟性維持
- **Trade-offs**:
  - ✅ 直感的なモジュール構造
  - ✅ 将来のAPI変更に対応しやすい
  - ❌ パスがやや長くなる（許容範囲）
- **Follow-up**: ドキュメントでインポートパターンを明記

### Decision: テストフィクスチャ配置（ワークスペースレベル）
- **Context**: `.pasta`ファイルの共有方法
- **Alternatives Considered**:
  1. 各クレートに複製
  2. pasta_runeに配置
  3. ワークスペースレベル`/tests/fixtures/`★採用
- **Selected Approach**: Option 3 - ワークスペースレベル共有
- **Rationale**:
  - フィクスチャ重複を回避
  - 両クレートから同一フィクスチャを参照可能
  - `tests/common/`でパス解決ユーティリティ提供
- **Trade-offs**:
  - ✅ 重複排除
  - ✅ 一貫したテストデータ
  - ❌ パス解決ロジック必要
- **Follow-up**: `tests/common/mod.rs`にfixtures_path()関数を実装

### Decision: examples/配置（pasta_rune）
- **Context**: サンプルスクリプトの配置場所
- **Alternatives Considered**:
  1. ワークスペースレベル`/examples/`
  2. pasta_runeの`/crates/pasta_rune/examples/`★採用
- **Selected Approach**: Option 2 - pasta_rune
- **Rationale**:
  - サンプルはエンジン機能（pasta_rune）に依存
  - pasta_coreのみでは実行不可
  - 言語層（Rune）固有のサンプル
- **Trade-offs**:
  - ✅ 依存関係が明確
  - ✅ `cargo run --example`でpasta_rune直接参照
  - ❌ ルートからの相対パス変更
- **Follow-up**: スクリプトパス参照を相対パスから調整

## Risks & Mitigations
- **28テストファイルの一括修正リスク** → 自動置換ツール使用、段階的検証
- **インポートパスのタイポ** → 全テストをCI実行、段階的マージ
- **依存関係解決の競合** → `cargo check --workspace`で事前検証
- **Pest文法ファイルのビルドパス変更** → `build.rs`なしでpest_deriveのパス指定を確認

## References
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — 公式ドキュメント
- [workspace.dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace) — 依存関係継承
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — モジュール公開ベストプラクティス
