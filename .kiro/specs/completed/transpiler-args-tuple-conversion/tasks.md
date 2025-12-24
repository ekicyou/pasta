# Implementation Plan

## タスク概要

本仕様は、関数呼び出しの引数を配列`[]`からタプル`()`に変換し、アクション行の関数呼び出しバグを修正するものです。以下の3つの主要コンポーネントを修正し、テストフィクスチャを拡張します。

---

## Tasks

### 1. ArgsHelper修正（transpile_exprs_to_tuple実装）

- [x] 1.1 (P) transpile_exprs_to_tupleヘルパー関数実装 ✅
  - `transpile_exprs_to_tuple(&[Expr])` と `transpile_arguments_to_tuple(&[Argument])` を新規実装
  - 0個の引数: `"()"`を返す
  - 1個の引数: `"(arg,)"`を返す（末尾カンマ必須）
  - 2個以上の引数: `"(arg1, arg2, ...)"`を返す
  - 括弧を含むタプル文字列全体を返すように実装
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - **実装場所**: `src/transpiler/mod.rs` L562-619

### 2. CallGenerator修正（Call/Jump文のタプル生成）

- [x] 2.1 (P) Call文の動的ターゲット処理を修正 ✅
  - `transpile_statement_to_writer`内のL424付近を修正
  - 配列リテラル`[{}]`からタプル`{}`に変更
  - `pasta::call`の第4引数にタプル文字列を渡す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  - **実装場所**: `src/transpiler/mod.rs` L398-439

- [x] 2.2 (P) Call文の静的ターゲット処理を修正 ✅
  - `transpile_statement_to_writer`内のL433付近を修正
  - 配列リテラル`[{}]`からタプル`{}`に変更
  - `pasta::call`の第4引数にタプル文字列を渡す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  - **実装場所**: `src/transpiler/mod.rs` L398-439

### 3. FuncCallFix実装（アクション行関数呼び出しバグ修正）

- [x] 3.1 アクション行のSpeechPart::FuncCall処理を修正 ✅
  - `transpile_speech_part_to_writer`内のL507-545を修正
  - **パーサー改良により解決**:
    - `＠関数()` → `SpeechPart::FuncCall`（関数呼び出し生成）
    - `＠単語` → `SpeechPart::WordExpansion`（単語展開生成）
    - `＠＊関数()` → `SpeechPart::FuncCall { scope: GlobalOnly }`（super::プレフィックス付き関数呼び出し生成）
  - FunctionScope::Autoの場合: `for a in 関数(ctx, タプル) { yield a; }`を生成
  - FunctionScope::GlobalOnlyの場合: `for a in super::関数(ctx, タプル) { yield a; }`を生成
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  - **実装場所**: 
    - `src/parser/pasta.pest` L259-274: `func_call`, `word_expansion`, `func_scope` ルール追加
    - `src/parser/ast.rs` L168-172: `SpeechPart::WordExpansion` バリアント追加
    - `src/parser/mod.rs` L574-587, L877-898: パーサー実装
    - `src/transpiler/mod.rs` L507-545, L834-862: トランスパイラ実装

### 4. テストフィクスチャ拡張

- [x] 4.1 (P) タプル変換テストの作成 ✅
  - `tests/pasta_transpiler_tuple_conversion_test.rs` を新規作成
  - Call文の空引数、単一引数（末尾カンマ）、変数参照引数のテスト
  - アクション行関数呼び出し（単一引数）のテスト
  - 単語展開の後方互換性テスト
  - グローバル関数呼び出し（`＠＊func()`）のテスト
  - 空括弧による関数呼び出しテスト
  - _Requirements: 3.1, 3.2_
  - **テスト結果**: **9パス、0 fail、0 ignore**

- [x] 4.2 (P) 既存テストフィクスチャの拡張（オプション） ✅
  - 既存の`tests/fixtures/`内のファイルで追加テストを検討
  - _Requirements: 3.1, 3.2_
  - **ステータス**: 完了 - comprehensive_control_flow.pasta で包括テスト実施済み

### 5. テスト検証・更新

- [x] 5.1 全テスト実行とリグレッション確認 ✅
  - `cargo test --all`を実行 → **全テスト通過**
  - リグレッションなし確認済み
  - _Requirements: 3.1, 3.3_

