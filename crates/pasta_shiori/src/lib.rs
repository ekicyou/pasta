//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

#[cfg(feature = "actor-poc")]
pub mod actor_poc;
// 本番アクターランタイム。task 5.1 で FFI 入口（windows.rs）が `actor::lifecycle` の
// `static MAILBOX` 所有モデルへ配線され、出荷経路となった。mailbox/thread/marshaling/
// teardown/lifecycle が既定（no-feature）ビルドでコンパイル・リンクされる。
pub mod actor;
pub mod error;
pub mod lua_request;
mod shiori;
mod util;

#[cfg(windows)]
mod windows;

// Re-export for integration tests
pub use shiori::{PastaShiori, Shiori};
