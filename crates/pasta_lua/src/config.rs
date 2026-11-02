//! Transpiler configuration for Lua code generation.
//!
//! This module provides configuration options for the transpilation process.

use pasta_core::SceneAliasTable;

/// Transpiler configuration.
#[derive(Debug, Clone, Default)]
pub struct TranspilerConfig {
    /// グローバルシーン名の別名表（既定は空の表で、宣言名を置き換えない）。
    pub scene_aliases: SceneAliasTable,
}

impl TranspilerConfig {
    /// グローバルシーンの宣言名を置き換える別名表を設定する。
    pub fn with_scene_aliases(mut self, aliases: SceneAliasTable) -> Self {
        self.scene_aliases = aliases;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scene_aliases_is_empty() {
        assert!(TranspilerConfig::default().scene_aliases.is_empty());
        let config =
            TranspilerConfig::default().with_scene_aliases(SceneAliasTable::builtin_default());
        assert_eq!(config.scene_aliases, SceneAliasTable::builtin_default());
    }
}
