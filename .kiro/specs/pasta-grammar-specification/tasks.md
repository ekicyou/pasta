# 実装タスク

## Phase 0: テスト層別化・グリーン確認

- [x] 1. テスト層別化とグリーン確認
- [x] 1.1 (P) Parser層テストファイルのリネーム
  - `parser_tests.rs` → `pasta_parser_main_test.rs`
  - `parser_error_tests.rs` → `pasta_parser_error_test.rs`
  - `parser_line_types.rs` → `pasta_parser_line_types_test.rs`
  - `sakura_script_tests.rs` → `pasta_parser_sakura_script_test.rs`
  - `pest_sakura_test.rs` → `pasta_parser_pest_sakura_test.rs`
  - `grammar_diagnostic.rs` → `pasta_parser_grammar_diagnostic_test.rs`
  - `parser_debug.rs` → `pasta_parser_debug_test.rs`
  - `parser_sakura_debug.rs` → `pasta_parser_sakura_debug_test.rs`
  - `pest_debug.rs` → `pasta_parser_pest_debug_test.rs`
  - _Requirements: REQ-QA-2_

- [x] 1.2 (P) Transpiler層テストファイルのリネーム
  - `transpile_comprehensive_test.rs` → `pasta_transpiler_comprehensive_test.rs`
  - `two_pass_transpiler_test.rs` → `pasta_transpiler_two_pass_test.rs`
  - `label_registry_test.rs` → `pasta_transpiler_label_registry_test.rs`
  - `actor_assignment_test.rs` → `pasta_transpiler_actor_assignment_test.rs`
  - `phase3_test.rs` → `pasta_transpiler_phase3_test.rs`
  - _Requirements: REQ-QA-2_

- [x] 1.3 (P) Engine層テストファイルのリネーム
  - `comprehensive_rune_vm_test.rs` → `pasta_engine_rune_vm_comprehensive_test.rs`
  - `rune_block_integration_test.rs` → `pasta_engine_rune_block_test.rs`
  - `rune_closure_test.rs` → `pasta_engine_rune_closure_test.rs`
  - `rune_compile_test.rs` → `pasta_engine_rune_compile_test.rs`
  - `rune_module_memory_test.rs` → `pasta_engine_rune_module_memory_test.rs`
  - `rune_module_merge_test.rs` → `pasta_engine_rune_module_merge_test.rs`
  - `rune_rust_module_test.rs` → `pasta_engine_rune_rust_module_test.rs`
  - `simple_rune_test.rs` → `pasta_engine_rune_simple_test.rs`
  - `label_resolution_runtime_test.rs` → `pasta_engine_label_resolution_test.rs`
  - `function_scope_tests.rs` → `pasta_engine_function_scope_test.rs`
  - `persistence_test.rs` → `pasta_engine_persistence_test.rs`
  - _Requirements: REQ-QA-2_

- [x] 1.4 (P) 統合テストファイルのリネーム
  - `engine_integration_test.rs` → `pasta_integration_engine_test.rs`
  - `engine_independence_test.rs` → `pasta_integration_engine_independence_test.rs`
  - `engine_two_pass_test.rs` → `pasta_integration_engine_two_pass_test.rs`
  - `end_to_end_simple_test.rs` → `pasta_integration_e2e_simple_test.rs`
  - `comprehensive_control_flow_test.rs` → `pasta_integration_control_flow_test.rs`
  - `concurrent_execution_test.rs` → `pasta_integration_concurrent_execution_test.rs`
  - `stdlib_integration_test.rs` → `pasta_integration_stdlib_test.rs`
  - `directory_loader_test.rs` → `pasta_integration_directory_loader_test.rs`
  - `error_handling_tests.rs` → `pasta_integration_error_handling_test.rs`
  - _Requirements: REQ-QA-2_

- [x] 1.5 グリーン確認とベースライン記録
  - `cargo test --all` で全テスト通過を確認
  - `test-baseline.log` に結果を記録
  - Git commit でグリーン状態を保存
  - _Requirements: REQ-QA-2_

