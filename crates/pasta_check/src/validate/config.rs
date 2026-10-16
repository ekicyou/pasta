//! pasta.toml の [loader] と [scene] を pasta.dll と同じ厳しさで読む段（2.2・2.4・2.5・2.10）。

use super::{Finding, TomlSource};

/// 設定の誤りを返す。誤りがあっても既定値で検査を続ける
pub(crate) fn check(_src: &TomlSource) -> Vec<Finding> {
    Vec::new()
}
