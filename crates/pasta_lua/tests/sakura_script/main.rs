// tests/sakura_script/main.rs
//
// SakuraScript関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod basic_test;
mod budoux_test;
mod output_test;
