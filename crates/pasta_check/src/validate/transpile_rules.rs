//! 構文としては読めるが pasta_lua のトランスパイルが失敗する 3 つの書き方を見つける段（3.5）。

use pasta_dsl::PastaFile;

use super::Finding;

pub(crate) fn check(_parsed: &[(String, PastaFile)]) -> Vec<Finding> {
    Vec::new()
}
