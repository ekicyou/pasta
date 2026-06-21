//! pasta-actor-feasibility 使い捨て検証ハーネス（PoC）の SHIORI 側。
//!
//! `actor-poc` feature 有効時のみコンパイルされる隔離モジュール。
//! SSP スレッド側の FFI 反転ハーネス（pest method 判定→GET block-on-reply／
//! NOTIFY 即 204 marshaling）を担う。
//!
//! 後続タスク（2.x 以降）が `ffi_marshal` 等のサブモジュールをここに追加する。
//! 出荷経路（`shiori::request`）は改変せず、request 文字列を独自に再パースする
//! 方針（design.md Q3）。feature 無効時はコンパイル単位に現れない。
