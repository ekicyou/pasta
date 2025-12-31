//! Transpiler2 module for converting Parser2 AST to Rune source code.
//!
//! This module provides a new transpiler that works with the parser2 AST types.
//! It is completely independent from the legacy transpiler module, using the
//! shared registry module for scene and word management.
//!
//! # Architecture
//!
//! - **Transpiler2**: Main entry point for AST-to-Rune conversion
//! - **TranspileContext2**: Context for tracking scope and attributes
//! - **CodeGenerator**: Generates Rune code from AST nodes
//! - **TranspileError**: Error type for transpilation failures
//!
//! # Two-Pass Strategy
//!
//! 1. **Pass 1 (Registration + Generation)**: Process FileItems sequentially,
//!    registering scenes/words and generating module code
//! 2. **Pass 2 (Selector Generation)**: Generate scene_selector and pasta wrapper
//!
//! # Example
//!
//! ```ignore
//! use pasta_core::parser;
//! use pasta_rune::transpiler::Transpiler;
//! use pasta_core::registry::{SceneRegistry, WordDefRegistry};
//!
//! let source = "＊会話\nさくら：こんにちは";
//! let file = parser::parse_str(source, "test.pasta").unwrap();
//!
//! let mut scene_registry = SceneRegistry::new();
//! let mut word_registry = WordDefRegistry::new();
//! let mut output = Vec::new();
//!
//! Transpiler::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output).unwrap();
//! Transpiler::transpile_pass2(&scene_registry, &mut output).unwrap();
//! ```

mod code_generator;
mod context;
mod error;

pub use code_generator::CodeGenerator;
pub use context::TranspileContext2;
pub use error::TranspileError;

use pasta_core::parser::{FileItem, GlobalSceneScope, PastaFile};
use pasta_core::registry::{SceneRegistry, WordDefRegistry};

/// Transpiler2 for parser2 AST to Rune code conversion.
///
/// This struct provides static methods for transpiling parser2 AST
/// to Rune source code using a two-pass strategy.
pub struct Transpiler2;

impl Transpiler2 {
    /// Pass 1: Register scenes/words and generate module code.
    ///
    /// This performs the first pass of the two-pass transpilation:
    /// - Registers all scenes in the SceneRegistry
    /// - Registers all word definitions in the WordDefRegistry
    /// - Generates Rune modules for each global scene
    ///
    /// # Arguments
    ///
    /// * `file` - The parsed parser2 AST
    /// * `scene_registry` - The scene registry for tracking scenes
    /// * `word_registry` - The word definition registry
    /// * `writer` - Output destination implementing Write trait
    #[allow(unused_variables)]
    pub fn transpile_pass1<W: std::io::Write>(
        file: &PastaFile,
        scene_registry: &mut SceneRegistry,
        word_registry: &mut WordDefRegistry,
        writer: &mut W,
    ) -> Result<(), TranspileError> {
        let mut context = TranspileContext2::new();

        for item in &file.items {
            match item {
                FileItem::FileAttr(attr) => {
                    context.accumulate_file_attr(attr);
                }
                FileItem::GlobalWord(word) => {
                    let values: Vec<String> = word.words.iter().map(|w| w.to_string()).collect();
                    word_registry.register_global(&word.name, values);
                }
                FileItem::GlobalSceneScope(scene) => {
                    Self::process_global_scene(
                        scene,
                        &mut context,
                        scene_registry,
                        word_registry,
                        writer,
                    )?;
                }
                FileItem::ActorScope(_actor) => {
                    // TODO: アクター定義の処理を実装
                    // 現時点ではスキップ（将来的にアクター情報をレジストリに登録）
                }
            }
        }

        Ok(())
    }

