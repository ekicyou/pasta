# Implementation Plan

---

タスク 1〜5 は互いに **並行実行可能**（操作対象ファイルが完全に分離している）。
タスク 6 はスナップショット再生成のため 1〜5 の完了後に実行する。
タスク 7 は全テスト検証のため 6 の完了後に実行する。
タスク 8 は 7 の完了後に実行する。

---

- [x] 1. (P) transpiler/ ドメインを整備する

- [x] 1.1 transpiler/ サブディレクトリと main.rs を作成する
  - `tests/transpiler/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成する
  - 7 つのテストモジュール（basic_test, comparison_test, scene_test, snapshot_test, actor_word_dictionary_test, fallback_search_integration_test, code_generator_test）を `mod` 宣言する
  - _Requirements: 1.2, 2.5_

- [x] 1.2 transpiler/ 系 7 ファイルを移動し common 参照を修正する
  - `git mv` で下記ファイルを移動する（リネーム検出を維持）:
    - transpiler_basic_test.rs → transpiler/basic_test.rs
    - transpiler_comparison_test.rs → transpiler/comparison_test.rs
    - transpiler_scene_test.rs → transpiler/scene_test.rs
    - transpiler_snapshot_test.rs → transpiler/snapshot_test.rs
    - actor_word_dictionary_test.rs → transpiler/actor_word_dictionary_test.rs
    - fallback_search_integration_test.rs → transpiler/fallback_search_integration_test.rs
    - code_generator_test.rs → transpiler/code_generator_test.rs
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する（common を使用するファイルのみ）
  - `cargo test -p pasta_lua -- transpiler` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 2. (P) loader/ ドメインを整備する

- [x] 2.1 loader/ サブディレクトリと main.rs を作成する
  - `tests/loader/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成する
  - 6 つのテストモジュール（cache_test, config_test, lifecycle_test, startup_test, config_actors_initialization_test, lua_passthrough_test）を `mod` 宣言する
  - _Requirements: 1.2, 2.5_

- [x] 2.2 loader/ 系 6 ファイルを移動し重複ヘルパーを排除する
  - `git mv` で下記ファイルを移動する:
    - loader_cache_test.rs → loader/cache_test.rs
    - loader_config_test.rs → loader/config_test.rs
    - loader_lifecycle_test.rs → loader/lifecycle_test.rs
    - loader_startup_test.rs → loader/startup_test.rs
    - config_actors_initialization_test.rs → loader/config_actors_initialization_test.rs
    - lua_passthrough_test.rs → loader/lua_passthrough_test.rs
  - `config_actors_initialization_test.rs` と `lua_passthrough_test.rs` の自前 `copy_dir_recursive` 定義を削除し `common::copy_dir_recursive` に置換する
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する（common を使用するファイルのみ）
  - `cargo test -p pasta_lua -- loader` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 3. (P) shiori/ ドメインを整備する

- [x] 3.1 shiori/ サブディレクトリと main.rs を作成する
  - `tests/shiori/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成する
  - 5 つのテストモジュール（event_dispatch_test, event_handler_test, res_test, virtual_event_config_test, virtual_event_dispatch_test）を `mod` 宣言する
  - _Requirements: 1.2, 2.5_

- [x] 3.2 shiori/ 系 5 ファイルを移動し重複ヘルパーを排除する
  - `git mv` で下記ファイルを移動する:
    - shiori_event_dispatch_test.rs → shiori/event_dispatch_test.rs
    - shiori_event_handler_test.rs → shiori/event_handler_test.rs
    - shiori_res_test.rs → shiori/res_test.rs
    - virtual_event_config_test.rs → shiori/virtual_event_config_test.rs
    - virtual_event_dispatch_test.rs → shiori/virtual_event_dispatch_test.rs
  - `shiori_res_test.rs` の自前 `create_empty_context`, `get_scripts_dir` 定義を削除し `common::` に置換する
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する（common を使用するファイルのみ）
  - `cargo test -p pasta_lua -- shiori` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 4. (P) runtime/ ドメインを整備する

- [x] 4.1 runtime/ サブディレクトリと main.rs を作成する
  - `tests/runtime/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成する
  - 8 つのテストモジュール（finalize_scene_test, scene_test, syntax_test, unit_test, persistence_integration_test, encoding_test, stdlib_modules_test, stdlib_regex_test）を `mod` 宣言する
  - _Requirements: 1.2, 2.5_

