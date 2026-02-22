// tests/search/main.rs
//
// 検索関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod module_test;
mod scene_search_test;
