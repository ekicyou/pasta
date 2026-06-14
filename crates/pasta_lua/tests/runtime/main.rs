// tests/runtime/main.rs
//
// ランタイム関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod debug_break_coalesce_fixture_test;
mod debug_integration_test;
mod debug_integration_zero_cost_test;
mod encoding_test;
mod finalize_scene_test;
mod local_scene_call_test;
mod persistence_integration_test;
mod runtime_api_test;
mod runtime_toggle_e2e_basic_test;
mod runtime_toggle_e2e_common;
mod runtime_toggle_e2e_initial_mode_test;
mod runtime_toggle_e2e_step_test;
mod scene_test;
mod source_map_handoff_test;
mod stdlib_modules_test;
mod stdlib_regex_test;
mod syntax_test;
mod unit_test;
