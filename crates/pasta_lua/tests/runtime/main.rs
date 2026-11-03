// tests/runtime/main.rs
//
// ランタイム関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod encoding_test;
mod finalize_scene_test;
mod literal_fixes_test;
mod local_scene_call_test;
mod module_searcher_test;
mod persistence_integration_test;
mod runtime_api_test;
mod scene_identity_format_test;
mod scene_test;
mod source_map_handoff_test;
mod syntax_test;
mod toml_module_test;
mod unit_test;
