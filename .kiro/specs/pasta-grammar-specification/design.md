# 設計ドキュメント

## プロジェクト説明（入力）

Pasta DSL の文法仕様を、現在の実装（pest定義）と乖離から修正し直す必要があります。特に：
- さくらスクリプトは**字句のみ認識**（非解釈）、**半角限定**
- ブラケット内での `\]` **許容**
- **Jump 文（`？`）廃止**、Call（`＞`）へ統一

この破壊的変更を層単位の段階化で実装し、リグレッション発生箇所を特定しやすくします。

---

## 設計概要

### 全体戦略: 層単位の段階的実装 + テスト層別化

1. **Phase 0（Pre-Implementation Preparation）**: テスト層別化・グリーン確認・commit
   - テストファイルを層別命名規則に統一
   - 全テストが通ることを確認し、git に保存
   - リグレッション検出の基盤を確立

2. **Phase 1（Parser 層）**: pest + AST を新仕様へ統一
   - pest.pest の全ルール修正（さくら・Jump）
   - AST 型修正（Jump 削除）
   - Parser テストが 100% パス

3. **Phase 2（Transpiler 層）**: Parser 出力に対応
   - Statement::Jump 削除
   - pasta::jump() 関数削除
   - Transpiler テストが 100% パス

4. **Phase 3（Runtime/Tests 層）**: 統合テスト・ドキュメント修正
   - テスト・フィクスチャ置換（`？` → `＞`、全角 → 半角）
   - GRAMMAR.md 改訂
   - 全テストが 100% パス

---

## 統合検証用最小スクリプト（Golden Test）

### 目的
grammar-specification.md の全機能を包括する最小の pasta スクリプトを定義し、Phase 1-3 の各段階で Parser・Transpiler・Runtime の3層すべてが正しく動作することを検証する。

### Golden Test スクリプト

**ファイル**: `tests/fixtures/golden/complete-feature-test.pasta`

```pasta
# コメント（2.10）
＃全角コメント

＆file_level_attr：test_file

＠global_word：apple　banana　orange

＊統合テスト
  ＆author：Alice
  ＆genre：test
  ＠local_word：choice1　choice2
  ```rune
  fn calculate(ctx, x, y) {
    x + y
  }
  fn get_greeting(ctx) {
    "こんにちは"
  }
  fn get_flag(ctx) {
    true
  }
  ```
  ＄＊global_var ： 100
  ＄local_var ： 「ローカル値」
  ＄result ： ＠calculate（x：10　y：20）
  ＄flag ： ＠get_flag()
  ＄message ： "Hello World"
  
  Alice：単語参照＠global_word　変数＄local_var
  Alice：関数呼び出し＠get_greeting()
  Alice：Sakura\n改行\w8待機\s[0]表情
  Alice：長い台詞は
    複数行に
    
    分けて記述
  Alice：＠＠はリテラルのアットマーク
  
  ・選択肢1
    Bob：選択肢1が選ばれました＠local_word
    ＞選択肢2
  
  ・選択肢2
    Bob：選択肢2が選ばれました
    ＞＊他のラベル
```

### Golden Test の網羅項目

| 機能カテゴリ | 仕様セクション | 検証項目 | 要件ID |
|------------|--------------|---------|--------|
| コメント | 2.10 | 半角 `#`、全角 `＃` | REQ-2.10 |
| ファイルレベル属性 | 8.3 | `＆file_level_attr` | REQ-8.3 |
| グローバル単語定義 | 10.1 | `＠global_word：value1　value2` | REQ-10.1 |
| グローバルラベル | 2.2, 3 | `＊統合テスト` | REQ-2.2.1, REQ-3.2.1 |
| 属性定義 | 8.1 | `＆author`, `＆genre` | REQ-8.1, REQ-8.2 |
| ローカル単語定義 | 10.2 | `＠local_word：choice1　choice2` | REQ-10.2 |
| Rune ブロック | 2.6 | 関数定義（`fn calculate`, `fn get_greeting`, `fn get_flag`） | REQ-2.6 |
| グローバル変数 | 9.1 | `＄＊global_var` | REQ-9.1.1 |
| ローカル変数 | 9.1 | `＄local_var` | REQ-9.1.2 |
| 関数呼び出し（引数付き） | 4.3 | `＠calculate（x：10　y：20）` | REQ-4.3 |
| bool 型リテラル | 5.2 | `＠get_flag()` が `true` を返す | REQ-5.2 |
| 英語文字列リテラル | 2.8, 5.2 | `＄message ： "Hello World"` | REQ-2.8, REQ-5.2 |
| アクション行 | 6.1-6.3 | `Alice：...` | REQ-6.1, REQ-6.2, REQ-6.3 |
| 単語参照 | 10.3 | `＠global_word` | REQ-10.3 |
| 変数参照 | 6.3 | `＄local_var` | REQ-6.3.1 |
| 関数呼び出し（引数なし） | 6.3 | `＠get_greeting()` | REQ-6.3.1 |
| ＠エスケープ | 6.3 | `＠＠` → `＠` | REQ-6.3.1 |
| Sakura スクリプト | 7 | `\n`, `\w8`, `\s[0]` | REQ-7 |
| 行継続 | 6.4 | 複数行台詞 | REQ-6.4 |
| 継続行内空行 | 6.5.3 | 空行による改行 | REQ-6.5.2 |
| ローカルラベル | 2.2, 3 | `・選択肢1`, `・選択肢2` | REQ-2.2.2, REQ-3.2.2 |
| Call（ローカル） | 4.1 | `＞選択肢2` | REQ-4.1.2 |
| Call（グローバル） | 4.1 | `＞＊他のラベル` | REQ-4.1.1 |

### 検証基準

#### Phase 1（Parser 層）

**ツール**: `cargo test pasta_parser_golden_test`

**合格条件**:
1. **構文解析成功**: Golden Test スクリプトが Parser を通過（Parse エラーなし）
2. **AST 構造検証**: 以下の AST ノードが正しく生成される
   - グローバルラベル（1個）: `統合テスト`
   - ローカルラベル（2個）: `選択肢1`, `選択肢2`
   - 属性（3個）: file_level, author, genre
   - Rune ブロック（1個）: 3関数定義（`calculate`, `get_greeting`, `get_flag`）
   - 変数代入（5個）: global_var, local_var, result, flag, message
   - アクション行（7個）: Alice（5個）, Bob（2個）
   - Call（2個）: `選択肢2`（ローカル）, `＊他のラベル`（グローバル）
   - 単語定義（2個）: global_word, local_word
