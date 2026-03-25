# 実装タスク: fn-call-expr-stmt

## 実装計画

- [ ] 1. DSL パーサーの AST 型を `var_set_none` に対応させる
- [ ] 1.1 `VarSet` 構造体の変数名フィールドを省略可能な型に変更する
  - `name: String` を `name: Option<String>` に変更する
  - Rust コンパイラのエラーをガイドに、`name` を参照している全箇所を `Option` 対応に修正する（`name.as_deref().unwrap_or("")` 等）
  - 既存の `var_set_local`/`var_set_global` では `Some(名前)` が設定されることを維持する
  - _Requirements: 2.1, 2.4_

- [ ] 1.2 PEG 文法に `var_set_none` ルールを追加する
  - `set` ルールから `id ~ s ~` の部分を切り出し、`var_set_local`/`var_set_global` 各ルールの先頭に移動する
  - `set` ルールを `set_marker ~ s ~ ( expr | word_ref )` のみに簡素化する
  - `var_set_none = { var_marker ~ set }` を新規追加する
  - `var_set` の選択肢に `var_set_none` を末尾 fallback として追加する
  - _Requirements: 2.1, 2.2, 2.3, 2.6_

- [ ] 1.3 パーサーに `var_set_none` の解析処理を追加する
  - `parse_var_set()` で `Rule::var_set_none` のときに `name: None` を返すよう対応する（`Rule::id` が出現しなければ `None` のまま）
  - `parse_local_start_scene_scope()` と `parse_local_scene_scope()` の match arm に `Rule::var_set_none` を追加して `LocalSceneItem::VarSet` として登録する
  - _Requirements: 2.1, 2.2_

- [ ] 2. パーサー層の単体テストを追加する
- [ ] 2.1 `var_set_none` のパーステストを追加する
  - `＄＝＠fn()` が `VarSet{ name: None, value: Expr::FnCall }` として解析されることを確認する
  - `＄＝＠fn（x：10）` のように引数付きでも同様に解析されることを確認する
  - `$=@fn()` の半角混在表記も受け入れられることを確認する
  - 既存の `＄変数名＝expr` が `VarSet{ name: Some("変数名"), ... }` のまま変わらないことをリグレッションとして確認する
  - _Requirements: 2.1, 2.2, 2.3, 4.1, 4.2_

- [ ] 3. グローバル関数呼び出しの Lua コード生成を修正する
- [ ] 3.1 (P) `＠＊XX()` の展開先を `GLOBAL.XX` に修正する
  - アクション行・式・バッファ生成の3箇所すべてで `FnScope::Global => "SCENE."` を `"GLOBAL."` に変更する
  - アクション行の場合の生成形式 `act.{actor}:talk(tostring(GLOBAL.XX(act, ...)))` を確認する
  - 変数代入右辺での `var.XX = GLOBAL.func(act)` の生成を確認する
  - _Requirements: 1.1, 1.2, 1.3, 1.5_

- [ ] 3.2 (P) 生成 Lua コードのヘッダーに GLOBAL モジュール読み込みを追加する
  - `write_header()` に `local GLOBAL = require "pasta.global"` を `local PASTA` の次行として追加する
  - GLOBAL の使用有無に関わらず常時出力する
  - _Requirements: 1.6_

- [ ] 4. `＄＝expr` 式文の Lua コード生成を実装する
- [ ] 4.1 変数名なし（`name: None`）の場合に式文を出力する
  - `generate_var_set()` で `name: None` のとき式を評価するだけで変数に代入しないコードを出力する
  - `＄＝＠fn()` → `SCENE.fn(act)` の式文出力を確認する
  - `＄＝＠＊fn()` → `GLOBAL.fn(act)` の式文出力を確認する（タスク 3.1 の変更と組み合わせ）
  - _Requirements: 2.4, 2.5_

- [ ] 5. スナップショットテストを更新して全テストを通過させる
- [ ] 5.1 GLOBAL ヘッダー追加によるスナップショット差分を承認する
  - `cargo test -p pasta_lua` を実行してスナップショット差分を確認する
  - `cargo insta review` で全スナップショットの `local GLOBAL = require "pasta.global"` 行追加を一括承認する
  - `cargo test --all` で全テストが通過することを確認する
  - _Requirements: 4.3_

- [ ] 5.2 `＠＊XX()` のスナップショットテストを追加する
  - `＠＊func()` → `GLOBAL.func(act)` の展開を確認するスナップショットテストを追加する
  - 引数付き `＠＊func（x：10）` → `GLOBAL.func(act, 10)` の展開テストを追加する
  - `＄YY＝＠＊func()` → `var.YY = GLOBAL.func(act)` の変数代入テストを追加する
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 5.3 `＄＝expr` 式文のスナップショットテストを追加する
  - `＄＝＠fn()` → `SCENE.fn(act)` の式文出力を確認するスナップショットテストを追加する
  - `＄＝＠＊fn()` → `GLOBAL.fn(act)` の式文出力テストを追加する
  - _Requirements: 2.4, 2.5_

- [ ] 6. LSP セマンティックトークンを `var_set_none` に対応させる
- [ ] 6.1 変数名なしのトークン化処理を追加する
  - `tokenize_var_set_text()` の変数名探索ステップを `Option` に対応させる（`vs.name` が `None` の場合はスキップ）
  - `name: None` 時に cursor が marker 終端に留まったまま `=` 検出ステップに進むことを確認する
  - `＄＝＠fn()` 行でマーカー・演算子・関数名の3トークンが正しく出力されることを確認する
  - _Requirements: 4.3_

- [ ] 7. 仕様ドキュメントを更新する
- [ ] 7.1 (P) 変数スコープ仕様に `＠＊` のグローバル展開先を追記する
  - `＠＊func()` が `GLOBAL.func(act)` に展開されることを変数代入例テーブルに追記する
  - `＠func()` → `SCENE.func(act)` との対比を明示する
  - _Requirements: 3.1_

- [ ] 7.2 (P) 文法モデル仕様に `＄＝expr` 式文構文を追記する
  - 式サポートセクションに `var_set_none` PEG ルール名と `＄＝expr` 構文を追記する
  - 使用例 `＄＝＠func()`、`＄＝＠＊global_func()` を追加する
  - _Requirements: 3.2, 3.3_
