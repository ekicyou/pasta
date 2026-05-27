# Implementation Plan

- [x] 1. Foundation: VarScope::Property enum 拡張
  - `VarScope` enum に `Property` バリアントを追加する（`crates/pasta_dsl/src/parser/ast/action.rs`）
  - 既存の `Copy + PartialEq + Eq + Debug + Clone` derive を維持する
  - 追加後にコンパイルすると element_gen.rs（4箇所）と visitors.rs（2箇所）で match 網羅性エラーが発生すること（後続タスクで解消）
  - _Requirements: 1.1, 2.1, 3.1, 4.1_

- [x] 2. Pestグラマー: プロパティ構文ルール追加
  - `property_marker`（`dollar ~ modulo`）、`property_id`（ASCII英字始まり）、`var_ref_property`、`var_set_property` の4ルールを `grammar.pest` に追加する
  - `var_ref` の選択順を `var_ref_property | var_ref_global | var_ref_local` に更新する
  - `var_set` の選択順を `var_set_property | var_set_global | var_set_local | var_set_none` に更新する
  - `cargo check -p pasta_dsl` がエラーなく通ること
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.1, 5.2, 5.3, 5.4, 6.1, 6.2_

- [ ] 3. パーサーモジュール拡張
- [x] 3.1 変数代入パーサー拡張（parse_elements.rs / parse_scene.rs）
  - `parse_var_set()` の scope match に `Rule::var_set_property => VarScope::Property` を追加する
  - name 抽出ループで `Rule::property_id` を `Rule::id` と同様に受理するよう拡張する
  - `parse_scene.rs` の2箇所の match パターンに `Rule::var_set_property` を追加する
  - `＄％prop＝value` が `VarSet { scope: Property, name: "prop" }` にパースされること
  - _Requirements: 2.1, 3.1, 3.2_
  - _Boundary: ParseVarSet, ParseScene_

- [x] 3.2 (P) アクション行パーサー拡張（parse_action.rs）
  - `parse_actions()` の match に `Rule::var_ref_property` arm を追加する
  - inner から `Rule::property_id` を抽出して `Action::VarRef { scope: Property, name }` を生成する
  - アクション行内の `＄％prop` が `Action::VarRef { scope: Property }` にパースされること
  - _Requirements: 1.1, 4.1_
  - _Boundary: ParseAction_

- [ ] 4. トランスパイラ拡張: Luaコード生成（element_gen.rs）
- [x] 4.1 generate_var_set: Property SET / GET代入分岐
  - scope match に `VarScope::Property` arm を追加し、SET は `generate_property_set()` ヘルパーに委譲する
  - `generate_property_set()` は `SetValue::Expr` → `act:set_property("name", expr)`、`SetValue::WordRef` → `act:set_property("name", act:word("word"))` を出力する
  - value match の前に `Expr::VarRef { scope: Property }` を検出し、`var.name = act:get_property("prop")` または `save.name = act:get_property("prop")` を直接出力する（中間変数なし）
  - `＄％prop＝123` → `act:set_property("prop", 123)`、`＄var＝＄％p` → `var.name = act:get_property("p")` が生成されること
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4_
  - _Boundary: GenerateVarSet_

- [x] 4.2 generate_action: VarRef Property インラインGET
  - `Action::VarRef` の scope match に `VarScope::Property` arm を追加する
  - `act.{actor}:talk(tostring(act:get_property("name")))` を出力する（nil → `"nil"` 文字列保証）
  - `さくら：＄％p` → `act.さくら:talk(tostring(act:get_property("p")))` が生成されること
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Boundary: GenerateAction_

- [x] 4.3 generate_expr: Property エラーガード + TranspileError 拡張
  - `TranspileError` に `property_in_expression()` バリアント（spanなし）を追加する
  - `generate_expr()` と `generate_expr_to_buffer()` の `Expr::VarRef` match に `VarScope::Property => TranspileError::property_in_expression()` arm を追加する
  - `＄var＝＄％a＋1` のような式中 Property で `TranspileError::property_in_expression()` が返ること
  - _Requirements: 3.1, 3.2_
  - _Boundary: GenerateExpr_

