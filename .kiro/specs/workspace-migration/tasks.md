# Implementation Tasks

## workspace-migration: Cargo ワークスペース構造への移行

**目標**: 単一クレート構成（pasta）をワークスペース構成（pasta_core + pasta_rune）に移行し、責任分界を明確化し保守性を向上させる

---

## Phase 1: ワークスペース基盤設定

- [ ] 1. ルート Cargo.toml をワークスペース構成に変更
  - `[workspace]` セクション追加、`members = ["crates/*"]` 設定
  - `[workspace.dependencies]` で共有依存関係（pest, thiserror, rune, rand 等）をバージョン一元管理
  - `[workspace.package]` で edition, authors, license を定義
  - 既存の `[package]` セクション削除
  - `resolver = "2"` を指定して dependency resolution 一貫性を確保
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 4.1, 4.2, 4.3, 4.4_

- [ ] 2. クレートディレクトリ構造を作成
  - `/crates/pasta_core/` および `/crates/pasta_rune/` ディレクトリを作成
  - 各クレート配下に `src/` を準備
  - `/tests/` と `/tests/common/` をワークスペースルートに配置
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 3. 各クレートの Cargo.toml を作成
  - `pasta_core/Cargo.toml`: pest, pest_derive, thiserror, fast_radix_trie, rand, tracing 依存設定
  - `pasta_rune/Cargo.toml`: pasta_core, rune, thiserror, glob, tracing, rand, futures, toml, fast_radix_trie 依存設定
  - 両クレートで `edition.workspace = true`, `authors.workspace = true`, `license.workspace = true` 使用
  - `publish = true` を設定
  - _Requirements: 2.1, 2.3, 3.1, 3.3_

---

## Phase 2: pasta_core 分離（言語非依存層）

- [ ] 4. パーサーモジュールを pasta_core に移動
  - `src/parser/` を `crates/pasta_core/src/parser/` に移動（mod.rs, ast.rs, grammar.pest を含む）
  - pest, pest_derive 依存関係を確認
  - _Requirements: 2.2, 2.4, 5.1, 9.1_

- [ ] 5. レジストリモジュールを pasta_core に移動
  - `src/registry/` を `crates/pasta_core/src/registry/` に移動（mod.rs, scene_registry.rs, word_registry.rs を含む）
  - fast_radix_trie, rand 依存関係を確認
  - _Requirements: 2.2, 5.1, 9.1_

- [ ] 6. ランタイム検索テーブルを pasta_core に抽出
  - `src/runtime/scene.rs` から SceneTable を `crates/pasta_core/src/registry/scene_table.rs` に抽出
  - `src/runtime/words.rs` から WordTable を `crates/pasta_core/src/registry/word_table.rs` に抽出
  - `src/runtime/random.rs` から RandomSelector トレイト + DefaultRandomSelector を `crates/pasta_core/src/registry/random.rs` に抽出
  - registry/mod.rs で再エクスポート（Rune 依存を削除して言語非依存化）
  - _Requirements: 2.2, 5.1, 5.2_

- [ ] 7. pasta_core エラー型を定義
  - `crates/pasta_core/src/error.rs` に ParseError, ParseErrorInfo, ParseResult を定義
  - ParseError バリアント: SyntaxError（file, line, column 含む）, PestError, IoError, MultipleErrors
  - thiserror 使用
  - _Requirements: 2.2, 2.4, 9.1_

- [ ] 8. pasta_core lib.rs を作成
  - モジュール宣言: `pub mod error;`, `pub mod parser;`, `pub mod registry;`
  - 公開 API 再エクスポート: ParseError, ParseErrorInfo, ParseResult, parse_file, parse_str, PastaFile, SceneEntry, SceneRegistry, WordDefRegistry, WordEntry
  - _Requirements: 2.1, 9.1_

---

## Phase 3: pasta_rune 分離（Rune実行層）

- [ ] 9. (P) トランスパイラモジュールを pasta_rune に移動
  - `src/transpiler/` を `crates/pasta_rune/src/transpiler/` に移動（mod.rs, code_generator.rs, context.rs, error.rs を含む）
  - インポート更新: `use crate::*` → `use pasta_core::{parser, registry, error}`
  - _Requirements: 3.2, 5.2_

- [ ] 10. (P) ランタイムモジュール（Rune固有部）を pasta_rune に移動
  - `src/runtime/` を `crates/pasta_rune/src/runtime/` に移動（mod.rs, generator.rs, variables.rs を含む）
  - scene.rs, words.rs, random.rs は移動しない（既に pasta_core にある）
  - インポート更新: 検索テーブルは `use pasta_core::registry::{SceneTable, WordTable, RandomSelector}`
  - _Requirements: 3.2, 5.2_

- [ ] 11. (P) Engine, Cache, Loader, IR を pasta_rune に移動
  - `src/engine.rs` → `crates/pasta_rune/src/engine.rs`
  - `src/cache.rs` → `crates/pasta_rune/src/cache.rs`
  - `src/loader.rs` → `crates/pasta_rune/src/loader.rs`
  - `src/ir/` → `crates/pasta_rune/src/ir/`
  - インポート更新: `use pasta_core::{parser, registry, error}`
  - _Requirements: 3.1, 3.2, 5.2_

