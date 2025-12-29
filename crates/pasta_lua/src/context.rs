//! Transpile context for Lua code generation.
//!
//! This module provides context management for the transpilation process.

use pasta_core::parser::{GlobalSceneScope, KeyWords, LocalSceneScope};
use pasta_core::registry::{SceneRegistry, WordDefRegistry};
use std::collections::HashMap;

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

    /// Register a global scene (Task 3.1).
    ///
    /// Registers the scene in SceneRegistry and returns (id, counter).
    pub fn register_global_scene(&mut self, scene: &GlobalSceneScope) -> (i64, usize) {
        let attrs: HashMap<String, String> = scene
            .attrs
            .iter()
            .map(|a| (a.key.clone(), a.value.to_string()))
            .collect();
        self.scene_registry.register_global(&scene.name, attrs)
    }

    /// Register a local scene (Task 3.1).
    ///
    /// Registers the local scene under the parent global scene.
    /// Returns the assigned scene ID.
    pub fn register_local_scene(
        &mut self,
        local_scene: &LocalSceneScope,
        parent_name: &str,
        parent_counter: usize,
        local_index: usize,
    ) -> i64 {
        let attrs: HashMap<String, String> = local_scene
            .attrs
            .iter()
            .map(|a| (a.key.clone(), a.value.to_string()))
            .collect();

        // Use scene name if present, otherwise use "__start__"
        let name = local_scene.name.as_deref().unwrap_or("__start__");

        self.scene_registry
            .register_local(name, parent_name, parent_counter, local_index, attrs)
    }

    /// Register global word definitions from file-level scope (Task 3.2).
    pub fn register_global_words(&mut self, words: &[KeyWords]) {
        for kw in words {
            self.word_registry
                .register_global(&kw.name, kw.words.clone());
        }
    }

    /// Register local word definitions within a scene (Task 3.2).
    pub fn register_local_words(&mut self, words: &[KeyWords], module_name: &str) {
        for kw in words {
            self.word_registry
                .register_local(module_name, &kw.name, kw.words.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_core::parser::Span;

    fn create_test_scene(name: &str) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![],
            code_blocks: vec![],
            local_scenes: vec![],
            span: Span::default(),
        }
    }

    fn create_test_local_scene(name: &str) -> LocalSceneScope {
        LocalSceneScope::named(name.to_string())
    }

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

    #[test]
    fn test_register_global_scene() {
        let mut ctx = TranspileContext::new();
        let scene = create_test_scene("メイン");

        let (id, counter) = ctx.register_global_scene(&scene);
        assert_eq!(id, 1);
        assert_eq!(counter, 1);

        let scenes = ctx.scene_registry.all_scenes();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].name, "メイン");
    }

    #[test]
    fn test_register_local_scene() {
        let mut ctx = TranspileContext::new();

        // First register parent
        let parent = create_test_scene("メイン");
        let (_, parent_counter) = ctx.register_global_scene(&parent);

        // Then register local scene
        let local = create_test_local_scene("自己紹介");
        let id = ctx.register_local_scene(&local, "メイン", parent_counter, 1);

        assert_eq!(id, 2);
        let scenes = ctx.scene_registry.all_scenes();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn test_register_global_words() {
        let mut ctx = TranspileContext::new();
        let words = vec![KeyWords {
            name: "挨拶".to_string(),
            words: vec!["こんにちは".to_string(), "やあ".to_string()],
            span: Span::default(),
        }];

        ctx.register_global_words(&words);

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "挨拶");
    }

    #[test]
    fn test_register_local_words() {
        let mut ctx = TranspileContext::new();
        let words = vec![KeyWords {
            name: "場所".to_string(),
            words: vec!["東京".to_string(), "大阪".to_string()],
            span: Span::default(),
        }];

        ctx.register_local_words(&words, "メイン_1");

        let entries = ctx.word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].key.contains(":メイン_1:場所"));
    }
}
