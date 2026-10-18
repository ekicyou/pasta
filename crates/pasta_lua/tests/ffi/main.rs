// tests/ffi/main.rs
//
// `ffi.load` の置き換えの統合テストのエントリーポイント（lua-cmodule-robustness）。
// DLL のモジュールはプロセス全体で共有されるので、ほかのテストのバイナリから切り離す。
// common ヘルパー（`PASTA_DEBUG` を中和する #[ctor] を含む）は #[path] で tests/common/ を参照する。
#![cfg(windows)]

#[path = "../common/mod.rs"]
mod common;

mod collision_test;
mod compare_test;
mod config_test;
mod dll_fixture;
mod lifetime_test;
mod load_test;
