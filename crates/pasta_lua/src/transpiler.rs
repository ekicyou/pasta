//! Lua Transpiler for Pasta DSL.
//!
//! This module provides the main transpiler interface for converting
//! Pasta AST to Lua code.

use pasta_core::parser::{FileItem, PastaFile};
use pasta_core::registry::SceneRegistry;

use super::code_generator::LuaCodeGenerator;
use super::config::TranspilerConfig;
use super::context::TranspileContext;
use super::error::TranspileError;
use super::normalize::normalize_output;

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

    /// Transpile PastaFile to Lua code (MAJOR-2).
    ///
    /// Processes FileItems in document order, accumulating file attributes
    /// and generating code for actors and scenes.
    ///
    /// # Arguments
    /// * `file` - The parsed PastaFile AST
    /// * `writer` - Output writer
    ///
    /// # Returns
    /// * `Ok(TranspileContext)` - Transpilation successful, context contains registries
    /// * `Err(TranspileError)` - Transpilation failed
    ///
    /// # Post-processing
    /// - Output is normalized: trailing blank lines removed, ends with exactly one newline
    pub fn transpile<W: Write>(
        &self,
        file: &PastaFile,
        writer: &mut W,
    ) -> Result<TranspileContext, TranspileError> {
        let mut context = TranspileContext::new();

        // Use intermediate buffer for code generation
        let mut intermediate_buffer: Vec<u8> = Vec::new();
        let mut codegen =
            LuaCodeGenerator::with_line_ending(&mut intermediate_buffer, self.config.line_ending);

        // Write header
        codegen.write_header()?;

        // Process FileItems in document order (MAJOR-2)
        for item in &file.items {
            match item {
                FileItem::FileAttr(attr) => {
                    // MAJOR-2.1: FileAttr累積（コード生成なし）
                    context.accumulate_file_attr(attr);
                }
                FileItem::GlobalWord(word) => {
                    // MAJOR-2.2: グローバル単語登録 (Rust側レジストリ + Lua出力)
                    // Register in Rust-side registry (for backward compatibility)
                    let values: Vec<String> = word.words.clone();
                    context.word_registry.register_global(&word.name, values);
                    // Generate Lua code for word definition (Requirement 2.1, Task 4.2)
                    codegen.generate_global_word(word)?;
                }
                FileItem::GlobalSceneScope(scene) => {
                    // MAJOR-2.3: シーン処理
                    // Register global scene in SceneRegistry
                    let (scene_id, counter) = context.register_global_scene(scene);

                    // Register scene-level word definitions in WordDefRegistry
                    let module_name =
                        format!("{}{}", SceneRegistry::sanitize_name(&scene.name), counter);
                    context.register_local_words(&scene.words, &module_name);

                    // Merge file attrs with scene attrs (MAJOR-1)
                    let merged_attrs = context.merge_attrs(&scene.attrs);

                    // Generate Lua code for the scene (MAJOR-3: file_attrs引数追加)
                    codegen.generate_global_scene(scene, counter, &context, &merged_attrs)?;

                    // Register local scenes
                    for (local_idx, local_scene) in scene.local_scenes.iter().enumerate() {
                        // Skip start scene (name is None) - already registered as part of global
                        if local_scene.name.is_some() {
                            context.register_local_scene(
                                local_scene,
                                &scene.name,
                                counter,
                                local_idx + 1, // 1-based index for named local scenes
                            );
                        }
                    }

                    // Store scene ID for potential future use
                    let _ = scene_id;
                }
                FileItem::ActorScope(actor) => {
                    // MAJOR-2.4: アクター処理（ファイル属性継承なし）
                    // Register actor word definitions in WordDefRegistry (Task 2.3)
                    for word_def in &actor.words {
                        let values: Vec<String> = word_def.words.clone();
                        context
                            .word_registry
                            .register_actor(&actor.name, &word_def.name, values);
                    }
                    codegen.generate_actor(actor)?;
                }
            }
        }

        // Convert intermediate buffer to UTF-8 string and normalize
        let raw_output = String::from_utf8(intermediate_buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let normalized_output = normalize_output(&raw_output);

        // Write normalized output to final writer
        writer.write_all(normalized_output.as_bytes())?;

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
    use pasta_core::parser::{ActorScope, GlobalSceneScope, KeyWords, LocalSceneScope, Span};
    use std::path::PathBuf;

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
            code_blocks: vec![],
            span: Span::default(),
        }
    }

    fn create_simple_scene(name: &str) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![],
            actors: vec![],
            code_blocks: vec![],
            local_scenes: vec![LocalSceneScope::start()],
            span: Span::default(),
        }
    }

    fn create_scene_with_words(
        name: &str,
        word_name: &str,
        word_values: Vec<&str>,
    ) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![KeyWords {
                name: word_name.to_string(),
                words: word_values.iter().map(|s| s.to_string()).collect(),
                span: Span::default(),
            }],
            actors: vec![],
            code_blocks: vec![],
            local_scenes: vec![LocalSceneScope::start()],
            span: Span::default(),
        }
    }

    fn create_scene_with_local(name: &str, local_name: &str) -> GlobalSceneScope {
        GlobalSceneScope {
            name: name.to_string(),
            is_continuation: false,
            attrs: vec![],
            words: vec![],
            actors: vec![],
            code_blocks: vec![],
            local_scenes: vec![
                LocalSceneScope::start(),
                LocalSceneScope::named(local_name.to_string()),
            ],
            span: Span::default(),
        }
    }

    /// Create a PastaFile with actors and scenes
    fn create_pasta_file(actors: Vec<ActorScope>, scenes: Vec<GlobalSceneScope>) -> PastaFile {
        let mut items: Vec<FileItem> = Vec::new();
        for actor in actors {
            items.push(FileItem::ActorScope(actor));
        }
        for scene in scenes {
            items.push(FileItem::GlobalSceneScope(scene));
        }
        PastaFile {
            path: PathBuf::from("test.pasta"),
            items,
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
        let file = create_pasta_file(vec![], vec![]);

        let result = transpiler.transpile(&file, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("local PASTA = require \"pasta\""));
    }

    #[test]
    fn test_transpile_actor() {
        let transpiler = LuaTranspiler::default();
        let actors = vec![create_simple_actor("さくら")];
        let file = create_pasta_file(actors, vec![]);
        let mut output = Vec::new();

        let result = transpiler.transpile(&file, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        assert!(lua_code.contains("PASTA.create_actor(\"さくら\")"));
        // Symmetric API: ACTOR:create_word(key):entry(...) (actor-word-dictionary)
        assert!(lua_code.contains("ACTOR:create_word(\"通常\"):entry([=[\\s[0]]=])"));
    }

    #[test]
    fn test_transpile_scene() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![create_simple_scene("メイン")];
        let file = create_pasta_file(vec![], scenes);
        let mut output = Vec::new();

        let result = transpiler.transpile(&file, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
        assert!(lua_code.contains("PASTA.create_scene(\"メイン\")"));
        assert!(lua_code.contains("function SCENE.__start__(act, ...)"));
    }

    #[test]
    fn test_transpile_multiple_scenes() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![
            create_simple_scene("メイン"),
            create_simple_scene("会話分岐"),
        ];
        let file = create_pasta_file(vec![], scenes);
        let mut output = Vec::new();

        let result = transpiler.transpile(&file, &mut output);
        assert!(result.is_ok());

        let lua_code = String::from_utf8(output).unwrap();
        // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
        // Both scenes use base name only - Lua will add counters at runtime
        assert!(lua_code.contains("PASTA.create_scene(\"メイン\")"));
        assert!(lua_code.contains("PASTA.create_scene(\"会話分岐\")"));
    }

    // Task 3.1: SceneRegistry integration tests
    #[test]
    fn test_transpile_registers_global_scene() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![create_simple_scene("メイン")];
        let file = create_pasta_file(vec![], scenes);
        let mut output = Vec::new();

        let context = transpiler.transpile(&file, &mut output).unwrap();

        let registered_scenes = context.scene_registry.all_scenes();
        assert_eq!(registered_scenes.len(), 1);
        assert_eq!(registered_scenes[0].name, "メイン");
        assert_eq!(registered_scenes[0].id, 1);
    }

    #[test]
    fn test_transpile_registers_local_scene() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![create_scene_with_local("メイン", "自己紹介")];
        let file = create_pasta_file(vec![], scenes);
        let mut output = Vec::new();

        let context = transpiler.transpile(&file, &mut output).unwrap();

        let registered_scenes = context.scene_registry.all_scenes();
        assert_eq!(registered_scenes.len(), 2); // global + local
        assert_eq!(registered_scenes[1].name, "自己紹介");
    }

    // Task 3.2: WordDefRegistry integration tests
    #[test]
    fn test_transpile_registers_local_words() {
        let transpiler = LuaTranspiler::default();
        let scenes = vec![create_scene_with_words(
            "メイン",
            "場所",
            vec!["東京", "大阪"],
        )];
        let file = create_pasta_file(vec![], scenes);
        let mut output = Vec::new();

        let context = transpiler.transpile(&file, &mut output).unwrap();

        let entries = context.word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].key.contains("場所"));
    }

    // MAJOR-2.2: GlobalWord processing test
    #[test]
    fn test_transpile_with_global_words() {
        let transpiler = LuaTranspiler::default();
        let global_words = KeyWords {
            name: "挨拶".to_string(),
            words: vec!["こんにちは".to_string(), "やあ".to_string()],
            span: Span::default(),
        };
        let scenes = vec![create_simple_scene("メイン")];

        // Create PastaFile with GlobalWord item
        let file = PastaFile {
            path: PathBuf::from("test.pasta"),
            items: vec![
                FileItem::GlobalWord(global_words),
                FileItem::GlobalSceneScope(scenes.into_iter().next().unwrap()),
            ],
            span: Span::default(),
        };
        let mut output = Vec::new();

        let context = transpiler.transpile(&file, &mut output).unwrap();

        let entries = context.word_registry.all_entries();
        // グローバル単語 + シーン内ローカル単語はないので1つ
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "挨拶");
    }

    // Task 2.3: ActorScope word registration test
    #[test]
    fn test_transpile_registers_actor_words() {
        let transpiler = LuaTranspiler::default();
        let actor = ActorScope {
            name: "さくら".to_string(),
            attrs: vec![],
            words: vec![
                KeyWords {
                    name: "通常".to_string(),
                    words: vec!["\\s[0]".to_string(), "\\s[1]".to_string()],
                    span: Span::default(),
                },
                KeyWords {
                    name: "照れ".to_string(),
                    words: vec!["\\s[2]".to_string()],
                    span: Span::default(),
                },
            ],
            var_sets: vec![],
            code_blocks: vec![],
            span: Span::default(),
        };
        let file = create_pasta_file(vec![actor], vec![]);
        let mut output = Vec::new();

        let context = transpiler.transpile(&file, &mut output).unwrap();

        let entries = context.word_registry.all_entries();
        assert_eq!(entries.len(), 2, "Expected 2 actor word entries");
        // Check key format: :__actor_{name}__:{word}
        assert_eq!(entries[0].key, ":__actor_さくら__:通常");
        assert_eq!(entries[1].key, ":__actor_さくら__:照れ");
        // Check values
        assert_eq!(entries[0].values, vec!["\\s[0]", "\\s[1]"]);
        assert_eq!(entries[1].values, vec!["\\s[2]"]);
    }
}
