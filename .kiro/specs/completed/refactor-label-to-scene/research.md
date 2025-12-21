# Research & Design Decisions

## Summary
- **Feature**: `refactor-label-to-scene`
- **Discovery Scope**: Simple Addition（用語リファクタリング、機能変更なし）
- **Key Findings**:
  1. 置換対象ファイル数: ソースファイル2個、テストファイル3個のリネーム + 全プロジェクト内の用語置換
  2. 主要構造体: 8個（LabelRegistry, LabelInfo×2, LabelDef, LabelScope, LabelTable, LabelId, LabelNotFound）
  3. 機械的置換が安全: プロジェクト内で「label」を別の意味で使用している箇所なし

## Research Log

### ファイル名変更対象の特定
- **Context**: リネームが必要なファイルを特定
- **Sources Consulted**: `src/` および `tests/` ディレクトリの検索
- **Findings**:
  - `src/transpiler/label_registry.rs` → `scene_registry.rs`
  - `src/runtime/labels.rs` → `scene.rs`（単数形化）
  - `tests/label_id_consistency_test.rs` → `scene_id_consistency_test.rs`
  - `tests/pasta_engine_label_resolution_test.rs` → `pasta_engine_scene_resolution_test.rs`
  - `tests/pasta_transpiler_label_registry_test.rs` → `pasta_transpiler_scene_registry_test.rs`
- **Implications**: git mv によるリネームで履歴を保持

### 主要構造体・型の特定
- **Context**: IDE Rename で変更すべき型名を特定
- **Sources Consulted**: `src/transpiler/label_registry.rs`, `src/runtime/labels.rs`, `src/parser/ast.rs`, `src/error.rs`
- **Findings**:
  | 現在の型名 | 変更後 | 定義ファイル |
  |----------|-------|------------|
  | `LabelRegistry` | `SceneRegistry` | `src/transpiler/label_registry.rs` |
  | `LabelInfo` (transpiler) | `SceneInfo` | `src/transpiler/label_registry.rs` |
  | `LabelInfo` (runtime) | `SceneInfo` | `src/runtime/labels.rs` |
  | `LabelDef` | `SceneDef` | `src/parser/ast.rs` |
  | `LabelScope` | `SceneScope` | `src/parser/ast.rs` |
  | `LabelTable` | `SceneTable` | `src/runtime/labels.rs` |
  | `LabelId` | `SceneId` | `src/runtime/labels.rs` |
  | `LabelNotFound` | `SceneNotFound` | `src/error.rs` |
- **Implications**: IDE Rename (rust-analyzer) で型安全に変更可能

### 生成されるRuneコードの確認
- **Context**: Transpilerが生成するRuneコード内の「label」使用箇所
- **Sources Consulted**: `src/transpiler/mod.rs`, `src/stdlib/mod.rs`
- **Findings**:
  - `label_selector` 関数名 → `scene_selector`
  - `select_label_to_id` 関数名 → `select_scene_to_id`
  - エラーメッセージ: `"ラベルID ${id} が見つかりませんでした"` → `"シーンID ${id} が見つかりませんでした"`
- **Implications**: Transpilerのstring literalを直接編集

### Markdownドキュメントの置換範囲
- **Context**: 日本語「ラベル」の出現箇所
- **Sources Consulted**: grep検索結果
- **Findings**:
  - 約110箇所の日本語「ラベル」
  - GRAMMAR.md, SPECIFICATION.md, README.md に集中
  - `.kiro/steering/` 4ファイル
  - `.kiro/specs/**/*.md` 100+ファイル
- **Implications**: スクリプトによる一括置換が効率的

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| IDE Rename + スクリプト | rust-analyzer Rename + sed/PowerShell | 型安全、高速 | 複雑な置換パターンで漏れリスク | **採用** |
| 手動リファクタリング | 手作業で全て変更 | 確実 | 時間がかかる、ヒューマンエラー | 非採用 |
| AST変換ツール | syn/quoteでRust ASTを変換 | 精密 | 開発コスト大 | 非採用 |

## Design Decisions

### Decision: 実行順序
- **Context**: ファイルリネーム、型名変更、文字列置換の順序
- **Alternatives Considered**:
  1. 型名変更 → ファイルリネーム → 文字列置換
  2. ファイルリネーム → 型名変更 → 文字列置換
- **Selected Approach**: ファイルリネーム → 型名変更 → 文字列置換
- **Rationale**: ファイルを先にリネームすることで、IDE Renameがmod.rsを自動更新
- **Trade-offs**: 中間状態でコンパイルエラーになるが、最終状態で解消
- **Follow-up**: 各段階でcargo checkを実行して確認

### Decision: 単数形化ルール
- **Context**: `labels.rs` → `scene.rs` or `scenes.rs`
- **Alternatives Considered**:
  1. 機械的に `scenes.rs`
  2. Rust慣例に従い `scene.rs`
- **Selected Approach**: `scene.rs`（単数形）
- **Rationale**: Rust標準ライブラリ・エコシステムでは単数形モジュール名が慣例（`error.rs`, `engine.rs`）
- **Trade-offs**: 機械的置換の例外となる
- **Follow-up**: mod.rsの宣言も手動で確認

## Risks & Mitigations

| リスク | 緩和策 |
|-------|-------|
| 置換漏れ | 最終検証で `grep -r "label\|ラベル"` を実行 |
| コンパイルエラー | 各段階で `cargo check` を実行 |
| テスト失敗 | `cargo test --all` で全テスト実行 |
| Markdownリンク切れ | ドキュメント内リンク確認（手動） |
| git履歴断絶 | `git mv` でファイルリネームし履歴保持 |

## References
- [Rust API Guidelines - Naming](https://rust-lang.github.io/api-guidelines/naming.html)
- [rust-analyzer rename refactoring](https://rust-analyzer.github.io/manual.html#rename)