3. **Sakura トークン検出**: `\n`, `\w8`, `\s[0]` が個別トークンとして検出される
4. **行継続検出**: 4行継続（「長い台詞は」→「複数行に」→空行→「分けて記述」）
5. **＠エスケープ検出**: `＠＠` がリテラル `＠` として認識される
6. **英語文字列リテラル**: `"Hello World"` が正しくパースされる
7. **Jump 非存在**: AST に `Statement::Jump` が存在しない

**実装**: `tests/pasta_parser_golden_test.rs`（新規作成）

#### Phase 2（Transpiler 層）

**ツール**: `cargo test pasta_transpiler_golden_test`

**合格条件**:
1. **Rune コード生成成功**: Golden Test が Rune コードへ変換される
2. **関数シグネチャ検証**: 
   - `pasta::call(ctx, "統合テスト", [], [])` 生成
   - `pasta::call(ctx, "他のラベル", [], [])` 生成（グローバル Call）
   - `pasta::call(ctx, "選択肢2", [], [])` 生成（ローカル Call）
   - `pasta::word_lookup(ctx, "global_word")` 生成
3. **変数スコープ**: `ctx.global["global_var"]`, `ctx.local["local_var"]` 生成
4. **Sakura 透過**: `\n`, `\w8`, `\s[0]` がそのまま文字列として出力
5. **＠エスケープ透過**: `＠＠` がリテラル文字列 `＠` に変換
6. **bool/String リテラル**: `true`, `"Hello World"` が正しく生成
7. **Jump コード非存在**: `pasta::jump()` 呼び出しが生成されない
8. **Rune コンパイル成功**: 生成コードが Rune VM で compile() を通過

**実装**: `tests/pasta_transpiler_golden_test.rs`（新規作成）

#### Phase 3（Runtime/Integration 層）

**ツール**: `cargo test pasta_integration_golden_test`

**合格条件**:
1. **エンドツーエンド実行成功**: Golden Test スクリプトが完全実行される
2. **出力トークン検証**: 
   - アクション行（7個）のトークンが正しい順序で yield される
   - 単語参照がランダム選択される（`apple`/`banana`/`orange` のいずれか）
   - 変数参照が正しく展開される（`「ローカル値」`、`"Hello World"`）
   - 関数呼び出し結果が展開される（`「こんにちは」`, `30`, `true`）
   - ＠エスケープが正しく展開される（`＠＠` → `＠`）
3. **Sakura スクリプト透過**: 出力に `\n`, `\w8`, `\s[0]` が含まれる
4. **Call 実行**: 
   - ローカル Call（`＞選択肢2`）が正しく呼び出される
   - グローバル Call（`＞＊他のラベル`）が呼び出される（実装済みラベルの場合）
5. **エラーなし**: Runtime エラー、panic なし

**実装**: `tests/pasta_integration_golden_test.rs`（新規作成）

### リグレッション検出方法

#### Phase 0 → Phase 1
- Phase 0 完了時: 既存テスト全通過（test-baseline.log 記録）
- Phase 1 修正後: `cargo test pasta_parser_` 実行
- **合格**: Parser 層テストすべて通過 + Golden Test 通過
- **リグレッション**: Parser 層テスト失敗 → pest 修正にバグ

#### Phase 1 → Phase 2
- Phase 1 完了時: Parser 層テスト全通過
- Phase 2 修正後: `cargo test pasta_transpiler_` 実行
- **合格**: Transpiler 層テストすべて通過 + Golden Test 通過
- **リグレッション**: Transpiler 層テスト失敗 → AST 変更の波及確認

#### Phase 2 → Phase 3
- Phase 2 完了時: Parser + Transpiler 層テスト全通過
- Phase 3 修正後: `cargo test --all` 実行
- **合格**: 全テスト通過 + Golden Test 通過
- **リグレッション**: Runtime/Integration 層テスト失敗 → フィクスチャ置換ミス

### Golden Test の保守

- **Phase 1-3 全体を通じて Golden Test は修正しない**
- Golden Test が通過しない場合は実装側を修正
- 仕様変更時のみ Golden Test を更新（その際は要件定義から見直し）

---

## Phase 0: Pre-Implementation Preparation

### 目的
リグレッション発生箇所を層別に特定しやすくするため、テストファイルを命名規則で統一し、変更前の「グリーン状態」を git で保存。

**重要**: Phase 0 完了後、**必ず git commit** してから Phase 0.5 へ進むこと。

### 実施内容

#### 0.1 テスト層別化・リネーム

**命名規則**:
```
pasta_parser_<name>_test.rs          — Parser 層テスト
pasta_transpiler_<name>_test.rs      — Transpiler 層テスト
pasta_engine_<name>_test.rs          — Engine/Runtime 層テスト
pasta_integration_<name>_test.rs     — 統合テスト
pasta_debug_<name>_test.rs           — デバッグ用（オプション）
```

**リネーム計画（約38ファイル）**:

##### Parser 層
- `parser_tests.rs` → `pasta_parser_main_test.rs`
- `parser_error_tests.rs` → `pasta_parser_error_test.rs`
- `parser_line_types.rs` → `pasta_parser_line_types_test.rs`
- `sakura_script_tests.rs` → `pasta_parser_sakura_script_test.rs`
- `pest_sakura_test.rs` → `pasta_parser_pest_sakura_test.rs`
- `grammar_diagnostic.rs` → `pasta_parser_grammar_diagnostic_test.rs`
- `parser_debug.rs` → `pasta_parser_debug_test.rs`
- `parser_sakura_debug.rs` → `pasta_parser_sakura_debug_test.rs`
- `pest_debug.rs` → `pasta_parser_pest_debug_test.rs`

##### Transpiler 層
- `transpile_comprehensive_test.rs` → `pasta_transpiler_comprehensive_test.rs`
- `two_pass_transpiler_test.rs` → `pasta_transpiler_two_pass_test.rs`
- `label_registry_test.rs` → `pasta_transpiler_label_registry_test.rs`
- `actor_assignment_test.rs` → `pasta_transpiler_actor_assignment_test.rs`
- `phase3_test.rs` → `pasta_transpiler_phase3_test.rs`

##### Engine/Runtime 層
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

