//! VSCode Lua デバッグ実現可能性の検証ハーネス本体（feature `lua-debug-poc`）。
//!
//! このファイル全体は `tests/runtime/main.rs` の
//! `#[cfg(feature = "lua-debug-poc")] mod lua_debug_poc_test;` によって
//! feature 有効時のみコンパイルされる。production クレート・既定テストへの
//! 影響はない（Requirement 5.1）。
//!
//! 各サブモジュールは段階的 GO 判定（R1〜R6）を分散検証する。`#[test]` は
//! 各サブモジュールへ co-locate する（後続タスク 1.2 以降で実装）。
//!
//! モジュールパス注: 非 mod.rs ファイルが宣言する子モジュールは
//! `lua_debug_poc_test/<name>.rs` に解決される。親モジュールが既に
//! feature-gate 済みのため、以下の `mod` 行には `#[cfg]` を付けない。

mod harness_types;
mod hook_probe;
mod pause_gate;
mod frame_inspector;
mod session;
mod transport_loop;
mod verdict;