    /// Pass 2: Generate scene_selector and pasta wrapper.
    ///
    /// This generates the `mod __pasta_trans2__` with scene_selector()
    /// and `mod pasta` with call() wrapper.
    #[allow(unused_variables)]
    pub fn transpile_pass2<W: std::io::Write>(
        scene_registry: &SceneRegistry,
        writer: &mut W,
    ) -> Result<(), TranspileError> {
        // Generate __pasta_trans2__ module with scene_selector
        // Note: scene_selector must be a pure function (not a generator) because it returns a function pointer
        writeln!(writer)?;
        writeln!(writer, "pub mod __pasta_trans2__ {{")?;
        writeln!(writer, "    use pasta_stdlib::*;")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "    pub fn scene_selector(scene, module_name, filters) {{"
        )?;
        writeln!(
            writer,
            "        let id = pasta_stdlib::select_scene_to_id(scene, module_name, filters);"
        )?;
        writeln!(writer, "        match id {{")?;
        writeln!(writer, "            Ok(id) => match id {{")?;

        // Generate match arms for each registered scene
        for scene_info in scene_registry.all_scenes() {
            // Use fn_name directly to get the correct function path
            // fn_name format: "モジュール名::関数名" (e.g., "会話_1::__start__" or "会話_1::返答_1")
            writeln!(
                writer,
                "                {} => crate::{},",
                scene_info.id, scene_info.fn_name
            )?;
        }

        // Default case for unknown scene IDs
        writeln!(
            writer,
            "                _ => |_ctx, _args| {{ yield Error(`シーンID ${{id}} が見つかりませんでした。`); }},"
        )?;
        writeln!(writer, "            }},")?;
        writeln!(
            writer,
            "            Err(e) => |_ctx, _args| {{ yield Error(`シーン解決エラー: ${{e}}`); }},"
        )?;
        writeln!(writer, "        }}")?;
        writeln!(writer, "    }}")?;
        writeln!(writer, "}}")?;

        // Generate pasta module with call wrapper
        writeln!(writer)?;
        writeln!(writer, "pub mod pasta {{")?;
        writeln!(
            writer,
            "    pub fn call(ctx, scene, module_name, filters, args) {{"
        )?;
        writeln!(
            writer,
            "        let func = crate::__pasta_trans2__::scene_selector(scene, module_name, filters);"
        )?;
        writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")?;
        writeln!(writer, "    }}")?;
        writeln!(writer, "}}")?;