##### 統合テスト
- `engine_integration_test.rs` → `pasta_integration_engine_test.rs`
- `engine_independence_test.rs` → `pasta_integration_engine_independence_test.rs`
- `engine_two_pass_test.rs` → `pasta_integration_engine_two_pass_test.rs`
- `end_to_end_simple_test.rs` → `pasta_integration_e2e_simple_test.rs`
- `comprehensive_control_flow_test.rs` → `pasta_integration_control_flow_test.rs`
- `concurrent_execution_test.rs` → `pasta_integration_concurrent_execution_test.rs`
- `stdlib_integration_test.rs` → `pasta_integration_stdlib_test.rs`
- `directory_loader_test.rs` → `pasta_integration_directory_loader_test.rs`
- `error_handling_tests.rs` → `pasta_integration_error_handling_test.rs`

##### Debug・Legacy（統合/削除候補）
- `sakura_debug_test.rs` → `pasta_debug_sakura_test.rs`（または削除）
- `label_id_consistency_test.rs` → `pasta_integration_label_id_consistency_test.rs`（または削除）

#### 0.2 事前グリーン確認

```bash
cargo test --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/test-baseline.log
# 全テスト "test result: ok" を確認
```

#### 0.3 Commit（グリーン状態の記録）

```bash
git add tests/*.rs
git commit -m "Refactor: Organize test files by layer hierarchy for regression tracking (Phase 0)"
```

### 成果物
- リネーム済みテストファイル
- test-baseline.log（全テスト通過の記録）
- Golden Test スクリプト（`tests/fixtures/golden/complete-feature-test.pasta`）
- Golden Test 実装（3層分、Phase 1-3 で順次作成）
- Git commit ID（グリーン状態の参照点）

### 検証方法
```bash
# 全テスト実行
cargo test --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/test-baseline.log

# グリーン確認
grep "test result:" test-baseline.log
# 期待: "test result: ok. X passed; 0 failed"

# Golden Test フィクスチャ作成
mkdir -p tests/fixtures/golden
# （上記 Golden Test スクリプトを作成）

# Commit
git add tests/*.rs tests/fixtures/golden/
git commit -m "Refactor: Organize test files by layer hierarchy + add Golden Test fixture (Phase 0)"
```

---

## Phase 0.5: 既存 Parser 実装の検証

### 目的
既存のパーサーテストは既存実装に依存して書かれている可能性があり、grammar-specification.md からの逸脱を見逃している可能性がある。

**既存テストを一切参照せず**、grammar-specification.md のみを根拠とした新規検証テストを作成し、既存実装が文法仕様書通りに動作するかを確認する。

### 前提条件
- Phase 0 完了・コミット済み
- 既存テスト全通過（グリーン状態）

### 検証方針

#### 0.5.1 テスト設計の原則

1. **既存コード非参照**
   - `tests/parser_*.rs` の既存テストコードは一切参照しない
   - `src/parser/pasta.pest` の既存 pest 定義も参照しない（仕様書のみ）
   - grammar-specification.md の各章・各節のみを根拠とする

2. **網羅的検証**
   - REQ-1〜REQ-11 の全章を章ごとにテストモジュール化
   - 特に「既存実装で対応済み」とした45項目を重点検証
   - 各文法要素について「正常系」「境界値」「異常系」を網羅

3. **現行仕様の確認**
   - 破壊的変更（REQ-BC-1〜3）については**現行仕様**（Jump あり、Sakura 全角あり）でテスト
   - Phase 1 以降で破壊的変更を実装した際、これらのテストが失敗することを期待

#### 0.5.2 テスト構成

**新規テストファイル**: `tests/pasta_parser_spec_validation_test.rs`

**モジュール構成**:
```rust
// grammar-specification.md の章構成に対応
mod chapter1_basic_principles;    // REQ-1（文法モデルの基本原則）
mod chapter2_keywords_markers;    // REQ-2（キーワード・マーカー定義）
mod chapter3_line_block;          // REQ-3（行とブロック構造）
mod chapter4_call;                // REQ-4（Call 詳細仕様）
mod chapter5_literals;            // REQ-5（リテラル型）
mod chapter6_action_lines;        // REQ-6（アクション行）
mod chapter7_sakura;              // REQ-7（Sakura スクリプト仕様）
mod chapter8_attributes;          // REQ-8（属性）
mod chapter9_variables_scope;     // REQ-9（変数・スコープ）
mod chapter10_word_definition;    // REQ-10（単語定義）
```

#### 0.5.3 検証項目詳細

##### Chapter 1: 文法モデルの基本原則（REQ-1）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `行指向文法_正常系` | 各行が独立して解析される | §1.1 |
| `行指向文法_複数行台詞` | アクション行の継続が正しく処理される | §1.1 |
| `ファイル構造_グローバルブロック` | グローバル宣言（単語・変数・Rune）の順序制約 | §1.2 |
| `ファイル構造_ラベルブロック` | ラベル内の構造（属性→宣言→本体）の順序制約 | §1.2 |
| `式の制約_変数宣言` | 変数宣言に式を記述できないことの確認 | §1.3 |
| `式の制約_単語定義` | 単語定義に式を記述できないことの確認 | §1.3 |

