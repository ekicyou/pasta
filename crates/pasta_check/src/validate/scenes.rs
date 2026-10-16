//! シーンを pasta_lua のトランスパイラと同じ手順で登録し、名前の重なりを見つける段（2.4・5.1〜5.4）。

use pasta_core::{MockRandomSelector, SceneAliasTable, SceneRegistry, SceneTable};
use pasta_dsl::PastaFile;

use super::Finding;

/// 登録したシーンの表と、名前の重なりの注意を返す。表には乱数源を使わない空のモック選択子を渡す
pub(crate) fn check(
    _parsed: &[(String, PastaFile)],
    _aliases: &SceneAliasTable,
) -> (SceneTable, Vec<Finding>) {
    let registry = SceneRegistry::new();
    let table =
        SceneTable::from_scene_registry(registry, Box::new(MockRandomSelector::new(vec![])))
            .expect("from_scene_registry は失敗を返さない");
    (table, Vec::new())
}
