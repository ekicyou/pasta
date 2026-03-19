# Implementation Plan

- [ ] 1. (P) ローカルシーン Lua 関数名の命名規約を修正する
  - `generate_local_scene()` 内のフォーマット文字列を `"__{}_{}__"` から `"{}_{}"`  に変更する
  - `__start__` を返す else 分岐（`counter == 0`）には触れない
  - `SceneRegistry::sanitize_name()` の呼び出しはそのまま維持する
  - 変更後、同一関数が生成する Lua コードが `SCENE.Head0_1` 形式になることをコンパイルで確認する
  - _Requirements: 1.1, 1.2, 2.1, 2.2, 2.4, 3.1_

- [ ] 2. (P) シーン検索の fn_name 変換とコメントを更新する
  - `parse_fn_name()` の else 節を `format!("__{}__", local_part)` から `local_part.to_string()` に変更する
  - else 節のインラインコメント `// Convert "選択肢_1" to "__選択肢_1__"` を削除または `// Return local_part as-is (already in Lua function name format)` に置き換える
  - `parse_fn_name` の docstring `/// # Returns` の例を `"選択肢_1"` に更新する
  - `search_scene` の `/// # Note` コメント中の例 `"__選択肢_1__"` を `"選択肢_1"` に更新する
  - _Requirements: 1.3_

- [ ] 3. (P) トランスパイラのスナップショットを更新する
  - Task 1 と Task 2 の完了後に `cargo test -p pasta_lua --test transpiler` を実行してスナップショット差分を発生させる
  - `cargo insta review` で差分を確認し、`__Name_N__` → `Name_N` 形式の変更のみを Accept する
  - `__start__`・`__global_name__` を含む箇所は変更されていないことを確認する（`tail_call_optimization.snap` 8箇所、`scene_with_call.snap` 1箇所が対象）
  - 更新後にテストがすべてグリーンになることを確認する
  - _Requirements: 2.3, 2.5, 3.1, 3.2, 3.3, 3.4_

- [ ] 4. (P) finalize 経路のローカルシーン検索テストを追加する
  - Task 1 と Task 2 の完了後に作業する
  - 既存の `test_scene_collection_local_scenes` を拡張し、`finalize_scene()` 呼び出し後に `search_scene` でローカルシーン名を検索できることを検証する
  - `search_scene("ローカルシーン名", Some("親シーン名"))` が `Some(...)` を返すことをアサートする
  - `register_global_raw` 経由で登録したローカルシーンの前方一致検索が正しく動作することも確認する
  - _Requirements: 4.2, 4.3_

- [ ] 5. ローカルシーン call の E2E 統合テストを作成する

- [ ] 5.1 新規テストファイルを作成し、test runner に登録する
  - Task 1 と Task 2 の完了後に作業する
  - `crates/pasta_lua/tests/runtime/local_scene_call_test.rs` を新規作成する
  - `crates/pasta_lua/tests/runtime/main.rs` に `mod local_scene_call_test;` を追加する
  - `e2e_helpers::create_runtime_with_finalize()` と `e2e_helpers::transpile()` を使う基本テスト構造を記述する
  - _Requirements: 4.1_

- [ ] 5.2 単純なローカルシーン call のテストケースを実装する
  - ローカルシーンを含む Pasta DSL ソースをインラインで定義し、`transpile()` でトランスパイルする
  - `create_runtime_with_finalize()` で Lua VM を構築し、トランスパイル済みコードを実行する
  - `finalize_scene()` を呼び出してランタイムレジストリを構築する
  - `act:call(global, "SubScene", {})` を実行し、対応するローカルシーン関数が呼ばれることをLuaグローバル変数への副作用で検証する
  - _Requirements: 4.1_

- [ ] 5.3 同名重複ローカルシーンのランダム選択テストを実装する
  - 同一グローバルシーン内に同名ローカルシーン（例: `・Head0`）が2回定義された DSL を用意する
  - `act:call` を複数回実行し、どちらの実装も（エラーなく）呼ばれうることを検証する
  - _Requirements: 4.1_

- [ ] 5.4 前方一致ローカルシーン検索のテストケースを実装する
  - `・Head0` と `・Head1` を持つ DSL を用意する
  - `act:call(global, "Head", {})` で前方一致検索が行われ、どちらかが正常実行されることを検証する
  - _Requirements: 4.1_