##### Chapter 2: キーワード・マーカー定義（REQ-2）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| **2.1 基本要素** |||
| `改行_LF` | LF（`\n`）が改行として認識される | §2.1 |
| `改行_CRLF` | CRLF（`\r\n`）が改行として認識される | §2.1 |
| `空白_全角半角` | 全角・半角空白が正しく認識される | §2.1 |
| `コロン_全角半角` | 全角・半角コロンが正しく認識される | §2.1 |
| `識別子_XID_START` | XID_START で始まる識別子が認識される | §2.1 |
| `識別子_XID_CONTINUE` | XID_CONTINUE を含む識別子が認識される | §2.1 |
| `識別子_異常系_数字始まり` | 数字始まりが識別子として認識されない | §2.1 |
| `インデント_バイナリ判定` | タブとスペース混在がエラーとなる | §2.1 |
| **2.2 ラベルマーカー** |||
| `グローバルラベル_全角` | `＊` で始まるグローバルラベル | §2.2 |
| `グローバルラベル_半角` | `*` で始まるグローバルラベル | §2.2 |
| `ローカルラベル_全角` | `・` で始まるローカルラベル | §2.2 |
| `ローカルラベル_半角` | `·` で始まるローカルラベル（Middle Dot U+00B7） | §2.2 |
| `属性マーカー_全角` | `＆` で始まる属性 | §2.2 |
| `属性マーカー_半角` | `&` で始まる属性 | §2.2 |
| **2.3 変数・関数マーカー** |||
| `単語登録_全角` | `＠word：values` | §2.3 |
| `単語登録_半角` | `@word：values` | §2.3 |
| `単語参照_全角` | `＠word` | §2.3 |
| `単語参照_半角` | `@word` | §2.3 |
| `変数宣言_全角` | `＄var：value` | §2.3 |
| `変数宣言_半角` | `$var：value` | §2.3 |
| `変数参照_全角` | `＄var` | §2.3 |
| `変数参照_半角` | `$var` | §2.3 |
| `グローバル修飾子_全角` | `＄＊var` | §2.3 |
| `グローバル修飾子_半角` | `$*var` | §2.3 |
| **2.4 制御フローマーカー** |||
| `Call_全角` | `＞＊label` | §2.4 |
| `Call_半角` | `>*label` | §2.4 |
| `Jump_全角_現行` | `？＊label`（現行仕様、Phase 1 以降は廃止） | §2.4 |
| `Jump_半角_現行` | `?*label`（現行仕様、Phase 1 以降は廃止） | §2.4 |
| **2.5 音声マーカー** |||
| `音声優先_全角` | `♪word` | §2.5 |
| `音声優先_半角` | `#word`（#は数値音声ID用） | §2.5 |
| **2.6 Rune コードブロック** |||
| `Rune開始_3backticks` | ` ``` ` でブロック開始 | §2.6 |
| `Rune終了_3backticks` | ` ``` ` でブロック終了 | §2.6 |
| `Rune言語指定` | ` ```rune ` の形式 | §2.6 |
| **2.7 演算子（将来予約）** |||
| `演算子_将来予約` | 現在は使用不可の確認 | §2.7 |
| **2.8 リテラル・文字列** |||
| `日本語文字列_鉤括弧` | `「文字列」` | §2.8 |
| `英語文字列_ダブルクォート` | `"string"` | §2.8 |
| `数値_整数` | `123` | §2.8 |
| `数値_浮動小数点` | `123.456` | §2.8 |
| `真偽値_true` | `true` | §2.8 |
| `真偽値_false` | `false` | §2.8 |
| **2.9 区切り文字** |||
| `単語値区切り_全角空白` | `apple　banana` | §2.9 |
| `単語値区切り_半角空白` | `apple banana` | §2.9 |
| `引数区切り_全角空白` | `x：10　y：20` | §2.9 |
| `引数区切り_半角空白` | `x：10 y：20` | §2.9 |
| **2.10 コメント** |||
| `コメント_全角` | `＃コメント` | §2.10 |
| `コメント_半角` | `#comment` | §2.10 |

##### Chapter 3: 行とブロック構造（REQ-3）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `行定義_独立解析` | 各行が独立して解析される | §3.1 |
| `インデント不要_グローバルラベル` | グローバルラベルがインデント不要 | §3.2 |
| `インデント不要_属性` | ファイルレベル属性がインデント不要 | §3.2 |
| `インデント必要_ローカルラベル` | ローカルラベルがインデント必要 | §3.2 |
| `インデント必要_アクション行` | アクション行がインデント必要 | §3.2 |
| `グローバルブロック_構造` | 単語定義→変数宣言→Runeブロックの順序 | §3.3 |
| `グローバルラベルブロック_構造` | 属性→宣言→本体の順序 | §3.3 |
| `ローカルブロック_構造` | ローカルラベル内の構造 | §3.3 |
| `Runeブロック_配置` | Runeブロックの配置ルール | §3.3 |
| `インデント規則_バイナリ判定` | タブ・スペース混在エラー | §3.4 |

##### Chapter 4: Call 詳細仕様（REQ-4）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `グローバルラベル参照` | `＞＊label` | §4.1 |
| `ローカルラベル参照` | `＞label` | §4.1 |
| `動的ターゲット_将来予約` | `＞＄var` は構文エラー（将来予約） | §4.1 |
| `前方一致解決` | 部分一致でのラベル解決 | §4.1 |
| `フィルター_構文のみ` | `＞＊label［＆attr：value］` の構文解析 | §4.2 |
| `引数リスト_名前付き` | `＞＊label（x：10　y：20）` | §4.3 |

##### Chapter 5: リテラル型（REQ-5）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `bool型` | `true`, `false` | §5 |
| `String型_日本語` | `「文字列」` | §5 |
| `String型_英語` | `"string"` | §5 |
| `f64型` | `123.456` | §5 |
| `i64型` | `123` | §5 |
| `型変換_bool_to_String` | `true` → `"true"` | §5.2 |
| `型変換_数値_to_String` | `123` → `"123"` | §5.2 |

##### Chapter 6: アクション行（REQ-6）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `基本構文_actor_action` | `Alice：こんにちは` | §6.1 |
| `Actor_識別子` | アクター名が識別子として認識される | §6.2 |
| `インライン要素_単語参照` | `＠word` | §6.3 |
| `インライン要素_変数参照` | `＄var` | §6.3 |
| `インライン要素_関数呼び出し` | `＠func()` | §6.3 |
| `インライン要素_Sakura` | `\n`, `\w8` 等 | §6.3 |
| `インライン要素_アットエスケープ` | `＠＠` → `＠` | §6.3 |
| `区切り文字_空白` | 単語値の空白区切り | §6.3 |
| `最長一致` | `＠long_word` vs `＠long` | §6.3 |
| `行継続_複数行台詞` | インデント継続 | §6.4 |
| `正規の改行_Sakura` | `\n` | §6.5 |
| `糖衣構文_継続行内空行` | 空行が改行として解釈される | §6.5 |
| `非継続領域_空行無視` | グローバルブロックの空行が無視される | §6.5 |

##### Chapter 7: Sakura スクリプト仕様（REQ-7）

**注**: Phase 0.5 では**現行仕様**（全角対応あり）でテスト

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `概要_字句のみ認識` | Sakura が解釈されず字句として透過される | §7.1 |
| `エスケープ_全角_現行` | `＼n`（Phase 1 以降は廃止） | §7.2 |
| `エスケープ_半角_現行` | `\n` | §7.2 |
| `コマンド_字句構造_現行` | 既存の複雑なパターンマッチング | §7.3 |
| `ブラケット内エスケープ_現行` | `\]` の扱い（Phase 1 で修正） | §7.3 |
| `文字種_全角括弧_現行` | `［］`（Phase 1 以降は廃止） | §7.4 |
| `文字種_半角括弧_現行` | `[]` | §7.4 |

