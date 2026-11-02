//! Pasta DSL - VSCode 拡張に埋め込む WebAssembly の解析器.
//!
//! `*.pasta` を解析して、次のものを返す:
//! - Semantic token-based syntax highlighting
//! - Parse error diagnostics

pub mod analysis;
pub mod transport;