---

## Phase 0.5: 既存Parser実装の仕様駆動検証

- [x] 2. 仕様駆動検証テストの実装
- [x] 2.1 Chapter 1-2 検証テスト（文法モデル・マーカー定義）
  - grammar-specification.md §1（行指向文法、ファイル構造、式の制約）の検証
  - grammar-specification.md §2.1（改行、空白、コロン、識別子、インデント）の検証
  - grammar-specification.md §2.2（ラベルマーカー全角・半角）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 20テスト全通過
  - _Requirements: REQ-1.1, REQ-1.2, REQ-1.3, REQ-2.1.1, REQ-2.1.2, REQ-2.1.3, REQ-2.1.4, REQ-2.1.5, REQ-2.2.1, REQ-2.2.2, REQ-2.2.3_

- [x] 2.2 Chapter 2 検証テスト（変数・関数マーカー）
  - grammar-specification.md §2.3（単語登録・参照、変数宣言・代入、スコープ修飾子）の検証
  - grammar-specification.md §2.4（Call マーカー全角・半角、Jump 現行仕様）の検証
  - grammar-specification.md §2.5（Sakura エスケープ現行仕様）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 15テスト全通過
  - _Requirements: REQ-2.3.1, REQ-2.3.2, REQ-2.3.3, REQ-2.4.1, REQ-2.5_

- [x] 2.3 Chapter 2 検証テスト（Rune・リテラル・コメント）
  - grammar-specification.md §2.6（Rune コードブロック開始・終了）の検証
  - grammar-specification.md §2.8（日本語文字列、英語文字列、数値、真偽値）の検証
  - grammar-specification.md §2.9（単語値・引数区切り文字）の検証
  - grammar-specification.md §2.10（コメント全角・半角）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 12テスト全通過
  - _Requirements: REQ-2.6, REQ-2.8, REQ-2.9, REQ-2.10_

- [x] 2.4 Chapter 3 検証テスト（行とブロック構造）
  - grammar-specification.md §3.1（行の定義、独立解析）の検証
  - grammar-specification.md §3.2（インデント不要行、インデント必要行）の検証
  - grammar-specification.md §3.3（グローバルブロック、ラベルブロック、Runeブロック配置）の検証
  - grammar-specification.md §3.4（インデント規則、バイナリ判定）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 9テスト全通過
  - _Requirements: REQ-3.1, REQ-3.2.1, REQ-3.2.2, REQ-3.3.1, REQ-3.3.2, REQ-3.3.3, REQ-3.3.4, REQ-3.4_

- [x] 2.5 Chapter 4-5 検証テスト（Call仕様・リテラル型）
  - grammar-specification.md §4.1（グローバル/ローカルラベル参照、前方一致解決）の検証
  - grammar-specification.md §4.3（引数リスト名前付き形式）の検証
  - grammar-specification.md §5（リテラル型、型変換ルール）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 5テスト全通過（位置引数は現在未サポート、将来拡張候補として記録）
  - _Requirements: REQ-4.1.1, REQ-4.1.2, REQ-4.1.4, REQ-4.3, REQ-5.1, REQ-5.2_

- [x] 2.6 Chapter 6 検証テスト（アクション行）
  - grammar-specification.md §6.1（基本構文 actor：action）の検証
  - grammar-specification.md §6.2（Actor 識別子認識）の検証
  - grammar-specification.md §6.3（インライン要素：単語参照、変数参照、関数呼び出し、Sakura、＠エスケープ）の検証
  - grammar-specification.md §6.4（行継続、複数行台詞）の検証
  - grammar-specification.md §6.5（改行：Sakura `\n`、継続行内空行、非継続領域空行）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 10テスト全通過（@@エスケープ、行継続は現在未サポート、将来拡張候補として記録）
  - _Requirements: REQ-6.1, REQ-6.2, REQ-6.3.1, REQ-6.3.2, REQ-6.4, REQ-6.5.1, REQ-6.5.2, REQ-6.5.3_

