//! Transpiler configuration for Lua code generation.
//!
//! This module provides configuration options for the transpilation process.

/// Transpiler configuration.
#[derive(Debug, Clone)]
pub struct TranspilerConfig {
    /// Enable comment mode (include Pasta source line references)
    pub comment_mode: bool,
}

impl Default for TranspilerConfig {
    fn default() -> Self {
        Self { comment_mode: true }
    }
}

impl TranspilerConfig {
    /// Create a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration with comment mode disabled.
    pub fn without_comments() -> Self {
        Self {
            comment_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TranspilerConfig::default();
        assert!(config.comment_mode);
    }

    #[test]
    fn test_without_comments() {
        let config = TranspilerConfig::without_comments();
        assert!(!config.comment_mode);
    }
}
