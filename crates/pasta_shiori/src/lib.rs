//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

// 本番アクターランタイム。task 5.1 で FFI 入口（windows.rs）が `actor::lifecycle` の
// `static LEGACY` 所有モデルへ配線され、出荷経路となった。mailbox/thread/marshaling/
// teardown/lifecycle が既定（no-feature）ビルドでコンパイル・リンクされる。
pub mod actor;
pub mod error;
pub mod lua_request;
mod shiori;
mod util;
mod v3;

#[cfg(windows)]
mod windows;

// 版 3 の口（全部の OS で書き出す）。旧来の入口と同じく、rlib の試験から到達させるために再エクスポートする。
pub use v3::shiori_get_api;

// Re-export for integration tests
pub use shiori::{PastaShiori, Shiori};

// 出荷 SHIORI extern FFI 入口（`#[unsafe(no_mangle)] extern "C"`）を Rust パスでも到達可能にする。
// これらは SSP 等のホストが C ABI で呼ぶ出荷シンボルそのものであり、cdylib エクスポートが
// 主用途。Rust 再エクスポートは追加的（非破壊）で、機能レベル統合 E2E（task 7.1）が
// 出荷シンボルを通じて load→request→unload を駆動・検証できるようにする
// （rlib 経由の統合テストでは no_mangle シンボルがリンク到達するために Rust から参照が必要）。
#[cfg(windows)]
pub use windows::{load, loadu, request, unload};
