# Implementation Tasks: word-ref-ast-support

## Overview
実装はパーサー層から始まり、トランスパイラー層への波及的な変更に対応します。4つの主要タスク（SetValue型定義、VarSet型変更、parse_var_set実装、generate_var_set対応）で構成され、各タスク内でユニット・統合テストも実施します。

---

## Tasks

- [x] 1. SetValue列挙型をast.rsに定義する
- [x] 1.1 SetValue列挙型の定義と導出トレイトの実装 (P)
  - SetValue列挙型をast.rsに新規追加（Expr(Expr)とWordRef {name: String}の2バリアント）
  - Debug、Clone、PartialEqトレイトをderiveして、既存のExpr列挙型と同じ扱いできるようにする
  - ExprバリアントはExpr型をラップ、WordRefバリアントはname: String フィールドを持たせる
  - Exprより後の適切な位置に配置し、他の型定義を侵害しない
  - _Requirements: 1_

- [x] 1.2 VarSet構造体のvalue フィールド型をExprからSetValueに変更 (P)
  - ast.rs内のVarSet構造体のvalue: Exprをvalue: SetValueに変更
  - VarSetを使用する他の定義への影響を確認（導出トレイト、メソッドなど）
  - この変更により、コンパイルエラーが全体に波及することを確認
  - _Requirements: 1_

---

- [x] 2. parse_var_set関数をword_ref対応に更新する
- [x] 2.1 parse_var_set関数内のword_ref検出ロジックを実装 (P)
  - parse_var_set関数の内部で、setルール値部分（exprまたはword_ref）を処理する分岐を追加
  - inner.peek() == Some(Rule::word_ref)でword_refペアを検出
  - word_refペア内の子ペアからid（word_marker は hidden rule で除去）を取得し、SetValue::WordRef { name } を構築
  - exprパターンで既存のtry_parse_expr処理を実行し、SetValue::Expr でラップ
  - _Requirements: 2, 3, 6_

- [x] 2.2 parse_var_setの修正後、ユニットテスト（パーサー層）を追加・修正 (P)
  - word_refパース：`＄場所＝＠場所` を正常パース、SetValue::WordRef { name: "場所" } を確認
  - exprパース：`＄x＝123` など既存expr入力が SetValue::Expr でラップされていることを確認
  - 複数のword_ref識別子形式（UNICODE、アンダースコア含む）をテスト
  - 既存のparse_var_setユニットテストが修正され、全て通過することを確認
  - _Requirements: 5, 6_

---

- [x] 3. トランスパイラー層（generate_var_set関数）をSetValue対応に更新する
- [x] 3.1 generate_var_set関数をSetValueのパターンマッチに対応させる
  - code_generator.rs内のgenerate_var_set関数をSetValueパターンマッチ対応に修正
  - SetValue::Expr(expr) パターン：既存のgenerate_expr呼び出しを実行
  - SetValue::WordRef { name } パターン：Err(TranspilerError::unimplemented(...))を返す
  - エラーメッセージにword_ref がセマンティクス未実装であることを明記
  - _Requirements: 4_

- [x] 3.2 トランスパイラー層のテスト修正・追加
  - code_generator.rs内のVarSetリテラル作成部分（テスト含む）を SetValue::Expr でラップ
  - existing_tests_pass：既存全テスト（cargo test --all）が通過することを確認（expr入力のみの既存テストはSetValue::Expr対応で通る）
  - generate_var_set_with_expr：SetValue::Exprのコード生成が既存出力と同じことを検証
  - generate_var_set_with_word_ref：SetValue::WordRefに対してTranspilerError::unimplementedが返されることを確認
  - _Requirements: 4, 5_

---

- [x] 4. 統合テスト（parser2_integration_test.rs）を修正・実行する
- [x] 4.1 既存統合テストをSetValue対応に修正
  - tests/parser2_integration_test.rs内の4箇所のvs.valueパターンマッチをSetValue::Expr でラップ
  - 既存テスト入力（expr のみ）はSetValue::Exprパターンで対応、既存期待値の修正は最小限
  - _Requirements: 5_

- [x] 4.2 統合テスト追加：word_ref パース・トランスパイル
  - word_refを含む.pastaスクリプトのパーサー統合テスト追加
  - word_refパース成功→AST中のVarSet.valueがSetValue::WordRefであることを確認
  - word_refを含むAST→Runeトランスパイル時にTranspilerError::unimplementedが発生することを確認
  - cargo test --all全テスト通過を確認（リグレッション0件）
  - _Requirements: 5, 6_

---

- [x] 5. 最終確認：cargo check/test全スイート実行
- [x] 5.1 ビルド・テスト成功確認
  - cargo check --allで全モジュール（pasta_core, pasta_rune）がコンパイル成功
  - cargo test --allで既存テスト全て通過、word_ref関連テスト成功
  - 要件1-6全ての受入れ基準が満たされていることを確認
  - _Requirements: 1, 2, 3, 4, 5, 6_