##### Chapter 8: 属性（REQ-8）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `構文_key_value` | `＆key：value` | §8.1 |
| `配置ルール_ラベル直後` | ラベル直後の配置 | §8.2 |
| `ファイルレベル属性_構文のみ` | `＆file_attr：value` の構文解析 | §8.3 |

##### Chapter 9: 変数・スコープ（REQ-9）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `グローバル変数_宣言` | `＄＊var：value` | §9.1 |
| `グローバル変数_参照` | `＄＊var` | §9.1 |
| `ローカル変数_宣言` | `＄var：value` | §9.1 |
| `ローカル変数_参照` | `＄var` | §9.1 |
| `代入制約_式不可` | `＄var：1 + 2` がエラー | §9.2 |
| `代入制約_関数呼び出しOK` | `＄var：＠func()` | §9.2 |

##### Chapter 10: 単語定義（REQ-10）

| テストケース | 検証内容 | grammar-specification.md 参照 |
|------------|---------|------------------------------|
| `グローバル単語定義` | `＠word：apple　banana` | §10.1 |
| `ローカル単語定義` | グローバルラベルスコープ内定義 | §10.2 |
| `単語参照_静的` | `＠word` | §10.3 |
| `単語参照_動的_将来予約` | `＠＄var` は構文エラー（将来予約） | §10.3 |

#### 0.5.4 実装手順

```bash
# 新規テストファイル作成
touch tests/pasta_parser_spec_validation_test.rs

# 各章のテストモジュールを順次実装
# （上記の検証項目詳細に基づく）

# テスト実行
cargo test pasta_parser_spec_validation

# 結果の確認
# - 全通過 → 既存実装は仕様書通り、Phase 1 へ進む
# - 失敗あり → 既存実装に逸脱あり、Phase 1 で修正項目に追加
```

#### 0.5.5 期待される結果

**成功ケース**:
- 全テストが通過
- 既存実装が grammar-specification.md に準拠していることが確認される
- Phase 1 の破壊的変更を自信を持って実装できる

**失敗ケース**:
- 一部のテストが失敗
- 失敗したテストケースから、既存実装と仕様書の乖離箇所が特定される
- 以下の**失敗時エスカレーションプロセス**に従う

#### 0.5.6 失敗時エスカレーションプロセス

**目的**: Phase 0.5 で失敗が発生した場合、原因を正しく分類し、適切な対処を行う。

##### 失敗の分類

各失敗テストケースを以下の3つのタイプに分類：

| Type | 名称 | 説明 | 対処 |
|------|------|------|------|
| **Type A** | 既存実装のバグ | 仕様書通りに動作すべきだが動作していない | Phase 1 で修正 |
| **Type B** | 仕様書の誤り | 仕様書が実装と乖離しており、実装が正しい | grammar-specification.md を修正し、requirements.md を再評価（要件定義フェーズに差し戻し） |
| **Type C** | テストコードの誤り | テストケースの期待値が間違っている | テストを修正して再実行 |

##### 判断プロセス

```
Phase 0.5 テスト実行
  ↓
失敗あり？
  ├─ No → Phase 1 へ進む
  └─ Yes → 失敗ケースごとに Type A/B/C を判定
           ↓
           Type B が1件でも存在？
             ├─ Yes → 要件定義フェーズに差し戻し
             │         (grammar-specification.md 修正)
             └─ No → Type A/C のみ
                      ↓
                      Type C を修正・再実行
                      ↓
                      Type A の修正項目を Phase 1 に追加
                      ↓
                      Phase 1 へ進む
```

##### ドキュメント更新手順

1. **検証レポート作成**:
   - ファイル: `.kiro/specs/pasta-grammar-specification/phase0.5-validation-report.md`
   - 内容:
     ```markdown
     # Phase 0.5 検証レポート
     
     ## 実行日時
     [YYYY-MM-DD HH:MM:SS]
     
     ## 実行結果サマリー
     - 総テストケース数: [N]
     - 成功: [M]
     - 失敗: [N-M]
     
     ## 失敗ケース詳細
     
     ### [失敗ケース1]
     - **テストケース名**: `chapter2_keywords_markers::Call_全角`
     - **失敗内容**: Parse エラー（`＞＊label` が認識されない）
     - **分類**: Type A（既存実装のバグ）
     - **根拠**: grammar-specification.md §2.4 に明記
     - **対処**: Phase 1 修正項目 B5 として追加
     
     ### [失敗ケース2]
     - **テストケース名**: `chapter7_sakura::エスケープ_全角_現行`
     - **失敗内容**: Parse エラー（`＼n` が認識される）
     - **分類**: Type A（期待通り）
     - **根拠**: Phase 1 で廃止予定のため、現行では失敗が正常
     - **対処**: Phase 1 で修正されることを確認
     
     ## 判定結果
     - Type A: [X]件
     - Type B: [Y]件
     - Type C: [Z]件
     
     ## 最終判断
     - [ ] Phase 1 へ進む（Type B = 0）
     - [ ] 要件定義フェーズに差し戻し（Type B ≥ 1）
     ```

2. **Phase 1 修正項目更新**（Type A のみ）:
   - design.md §1.1「修正項目一覧」に追加項目を記録
   - 例: 「B5: Call_全角対応の追加」

3. **Commit**:
   ```bash
   git add .kiro/specs/pasta-grammar-specification/phase0.5-validation-report.md
   git add .kiro/specs/pasta-grammar-specification/design.md  # Phase 1 修正項目更新
   git commit -m "test: Phase 0.5 validation report - [Type A: X, Type B: Y, Type C: Z]"
   ```

##### Type B 発生時の特別対応

Type B（仕様書の誤り）が1件でも発生した場合：

1. **要件定義フェーズへ差し戻し**:
   - spec.json の `phase` を `requirements` に戻す
   - grammar-specification.md を修正
   - requirements.md を再評価
   
2. **差し戻しの記録**:
   ```bash
   git add .kiro/specs/pasta-grammar-specification/spec.json
   git add .kiro/specs/pasta-grammar-specification/grammar-specification.md
   git commit -m "fix(spec): Revert to requirements phase - grammar spec correction needed

   Type B failures detected in Phase 0.5 validation:
   - [失敗ケース名]: [理由]
   
   grammar-specification.md corrections required before design can proceed."
   ```

3. **再レビュー**:
   - `/kiro-spec-requirements pasta-grammar-specification` で要件定義を再生成
   - 修正内容を確認後、再度設計フェーズへ

#### 0.5.7 Commit（成功時）

