//! Scope-level code generation: actors, global scenes, local scenes.

use super::LuaCodeGenerator;
use crate::context::TranspileContext;
use crate::error::TranspileError;
use crate::string_literalizer::StringLiteralizer;
use pasta_core::registry::SceneRegistry;
use pasta_dsl::parser::{
    ActorScope, AttrValue, GlobalSceneScope, LocalSceneItem, LocalSceneScope, SceneActorItem,
};
use std::collections::HashMap;
use std::io::Write;

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Generate actor definition block (Requirement 3a, actor-word-dictionary).
    ///
    /// Generates:
    /// ```lua
    /// do
    ///     local ACTOR = PASTA.create_actor("アクター名")
    ///     ACTOR.通常 = { [=[\s[0]]=], [=[\s[100]]=] }
    ///     
    ///     function ACTOR.時刻(act)
    ///         -- Lua関数定義
    ///     end
    /// end
    /// ```
    pub fn generate_actor(&mut self, actor: &ActorScope) -> Result<(), TranspileError> {
        // do block for scope separation (Requirement 1)
        self.writeln("do")?;
        self.indent();

        // Create actor
        self.writeln(&format!(
            "local ACTOR = PASTA.create_actor(\"{}\")",
            actor.name
        ))?;

        // Generate word definitions (Requirement 2, actor-word-dictionary Task 3.1)
        // ACTOR:create_word() registers both in word.lua (L2 prefix search) and as actor attribute (L1 exact match)
        for word_def in &actor.words {
            if word_def.words.is_empty() {
                continue;
            }

            // Literalize all words in the array
            let literals: Result<Vec<String>, _> = word_def
                .words
                .iter()
                .map(|w| StringLiteralizer::literalize_with_span(w, &word_def.span))
                .collect();
            let literals = literals?;

            // Use symmetric API: ACTOR:create_word(key):entry(...)
            // This pattern matches SCENE:create_word(key):entry(...)
            let entry_args = literals.join(", ");
            for name in &word_def.names {
                self.writeln(&format!(
                    "ACTOR:create_word(\"{}\"):entry({})",
                    name, entry_args
                ))?;
            }
        }

        // Generate code blocks (Requirement 4.関数定義)
        for code_block in &actor.code_blocks {
            // Only expand Lua code blocks
            if code_block.language.as_deref() == Some("lua") {
                self.write_blank_line()?;
                // Write code content line by line, preserving indentation
                for line in code_block.content.lines() {
                    self.writeln(line)?;
                }
            }
        }

        self.dedent();
        self.writeln("end")?;
        self.write_blank_line()?;

        Ok(())
    }

    /// Generate global scene block (Requirement 3b, MAJOR-3).
    ///
    /// Generates:
    /// ```lua
    /// do
    ///     local SCENE = PASTA.create_scene("モジュール名_N")
    ///     
    ///     function SCENE.__start__(ctx, ...)
    ///         local args = { ... }
    ///         local act, save, var = PASTA.create_session(SCENE, ctx)
    ///         -- ...
    ///     end
    ///     
    ///     function SCENE.__シーン名_1__(ctx, ...)
    ///         -- ...
    ///     end
    /// end
    /// ```
    ///
    /// # Arguments
    /// * `scene` - The global scene scope
    /// * `scene_counter` - Scene counter for name uniqueness
    /// * `_context` - Transpile context (currently unused)
    /// * `_file_attrs` - Merged file+scene attributes (MAJOR-3, currently unused for future extension)
    #[allow(unused_variables)]
    pub fn generate_global_scene(
        &mut self,
        scene: &GlobalSceneScope,
        _scene_counter: usize,
        _context: &TranspileContext,
        _file_attrs: &HashMap<String, AttrValue>,
    ) -> Result<(), TranspileError> {
        let sanitized_name = SceneRegistry::sanitize_name(&scene.name);
        // Use base name only - counter is assigned by Lua runtime (Requirement 8.5)
        let base_name = sanitized_name;

        // do block for scope separation (Requirement 1)
        self.writeln("do")?;
        self.indent();

        // Create scene with base name - Lua side assigns counter (Requirement 8.2, 8.5)
        self.writeln(&format!(
            "local SCENE = PASTA.create_scene(\"{}\")",
            base_name
        ))?;
        self.write_blank_line()?;

        // Generate scene-level word definitions (Requirement 2.2, Task 4.3)
        // These are registered under the current global scene name
        for word in &scene.words {
            self.generate_local_word(word)?;
        }
        if !scene.words.is_empty() {
            self.write_blank_line()?;
        }

        // Generate local scenes with per-name counters
        // Same-name scenes get incrementing numbers (_1, _2, ...)
        let mut name_counters: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for local_scene in &scene.local_scenes {
            let counter = if let Some(ref name) = local_scene.name {
                let count = name_counters.entry(name.clone()).or_insert(0);
                *count += 1;
                *count
            } else {
                0 // start scene doesn't use counter
            };
            self.generate_local_scene(local_scene, counter, &scene.actors)?;
        }

        // Generate code blocks at module level (after all local scene functions)
        // First: global scene level code blocks
        for code_block in &scene.code_blocks {
            self.generate_code_block(code_block)?;
        }
        // Second: code blocks from local scenes (these are stored in local scenes but should
        // appear at the global scene level, after all function definitions)
        for local_scene in &scene.local_scenes {
            for code_block in &local_scene.code_blocks {
                self.generate_code_block(code_block)?;
            }
        }

        self.dedent();
        self.writeln("end")?;
        self.write_blank_line()?;

        Ok(())
    }

    /// Generate local scene function (Requirement 3c).
    ///
    /// Generates:
    /// ```lua
    /// function SCENE.__シーン名_N__(ctx, ...)
    ///     local args = { ... }
    ///     local act, save, var = PASTA.create_session(SCENE, ctx)
    ///     -- items...
    /// end
    /// ```
    ///
    /// The `counter` parameter is the per-name counter (1, 2, 3... for same-name scenes).
    /// For start scenes (name is None), counter is ignored.
    ///
    /// Note: Code blocks associated with local scenes are NOT generated here.
    /// They are generated at the global scene level by generate_global_scene.
    pub fn generate_local_scene(
        &mut self,
        scene: &LocalSceneScope,
        counter: usize,
        actors: &[SceneActorItem],
    ) -> Result<(), TranspileError> {
        let fn_name = if let Some(ref name) = scene.name {
            let sanitized = SceneRegistry::sanitize_name(name);
            format!("{}_{}", sanitized, counter)
        } else {
            "__start__".to_string()
        };

        self.writeln(&format!("function SCENE.{}(act, ...)", fn_name))?;
        self.indent();

        // Session initialization: args and init_scene come first
        self.writeln("local args = { ... }")?;
        self.writeln("local save, var = act:init_scene(SCENE)")?;

        // Generate actor initialization block for __start__ only (counter == 0)
        // Order: init_scene -> clear_spot -> set_spot(s)
        if counter == 0 && !actors.is_empty() {
            // clear_spot at the start of actor initialization block (Requirement 2.1)
            self.writeln("act:clear_spot()")?;
            // set_spot with new format: act:set_spot("name", number) (Requirement 3.1, 3.2)
            for actor in actors {
                self.writeln(&format!(
                    r#"act:set_spot("{}", {})"#,
                    actor.name, actor.number
                ))?;
            }
        }

        self.write_blank_line()?;

        // Generate local scene items
        self.generate_local_scene_items(&scene.items)?;

        // Code blocks are NOT generated here - they are generated at global scene level
        // This ensures code blocks appear after all local scene function definitions

        self.dedent();
        self.writeln("end")?;
        self.write_blank_line()?;

        Ok(())
    }

    /// Check if a LocalSceneItem is a "callable" item (TCO optimization target).
    ///
    /// Currently only `CallScene` is considered callable. When new variants like
    /// `FnCall` are added in the future, simply extend the `matches!` condition:
    ///
    /// ```ignore
    /// // Future extension example:
    /// // matches!(item, LocalSceneItem::CallScene(_) | LocalSceneItem::FnCall(_))
    /// ```
    fn is_callable_item(item: &LocalSceneItem) -> bool {
        matches!(item, LocalSceneItem::CallScene(_))
    }

    /// Generate local scene items (action lines, var sets, calls).
    ///
    /// Tail call optimization: The last item in the list gets a `return` prefix
    /// if it is a CallScene, enabling Lua TCO.
    fn generate_local_scene_items(
        &mut self,
        items: &[LocalSceneItem],
    ) -> Result<(), TranspileError> {
        // Calculate the index of the last callable item for TCO
        // TCO only applies if the last item itself is callable
        let last_index = items.len().saturating_sub(1);
        let last_is_callable = items.last().is_some_and(Self::is_callable_item);

        let mut last_actor: Option<String> = None;

        for (index, item) in items.iter().enumerate() {
            match item {
                LocalSceneItem::VarSet(var_set) => {
                    self.generate_var_set(var_set)?;
                }
                LocalSceneItem::CallScene(call_scene) => {
                    let is_tail_call = last_is_callable && index == last_index;
                    self.generate_call_scene(call_scene, is_tail_call)?;
                }
                LocalSceneItem::ActionLine(action_line) => {
                    self.generate_action_line(action_line, &mut last_actor)?;
                }
                LocalSceneItem::ContinueAction(continue_action) => {
                    self.generate_continue_action(continue_action, &last_actor)?;
                }
                LocalSceneItem::CueCommand(_) => {
                    // キューコマンドは Lua コード生成の対象外（dola 側で処理）
                }
            }
        }

        Ok(())
    }
}