        Ok(())
    }

    /// Helper: Transpile a single file to a string (for testing).
    pub fn transpile_to_string(file: &PastaFile) -> Result<String, TranspileError> {
        let mut scene_registry = SceneRegistry::new();
        let mut word_registry = WordDefRegistry::new();
        let mut output = Vec::new();

        Self::transpile_pass1(file, &mut scene_registry, &mut word_registry, &mut output)?;
        Self::transpile_pass2(&scene_registry, &mut output)?;

        String::from_utf8(output).map_err(|e| TranspileError::internal(e.to_string()))
    }

    /// Process a global scene scope.
    fn process_global_scene<W: std::io::Write>(
        scene: &GlobalSceneScope,
        context: &mut TranspileContext2,
        scene_registry: &mut SceneRegistry,
        word_registry: &mut WordDefRegistry,
        writer: &mut W,
    ) -> Result<(), TranspileError> {
        use std::collections::HashMap;

        // Merge file-level attrs with scene attrs
        let merged_attrs = context.merge_attrs(&scene.attrs);

        // Convert AttrValue to String for SceneRegistry
        let string_attrs: HashMap<String, String> = merged_attrs
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect();

        // Register the global scene
        let (_scene_id, scene_counter) = scene_registry.register_global(&scene.name, string_attrs);

        // Update context with current module
        let sanitized_name = SceneRegistry::sanitize_name(&scene.name);
        let module_name = format!("{}_{}", sanitized_name, scene_counter);
        context.set_current_module(module_name.clone());

        // Register scene-level words
        for word in &scene.words {
            let values: Vec<String> = word.words.iter().map(|w| w.to_string()).collect();
            word_registry.register_local(&context.current_module(), &word.name, values);
        }

        // Register local scenes with index matching CodeGenerator
        for (idx, local_scene) in scene.local_scenes.iter().enumerate() {
            if let Some(ref local_name) = local_scene.name {
                let local_attrs: HashMap<String, String> = local_scene
                    .attrs
                    .iter()
                    .map(|a| (a.key.clone(), a.value.to_string()))
                    .collect();
                // local_index is 1-based to match CodeGenerator.generate_local_scene
                let local_index = idx + 1;
                scene_registry.register_local(
                    local_name,
                    &scene.name,
                    scene_counter,
                    local_index,
                    local_attrs,
                );
            }
        }

        // Generate module code for this scene
        let mut generator = CodeGenerator::new(writer);
        generator.generate_global_scene(scene, scene_counter, context)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_transpile_empty_file() {
        let file = PastaFile::new(std::path::PathBuf::from("test.pasta"));
        let result = Transpiler2::transpile_to_string(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_pass1_registers_scene() {
        // action_line requires: pad + id + colon + actions
        // local_start_scene_scope requires at least one local_scene_item
        let source = "＊会話\n  sakura：hello\n";
        let file = parser::parse_str(source, "test.pasta").unwrap();

        let mut scene_registry = SceneRegistry::new();
        let mut word_registry = WordDefRegistry::new();
        let mut output = Vec::new();

        let result = Transpiler2::transpile_pass1(
            &file,
            &mut scene_registry,
            &mut word_registry,
            &mut output,
        );
        assert!(result.is_ok());

        // Verify scene was registered
        let scenes = scene_registry.all_scenes();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].name, "会話");
    }

    #[test]
    fn test_transpile_pass1_registers_words() {
        // file_word_line: word_marker ~ key_words ~ eol
        let source = "＠挨拶：こんにちは、おはよう\n＊会話\n  sakura：hi\n";
        let file = parser::parse_str(source, "test.pasta").unwrap();

        let mut scene_registry = SceneRegistry::new();
        let mut word_registry = WordDefRegistry::new();
        let mut output = Vec::new();

        let result = Transpiler2::transpile_pass1(
            &file,
            &mut scene_registry,
            &mut word_registry,
            &mut output,
        );
        assert!(result.is_ok());

        // Verify word was registered
        let entries = word_registry.all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "挨拶");
        assert_eq!(entries[0].values, vec!["こんにちは", "おはよう"]);
    }

    #[test]
    fn test_transpile_pass2_generates_selector() {
        let source = "＊会話\n  sakura：hello\n";
        let file = parser::parse_str(source, "test.pasta").unwrap();

        let mut scene_registry = SceneRegistry::new();
        let mut word_registry = WordDefRegistry::new();
        let mut output = Vec::new();

        // Pass 1
        Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
            .unwrap();

        // Pass 2
        Transpiler2::transpile_pass2(&scene_registry, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();

        // Verify generated code contains scene_selector module
        assert!(
            result.contains("mod __pasta_trans2__"),
            "Missing __pasta_trans2__ module"
        );
        assert!(
            result.contains("pub fn scene_selector"),
            "Missing scene_selector function"
        );
        assert!(result.contains("mod pasta"), "Missing pasta module");
        assert!(result.contains("pub fn call"), "Missing call function");
    }

    #[test]
    fn test_transpile_pass2_scene_match_cases() {
        let source = "＊挨拶\n  sakura：hello\n＊会話\n  kero：yo\n";
        let file = parser::parse_str(source, "test.pasta").unwrap();

        let mut scene_registry = SceneRegistry::new();
        let mut word_registry = WordDefRegistry::new();
        let mut output = Vec::new();

        // Pass 1 + Pass 2
        Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
            .unwrap();
        Transpiler2::transpile_pass2(&scene_registry, &mut output).unwrap();

        let result = String::from_utf8(output).unwrap();

        // Check that match cases are generated for each scene
        assert!(result.contains("挨拶_1"), "Missing 挨拶_1 module reference");
        assert!(result.contains("会話_1"), "Missing 会話_1 module reference");
    }
}
