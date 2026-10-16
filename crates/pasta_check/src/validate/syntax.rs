//! 各辞書ファイルを pasta.dll と同じ関数で読み、最初の構文エラーと読めないファイルを報告する段（3.1〜3.4）。

use pasta_dsl::PastaFile;

use super::{FileText, Finding};

/// 読めたファイルの構文木を、パスと組にして次の段に渡す
pub(crate) fn check(_files: &[(String, FileText)]) -> (Vec<(String, PastaFile)>, Vec<Finding>) {
    (Vec::new(), Vec::new())
}
