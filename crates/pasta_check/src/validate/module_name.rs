//! モジュール名を作り、辞書ファイル・辞書モジュールの重なりを分類する段（4.1〜4.4）。

use super::Finding;
use super::discovery::Discovered;

pub(crate) fn check(_discovered: &Discovered) -> Vec<Finding> {
    Vec::new()
}
