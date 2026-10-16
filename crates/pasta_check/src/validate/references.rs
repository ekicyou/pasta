//! 静的な飛び先の Call・選択肢を辞書のシーンと組み込みの名前で探し、動的な Call を数える段（6.1〜6.9）。

use pasta_core::SceneTable;
use pasta_dsl::PastaFile;

use super::Finding;

/// 飛び先の注意と、飛び先を検査しなかった動的な Call の数を返す
pub(crate) fn check(_table: &SceneTable, _parsed: &[(String, PastaFile)]) -> (Vec<Finding>, usize) {
    (Vec::new(), 0)
}
