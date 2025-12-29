//! Transpile context for Lua code generation.
//!
//! This module provides context management for the transpilation process.

use pasta_core::registry::{SceneRegistry, WordDefRegistry};

/// Transpile context for sharing state during transpilation.
#[derive(Default)]
pub struct TranspileContext {
    /// Scene registry for global/local scene registration
    pub scene_registry: SceneRegistry,
    /// Word definition registry for global/local word registration
    pub word_registry: WordDefRegistry,
    /// Current module name being processed
    pub current_module: Option<String>,
}

impl TranspileContext {
    /// Create a new transpile context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current module name.
    pub fn set_current_module(&mut self, module: String) {
        self.current_module = Some(module);
    }

    /// Get the current module name.
    pub fn get_current_module(&self) -> Option<&str> {
        self.current_module.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = TranspileContext::new();
        assert!(ctx.current_module.is_none());
    }

    #[test]
    fn test_context_set_module() {
        let mut ctx = TranspileContext::new();
        ctx.set_current_module("メイン1".to_string());
        assert_eq!(ctx.get_current_module(), Some("メイン1"));
    }
}
