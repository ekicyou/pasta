//! pasta_check ライブラリ: ゴーストの辞書の検査の核。
//!
//! ファイルシステムに触れず、WASM（VSCode 拡張）とネイティブ（`pasta_check` バイナリ）の両方から使う。

pub mod validate;
