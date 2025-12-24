# Research & Design Decisions

---
**Purpose**: parser2/transpiler2移行に関するディスカバリ調査結果と設計判断の記録

---

## Summary
- **Feature**: `engine-parser2-transpiler2-migration`
- **Discovery Scope**: Extension（既存engine.rsの内部実装変更）
- **Key Findings**:
  - engine.rsの変更箇所は3箇所（Import, Parse/Merge, Transpile）に局所化
  - parser2/transpiler2は完成済み、611テスト成功で安定
  - ランタイム層は変更不要、SceneTable/WordTable APIは互換維持

## Research Log

### AST構造の差異調査
- **Context**: 旧/新PastaFileの構造差異を把握し、マージロジック設計に反映
- **Sources Consulted**: `src/parser/ast.rs`, `src/parser2/ast.rs`, gap-analysis.md
- **Findings**:
  - 旧PastaFile: `{ path, global_words: Vec<WordDef>, scenes: Vec<SceneDef>, span }`
  - 新PastaFile: `{ path, items: Vec<FileItem>, span }`
  - FileItem = `FileAttr | GlobalWord | GlobalSceneScope`
  - ヘルパーメソッド: `file_attrs()`, `words()`, `global_scene_scopes()`利用可能
- **Implications**: マージ時にitemsをextendするだけで順序を保持できる

### Transpiler API差異調査
- **Context**: 2-pass呼び出しへの移行方法を明確化
- **Sources Consulted**: `src/transpiler/mod.rs`, `src/transpiler2/mod.rs`
- **Findings**:
  - 旧API: `Transpiler::transpile_with_registry(&PastaFile) -> (String, SceneRegistry, WordDefRegistry)`
  - 新API: 
    - `Transpiler2::transpile_pass1<W: std::io::Write>(&PastaFile, &mut SceneRegistry, &mut WordDefRegistry, &mut W)`
    - `Transpiler2::transpile_pass2<W: std::io::Write>(&SceneRegistry, &mut W)`
  - 戻り値がwriterパターンに変更、レジストリは事前作成が必要
  - **検証済み**: `Vec<u8>`は`std::io::Write`を実装しており、バッファとして適切
  - **検証済み**: WordDefRegistry.register_global(&str, Vec<String>)はKeyWords型(name: String, words: Vec<String>)と完全互換
- **Implications**: Vec<u8>バッファを用意し、2回のpass呼び出し後にString変換

### エラーハンドリング調査
- **Context**: parser2/transpiler2エラーのPastaError統合方法
- **Sources Consulted**: `src/error.rs`, `src/parser2/mod.rs`, `src/transpiler2/error.rs`
- **Findings**:
  - parser2: `PastaError::ParseError`に行・列付きで変換済み（本仕様で実装完了）
  - transpiler2: `TranspileError::into_pasta_error(pass)`でPass段階付き変換可能（本仕様で実装完了）
  - `PastaError::Transpiler2Error { pass, message }`バリアント追加済み
  - **検証済み**: `into_pasta_error()`は`src/transpiler2/error.rs`に実装済み (L74-79)
- **Implications**: エラー変換は設計済み、実装時に呼び出すだけ

### ランタイム層互換性調査
- **Context**: SceneTable/WordTable構築の互換性確認
- **Sources Consulted**: `src/runtime/label_table.rs`, `src/registry/`
- **Findings**:
  - `SceneTable::from_scene_registry(registry, random_selector)`は共有registryに対応
  - `WordTable::from_word_def_registry(registry, random_selector)`も同様
  - ランタイム層のコード変更は不要
- **Implications**: transpiler2出力のregistryをそのままランタイム層に渡せる

## Architecture Pattern Evaluation

| Option               | Description                 | Strengths                    | Risks / Limitations    | Notes             |
| -------------------- | --------------------------- | ---------------------------- | ---------------------- | ----------------- |
| A: engine.rsのみ修正 | 既存ファイル内で3箇所を変更 | 最小変更、ナビゲーション容易 | マージロジック複雑化   | **採用**          |
| B: Engine2新規作成   | 並行保持で段階移行          | 旧実装を破壊しない           | ファイル増、保守負担増 | 不採用            |
| C: 段階的移行        | A+Bの組み合わせ、Phase分割  | 検証容易、リスク分散         | 計画遵守必要           | Aベースで段階実施 |

## Design Decisions

### Decision: ASTマージ戦略
- **Context**: 複数ファイルのPastaFileをどう統合するか
- **Alternatives Considered**:
  1. 全itemsを1つのVecに集約してから新PastaFileを構築
  2. ファイルごとにFileAttr/GlobalWord/GlobalSceneScopeを分類してから統合
- **Selected Approach**: Option 1（全items集約）
- **Rationale**: シンプルで理解しやすく、順序も自然に保持される
- **Trade-offs**: 型別分類の柔軟性は失うが、現状のユースケースでは不要
- **Follow-up**: パフォーマンス問題が発生すれば再検討

### Decision: バッファ管理方式
- **Context**: 2-passトランスパイルの出力をどう管理するか
- **Alternatives Considered**:
  1. `Vec<u8>`を使用し、最後に`String::from_utf8()`
  2. Stringを直接使用（`write_str`等）
- **Selected Approach**: Option 1（Vec<u8> + String::from_utf8）
- **Rationale**: transpiler2のAPI設計（`Write`トレイト）に合致
- **Trade-offs**: UTF-8変換エラーの可能性（実質発生しない）
- **Follow-up**: なし

### Decision: エラー変換方式
- **Context**: parser2/transpiler2エラーをPastaErrorへどう変換するか
- **Alternatives Considered**:
  1. From trait実装で自動変換
  2. map_err()で個別変換
  3. 専用バリアント追加
- **Selected Approach**: Option 3（専用バリアント + 変換メソッド）
- **Rationale**: Pass段階情報を保持でき、エラー情報の欠落がない
- **Trade-offs**: PastaErrorの拡張が必要（実装済み）
- **Follow-up**: なし

## Risks & Mitigations
- **ASTマージロジックの複雑化** — items構造のextendで単純化、ヘルパーメソッド活用
- **2-pass呼び出しの順序誤り** — 明確なシーケンス設計とエラーハンドリング
- **リグレッション発生** — 611テストで即座に検出、段階的コミットで切り戻し容易

## References
- [gap-analysis.md](.kiro/specs/engine-parser2-transpiler2-migration/gap-analysis.md) — 詳細な現状調査とギャップ分析
- [src/parser2/ast.rs](src/parser2/ast.rs) — 新AST定義
- [src/transpiler2/mod.rs](src/transpiler2/mod.rs) — 2-pass API定義
- [src/registry/](src/registry/) — 共有レジストリモジュール