- [x] 2.7 Chapter 7 検証テスト（Sakuraスクリプト現行仕様）
  - grammar-specification.md §7.1（字句のみ認識、非解釈）の検証
  - grammar-specification.md §7.2（エスケープ現行仕様：全角・半角両対応）の検証
  - grammar-specification.md §7.3（コマンド字句構造現行仕様）の検証
  - grammar-specification.md §7.4（文字種現行仕様：全角括弧対応）の検証
  - Phase 1 で破壊的変更後に失敗することを期待するテスト
  - 結果: 5テスト全通過
  - _Requirements: REQ-7.1, REQ-7.2, REQ-7.3, REQ-7.4_

- [x] 2.8 Chapter 8-10 検証テスト（属性・変数・単語定義）
  - grammar-specification.md §8（属性構文、配置ルール）の検証
  - grammar-specification.md §9（グローバル変数、ローカル変数、代入制約）の検証
  - grammar-specification.md §10（グローバル/ローカル単語定義、単語参照）の検証
  - 既存テストコード非参照、仕様書のみを根拠
  - 結果: 8テスト全通過
  - _Requirements: REQ-8.1, REQ-8.2, REQ-9.1.1, REQ-9.1.2, REQ-9.2, REQ-10.1, REQ-10.2, REQ-10.3_

- [x] 2.9 検証レポート作成とエスカレーション判定
  - `test-baseline-phase0.5.log` を作成（208テスト全通過）
  - 全テスト結果: 84仕様検証テスト全通過
  - 発見された仕様と実装の乖離（将来拡張候補）:
    - 位置引数付きCall（名前付き引数のみサポート）
    - @@エスケープ（未サポート）
    - 「：」のみでの行継続（未サポート）
  - Type B（仕様書の誤り）なし → Phase 1へ進行可能
  - Git commit で Phase 0.5 完了を記録
  - _Requirements: REQ-QA-1_

---

## Phase 0.5+: Golden Test フィクスチャ作成

- [ ] 3. Golden Test フィクスチャの作成
- [ ] 3.1 Golden Test スクリプト作成
  - `tests/fixtures/golden/complete-feature-test.pasta` を作成
  - 22機能カテゴリ（コメント、属性、ラベル、Rune、変数、関数呼び出し、アクション行、Sakura、行継続、Call等）を包括
  - design.md の Golden Test セクションに準拠
  - _Requirements: REQ-QA-1, REQ-2.10, REQ-8.1, REQ-8.2, REQ-2.2.1, REQ-2.2.2, REQ-10.1, REQ-10.2, REQ-2.6, REQ-9.1.1, REQ-9.1.2, REQ-4.3, REQ-5.2, REQ-2.8, REQ-6.1, REQ-6.2, REQ-6.3.1, REQ-6.4, REQ-7.2, REQ-7.3, REQ-4.1.1, REQ-4.1.2_

---

## Phase 1: Parser層修正

- [ ] 4. Sakuraスクリプト関連の pest 修正
- [ ] 4.1 sakura_escape を半角バックスラッシュのみに修正
  - `sakura_escape = { "\\" | "＼" }` → `sakura_escape = { "\\" }`
  - 全角バックスラッシュが認識されなくなることを確認
  - _Requirements: REQ-7.2, REQ-BC-2_

- [ ] 4.2 sakura_command を簡素化
  - 既存5パターンを `sakura_token ~ sakura_bracket_content?` に統一
  - `sakura_token = @{ (ASCII_ALPHA | ASCII_DIGIT | "_" | "!")+ }`
  - 未知トークンを許容する設計（仕様「字句のみ、非解釈」に準拠）
  - _Requirements: REQ-7.3, REQ-BC-2_

