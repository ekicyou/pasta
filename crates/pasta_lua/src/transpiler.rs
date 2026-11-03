//! Lua Transpiler for Pasta DSL.
//!
//! This module provides the main transpiler interface for converting
//! Pasta AST to Lua code.

use pasta_core::registry::SceneRegistry;
use pasta_dsl::parser::{ActorScope, Attr, FileItem, GlobalSceneScope, KeyWords, PastaFile};

use super::code_gen::source_map::SourceMapSink;
use super::code_gen::{LineMapSlot, LuaCodeGenerator};
use super::config::TranspilerConfig;
use super::context::{TranspileContext, local_scene_counters};
use super::error::TranspileError;
use super::normalize::{LineShift, normalize_output_with_shift};
use super::string_literalizer::StringLiteralizer;

use pasta_dsl::parser::Span;
use std::collections::BTreeMap;
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
    ///
    /// This is [`transpile_with_source_map`](Self::transpile_with_source_map)
    /// with no sink attached, keeping only the context. The line recorder is
    /// always attached, so the output bytes are the same with or without a sink.
    pub fn transpile<W: Write>(
        &self,
        file: &PastaFile,
        writer: &mut W,
    ) -> Result<TranspileContext, TranspileError> {
        self.transpile_with_source_map(file, writer, None)
            .map(|(context, _)| context)
    }

    /// Transpile PastaFile to Lua code, recording into a source-map sink and
    /// returning the normalize [`LineShift`] (source-map build path).
    ///
    /// This is the loader-side integration entry for `pasta-source-map` task 4.3:
    /// it is the SAME code-generation + normalize pipeline as
    /// [`transpile`](Self::transpile), but it additionally
    /// returns the [`LineShift`] produced by output normalization so the loader can
    /// rebase the sink's pre-normalize records onto the final `.lua` line numbers
    /// (`MapBuilderSink::finish(&shift)` — Requirement 2.1) and complete the
    /// per-chunk [`crate::debug::source_map::ChunkSourceMap`].
    ///
    /// # Byte-invariance (Requirement 7.1)
    ///
    /// The line recorder is always attached; `sink` is optional and only receives
    /// the same records. Normalization ([`normalize_output_with_shift`]) only trims
    /// the end of the output, so the bytes written to `writer` are the same whether
    /// or not a sink is passed.
    ///
    /// # Arguments
    /// * `file` - The parsed PastaFile AST
    /// * `writer` - Output writer
    /// * `sink` - Optional source-map recording sink (the loader attaches one)
    pub fn transpile_with_source_map<W: Write>(
        &self,
        file: &PastaFile,
        writer: &mut W,
        sink: Option<&mut dyn SourceMapSink>,
    ) -> Result<(TranspileContext, LineShift), TranspileError> {
        let mut context = TranspileContext::new();

        // Use intermediate buffer for code generation
        let mut intermediate_buffer: Vec<u8> = Vec::new();
        // シーンの行の対応（runtime-error-balloon 2.5）のために、デバッガの対応表と同じ記録を
        // いつも受け取る。渡されたシンクへはそのまま流す（デバッガの対応表は変わらない）。
        let mut recorder = LineRecorder {
            lines: BTreeMap::new(),
            inner: sink,
        };
        let mut codegen = LuaCodeGenerator::new(&mut intermediate_buffer);
        codegen.set_source_map(&mut recorder);

        // Write header
        codegen.write_header()?;

        // n 番目のグローバルシーンの属性（ファイル属性の受け継ぎと畳みの後）
        let mut scene_attrs = file.global_scene_attrs().into_iter();

        // Process FileItems in document order (MAJOR-2)
        for item in &file.items {
            match item {
                // 受け継ぎは global_scene_attrs が決める
                FileItem::FileAttr(_) => {}
                FileItem::GlobalWord(word) => {
                    Self::process_global_word(&mut context, &mut codegen, word)?;
                }
                FileItem::GlobalSceneScope(scene) => {
                    let attrs = scene_attrs.next().unwrap_or_default();
                    self.process_global_scene(&mut context, &mut codegen, scene, &attrs)?;
                }
                FileItem::ActorScope(actor) => {
                    Self::process_actor(&mut context, &mut codegen, actor)?;
                }
            }
        }
        let slots = codegen.take_line_map_slots();

        // Convert intermediate buffer to UTF-8 string and normalize. Use the
        // shift-returning normalize so the caller can rebase recorded pre-normalize
        // lines onto final `.lua` lines (2.1).
        let raw_output = String::from_utf8(intermediate_buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let (normalized_output, shift) = normalize_output_with_shift(&raw_output);
        let normalized_output = fill_line_maps(
            &normalized_output,
            &shift,
            &slots,
            &recorder.lines,
            &file.path.to_string_lossy(),
        )?;

        // Write normalized output to final writer
        writer.write_all(normalized_output.as_bytes())?;

        Ok((context, shift))
    }

    /// Process GlobalWord: register in WordDefRegistry and generate Lua code.
    fn process_global_word<W: Write>(
        context: &mut TranspileContext,
        codegen: &mut LuaCodeGenerator<W>,
        word: &KeyWords,
    ) -> Result<(), TranspileError> {
        for name in &word.names {
            context
                .word_registry
                .register_global(name, word.words.clone());
        }
        codegen.generate_global_word(word)?;
        Ok(())
    }

    /// Process GlobalSceneScope: register scene/words, generate code, register locals.
    ///
    /// `attrs` は受け継ぎと畳みの後の、このシーンの属性の並び（`PastaFile::global_scene_attrs`）。
    ///
    /// 宣言名（別名表で置き換えた後の名前）をここで 1 回だけ決め、登録・単語のモジュール名・
    /// 生成コード・ローカルの親名のすべてに同じ値を渡す。単独の `＊` 行はパーサが受け継いだ
    /// `scene.name` を持つので同じ経路で置き換わる。
    fn process_global_scene<W: Write>(
        &self,
        context: &mut TranspileContext,
        codegen: &mut LuaCodeGenerator<W>,
        scene: &GlobalSceneScope,
        attrs: &[Attr],
    ) -> Result<(), TranspileError> {
        let declared_name = self
            .config
            .scene_aliases
            .resolve(&scene.name)
            .unwrap_or(&scene.name);
        let (_, counter) = context.register_global_scene_named(declared_name, attrs);

        // Register scene-level word definitions in WordDefRegistry
        let module_name = SceneRegistry::registered_name(declared_name, counter);
        for kw in &scene.words {
            for name in &kw.names {
                context
                    .word_registry
                    .register_local(&module_name, name, kw.words.clone());
            }
        }

        codegen.generate_global_scene(scene, declared_name, counter, attrs)?;

        // Register named local scenes (start scene is part of global)
        // 通し番号は生成器と同じ local_scene_counters（照合用の名前ごと）を使う（2.12）。
        let local_counters = local_scene_counters(&scene.local_scenes);
        for (local_scene, local_counter) in scene.local_scenes.iter().zip(local_counters) {
            if local_scene.name.is_some() {
                context.register_local_scene(local_scene, declared_name, counter, local_counter);
            }
        }

        Ok(())
    }

    /// Process ActorScope: register actor words and generate Lua code.
    fn process_actor<W: Write>(
        context: &mut TranspileContext,
        codegen: &mut LuaCodeGenerator<W>,
        actor: &ActorScope,
    ) -> Result<(), TranspileError> {
        for word_def in &actor.words {
            for name in &word_def.names {
                context
                    .word_registry
                    .register_actor(&actor.name, name, word_def.words.clone());
            }
        }
        codegen.generate_actor(actor)?;
        Ok(())
    }
}