- [x] 5. (P) Lua API: get_property トークンバッファ保全
  - `SHIORI_ACT_IMPL.get_property()` のバリデーション完了直後に `local saved_tokens = self.token` で退避し、`self.token = {}` で空にする
  - get タグのみを新バッファに登録し `coroutine.yield(self:build())` を呼ぶ
  - `coroutine.yield` の直後（成功・エラー両経路）で `self.token = saved_tokens` を実行する
  - `get_property()` 呼び出し前後で `self.token` の内容が不変であること
  - yield 時に送信されるスクリプトが get タグのみを含むこと
  - _Requirements: 3.5, 4.4_
  - _Boundary: GetPropertyLua_

- [x] 6. (P) LSP シンタックスハイライト対応
  - `visitors.rs` の `Expr::VarRef` と `Action::VarRef` の `match scope` 2箇所に `VarScope::Property` arm を追加する
  - パターン: `["＄％{name}", "$%{name}"]` で `token_type::VARIABLE` を付与する
  - `cargo check -p pasta_lsp` がエラーなく通ること
  - `.pasta` ファイル内の `＄％prop` が VARIABLE トークンとしてハイライトされること
  - _Requirements: 1.1_
  - _Boundary: LspVisitors_
  - _Depends: 1_

- [ ] 7. パーサーテスト
- [ ] 7.1 プロパティ構文パーサーテスト新規作成
  - `crates/pasta_dsl/tests/property_scope_test.rs` を作成する
  - `＄％simple`・`＄％system.name`・`＄％scope(0).validwidth.initial` が `VarScope::Property` にパースされることを検証する
  - `$%half` と `＄％full` が同一 AST 構造に解決されることを検証する
  - `＄％1abc`・`＄％ `（末尾空白）でパースエラーが返ることを検証する
  - `cargo test -p pasta_dsl property_scope` が全件グリーンであること
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 6.1, 6.2_

- [ ] 7.2 既存パーサーテスト回帰確認
  - `cargo test -p pasta_dsl` を実行し、既存テストがすべてグリーンであること
  - `＄var`・`＄＊var`・`＄＝expr`・`＄０`〜`＄９` の各テストが pass すること
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 8. トランスパイラテスト
- [ ] 8.1 プロパティコード生成テスト新規作成
  - `crates/pasta_lua/tests/property_scope_codegen_test.rs` を作成する
  - SET（リテラル・変数・単語・式）→ `act:set_property(...)` の Lua 出力を検証する
  - GET代入（ローカル・グローバル）→ `var.name = act:get_property(...)` / `save.name = ...` の出力を検証する
  - インラインGET → `act.X:talk(tostring(act:get_property(...)))` の出力を検証する
  - 式中Property → `TranspileError::property_in_expression` を検証する
  - `cargo test -p pasta_lua property_scope_codegen` が全件グリーンであること
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 4.1, 4.5_

- [ ] 8.2 既存トランスパイラテスト回帰確認
  - `cargo test -p pasta_lua` を全件実行し、既存テストがすべてグリーンであること
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 9. Lua API トークン保全テスト
  - `crates/pasta_lua/tests/property_token_preservation_test.rs` を作成する（既存 shiori-event-test-framework を活用）
  - `act:talk("A"); act:get_property("p"); act:talk("B")` を実行し、最初の SSP 送信スクリプトに "A" が含まれないことを検証する
  - resume 後の最終出力に "A" と "B" が両方含まれることを検証する
  - `act:get_property()` の戻り値が呼び出し側に正しく届くことを検証する
  - `cargo test -p pasta_lua property_token_preservation` が全件グリーンであること
  - _Requirements: 3.5, 4.4_

- [ ] 10. 統合テスト: 全体回帰確認
  - `pasta_sample_ghost` の任意のシーンに `＄ゴースト名＝＄％currentghost.name` + 後続トークを追加する
  - `cargo test --workspace` を全件実行し、すべてグリーンであること
  - 既存の 950+ テストを含めてリグレッション 0 件であること
  - _Requirements: 1.1, 2.1, 3.1, 3.5, 4.1, 5.1_
