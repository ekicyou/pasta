# Research & Design Decisions

## Summary
- **Feature**: word-ref-ast-support
- **Discovery Scope**: Extension（既存パーサー層への拡張）
- **Key Findings**:
  - VarSet.value の型変更により、4箇所以上のコード修正が必要
  - SetValue 列挙型はExprと同じファイル（ast.rs）に配置が適切
  - grammar.pest の `set = ( expr | word_ref )` に対応するために、parse_var_set 内で Rule::word_ref を明示的に処理する必要がある

## Research Log

### VarSet.value の使用箇所調査
- **Context**: VarSet.value の型を Expr から SetValue に変更した場合の影響範囲を特定
- **Sources Consulted**: crates/pasta_core/src/parser/ast.rs, crates/pasta_rune/src/transpiler/code_generator.rs, tests/parser2_integration_test.rs
- **Findings**:
  - `code_generator.rs` line 192: `self.generate_expr(&var_set.value)` - パターンマッチ必要
  - `code_generator.rs` line 518: VarSetリテラル作成（テスト用）- value フィールド型変更
  - `tests/parser2_integration_test.rs` lines 274, 290: `matches!(vs.value, Expr::Integer(123))` - SetValue::Expr でラップ
  - `crates/pasta_rune/tests/parser2_integration_test.rs` lines 305, 321: 同上
  - `parser/mod.rs` line 619: VarSetリテラル作成 - value フィールド型変更
- **Implications**: コンパイルエラーにより漏れなく検出可能、修正は機械的

### parse_var_set 内での word_ref 処理方法
- **Context**: grammar.pest の set ルールでは expr と word_ref が分離されているが、現在の parse_var_set は expr のみ処理
- **Sources Consulted**: crates/pasta_core/src/parser/mod.rs lines 569-625
- **Findings**:
  - 現在の実装: `try_parse_expr(inner)` で全ての値をパースし、terms に追加
  - word_ref は try_parse_expr で処理されない（別ルール）
  - Rule::word_ref を明示的に検出し、SetValue::WordRef を構築する必要あり
  - 二項演算処理（operators + terms）は expr にのみ適用され、word_ref には不要
- **Implications**: parse_var_set の内部ロジックを更新し、word_ref と expr を分離処理

### トランスパイラー層での SetValue 処理
- **Context**: SetValue::WordRef が返された場合のコード生成戦略
- **Sources Consulted**: crates/pasta_rune/src/transpiler/code_generator.rs lines 175-230
- **Findings**:
  - generate_var_set は `self.generate_expr(&var_set.value)` を呼び出し
  - SetValue への変更後: パターンマッチが必要
  - SetValue::Expr(expr) → 既存の generate_expr を呼び出し
  - SetValue::WordRef → 本仕様では無視（別仕様で扱う）
- **Implications**: コード生成はシンプルなパターンマッチ追加のみ

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: SetValue列挙型導入 | VarSet.value を SetValue に変更 | 型安全、設計意図反映 | 複数ファイル修正 | **選択** |
| B: Expr::WordRef追加 | Expr に WordRef バリアントを追加 | 修正箇所最小 | 設計意図が曖昧 | 却下 |
| C: Option<String>追加 | word_ref を別フィールドに | Option型の条件分岐 | 不正状態を許容 | 却下 |

## Design Decisions

### Decision: SetValue 列挙型の導入
- **Context**: grammar.pest の `( expr | word_ref )` 分離を AST 上で反映する必要
- **Alternatives Considered**:
  1. Option A - SetValue列挙型（Expr(Expr) | WordRef { name }）
  2. Option B - Expr::WordRef バリアント追加
  3. Option C - VarSet に Option<String> フィールド追加
- **Selected Approach**: Option A - SetValue 列挙型
- **Rationale**: 
  - grammar.pest の設計意図を型レベルで正確に表現
  - expr と word_ref の構造的な違いを明確化
  - コンパイルエラーで漏れなく修正箇所を検出
- **Trade-offs**: 
  - 複数ファイルへの修正が必要
  - 既存テストの更新が必要
- **Follow-up**: word_ref のセマンティクス実装は別仕様

### Decision: parse_var_set 内での word_ref 処理
- **Context**: try_parse_expr は expr 専用であり、word_ref を処理できない
- **Alternatives Considered**:
  1. try_parse_expr を拡張して SetValue を返す
  2. parse_var_set 内で Rule::word_ref を明示的に検出
- **Selected Approach**: parse_var_set 内で Rule::word_ref を明示的に検出
- **Rationale**: 
  - try_parse_expr は expr 専用の責務を維持
  - word_ref は set ルール固有の処理として局所化
  - 二項演算処理は expr にのみ適用（word_ref は単独値）
- **Trade-offs**: parse_var_set の複雑度がわずかに増加
- **Follow-up**: なし

### Decision: トランスパイラー層での WordRef 無視
- **Context**: word_ref のセマンティクス実装は本仕様のスコープ外
- **Selected Approach**: SetValue::WordRef に対しては何もしない（無視）
- **Rationale**: 
  - 本仕様は AST 層の変更のみ
  - コンパイルを通すための最小限の対応
  - 将来の仕様で適切に実装
- **Trade-offs**: word_ref を含む VarSet はコード生成されない
- **Follow-up**: word_ref のセマンティクス実装仕様を別途作成

## Risks & Mitigations
- **Risk**: VarSet.value 使用箇所の見落とし → コンパイルエラーで検出されるため低リスク
- **Risk**: word_ref を含むスクリプトが意図せず無視される → テストで検証、ドキュメント化
- **Risk**: 二項演算と word_ref の組み合わせ → grammar.pest 上で禁止されている（`set = expr | word_ref`）

## References
- [pasta SPECIFICATION.md](../../../SPECIFICATION.md) - Pasta DSL言語仕様
- [grammar.pest](../../../crates/pasta_core/src/parser/grammar.pest) - PEG文法定義
- [gap-analysis.md](./gap-analysis.md) - 詳細なギャップ分析