```bash
git add tests/pasta_parser_spec_validation_test.rs
git add .kiro/specs/pasta-grammar-specification/phase0.5-validation-report.md
git commit -m "test: Add specification-driven parser validation tests (Phase 0.5)

- Created comprehensive validation tests based solely on grammar-specification.md
- 10 chapters, ~100 test cases covering REQ-1 to REQ-10
- No reference to existing test code or pest definitions
- Validates existing implementation against authoritative specification
- All tests passed: existing implementation conforms to spec"
```

---

## Phase 1: Parser 層修正

### 目的
pest 定義と AST 型を正規仕様（grammar-specification.md）に統一。Parser テストが 100% パス。

### 修正項目一覧（grammar-specification.md 基準）

| ID | カテゴリ | 仕様箇所 | 修正内容 |
|----|---------|---------|---------|
| A1 | Sakura | 7.2 | `sakura_escape` 半角のみ |
| A2 | Sakura | 7.3 | `sakura_command` 簡素化（5パターン → 単純形） |
| A3 | Sakura | 7.3 | `bracket_content` に `\]` 許容 |
| A4 | Sakura | 7.4 | `sakura_bracket_open/close` 半角のみ |
| A5 | Sakura | 7.4 | `sakura_letter` ルール削除（不要） |
| A6 | Sakura | 7.4 | `sakura_digit` ルール削除（不要） |
| A7 | Sakura | 7.4 | `sakura_underscore` ルール削除（不要） |
| B1 | Jump | 2.4 | `jump_marker` ルール削除 |
| B2 | Jump | — | `jump_content` ルール削除 |
| B3 | Jump | — | `label_body_line` から jump 分岐削除 |
| B4 | Jump | — | `local_label_body_line` から jump 分岐削除 |
| C1 | text_part | 6.3 | `dollar_marker` を除外対象に追加 |
| D1 | 行継続 | 6.5.3 | 継続行内空行の対応（設計判断） |

### 実施内容

#### 1.1 pasta.pest の修正

##### 1.1.1 Sakura スクリプト関連（A1-A7）

**変更前**:
```pest
sakura_escape = { "\\" | "＼" }
sakura_bracket_open = { "[" | "［" }
sakura_bracket_close = { "]" | "］" }
sakura_letter = { ASCII_ALPHA | '\u{FF41}'..'\u{FF5A}' | '\u{FF21}'..'\u{FF3A}' }
sakura_digit = { ASCII_DIGIT | '\u{FF10}'..'\u{FF19}' }
sakura_underscore = { "_" | "＿" }

sakura_command = @{
    sakura_underscore ~ sakura_letter+ ~ (sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close)? |
    ("!" | "！" | sakura_letter+) ~ sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close |
    sakura_letter ~ sakura_digit+ ~ !sakura_letter |
    sakura_letter |
    sakura_digit+
}
```

**変更後（grammar-specification.md 7.3 準拠）**:
```pest
// Sakura スクリプト（7.2-7.4 準拠）
// エスケープは厳密に半角バックスラッシュのみ
sakura_escape = { "\\" }

// 括弧は半角のみ
sakura_bracket_open = { "[" }
sakura_bracket_close = { "]" }

// コマンドの字句構造（7.3 簡略版）
// sakura_token ::= [!_a-zA-Z0-9]+
// bracket_content ::= "[" ~ bracket_chars ~ "]"
// bracket_chars ::= ( "\\]" | [^\]] )*
sakura_command = @{
    sakura_token ~ sakura_bracket_content?
}

sakura_token = @{
    (ASCII_ALPHA | ASCII_DIGIT | "_" | "!")+
}

sakura_bracket_content = {
    sakura_bracket_open ~ sakura_bracket_chars ~ sakura_bracket_close
}

sakura_bracket_chars = @{
    (("\\" ~ "]") | (!"]" ~ ANY))*
}

// 以下のルールは削除（A5-A7）:
// - sakura_letter（全角英字含む → 不要）
// - sakura_digit（全角数字含む → 不要）
// - sakura_underscore（全角アンダースコア含む → 不要）
```

**理由**:
- 仕様 7.2: エスケープは半角 `\` のみ
- 仕様 7.3: `sakura_token = [!_a-zA-Z0-9]+`（ASCII のみ）
- 仕様 7.3: `bracket_chars = ( "\\]" | [^\]] )*`（`\]` 許容）
- 仕様 7.4: 括弧は半角のみ

##### 1.1.2 Jump 削除（B1-B4）

**削除対象ルール**:
```pest
// B1: 削除
jump_marker = { "？" | "?" }

// B2: 削除
jump_content = { jump_target ~ filter_list? ~ arg_list? ~ NEWLINE }
```

**修正対象ルール**:
```pest
// B3: label_body_line から jump 分岐削除
label_body_line = {
    indent ~ (
        comment_marker ~ comment_content ~ NEWLINE |
        rune_start ~ rune_block_content |
        at_marker ~ word_def_content |
        amp_marker ~ attribute_content |
        dollar_marker ~ var_assign_content |
        local_label_marker ~ local_label_content |
        call_marker ~ call_content |
        // jump_marker ~ jump_content |  ← 削除
        speech_line_content
    )
}

