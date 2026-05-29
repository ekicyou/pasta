// tests/transpiler/main.rs
//
// トランスパイラ関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod actor_word_dictionary_test;
mod basic_test;
mod choice_transpile_test;
mod choice_timeout_transpile_test;
mod code_generator_test;
mod comparison_test;
mod cue_command_passthrough_test;
mod dynamic_call_test;
mod fallback_search_integration_test;
mod scene_test;
mod snapshot_test;
