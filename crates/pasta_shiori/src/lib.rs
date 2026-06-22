//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

#[cfg(feature = "actor-poc")]
pub mod actor_poc;
// 本番アクターランタイム（task 3.x で出荷経路へ配線）。mailbox は flume のみ依存ゆえ
// default ビルドに同居する。FFI 入口への配線は task 5.1 のため、それまでは pub API が
// クレート内利用されず dead_code 警告対象になりうる。pub 維持＋ユニットテストで実証する
// 方針（design.md「Placement / gating」）に従い、配線完了まではモジュールごとに
// dead_code を許容する。
#[allow(dead_code)]
pub mod actor;
pub mod error;
pub mod lua_request;
mod shiori;
mod util;

#[cfg(windows)]
mod windows;

// Re-export for integration tests
pub use shiori::{PastaShiori, Shiori};