- [ ] 4.3 sakura_bracket_content に `\]` 許容を追加
  - `sakura_bracket_chars = @{ (("\\" ~ "]") | (!"]" ~ ANY))* }`
  - ブラケット内での `\]` エスケープが正しく動作することを確認
  - _Requirements: REQ-7.3, REQ-BC-2_

- [ ] 4.4 sakura_bracket_open/close を半角のみに修正
  - `sakura_bracket_open = { "[" | "［" }` → `sakura_bracket_open = { "[" }`
  - `sakura_bracket_close = { "]" | "］" }` → `sakura_bracket_close = { "]" }`
  - 全角括弧が認識されなくなることを確認
  - _Requirements: REQ-7.4, REQ-BC-2_

- [ ] 4.5 不要な sakura_* ルールを削除
  - `sakura_letter` ルール削除（全角英字含む → 不要）
  - `sakura_digit` ルール削除（全角数字含む → 不要）
  - `sakura_underscore` ルール削除（全角アンダースコア含む → 不要）
  - _Requirements: REQ-7.4, REQ-BC-2_

- [ ] 5. Jump 関連の pest ルール削除
- [ ] 5.1 jump_marker ルールを削除
  - `jump_marker = { "？" | "?" }` ルールを pest 定義から完全削除
  - _Requirements: REQ-2.4.1, REQ-BC-1_

- [ ] 5.2 jump_content ルールを削除
  - `jump_content = { jump_target ~ filter_list? ~ arg_list? ~ NEWLINE }` ルールを削除
  - _Requirements: REQ-BC-1_

- [ ] 5.3 label_body_line から Jump 分岐を削除
  - `jump_marker ~ jump_content` の選択肢を `label_body_line` から削除
  - _Requirements: REQ-BC-1_

- [ ] 5.4 local_label_body_line から Jump 分岐を削除
  - `jump_marker ~ jump_content` の選択肢を `local_label_body_line` から削除
  - _Requirements: REQ-BC-1_

- [ ] 6. text_part バグ修正
- [ ] 6.1 text_part に dollar_marker 除外を追加
  - `text_part = @{ (!(at_marker | sakura_escape | NEWLINE) ~ ANY)+ }`
  - ↓ 修正後
  - `text_part = @{ (!(at_marker | dollar_marker | sakura_escape | NEWLINE) ~ ANY)+ }`
  - `＄var_name` がインライン変数参照として正しく認識されることを確認
  - _Requirements: REQ-6.3.1, REQ-BC-3_

- [ ] 7. AST型の修正
- [ ] 7.1 Statement enum から Jump を削除
  - `Statement::Jump { target, filters, args }` 分岐を削除
  - 関連する `JumpTarget` enum の使用箇所を確認（Call で使用なら維持）
  - _Requirements: REQ-BC-1_

- [ ] 7.2 Parser mod.rs から Jump 処理を削除
  - `Rule::jump_content` 処理を削除
  - `Rule::jump_marker` 処理を削除
  - `Statement::Jump` 構築コードを削除
  - _Requirements: REQ-BC-1_

- [ ] 8. Parser層テストの修正
- [ ] 8.1 Jump 関連テストケースを削除
  - `？` を使用するテストケースを削除
  - Jump 検証ロジックを削除
  - _Requirements: REQ-BC-1_

- [ ] 8.2 全角 Sakura テストケースを削除
  - `＼` を使用するテストケースを削除
  - `［］` を使用するテストケースを削除
  - 半角のみのテストケースに統一
  - _Requirements: REQ-BC-2_

- [ ] 8.3 text_part テストケースを追加
  - `＄` が text_part に吸収されないことを確認するテストを追加
  - インライン変数参照が正しく分離されることを確認
  - _Requirements: REQ-BC-3_

- [ ] 9. Phase 1 完了検証
- [ ] 9.1 Parser層テスト全通過確認
  - `cargo test pasta_parser_ --all` 実行
  - 全 Parser 層テストが通過することを確認
  - _Requirements: REQ-QA-1_