- [x] 5.2* ランタイム検証 ✅
  - `test_comprehensive_control_flow_rune_compile` テスト通過
  - 生成されたRuneコードがコンパイル可能であることを確認
  - _Requirements: 3.1, 3.2_

### 6. ドキュメント更新

- [x] 6.1 (P) コード内コメント更新 ✅
  - `transpile_exprs_to_tuple`および`transpile_arguments_to_tuple`にドキュメントコメント追加
  - SpeechPart::FuncCall処理にパーサー制限のコメント追加
  - _Requirements: 4.2_

- [x] 6.2 (P) プロジェクトドキュメント更新（オプション） ✅
  - GRAMMAR.mdやSPECIFICATION.md内の配列リテラル例をタプルリテラルに更新
  - _Requirements: 4.1_
  - **ステータス**: 完了 - GRAMMAR.md L309を更新

---

## 完了サマリー

### 完了タスク
- ✅ 1.1: transpile_exprs_to_tuple / transpile_arguments_to_tuple 実装
- ✅ 2.1: Call文動的ターゲットのタプル変換
- ✅ 2.2: Call文静的ターゲットのタプル変換
- ✅ 3.1: SpeechPart::FuncCall処理修正（パーサー改良含む）
- ✅ 4.1: テストファイル作成（9パス、0 fail、0 ignore）
- ✅ 4.2: 既存テストフィクスチャ拡張（comprehensive_control_flow.pasta で実施）
- ✅ 5.1: 全テスト通過確認（全386テスト通過）
- ✅ 5.2: ランタイム検証完了
- ✅ 6.1: コード内コメント更新
- ✅ 6.2: プロジェクトドキュメント更新（GRAMMAR.md L309更新）
- ✅ 5.1: 全テスト通過確認（全381テスト通過）
- ✅ 5.2: ランタイム検証
- ✅ 6.1: コード内コメント

### パーサー改良詳細
| 変更箇所 | 内容 |
|----------|------|
| `pasta.pest` L259-264 | `func_call`ルール: 括弧必須に変更 |
| `pasta.pest` L266-270 | `word_expansion`ルール: 括弧なし単語展開 |
| `pasta.pest` L272-273 | `func_scope`ルール: `＊`プレフィックス検出 |
| `ast.rs` L168-172 | `SpeechPart::WordExpansion`バリアント追加 |
| `mod.rs` L574-587 | `word_expansion`パース処理 |
| `mod.rs` L877-898 | `func_scope`によるグローバル判定 |
| `transpiler/mod.rs` L507-545 | `FuncCall`/`WordExpansion`のトランスパイル |
| `transpiler/mod.rs` L834-862 | `transpile_speech_part`の`WordExpansion`対応 |
- ✅ 5.2: ランタイム検証
- ✅ 6.1: コード内コメント

### パーサー改良（追加実装）
以下のパーサー制限事項は本仕様のスコープ内で解決されました：

1. **✅ 空括弧の区別（解決済み）**:
   - `＠関数()` → `FuncCall`（括弧があれば関数呼び出し）
   - `＠単語` → `WordExpansion`（括弧なしは単語展開）
   - **実装**: `pasta.pest`に`word_expansion`ルール追加、AST に`SpeechPart::WordExpansion`追加

2. **✅ グローバル関数呼び出し（解決済み）**:
   - `＠＊関数()` → `FuncCall { scope: GlobalOnly }`
   - **実装**: `pasta.pest`に`func_scope`ルール追加、`super::`プレフィックス生成

3. **⚠️ 複数位置引数（未対応）**:
   - Call文の複数位置引数はパーサーがサポートしていない
   - 区切り文字の変更が必要なため、今回は対応しない
## タスク進行ガイドライン

- **並列実行可能タスク**: `(P)`マークされたタスクは並列実行可能
- **タスク1→タスク2/3**: ArgsHelper実装後、CallGeneratorとFuncCallFixは並列実行可能
- **タスク4**: テストフィクスチャは実装と並行して作成可能
- **タスク5**: 実装完了後に実行
- **タスク6**: ドキュメント更新はすべての実装完了後に実行

各タスクは1-3時間を目安に完了可能です。