- [ ] 12. (P) stdlib モジュールを pasta_rune に移動
  - `src/stdlib/` を `crates/pasta_rune/src/stdlib/` に移動（mod.rs, persistence.rs を含む）
  - インポート確認（pasta_core 依存がないことを検証）
  - _Requirements: 3.2, 5.2_

- [ ] 13. pasta_rune エラー型を定義
  - `crates/pasta_rune/src/error.rs` に PastaError を定義
  - ParseError から自動変換: `#[from] pasta_core::error::ParseError`
  - ランタイムエラーバリアント含む: SceneNotFound, RuneCompileError, VmError, IoError
  - `pub type Result<T> = std::result::Result<T, PastaError>;`
  - _Requirements: 3.2, 9.2_

- [ ] 14. pasta_rune lib.rs を作成
  - モジュール宣言: `pub mod cache;`, `pub mod engine;`, `pub mod error;`, `pub mod ir;`, `mod loader;` (private), `pub mod runtime;`, `pub mod stdlib;`, `pub mod transpiler;`
  - 間接公開: `pub use pasta_core as core;`
  - 公開 API 再エクスポート: ParseCache, PastaEngine, PastaError, Result, ScriptEvent, ContentPart, DirectoryLoader, LoadedFiles, RandomSelector, SceneTable, ScriptGenerator 等
  - _Requirements: 3.1, 3.4, 9.2_

---

## Phase 4: テスト・ドキュメント統合

- [ ] 15. テスト共通ユーティリティを作成
  - `tests/common/mod.rs` に `fixtures_path()`, `fixture(name: &str)` 関数を実装
  - ワークスペースレベル `/tests/fixtures/` へのパス解決を両クレート対応で実装
  - `CARGO_MANIFEST_DIR` + ナビゲーションロジック使用
  - _Requirements: 6.5, 6.6_

- [ ] 16. テストインポートを更新・実行
  - `/tests/` 内すべてのテストファイルのインポート更新: `use pasta_rune::core::{parser, registry, error};`, `use pasta_rune::{engine, runtime, ...};`
  - フィクスチャパス参照: `common::fixture(...)` を使用
  - `cargo test --workspace` 実行、すべてテスト成功確認（parser, transpiler, engine, integration）
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 17. ドキュメントを更新
  - `.kiro/steering/structure.md`: ワークスペース構造、/crates/ 階層を記述
  - `README.md`: ビルド手順 `cargo build --workspace`, `cargo test --workspace` に更新、ワークスペースアーキテクチャ概要追加
  - `.kiro/steering/tech.md`: ワークスペース構成をアーキテクチャ原則に追加、pasta_rune → pasta_core 依存関係図含める
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

---

## 実行戦略

### 依存関係
- **Phase 1 → Phase 2 → Phase 3 → Phase 4** : 順序実行が必須
- **Phase 3 内**: タスク 9, 10, 11, 12 は並列実行可（`(P)` マーク）
  - タスク 9, 10, 11, 12 完了後 → タスク 13, 14 順序実行
- **Phase 4**: タスク 15, 16, 17 順序実行

### 検証
- 各 Phase 終了後: `cargo check --workspace`, `cargo build --workspace`
- 最終: `cargo test --workspace`

---

## 要件カバレッジ

| 要件グループ | タスク ID | 説明 |
|-----------|---------|------|
| 1, 4 | 1 | ワークスペース構成 |
| 2, 9 | 4, 5, 6, 7, 8 | pasta_core 分離（言語非依存） |
| 3, 9 | 9, 10, 11, 12, 13, 14 | pasta_rune 分離（Rune層） |
| 5, 6 | 2, 15, 16 | ディレクトリ移行・テスト継続性 |
| 7 | 16 | ビルド互換性 |
| 8 | 17 | ドキュメント更新 |

**全 9 要件カバレッジ: 100%**
**全 14 デザイン要素カバレッジ: 100%**

---

## 成功基準

- [x] ✅ すべての要件（9/9）がタスク（17個）にマッピング
- [x] ✅ すべてのコンポーネント（14/14）が設計から実装タスクに反映
- [x] ✅ タスク依存関係が明確で循環なし
- [x] ✅ 各タスクは 1-3 時間程度の作業量
- [x] ✅ 並列実行可能なタスク（Phase 3）に `(P)` マーク付与
- [x] ✅ テスト・検証タスク含まれている
- [x] ✅ 2 階層構造（大タスク + サブタスク不要、番号付けシンプル）

---

## 実装開始

最初のタスク実行:
```bash
/kiro-spec-impl workspace-migration 1
```

複数タスク実行（コンテキスト管理に注意）:
```bash
/kiro-spec-impl workspace-migration 1,2,3
```

すべてのタスク（非推奨 - コンテキスト上限に注意）:
```bash
/kiro-spec-impl workspace-migration
```
