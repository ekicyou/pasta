# 要件定義書

## プロジェクト説明（入力）
文法「＄場所＝＠場所」を処理できるように、「grammar.pest」を変更しました。「set            =_{ id ~ s ~ set_marker ~ s ~ ( expr | word_ref ) }」というように、word_refを追加しています。パーサーのASTに、word_ref対応を追加してください。

## 導入
本機能は、Pasta DSLの変数代入文（VarSet）において、単語参照（word_ref: `@単語名`）を値として設定できるようにするものです。grammar.pestでは`set`ルールが`expr | word_ref`として定義されており、exprとword_refを**意図的に分離**しています。AST上でもこの分離を正確に反映するため、新たに **SetValue 列挙型**を導入し、VarSet.value の型を Expr から SetValue に変更します。SetValue は Expr と WordRef の2つのバリアントを持ちます。

## 必須要件（すべて満たすこと）
1. **コンパイルエラー**: 全てのモジュール（pasta_core, pasta_rune等）で `cargo check --all` が成功すること
2. **リグレッション**: 既存のテストコード（`cargo test --all`）が全て合格し、リグレッションが発生しないこと
3. **設計意図**: grammar.pestの`( expr | word_ref )`という分離の意図をAST上で正確に反映すること

## 要件

### 要件1: SetValue列挙型の導入
**目的:** 開発者として、grammar.pestの`( expr | word_ref )`という分離の意図をAST上で型安全に反映したい。expr と word_ref の構造的な違いを列挙型で表現するため。

#### 受け入れ基準
1. pasta_core パーサーは、ast.rs に新たな `SetValue` 列挙型を定義すること。
2. `SetValue` は以下の2つのバリアントを持つこと：
   - `Expr(Expr)`: expr が返された場合
   - `WordRef { name: String }`: word_ref が返された場合
3. `SetValue` 列挙型は `Debug` と `Clone` トレイトを derive していること。
4. VarSet 構造体の `value` フィールドの型を `Expr` から `SetValue` に変更すること。

### 要件2: parse_var_set関数でのSetValue構築
**目的:** 開発者として、grammar.pestの`set = ( expr | word_ref )`をパース時に、expr か word_ref かを判別して SetValue を構築したい。

#### 受け入れ基準
1. parse_var_set 関数内で、set ルールの値部分（expr または word_ref）に対応する Pair を処理すること。
2. Pair が word_ref ルールの場合、SetValue::WordRef { name } を構築し、内部から id を抽出して name フィールドに保存すること。
3. Pair が expr ルールの場合（または try_parse_expr で処理可能な場合）、SetValue::Expr(expr) を構築すること。
4. 構築した SetValue を VarSet.value に割り当てること。
5. パースに失敗した場合は、適切にエラーを処理すること。

### 要件3: parse_var_set関数の内部処理更新
**目的:** 開発者として、parse_var_set関数内の処理を更新し、SetValue を正しく構築してVarSet.value に割り当てたい。

#### 受け入れ基準
1. parse_var_set 関数の内部で、set ルールの値部分（expr/word_ref）を処理するロジックを更新すること。
2. Rule::word_ref が検出された場合、SetValue::WordRef { name } を構築すること。
3. その他のルール（expr）は try_parse_expr で処理し、SetValue::Expr でラップすること。
4. VarSet.value は SetValue 型になり、`Expr` 型ではなくなること。

### 要件4: トランスパイラー層へのAPI破壊的変更への対応
**目的:** 本仕様のスコープは AST 層（SetValue 列挙型導入と VarSet.value 型変更）のみであり、トランスパイラー層など受け入れ側のコンパイルエラーは本仕様では対応せず、別仕様で扱うため。コンパイルを通すための最小限の対応のみを行うこと。

#### 受け入れ基準
1. VarSet.value の型が SetValue に変更されることで発生するコンパイルエラー（pasta_rune の generate_var_set, generate_expr等）を、最小限のパターンマッチング追加で対応すること。
2. SetValue::WordRef が返された場合、トランスパイラー層の既存の関数群は何もしない（無視する）実装で対応すること。
3. SetValue::Expr(expr) の場合は、既存の処理を expr に対して実行すること。
4. API の公開シグネチャ変更は最小限に留めること。
5. WordRef のセマンティクス実装は別仕様で扱い、本仕様では考慮しないこと。

### 要件5: 既存テストのリグレッション防止
**目的:** 開発者として、既存のテストが引き続き合格することを保証したい。`cargo test --all`が全て成功する状態を維持するため。

#### 受け入れ基準
1. pasta_core パーサーは、パーサーモジュール内の既存の全てのユニットテストを修正して合格させること。
2. pasta_rune トランスパイラーは、コード生成に関連した全ての既存テスト（code_generator_test等）を修正して合格させること。
3. `tests/parser2_integration_test.rs` 内の全ての統合テストが合格すること。

### 要件6: word_ref構文のパース成功確認
**目的:** 開発者として、word_ref 構文がパーサー層で正しくパースされることを検証したい。AST上での表現を確認するため。

#### 受け入れ基準
1. パーサー層のテストは、word_ref 構文（`@単語名`）のパース成功を検証すること。
2. パースされた SetValue::WordRef の name フィールドが正しく抽出されていることを確認すること。
3. パーサー層では、word_ref のセマンティクスは検証しないこと（検証は別仕様で扱う）。
