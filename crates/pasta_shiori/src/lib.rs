//! pasta_shiori - SHIORI DLL interface for pasta script engine
//!
//! This crate provides the SHIORI protocol interface as a Windows DLL.

// Re-export dependencies for internal use
use pasta_core as _;
use pasta_lua as _;

pub mod error;

#[cfg(windows)]
pub mod windows;

pub mod util;
