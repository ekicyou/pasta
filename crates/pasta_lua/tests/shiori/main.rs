// tests/shiori/main.rs
//
// SHIORI関連の統合テストをグルーピングするエントリーポイント。
// common ヘルパーは #[path] で tests/common/ を参照する。

#[path = "../common/mod.rs"]
mod common;

mod event_dispatch_test;
mod event_handler_test;
mod res_test;
mod virtual_event_config_resolution_test;
mod virtual_event_dispatch_test;
mod virtual_event_status_skip_test;
