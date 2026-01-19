//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

mod error;
mod shiori;
mod util;

#[cfg(windows)]
mod windows;

// Re-export for integration tests
pub use shiori::{PastaShiori, Shiori};