// B4: local_label_body_line から jump 分岐削除
local_label_body_line = {
    indent ~ (
        comment_marker ~ comment_content ~ NEWLINE |
        rune_start ~ rune_block_content |
        at_marker ~ word_def_content |
        amp_marker ~ attribute_content |
        dollar_marker ~ var_assign_content |
        call_marker ~ call_content |
        // jump_marker ~ jump_content |  ← 削除
        speech_line_content
    )
}
```

##### 1.1.3 text_part バグ修正（C1）

**変更前**:
```pest
text_part = @{ (!(at_marker | sakura_escape | NEWLINE) ~ ANY)+ }
```

**変更後（6.3 インライン要素準拠）**:
```pest
// 6.3: インライン要素（＠、＄、\）で分岐するため、これらを除外
text_part = @{ (!(at_marker | dollar_marker | sakura_escape | NEWLINE) ~ ANY)+ }
```

**理由**: 仕様 6.3 では `＄var_name` が変数参照としてインライン要素。現行 pest では `＄` が text_part に吸収されるバグ。

##### 1.1.4 行継続内空行（D1）

**仕様 6.5.3**: 継続行内のインデントのみの行は改行として解釈

**設計判断**: pest レベルでは現行維持（`continuation_line` は `speech_content` 必須）。空行の改行解釈は **AST 構築時（mod.rs）または Transpiler** で対応。

**理由**:
- pest で空行を許容すると `speech_content` が空になり、構文的に曖昧になる
- AST レベルで `SpeechPart::Newline` を挿入する方が明確

**チェックリスト（1.1）**:
- [ ] A1: `sakura_escape` を `{ "\\" }` に変更
- [ ] A2: `sakura_command` を簡素化（`sakura_token ~ sakura_bracket_content?`）
- [ ] A3: `sakura_bracket_chars` に `\]` 許容ルール追加
- [ ] A4: `sakura_bracket_open/close` を半角のみに変更
- [ ] A5-A7: `sakura_letter`, `sakura_digit`, `sakura_underscore` ルール削除
- [ ] B1: `jump_marker` ルール削除
- [ ] B2: `jump_content` ルール削除
- [ ] B3: `label_body_line` から `jump_marker ~ jump_content` 削除
- [ ] B4: `local_label_body_line` から `jump_marker ~ jump_content` 削除
- [ ] C1: `text_part` に `dollar_marker` 除外追加
- [ ] ビルド確認: `cargo build`

#### 1.2 AST 型（src/parser/ast.rs）の修正

**対象**:
```rust
pub enum Statement {
    Speech { ... },
    VarAssign { ... },
    Call { ... },
    Jump { target: JumpTarget, ... },  // ← 削除対象
    RuneBlock { ... },
    WordDef { ... },
    ...
}

pub enum JumpTarget {
    Local(String),
    Global(String),
    LongJump { global: String, local: String },
    Dynamic(String),
}  // ← Jump がなければ、削除または統合を検討
```

**修正案**:
```rust
pub enum Statement {
    Speech { ... },
    VarAssign { ... },
    Call { ... },  // Jump を統一
    RuneBlock { ... },
    WordDef { ... },
    ...
}

// JumpTarget は Call で使用されている場合は維持
// Jump 専用なら削除
```

**チェックリスト（1.2）**:
- [ ] Statement enum から `Jump` 分岐削除
- [ ] `JumpTarget` enum の使用箇所確認（Call で使用なら維持）
- [ ] 関連する impl ブロック修正
- [ ] コンパイル確認: `cargo build`

#### 1.3 Parser 実装（src/parser/mod.rs）の修正

**対象**: AST 構築ロジックから Jump 関連コード削除

**修正箇所**:
```rust
// 削除対象
Rule::jump_content => { ... }
Rule::jump_marker => { ... }

// Statement::Jump 構築コード削除
```

**チェックリスト（1.3）**:
- [ ] `Rule::jump_content` 処理削除
- [ ] `Rule::jump_marker` 処理削除
- [ ] `Statement::Jump` 構築コード削除
- [ ] コンパイル確認: `cargo build`

#### 1.4 Parser テスト修正

**対象ファイル** (`tests/pasta_parser_*.rs` or 現行命名):
- Jump 検証ロジック削除
- 全角 Sakura テストケース削除
- `text_part` テストケース追加（`＄` が正しく分離されるか）

**チェックリスト（1.4）**:
- [ ] Jump を使用するテストケース削除
- [ ] 全角 `＼` `［］` テストケース削除
- [ ] `＄` 変数参照が text_part に吸収されないテスト追加
- [ ] `cargo test` で Parser 関連テストが 100% パス

### 成果物
- 修正済み pasta.pest
- 修正済み ast.rs
- 修正済み parser/mod.rs
- 修正済み Parser テスト
- Golden Test（Parser 層）実装
- Git commit 記録（Phase 1 完了）

### 検証方法
```bash
# Parser 層テスト実行
cargo test pasta_parser_ --all

# Golden Test（Parser 層）実行
cargo test pasta_parser_golden_test

# 合格条件
# 1. 全 Parser 層テストが通過
# 2. Golden Test の AST 構造検証が通過
# 3. Jump 関連の AST ノードが存在しない

# 検証レポート出力
cargo test pasta_parser_ --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/phase1-test-result.log
```

---

## Phase 2: Transpiler 層修正

### 目的
新 Parser AST（Jump 削除）に対応する Transpiler を実装。Transpiler テストが 100% パス。

### 実施内容

#### 2.1 src/transpiler/mod.rs の修正

**対象コード**:
```rust
// 削除対象
Statement::Jump {
    target,
    filters,
    args,
} => {
    let search_key = Self::transpile_jump_target_to_search_key(target);
    writeln!(
        writer,
        "        for a in crate::pasta::jump(ctx, \"{}\", {}, []) {{ yield a; }}",
        search_key, ...
    )?;
}

// 削除対象メソッド
fn transpile_jump_target(target: &JumpTarget) -> String { ... }
fn transpile_jump_target_to_search_key(target: &JumpTarget) -> String { ... }

// 削除対象ランタイム関数定義
pub fn jump(ctx, label, filters, args) { ... }
```

**修正案**:
- `Statement::Jump` 分岐削除
- `transpile_jump_target*` メソッド削除
- `pasta::jump()` ランタイム関数削除

**チェックリスト**:
- [ ] Statement::Jump 分岐削除
- [ ] transpile_jump_target* メソッド削除
- [ ] pasta::jump() 関数削除
- [ ] コンパイル確認: `cargo build`
- [ ] 型推論エラーなし

#### 2.2 Transpiler テスト修正

**対象ファイル** (`pasta_transpiler_*.rs`):
- `pasta_transpiler_two_pass_test.rs`: Jump 前提のテスト削除
- `pasta_transpiler_phase3_test.rs`: Jump ケース削除

**チェックリスト**:
- [ ] Jump 関連テスト削除
- [ ] `cargo test pasta_transpiler_ --all` が 100% パス

### 成果物
- 修正済み transpiler/mod.rs
- 修正済み Transpiler テスト
- Golden Test（Transpiler 層）実装
- Git commit 記録

### 検証方法
```bash
# Transpiler 層テスト実行
cargo test pasta_transpiler_ --all

# Golden Test（Transpiler 層）実行
cargo test pasta_transpiler_golden_test

# 合格条件
# 1. 全 Transpiler 層テストが通過
# 2. Golden Test の Rune コード生成が成功
# 3. pasta::jump() 呼び出しが生成されない
# 4. 生成 Rune コードが compile() を通過

