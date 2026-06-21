//! pasta-actor-feasibility 使い捨て検証ハーネス（PoC）の SHIORI 側。
//!
//! `actor-poc` feature 有効時のみコンパイルされる隔離モジュール。
//! SSP スレッド側の FFI 反転ハーネス（pest method 判定→GET block-on-reply／
//! NOTIFY 即 204 marshaling）を担う。
//!
//! 出荷経路（`shiori::request`）は改変せず、request 文字列を独自に再パースする
//! 方針（design.md Q3）。feature 無効時はコンパイル単位に現れない。

/// SSP スレッド側 FFI 反転ハーネス（pest method 再パース→GET block-on-reply／
/// NOTIFY 即 204・R2.1/R2.2/R2.4）。
pub mod ffi_marshal;