- [ ] 9.2 Golden Test（Parser層）の実装と実行
  - `tests/pasta_parser_golden_test.rs` を作成
  - Golden Test スクリプトが Parser を通過することを確認
  - AST 構造検証（ラベル数、変数数、アクション行数等）
  - Jump 非存在を確認
  - _Requirements: REQ-QA-1_

- [ ] 9.3 Phase 1 完了コミット
  - 修正済み pasta.pest、ast.rs、parser/mod.rs、テストをコミット
  - `phase1-test-result.log` を記録
  - _Requirements: REQ-QA-1_

---

## Phase 2: Transpiler層修正

- [ ] 10. Transpiler から Jump を削除
- [ ] 10.1 Statement::Jump 分岐を削除
  - `transpiler/mod.rs` から `Statement::Jump` の match 分岐を削除
  - _Requirements: REQ-BC-1_

- [ ] 10.2 transpile_jump_target メソッドを削除
  - `transpile_jump_target()` メソッドを削除
  - `transpile_jump_target_to_search_key()` メソッドを削除
  - _Requirements: REQ-BC-1_

- [ ] 10.3 pasta::jump() ランタイム関数を削除
  - Rune 側の `pasta::jump()` 関数定義を削除
  - _Requirements: REQ-BC-1_

- [ ] 11. Transpiler層テストの修正
- [ ] 11.1 Jump 関連テストケースを削除
  - `pasta_transpiler_two_pass_test.rs` から Jump 前提のテストを削除
  - `pasta_transpiler_phase3_test.rs` から Jump ケースを削除
  - _Requirements: REQ-BC-1_

- [ ] 12. Phase 2 完了検証
- [ ] 12.1 Transpiler層テスト全通過確認
  - `cargo test pasta_transpiler_ --all` 実行
  - 全 Transpiler 層テストが通過することを確認
  - _Requirements: REQ-QA-1_

- [ ] 12.2 Golden Test（Transpiler層）の実装と実行
  - `tests/pasta_transpiler_golden_test.rs` を作成
  - Golden Test が Rune コードへ正しく変換されることを確認
  - `pasta::jump()` 呼び出しが生成されないことを確認
  - 生成 Rune コードが compile() を通過することを確認
  - _Requirements: REQ-QA-1_

- [ ] 12.3 Phase 2 完了コミット
  - 修正済み transpiler/mod.rs、テストをコミット
  - `phase2-test-result.log` を記録
  - _Requirements: REQ-QA-1_

---

## Phase 3: Runtime/Tests・ドキュメント修正

- [ ] 13. テストフィクスチャの置換
- [ ] 13.1 Jump マーカーを Call マーカーに置換
  - `tests/fixtures/*.pasta` 内の `？` を `＞` へ置換
  - `find tests/fixtures -name "*.pasta" -exec sed -i 's/？/＞/g' {} \;`
  - 残存確認: `grep -r "？" tests/fixtures/`
  - _Requirements: REQ-BC-1_

- [ ] 14. テストコードの修正
- [ ] 14.1 Engine層テストから全角Sakuraケースを削除
  - `pasta_engine_*.rs` から `＼` を使用するテストケースを削除
  - `pasta_engine_*.rs` から `［］` を使用するテストケースを削除
  - 半角へ統一したテストケースに置換
  - _Requirements: REQ-BC-2_

- [ ] 14.2 統合テストから Jump 依存テストを削除
  - `pasta_integration_*.rs` から Jump 依存テストを削除
  - Call のみで制御フローをテスト
  - _Requirements: REQ-BC-1_

- [ ] 15. GRAMMAR.md の改訂
- [ ] 15.1 Sakura スクリプトセクションの更新
  - §7（Sakura スクリプト）に「字句のみ、非解釈」を明記
  - §7.3 に「半角 `\[...]` + `\]` 許容」を具体記述
  - 全角バックスラッシュ・全角括弧が不可であることを明記
  - _Requirements: REQ-QA-3, REQ-BC-2_

