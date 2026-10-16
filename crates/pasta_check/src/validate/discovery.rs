//! pasta.dll と同じ辞書ファイルと辞書モジュールを選び出す段（2.1〜2.3・2.6〜2.8・11.8）。

use super::{Finding, GhostInput};

/// 辞書ファイル（.pasta）の相対パスを昇順で返し、集め方の注意・誤りを添える
pub(crate) fn discover(_input: &GhostInput) -> (Vec<String>, Vec<Finding>) {
    (Vec::new(), Vec::new())
}
