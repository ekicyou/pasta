// tests/loader/main.rs
//
// ローダー関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod cache_test;
mod config_actors_initialization_test;
mod config_test;
mod lifecycle_test;
mod lua_passthrough_test;
mod startup_test;