# 検証レポート出力
cargo test pasta_transpiler_ --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/phase2-test-result.log
```

---

## Phase 3: Runtime/Tests・ドキュメント修正

### 目的
統合テスト・テストフィクスチャ・ドキュメントを新仕様に更新。全テストが 100% パス。

### 実施内容

#### 3.1 テスト・フィクスチャの置換

**対象ファイル** (`tests/fixtures/`):
- `*.pasta` 内の `？` を `＞` へ置換

**置換コマンド**:
```bash
find tests/fixtures -name "*.pasta" -exec sed -i 's/？/＞/g' {} \;
```

**チェックリスト**:
- [ ] `tests/fixtures/*.pasta` の `？` をすべて `＞` に置換
- [ ] `grep -r "？" tests/fixtures/` で残存確認（なし）

#### 3.2 全角テストケースの削除

**対象ファイル** (`pasta_engine_*.rs`, `pasta_integration_*.rs`):
- 全角 `＼` を使用するテストケース削除
- 全角 `［］` を使用するテストケース削除
- 半角へ統一したテストケースに置換

**例**:
```rust
// 削除対象
さくら：こんにちは＼ｓ［０］

// 置換後
さくら：こんにちは\s[0]
```

**チェックリスト**:
- [ ] `grep -r "＼" tests/` で全角バックスラッシュ削除（パスタスクリプト内のみ許容）
- [ ] `grep -r "［" tests/` で全角括弧削除（パスタスクリプト内のみ許容）

#### 3.3 GRAMMAR.md 改訂

**対象セクション**:

1. **7章（さくらスクリプト）**:
   - 7.1（概要）: 「字句のみ認識、非解釈」を明記
   - 7.3（字句文法）: 「半角 `\` + ASCIIトークン + 任意の非ネスト `[...]`（`\]` 許容）」に統一
   - 7.4（制約）: 「半角バックスラッシュ・半角角括弧のみ」を明記

2. **11章（設計決定）**:
   - 11.6–11.20: 現在の決定状況（✓ 確定項目）
   - 11.16（エスケープ・引用）: `\]` 対応を明記

3. **制御フロー セクション**:
   - Jump 文（`？`）削除
   - Call 文（`＞`）へ統一
   - 既存ドキュメントから Jump 記述削除

**チェックリスト**:
- [ ] 7章に「字句のみ、非解釈」を明記
- [ ] 7.3 に「半角 `\[...]` + `\]` 許容」を具体記述
- [ ] Jump 記述すべて削除
- [ ] Call のみで制御フローを説明

#### 3.4 統合テスト・エンジンテスト修正

**対象ファイル** (`pasta_engine_*.rs`, `pasta_integration_*.rs`):
- Jump 依存テスト削除
- Call のみで制御フローをテスト

**チェックリスト**:
- [ ] `grep -r "Jump" tests/` でコメント検索（意図確認）
- [ ] Jump テストコード削除
- [ ] `cargo test pasta_engine_ --all` が 100% パス
- [ ] `cargo test pasta_integration_ --all` が 100% パス

### 成果物
- 置換済みテストフィクスチャ（`？` → `＞`）
- 修正済みテストコード（全角削除）
- 改訂済み GRAMMAR.md
- Golden Test（Runtime/Integration 層）実装
- Git commit 記録

### 検証方法
```bash
# 全テスト実行（Phase 3 完了検証）
cargo test --all

# Golden Test（Runtime/Integration 層）実行
cargo test pasta_integration_golden_test

# 合格条件
# 1. 全テストが通過（Parser + Transpiler + Engine + Integration）
# 2. Golden Test のエンドツーエンド実行が成功
# 3. 出力トークン検証が通過
# 4. Runtime エラーなし

# 最終検証レポート出力
cargo test --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/phase3-test-result.log

# Phase 0 baseline との比較
diff test-baseline.log phase3-test-result.log
# 期待: テスト数の変化のみ（Golden Test 3件追加）、失敗なし
```

---

## 設計の判断ポイント・トレードオフ

### 1. Sakura コマンドの簡素化レベル

- **案 A**: 現在の詳細5パターンを維持し、半角・`\]` 対応のみ
- **案 B**: 「未知トークン許容」で完全に簡素化（仕様「非解釈」に最も準拠）

**決定**: ✅ **案 B（完全簡素化）を採用**

**理由**:
- 仕様「Sakura は字句のみ認識、非解釈」に忠実
- 詳細5パターン区別は必要ない（実装側も複雑化しない）
- **ただしブラケット内の `\]` エスケープ対応は必須**
- 未知トークンを通すことで、将来の拡張性確保
- テスト修正も最小化（詳細パターン検証ケース削除）

### 2. Jump 削除の最終決定

- **案 A**: Jump 廃止は必須 → Phase 1-3 で Jump 削除を進行
- **案 B**: Jump 維持 → design.md・gap-analysis を修正し、Jump は継続サポート

**決定**: ✅ **案 A（Jump 廃止は必須）**

**理由**:
- MVP 達成前の段階における積極的な破壊的変更対応
- Jump と Call のセマンティクス上の区別がなく、統一してシンプル化
- DSL 文法の整理・保守性向上
- テスト修正規模は大きいが、MVP 達成前だからこそ対応が容易

### 3. テスト修正の優先順序

- **案 A**: Parser → Transpiler → Engine/Integration
- **案 B**: 層別に並行修正

**推奨**: 案 A（依存関係に従い、下層完全化が上層前提）

---

## リスク・緩和策

| リスク | 発生箇所 | 緩和策 |
|-------|--------|------|
| Phase 1 ast.rs 型変更の波及 | Parser 出力の型が変更 | Transpiler で compiler error として即座に検出 |
| Jump 削除漏れ | 複数ファイルに分散 | grep で Jump 関連コード検索、チェックリスト活用 |
| テスト置換ミス（`？` → `＞`） | fixtures が混在状態 | Phase 0 の test-baseline.log と比較 |
| 全角テスト削除漏れ | 複数ファイル | `grep -r "＼" tests/` で確認 |

---

## 実装スケジュール（推定）

| Phase | 工数 | 説明 |
|-------|------|------|
| Phase 0 | 1日 | テスト層別化・グリーン確認・commit |
| Phase 1 | 5–7日 | Parser 層（pest/AST）完全修正 |
| Phase 2 | 3–5日 | Transpiler 層修正 |
| Phase 3 | 5–10日 | Runtime/Tests/GRAMMAR.md 修正 |
| **合計** | **14–23 日** | |

---

## 次ステップ

- 要件承認（requirements.md ✓）
- **本設計の承認**
- Phase 0 実装開始
