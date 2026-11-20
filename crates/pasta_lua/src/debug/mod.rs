//! 開発用の部品の入口（pasta_lua。SHIORI に依存しない）。
//!
//! - [`resolve_enabled`] は `PASTA_DEBUG` ＞ 保存変数 `デバッグ`／`debug` ＞ pasta.toml の `[debug] enabled` ＞ 無効 の順で「有効か」を決める。
//! - [`source_map`] はシーンマップ（シーンの宣言の記録と索引）。有効なときだけ作る。
//! - `playscene` は `OnPastaSceneAt` の位置の引き（`@pasta_debug.scene_at`）。

pub use crate::loader::DebugFileConfig;

pub(crate) mod playscene;

mod config;

// `.pasta` のシーンの宣言を控えるシーンマップの consumer 側モジュール。
// 本仕様 `pasta-source-map` で本番化（常時コンパイル）した（7.3）。
pub mod source_map;

pub use config::resolve_enabled;