- [x] 4.2 runtime/ 系 8 ファイルを移動し common 参照・重複を修正する
  - `git mv` で下記ファイルを移動する（リネームを含む）:
    - finalize_scene_test.rs → runtime/finalize_scene_test.rs
    - runtime_scene_test.rs → runtime/scene_test.rs
    - runtime_syntax_test.rs → runtime/syntax_test.rs
    - runtime_test.rs → runtime/unit_test.rs
    - persistence_integration_test.rs → runtime/persistence_integration_test.rs
    - pasta_lua_encoding_test.rs → runtime/encoding_test.rs
    - stdlib_modules_test.rs → runtime/stdlib_modules_test.rs
    - stdlib_regex_test.rs → runtime/stdlib_regex_test.rs
  - `stdlib_modules_test.rs` の自前 `create_empty_context` 定義を削除し `common::` に置換する
  - `stdlib_regex_test.rs` の自前 `create_empty_context`, `value_to_string` 定義を削除し `common::create_empty_context`, `common::value_as_str` に置換する
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する（common を使用するファイルのみ）
  - `cargo test -p pasta_lua -- runtime` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 5. (P) log/ / sakura_script/ / search/ ドメインを一括整備する

- [x] 5.1 log/ を整備し 3 ファイルを移動する
  - `tests/log/` ディレクトリを作成する
  - 3 つのテストモジュール（integration_test, module_test, stack_level_test）を宣言する main.rs を作成する（common 参照なし）
  - `git mv` で log_integration_test.rs → log/integration_test.rs, log_module_test.rs → log/module_test.rs, log_stack_level_test.rs → log/stack_level_test.rs を移動する
  - `cargo test -p pasta_lua -- log` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2_

- [x] 5.2 sakura_script/ を整備し 2 ファイルを移動・修正する
  - `tests/sakura_script/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成し、2 つのテストモジュール（basic_test, output_test）を宣言する
  - `git mv` で sakura_script_basic_test.rs → sakura_script/basic_test.rs, sakura_script_output_test.rs → sakura_script/output_test.rs を移動する
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する
  - `cargo test -p pasta_lua -- sakura_script` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 5.3 search/ を整備し 2 ファイルを移動・修正する
  - `tests/search/` ディレクトリを作成する
  - common を `#[path = "../common/mod.rs"] mod common;` で参照する main.rs を作成し、2 つのテストモジュール（scene_search_test, module_test）を宣言する
  - `git mv` で scene_search_test.rs → search/scene_search_test.rs, search_module_test.rs → search/module_test.rs を移動する
  - 旧 `mod common;` 宣言を `use crate::common;` に置換する
  - `cargo test -p pasta_lua -- search` で部分検証する
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 6. insta スナップショットを再生成して検証する
  - タスク 1〜5 の完了後に実行する
  - `cargo insta test -p pasta_lua --review` を実行しスナップショットを再生成する
  - 新スナップショット（tests/transpiler/snapshots/snapshot_test__*.snap）の内容が旧スナップショットと一致することを確認する
  - 旧スナップショットディレクトリ `tests/snapshots/` が空になれば削除する（残存ファイルがある場合は維持）
  - _Requirements: 2.4_

- [x] 7. pasta_lua の全テスト通過を確認する
  - タスク 6 の完了後に実行する
  - `cargo test -p pasta_lua --all-targets` で全テストが Pass することを確認する
  - `cargo clippy -p pasta_lua --all-targets` で警告がないことを確認する
  - `cargo build --workspace` でワークスペース全体のビルドが成功することを確認する
  - `git log --follow <moved-file>` で代表ファイルのリネーム検出が機能していることをスポットチェックする
  - _Requirements: 1.3, 2.5_

- [x] 8. src/ レビュー結果の記録と steering を更新する

- [x] 8.1 pasta_lua/src/ のモジュール配置を検証し結果を確定する
  - pasta_lua/src/ の全モジュール（transpiler.rs, context.rs, config.rs, error.rs, string_literalizer.rs, normalize.rs, code_gen/, runtime/, loader/, encoding/, logging/, sakura_script/, search/）を対象に、各モジュールが所属先の責務と合致しているか確認する
  - 設計の判定（適切 / 許容）に変化がないことを確認する
  - 問題が見つかった場合は design.md の C7 を更新する
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 8.2 steering/structure.md をリファクタリング後の実体に同期する
  - pasta_lua/tests/ のディレクトリツリーを実際の構造（7 サブディレクトリ + 3 フラット残留 + common/ + fixtures/ + lua_specs/）に更新する
  - テストサブモジュール化方針（`tests/<category>/main.rs` + `#[path = "../common/mod.rs"] mod common;` パターン、3 ファイル未満は統合またはフラット残留）を追記する
  - src/ 内テスト配置方針（`#[cfg(test)] #[path]` パターン、private フィールドアクセスが必要な場合のみ src/ 内に配置）を追記する
  - `lua_unittest_runner.rs` の命名例外（テストランナーは `_runner.rs` サフィックスを使用する）を命名規則に追記する
  - _Requirements: 3.1, 3.2, 3.3, 5.1, 5.2, 6.1, 6.2, 6.3_
