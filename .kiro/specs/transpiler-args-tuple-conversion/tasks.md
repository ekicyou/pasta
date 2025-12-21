# Implementation Plan

## タスク概要

本仕様は、関数呼び出しの引数を配列`[]`からタプル`()`に変換し、アクション行の関数呼び出しバグを修正するものです。以下の3つの主要コンポーネントを修正し、テストフィクスチャを拡張します。

---

## Tasks

### 1. ArgsHelper修正（transpile_exprs_to_tuple実装）

- [ ] 1.1 (P) transpile_exprs_to_tupleヘルパー関数実装
  - 既存の`transpile_exprs_to_args`関数を`transpile_exprs_to_tuple`に改名
  - 0個の引数: `"()"`を返す
  - 1個の引数: `"(arg,)"`を返す（末尾カンマ必須）
  - 2個以上の引数: `"(arg1, arg2, ...)"`を返す
  - 括弧を含むタプル文字列全体を返すように実装
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

### 2. CallGenerator修正（Call/Jump文のタプル生成）

- [ ] 2.1 (P) Call文の動的ターゲット処理を修正
  - `transpile_statement_to_writer`内のL424付近を修正
  - 配列リテラル`[{}]`からタプル`{}`に変更（`{}`は`transpile_exprs_to_tuple`の戻り値）
  - `pasta::call`の第4引数にタプル文字列を渡す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [ ] 2.2 (P) Call文の静的ターゲット処理を修正
  - `transpile_statement_to_writer`内のL433付近を修正
  - 配列リテラル`[{}]`からタプル`{}`に変更（`{}`は`transpile_exprs_to_tuple`の戻り値）
  - `pasta::call`の第4引数にタプル文字列を渡す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

### 3. FuncCallFix実装（アクション行関数呼び出しバグ修正）

- [ ] 3.1 アクション行のSpeechPart::FuncCall処理を完全書き換え
  - `transpile_speech_part_to_writer`内のL507-520を完全書き換え
  - 現在の単語展開処理（バグ）を削除
  - `transpile_exprs_to_tuple`を使用して引数タプル文字列を生成
  - FunctionScope::Autoの場合: `for a in 関数(ctx, タプル) { yield a; }`を生成
  - FunctionScope::GlobalOnlyの場合: `for a in super::関数(ctx, タプル) { yield a; }`を生成
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

### 4. テストフィクスチャ拡張

- [ ] 4.1 (P) アクション行関数呼び出しのテストフィクスチャ作成
  - `tests/fixtures/`に新規Pastaスクリプトファイルを作成
  - `＠関数()`形式のアクション行サンプルを追加
  - `＠＊関数()`形式のグローバル関数呼び出しサンプルを追加
  - Runeブロック内に対応する関数定義を追加（`fn 関数(ctx, args) { ... }`、`pub fn グローバル関数(ctx, args) { ... }`）
  - _Requirements: 3.1, 3.2_

- [ ] 4.2 (P) Call/Jump文のテストフィクスチャ拡張
  - 既存の`tests/fixtures/`内のファイルで、Call/Jump文の引数タプル生成を検証
  - 0個、1個、2個以上の引数パターンをカバー
  - _Requirements: 3.1, 3.2_

### 5. テスト検証・更新

- [ ] 5.1 全テスト実行とリグレッション確認
  - `cargo test --all`を実行
  - 失敗したテストの期待値を配列からタプルに更新
  - トランスパイラー出力がタプル形式（括弧と末尾カンマ）であることを確認
  - _Requirements: 3.1, 3.3_

- [ ] 5.2* アクション行関数呼び出しのランタイム検証テスト
  - 生成されたRuneコードがコンパイル可能であることを確認
  - 関数呼び出しが正しく解決されること（ローカル優先/グローバル明示）を検証
  - テストフィクスチャ内のRune関数定義が正しく参照されることを確認
  - _Requirements: 3.1, 3.2_

### 6. ドキュメント更新

- [ ] 6.1 (P) コード内コメント更新
  - `transpile_exprs_to_tuple`関数のドキュメントコメントを追加
  - タプル構文の使用を明記（戻り値フォーマット、末尾カンマの扱い）
  - _Requirements: 4.2_

- [ ] 6.2 (P) プロジェクトドキュメント更新
  - GRAMMAR.mdやSPECIFICATION.md内の配列リテラル例をタプルリテラルに更新
  - アクション行関数呼び出しの仕様を明記
  - _Requirements: 4.1_

---

## タスク進行ガイドライン

- **並列実行可能タスク**: `(P)`マークされたタスクは並列実行可能
- **タスク1→タスク2/3**: ArgsHelper実装後、CallGeneratorとFuncCallFixは並列実行可能
- **タスク4**: テストフィクスチャは実装と並行して作成可能
- **タスク5**: 実装完了後に実行
- **タスク6**: ドキュメント更新はすべての実装完了後に実行

各タスクは1-3時間を目安に完了可能です。