/// デバッガの対応表と同じ記録（`.lua` の行 → `.pasta` の行・同じ行は後勝ち）を受け取り、
/// 渡されたシンクがあればそのまま流す。
struct LineRecorder<'s> {
    lines: BTreeMap<u32, u32>,
    inner: Option<&'s mut dyn SourceMapSink>,
}

impl SourceMapSink for LineRecorder<'_> {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.lines.insert(lua_line, pasta_line);
        if let Some(inner) = self.inner.as_deref_mut() {
            inner.record_line(lua_line, pasta_line);
        }
    }

    fn record(&mut self, lua_line: u32, span: Span) {
        self.lines.insert(lua_line, span.start_line as u32);
        if let Some(inner) = self.inner.as_deref_mut() {
            inner.record(lua_line, span);
        }
    }

    fn record_scene(&mut self, scene_join_key: &str, span: Span) {
        if let Some(inner) = self.inner.as_deref_mut() {
            inner.record_scene(scene_join_key, span);
        }
    }
}

/// 整形の後の生成コードの各 `init_scene` の行の `)` の前に、そのシーンの行の対応
/// `"<.pasta のファイル>|<.lua の行>:<.pasta の行>,..."`（1 行の文字列の定数）を埋める。
/// 行は増やさない（runtime-error-balloon 2.5）。行は整形の後の最終の行。
fn fill_line_maps(
    normalized: &str,
    shift: &LineShift,
    slots: &[LineMapSlot],
    lines: &BTreeMap<u32, u32>,
    pasta_file: &str,
) -> Result<String, TranspileError> {
    let mut out: Vec<String> = normalized.lines().map(str::to_string).collect();
    for slot in slots {
        let pairs: Vec<String> = slot
            .ranges
            .iter()
            .filter(|(from, to)| from <= to)
            .flat_map(|&(from, to)| lines.range(from..=to))
            .filter_map(|(&pre, &pasta)| shift.map(pre).map(|lua| format!("{lua}:{pasta}")))
            .collect();
        let literal = StringLiteralizer::literalize(&format!("{pasta_file}|{}", pairs.join(",")))?;
        let line = shift
            .map(slot.init_line)
            .and_then(|l| out.get_mut(l as usize - 1))
            .expect("init_scene の行は整形で消えない");
        let close = line.rfind(')').expect("init_scene の行は ) で閉じる");
        line.insert_str(close, &format!(", {literal}"));
    }
    let mut result = out.join("\n");
    result.push('\n');
    Ok(result)
}

#[cfg(test)]
#[path = "transpiler_tests.rs"]
mod tests;
