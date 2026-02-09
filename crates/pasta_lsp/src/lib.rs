//! Pasta DSL - Language Server Protocol implementation.
//!
//! This crate provides an LSP server for `*.pasta` files, offering:
//! - Semantic token-based syntax highlighting
//! - Parse error diagnostics
//! - Document synchronization
//!
//! Designed to be built as a WebAssembly module for embedding in VSCode extensions.

pub mod analysis;
pub mod document;
pub mod error;
pub mod server;
pub mod transport;

pub use error::LangServerError;
pub use server::PastaLangServer;
