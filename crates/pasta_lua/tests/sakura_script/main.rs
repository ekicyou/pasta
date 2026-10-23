// tests/sakura_script/main.rs
//
// SakuraScript関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod backslash_break_test;
mod basic_test;
mod budoux_test;
mod conformance_test;
mod edge_case_test;
mod grapheme_test;
mod output_test;
