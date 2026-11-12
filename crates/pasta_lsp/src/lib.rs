//! Pasta DSL - VSCode 拡張に埋め込む WebAssembly の解析器.
//!
//! `*.pasta` を解析して、次のものを返す:
//! - Semantic token-based syntax highlighting
//! - Parse error diagnostics
//! - シーンの並び（グローバルシーン ＞ ローカルシーン。拡張のアウトライン・▶ と `pasta-recipe` が同じものを受ける）

pub mod analysis;
pub mod transport;
