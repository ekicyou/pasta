//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

pub mod actor;
pub mod error;
pub mod lua_request;
mod shiori;
mod util;
mod v3;

#[cfg(windows)]
mod windows;

// 版 3 の口は全部の OS で書き出す。
pub use v3::shiori_get_api;

pub use shiori::PastaShiori;

// rlib 経由の統合テストが C ABI の出荷シンボルを駆動できるよう、Rust パスでも再エクスポートする。
#[cfg(windows)]
pub use windows::{load, loadu, request, unload};
