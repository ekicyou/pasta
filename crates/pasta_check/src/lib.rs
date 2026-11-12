//! pasta_check ライブラリ: ゴーストの辞書の検査の核。
//!
//! `fs` の機能を除き、ファイルシステムに触れず、WASM（VSCode 拡張）とネイティブ（`pasta_check` バイナリ）の両方から使う。

#[cfg(feature = "fs")]
pub mod ghost_files;
pub mod validate;