- [ ] 15.2 制御フローセクションの更新
  - Jump 文（`？`）記述をすべて削除
  - Call 文（`＞`）のみで制御フローを説明
  - 破壊的変更であることを明記
  - _Requirements: REQ-QA-3, REQ-BC-1_

- [ ] 15.3 各機能の実用的な例を追加
  - 各機能に 3 つ以上の実用的な例を含める
  - 新仕様に準拠した例のみを掲載
  - _Requirements: REQ-QA-3_

- [ ] 16. Phase 3 完了検証
- [ ] 16.1 全テスト通過確認
  - `cargo test --all` 実行
  - Parser + Transpiler + Engine + Integration 全テストが通過することを確認
  - _Requirements: REQ-QA-1_

- [ ] 16.2 Golden Test（Runtime/Integration層）の実装と実行
  - `tests/pasta_integration_golden_test.rs` を作成
  - Golden Test スクリプトのエンドツーエンド実行が成功することを確認
  - 出力トークン検証（アクション行、単語参照、変数参照、関数呼び出し、＠エスケープ）
  - Sakura スクリプト透過確認
  - Call 実行確認（ローカル・グローバル）
  - Runtime エラー、panic なしを確認
  - _Requirements: REQ-QA-1_

- [ ] 16.3 Phase 3 完了コミット
  - 全修正（フィクスチャ、テスト、GRAMMAR.md）をコミット
  - `phase3-test-result.log` を記録
  - Phase 0 の `test-baseline.log` と比較し、変化を確認
  - _Requirements: REQ-QA-1, REQ-QA-3_

---

## 要件カバレッジ検証

### 要件マッピングサマリー

| 要件カテゴリ | 要件数 | カバータスク |
|------------|--------|-------------|
| REQ-1（基本原則） | 3 | 2.1, 2.4 |
| REQ-2（マーカー定義） | 16 | 2.1, 2.2, 2.3, 3.1, 4.1-4.5, 5.1-5.4 |
| REQ-3（ブロック構造） | 8 | 2.4 |
| REQ-4（Call仕様） | 4 | 2.5, 3.1 |
| REQ-5（リテラル型） | 2 | 2.5, 3.1 |
| REQ-6（アクション行） | 9 | 2.6, 3.1, 6.1 |
| REQ-7（Sakura仕様） | 4 | 2.7, 4.1-4.5 |
| REQ-8（属性） | 2 | 2.8, 3.1 |
| REQ-9（変数・スコープ） | 3 | 2.8, 3.1 |
| REQ-10（単語定義） | 3 | 2.8, 3.1 |
| REQ-BC-1（Jump廃止） | 1 | 5.1-5.4, 7.1-7.2, 8.1, 10.1-10.3, 11.1, 13.1, 14.2, 15.2 |
| REQ-BC-2（Sakura半角限定） | 1 | 4.1-4.5, 8.2, 14.1, 15.1 |
| REQ-BC-3（text_partバグ修正） | 1 | 6.1, 8.3 |
| REQ-QA-1（Golden Test） | 1 | 2.9, 3.1, 9.1-9.3, 12.1-12.3, 16.1-16.3 |
| REQ-QA-2（テスト層別化） | 1 | 1.1-1.5 |
| REQ-QA-3（GRAMMAR.md改訂） | 1 | 15.1-15.3, 16.3 |

### 将来予約要件（スコープ外）

以下の要件は grammar-specification.md で「将来予約」と定義されており、本タスクのスコープ外：

- REQ-2.7（演算子）
- REQ-4.1.3（動的ターゲット）
- REQ-4.2（フィルター）
- REQ-8.3（ファイルレベル属性）
- REQ-11（未確定事項19項目）

### 全64要件のカバレッジ

- **既存実装で対応済み（検証タスクでカバー）**: 45要件
- **設計タスクで対応**: 14要件
- **将来予約（スコープ外）**: 5要件
- **合計カバレッジ**: 100%（64/64）
