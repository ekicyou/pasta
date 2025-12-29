//! Lua Transpiler for Pasta DSL.
//!
//! This module provides the main transpiler interface for converting
//! Pasta AST to Lua code.

use pasta_core::parser::{ActorScope, GlobalSceneScope};

use super::code_generator::LuaCodeGenerator;
use super::config::TranspilerConfig;
use super::context::TranspileContext;
use super::error::TranspileError;

use std::io::Write;

/// Lua transpiler for Pasta DSL.
///
/// Converts Pasta AST to Lua source code.
pub struct LuaTranspiler {
    /// Transpiler configuration
    config: TranspilerConfig,
}

impl Default for LuaTranspiler {
    fn default() -> Self {
        Self::new(TranspilerConfig::default())
    }
}

impl LuaTranspiler {
    /// Create a new transpiler with the given configuration.
    pub fn new(config: TranspilerConfig) -> Self {
        Self { config }
    }

    /// Create a transpiler with default configuration.
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Transpile Pasta AST to Lua code.
    ///
    /// # Arguments
    /// * `actors` - Actor definitions
    /// * `scenes` - Global scene definitions
    /// * `writer` - Output writer
    ///
    /// # Returns
    /// * `Ok(TranspileContext)` - Transpilation successful, context contains registries
    /// * `Err(TranspileError)` - Transpilation failed
    pub fn transpile<W: Write>(
        &self,
        actors: &[ActorScope],
        scenes: &[GlobalSceneScope],
        writer: &mut W,
    ) -> Result<TranspileContext, TranspileError> {
        let context = TranspileContext::new();
        let mut codegen = LuaCodeGenerator::new(writer);

        // Write header
        codegen.write_header()?;

        // Generate actor definitions
        for actor in actors {
            codegen.generate_actor(actor)?;
        }

        // Generate scene definitions
        for (idx, scene) in scenes.iter().enumerate() {
            codegen.generate_global_scene(scene, idx + 1, &context)?;
        }

        Ok(context)
    }

    /// Get the transpiler configuration.
    pub fn config(&self) -> &TranspilerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_core::parser::{KeyWords, LocalSceneScope, Span};

    fn create_simple_actor(name: &str) -> ActorScope {
        ActorScope {
            name: name.to_string(),
            attrs: vec![],
            words: vec![KeyWords {
                name: "通常".to_string(),
                words: vec!["\\s[0]".to_string()],
                span: Span::default(),
            }],
            var_sets: vec![],
            span: Span::default(),
        }
    }

    fn create_simple_scene(name: &str) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![],
            code_blocks: vec![],
            local_scenes: vec![LocalSceneScope::start()],
            span: Span::default(),
        }
    }

    #[test]
    fn test_transpiler_default() {
        let transpiler = LuaTranspiler::default();
        assert!(transpiler.config().comment_mode);
    }

    #[test]
    fn test_transpile_empty() {
        let transpiler = LuaTranspiler::default();
        let mut output = Vec::new();

        let result = transpiler.transpile(&[], &[], &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("local PASTA = require \"pasta.runtime\""));
    }

    #[test]
    fn test_transpile_actor() {
        let transpiler = LuaTranspiler::default();
        let actors = vec![create_simple_actor("さくら")];
        let mut output = Vec::new();

        let result = transpiler.transpile(&actors, &[], &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("PASTA:create_actor(\"さくら\")"));
        assert!(lua_code.contains("ACTOR.通常 = [=[\\s[0]]=]"));
    }

    #[test]
    fn test_transpile_scene() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![create_simple_scene("メイン")];
        let mut output = Vec::new();

        let result = transpiler.transpile(&[], &scenes, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("PASTA:create_scene(\"メイン1\")"));
        assert!(lua_code.contains("function SCENE.__start__(ctx, ...)"));
    }

    #[test]
    fn test_transpile_multiple_scenes() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![
            create_simple_scene("メイン"),
            create_simple_scene("会話分岐"),
        ];
        let mut output = Vec::new();

        let result = transpiler.transpile(&[], &scenes, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("PASTA:create_scene(\"メイン1\")"));
        assert!(lua_code.contains("PASTA:create_scene(\"会話分岐2\")"));
    }
}
