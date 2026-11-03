//! 開発用の部品の入口（pasta_lua。SHIORI に依存しない）。
//!
//! - [`DebugConfig`] は pasta.toml の `[debug]` と環境変数から「有効か」を決める。
//! - [`source_map`] はシーンマップ（シーンの宣言の記録と索引）。有効なときだけ作る。
//! - `playscene` は `OnPastaSceneAt` の位置の引き（`@pasta_debug.scene_at`）。

pub use crate::loader::{DebugFileConfig, default_debug_port};

pub(crate) mod playscene;

mod config;
mod source_mode;

// `.pasta`↔生成 `.lua` ソースマップの consumer 側モジュール。
// 本仕様 `pasta-source-map` で本番化（常時コンパイル）した（7.3）。
pub mod source_map;

pub use config::DebugConfig;
// `debug_mod_tests` (a descendant of `debug`) reaches the `pub(super)`
// `config::parse_env_bool` helper through this in-module re-export; previously the
// helper lived directly in this file. Test-only, so it stays gated.
#[cfg(test)]
use config::parse_env_bool;
pub use source_mode::SourceMode;

#[cfg(test)]
#[path = "debug_mod_tests.rs"]
mod debug_mod_tests;
