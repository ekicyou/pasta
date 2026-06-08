// tests/runtime/main.rs
//
// ランタイム関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod debug_integration_test;
mod encoding_test;
mod finalize_scene_test;
mod local_scene_call_test;
mod persistence_integration_test;
mod scene_test;
mod source_map_handoff_test;
mod stdlib_modules_test;
mod stdlib_regex_test;
mod syntax_test;
mod unit_test;
