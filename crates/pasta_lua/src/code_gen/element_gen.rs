//! Element-level code generation: variables, calls, actions, expressions, words.

use super::LuaCodeGenerator;
use crate::error::TranspileError;
use crate::string_literalizer::StringLiteralizer;
use pasta_dsl::parser::{
    Action, ActionLine, Args, CallScene, CodeBlock, ContinueAction, Expr, KeyWords, SetValue,
    Span, VarScope, VarSet,
};
use std::io::Write;

/// Format optional arguments suffix: empty when no args, otherwise ", arg1, arg2, ..."
fn format_args_suffix(args_str: &str) -> String {
    if args_str.is_empty() {
        String::new()
    } else {
        format!(", {}", args_str)
    }
}

/// Extract the `.pasta` [`Span`] carried by every [`Action`] variant.
///
/// Source-map seam helper (R4): used to thread the originating `.pasta` location to
/// the output-line recording point. Defined here (not on the `pasta_dsl` AST) to keep
/// the change within the `code_gen` boundary.
fn action_span(action: &Action) -> Span {
    match action {
        Action::Talk { span, .. }
        | Action::WordRef { span, .. }
        | Action::VarRef { span, .. }
        | Action::FnCall { span, .. }
        | Action::SakuraScript { span, .. }
        | Action::Escape { span, .. } => *span,
    }
}

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Resolve a VarScope to its Lua variable path (e.g., `var.x`, `save.x`, `args[1]`).
    ///
    /// Returns an error for `VarScope::Property`, which must be handled separately
    /// by callers before reaching this function.
    fn resolve_var_path(name: &str, scope: &VarScope) -> Result<String, TranspileError> {
        match scope {
            VarScope::Local => Ok(format!("var.{}", name)),
            VarScope::Global => Ok(format!("save.{}", name)),
            VarScope::Args(index) => Ok(format!("args[{}]", index + 1)),
            VarScope::Property => Err(TranspileError::property_in_expression()),
        }
    }

    /// Generate variable assignment (Requirement 3d).
    ///
    /// Local: `var.変数名 = 値`
    /// Global: `save.変数名 = 値`
    ///
    /// Source-map wiring (Requirements 1.1, 1.3): records the assignment's `.pasta`
    /// [`Span`] against the output line(s) it emits, following `generate_action`'s
    /// `out_line` delta-detection pattern. Every branch emits exactly one line, but
    /// the delta check makes recording robust to non-emitting paths (e.g. the error
    /// branch). The `Property` branch delegates to `generate_property_set`, which is
    /// covered by the same surrounding delta.
    pub fn generate_var_set(&mut self, var_set: &VarSet) -> Result<(), TranspileError> {
        let out_line_before = self.out_line();
        match &var_set.name {
            Some(name) => {
                let var_path = match var_set.scope {
                    VarScope::Local => format!("var.{}", name),
                    VarScope::Global => format!("save.{}", name),
                    VarScope::Property => {
                        self.generate_property_set(name, &var_set.value)?;
                        if self.out_line() > out_line_before {
                            self.record_span(var_set.span);
                        }
                        return Ok(());
                    }
                    VarScope::Args(_) => {
                        // Cannot assign to scene arguments
                        return Err(TranspileError::invalid_ast(
                            &var_set.span,
                            "Cannot assign to scene argument",
                        ));
                    }
                };

                // GET代入: 右辺が単一Property参照なら直接代入
                if let SetValue::Expr(Expr::VarRef {
                    name: ref prop_name,
                    scope: VarScope::Property,
                }) = var_set.value
                {
                    let prop_literal = StringLiteralizer::literalize(prop_name)?;
                    self.writeln(&format!(
                        "{} = act:get_property({})",
                        var_path, prop_literal
                    ))?;
                    if self.out_line() > out_line_before {
                        self.record_span(var_set.span);
                    }
                    return Ok(());
                }

                match &var_set.value {
                    SetValue::Expr(expr) => {
                        self.write_indent()?;
                        self.write_raw(&format!("{} = ", var_path))?;
                        self.generate_expr(expr)?;
                        self.write_line_terminator()?;
                    }
                    SetValue::WordRef { name } => {
                        // Generate: var.変数名 = act:word("単語名") or save.変数名 = act:word("単語名")
                        let word_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!("{} = act:word({})", var_path, word_literal))?;
                    }
                }
            }
            None => {
                // Expression statement: evaluate expression without assignment
                match &var_set.value {
                    SetValue::Expr(expr) => {
                        self.write_indent()?;
                        self.generate_expr(expr)?;
                        self.write_line_terminator()?;
                    }
                    SetValue::WordRef { name } => {
                        let word_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!("act:word({})", word_literal))?;
                    }
                }
            }
        }

        // Record the (out_line -> span) correspondence for the line just emitted
        // (Requirement 1.1). Skipped when no line was emitted or no sink is attached.
        if self.out_line() > out_line_before {
            self.record_span(var_set.span);
        }

        Ok(())
    }

    /// Generate property SET assignment.
    ///
    /// `SetValue::Expr` → `act:set_property("name", expr)`
    /// `SetValue::WordRef` → `act:set_property("name", act:word("word"))`
    fn generate_property_set(
        &mut self,
        name: &str,
        value: &SetValue,
    ) -> Result<(), TranspileError> {
        let name_literal = StringLiteralizer::literalize(name)?;
        match value {
            SetValue::Expr(expr) => {
                let expr_str = self.expr_to_string(expr)?;
                self.writeln(&format!("act:set_property({}, {})", name_literal, expr_str))?;
            }
            SetValue::WordRef { name: word_name } => {
                let word_literal = StringLiteralizer::literalize(word_name)?;
                self.writeln(&format!(
                    "act:set_property({}, act:word({}))",
                    name_literal, word_literal
                ))?;
            }
        }
        Ok(())
    }

    /// Generate scene call (Requirement 3d, 3g).
    ///
    /// Generates: `act:call(SCENE.__global_name__, "ラベル名", {}, table.unpack(args))`
    ///
    /// When `is_tail_call` is true, prepends `return` to enable Lua TCO.
    ///
    /// Source-map wiring (Requirement 1.1): records the call's `.pasta` [`Span`]
    /// against the single output line it emits, following `generate_action`'s
    /// `out_line` delta-detection pattern.
    pub(super) fn generate_call_scene(
        &mut self,
        call_scene: &CallScene,
        is_tail_call: bool,
    ) -> Result<(), TranspileError> {
        let out_line_before = self.out_line();
        // Generate argument list
        let args_str = if let Some(ref args) = call_scene.args {
            let parts = self.generate_args_string(args)?;
            if parts.is_empty() {
                "table.unpack(args)".to_string()
            } else {
                format!("{}, table.unpack(args)", parts)
            }
        } else {
            "table.unpack(args)".to_string()
        };

        // Generate call statement based on target type
        let call_stmt = match &call_scene.target {
            pasta_dsl::parser::CallTarget::Static(name) => {
                format!(
                    "act:call(SCENE.__global_name__, \"{}\", {{}}, {})",
                    name, args_str
                )
            }
            pasta_dsl::parser::CallTarget::Dynamic(expr) => {
                let expr_str = self.expr_to_string(expr)?;
                format!(
                    "act:call(SCENE.__global_name__, tostring({}), {{}}, {})",
                    expr_str, args_str
                )
            }
        };

        // Tail call optimization: prepend 'return' for the last callable item
        if is_tail_call {
            self.writeln(&format!("return {}", call_stmt))?;
        } else {
            self.writeln(&call_stmt)?;
        }

        // Record the (out_line -> span) correspondence for the line just emitted
        // (Requirement 1.1).
        if self.out_line() > out_line_before {
            self.record_span(call_scene.span);
        }

        Ok(())
    }

    /// Generate action line (with speaker).
    pub(super) fn generate_action_line(
        &mut self,
        action_line: &ActionLine,
        last_actor: &mut Option<String>,
    ) -> Result<(), TranspileError> {
        let actor = &action_line.actor;
        *last_actor = Some(actor.clone());

        // Generate actions
        for action in &action_line.actions {
            self.generate_action(action, actor)?;
        }

        Ok(())
    }

    /// Generate continue action line (without speaker).
    pub(super) fn generate_continue_action(
        &mut self,
        continue_action: &ContinueAction,
        last_actor: &Option<String>,
    ) -> Result<(), TranspileError> {
        let actor = match last_actor {
            Some(a) => a,
            None => {
                return Err(TranspileError::invalid_continuation(&continue_action.span));
            }
        };

        // Generate actions (speaker is inherited)
        for action in &continue_action.actions {
            self.generate_action(action, actor)?;
        }

        Ok(())
    }

    /// Generate a single action (Requirement 3d, 3e).
    ///
    /// Source-map seam (R4): this is the representative span-bearing path. The
    /// action's `.pasta` [`Span`](pasta_dsl::parser::Span) is recorded against the
    /// generated Lua line via [`record_span`](Self::record_span) when a sink is
    /// attached (otherwise inert / byte-identical). Every variant emits exactly one
    /// line except the rare empty-escape case, so we detect actual emission by the
    /// `out_line` delta before recording, ensuring the span maps to the line that was
    /// actually written.
    pub fn generate_action(&mut self, action: &Action, actor: &str) -> Result<(), TranspileError> {
        let span = action_span(action);
        let out_line_before = self.out_line();
        match action {
            Action::Talk { text, .. } => {
                // act.アクター:talk("文字列")
                let literal = StringLiteralizer::literalize(text)?;
                self.writeln(&format!("act.{}:talk({})", actor, literal))?;
            }
            Action::WordRef {
                name: word_name, ..
            } => {
                // act.アクター:talk(act.アクター:word("単語名"))
                let word_literal = StringLiteralizer::literalize(word_name)?;
                self.writeln(&format!(
                    "act.{}:talk(act.{}:word({}))",
                    actor, actor, word_literal
                ))?;
            }
            Action::VarRef { name, scope, .. } => {
                // Variable interpolation: generate talk with concatenation
                match scope {
                    VarScope::Property => {
                        let prop_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!(
                            "act.{}:talk(tostring(act:get_property({})))",
                            actor, prop_literal
                        ))?;
                    }
                    _ => {
                        // SAFETY: VarScope::Property is handled in the arm above;
                        // resolve_var_path returns Err for Property as a defensive guard.
                        let var_path = Self::resolve_var_path(name, scope)?;
                        self.writeln(&format!("act.{}:talk(tostring({}))", actor, var_path))?;
                    }
                }
            }
            Action::FnCall {
                name, args, scope, ..
            } => {
                let args_str = self.generate_args_string(args)?;
                match scope {
                    pasta_dsl::parser::FnScope::Local => {
                        // act.アクター:expr_fn("関数名", 引数...)
                        let name_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!(
                            "act.{}:talk(tostring(act.{}:expr_fn({}{})))",
                            actor,
                            actor,
                            name_literal,
                            format_args_suffix(&args_str)
                        ))?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // GLOBAL.関数名(act, 引数...)
                        self.writeln(&format!(
                            "act.{}:talk(tostring(GLOBAL.{}(act{})))",
                            actor,
                            name,
                            format_args_suffix(&args_str)
                        ))?;
                    }
                }
            }
            Action::SakuraScript { script, .. } => {
                // SakuraScript is output as act.{actor}:sakura_script()
                let literal = StringLiteralizer::literalize(script)?;
                self.writeln(&format!("act.{}:sakura_script({})", actor, literal))?;
            }
            Action::Escape {
                sequence: escape, ..
            } => {
                // Extract the escaped character (second char) and literalize
                if let Some(c) = escape.chars().nth(1) {
                    let literal = StringLiteralizer::literalize(&c.to_string())?;
                    self.writeln(&format!("act.{}:talk({})", actor, literal))?;
                }
            }
        }

        // Record the (out_line -> span) correspondence for the line(s) just emitted.
        // Skipped automatically when no line was emitted (empty escape) or no sink is
        // attached. `record_span` uses the current `out_line`, which now points at the
        // last line written by this action.
        if self.out_line() > out_line_before {
            self.record_span(span);
        }

        Ok(())
    }

    /// Generate an expression (delegates to `generate_expr_to_buffer`).
    fn generate_expr(&mut self, expr: &Expr) -> Result<(), TranspileError> {
        let mut buf = Vec::new();
        self.generate_expr_to_buffer(expr, &mut buf)?;
        self.writer.write_all(&buf)?;
        Ok(())
    }

    /// Render an expression to a `String` (helper over `generate_expr_to_buffer`).
    fn expr_to_string(&self, expr: &Expr) -> Result<String, TranspileError> {
        let mut buf = Vec::new();
        self.generate_expr_to_buffer(expr, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap_or_default())
    }

    /// Generate expression to a separate buffer.
    fn generate_expr_to_buffer(
        &self,
        expr: &Expr,
        buf: &mut Vec<u8>,
    ) -> Result<(), TranspileError> {
        match expr {
            Expr::Integer(n) => {
                write!(buf, "{}", n)?;
            }
            Expr::Float(f) => {
                write!(buf, "{}", f)?;
            }
            Expr::String(s) => {
                let literal = StringLiteralizer::literalize(s)?;
                write!(buf, "{}", literal)?;
            }
            Expr::BlankString => {
                write!(buf, "\"\"")?;
            }
            Expr::VarRef { name, scope } => {
                write!(buf, "{}", Self::resolve_var_path(name, scope)?)?;
            }
            Expr::FnCall { name, args, scope } => {
                let args_str = self.generate_args_string(args)?;
                match scope {
                    pasta_dsl::parser::FnScope::Local => {
                        // act:expr_fn("関数名", 引数...)
                        let name_literal = StringLiteralizer::literalize(name)?;
                        write!(
                            buf,
                            "act:expr_fn({}{})",
                            name_literal,
                            format_args_suffix(&args_str)
                        )?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // GLOBAL.関数名(act, 引数...)
                        write!(
                            buf,
                            "GLOBAL.{}(act{})",
                            name,
                            format_args_suffix(&args_str)
                        )?;
                    }
                }
            }
            Expr::Paren(inner) => {
                write!(buf, "(")?;
                self.generate_expr_to_buffer(inner, buf)?;
                write!(buf, ")")?;
            }
            Expr::Binary { op, lhs, rhs } => {
                self.generate_expr_to_buffer(lhs, buf)?;
                let op_str = match op {
                    pasta_dsl::parser::BinOp::Add => " + ",
                    pasta_dsl::parser::BinOp::Sub => " - ",
                    pasta_dsl::parser::BinOp::Mul => " * ",
                    pasta_dsl::parser::BinOp::Div => " / ",
                    pasta_dsl::parser::BinOp::Mod => " % ",
                };
                write!(buf, "{}", op_str)?;
                self.generate_expr_to_buffer(rhs, buf)?;
            }
        }

        Ok(())
    }

    /// Generate arguments as a string.
    fn generate_args_string(&self, args: &Args) -> Result<String, TranspileError> {
        let mut parts = Vec::new();
        for arg in &args.items {
            let expr = match arg {
                pasta_dsl::parser::Arg::Positional(expr) => expr,
                pasta_dsl::parser::Arg::Keyword { key: _, value } => value,
            };
            parts.push(self.expr_to_string(expr)?);
        }
        Ok(parts.join(", "))
    }

    /// Generate code block (Requirement 3f).
    ///
    /// Outputs the code block content directly without transformation.
    ///
    /// Source-map wiring (Requirements 1.1, 1.3): the block is emitted one `.lua`
    /// line per `.pasta` content line (1:1 line correspondence — `content.lines()`
    /// is each emitted via exactly one `writeln`). Each output line is mapped
    /// INDIVIDUALLY to its originating `.pasta` line so in-block breakpoints resolve
    /// precisely, instead of collapsing the whole block onto a single span line.
    ///
    /// # Line offset
    ///
    /// `block.span.start_line` is the `.pasta` line of the OPENING fence
    /// (` ```lang `, the `code_open` grammar rule). The block `content`
    /// (`code_contents`) begins on the line immediately AFTER the fence. Therefore
    /// content line `offset` (0-based) originates from
    /// `block.span.start_line + 1 + offset` — a constant `+1` correction for the
    /// fence line. The closing fence is not part of `content`, so no trailing
    /// adjustment is needed.
    pub fn generate_code_block(&mut self, block: &CodeBlock) -> Result<(), TranspileError> {
        // The opening fence occupies `span.start_line`; content starts on the next
        // line. `+ 1` skips the fence; `+ offset` walks each content line. Only map
        // when the span is valid, mirroring `record_span`'s guard so default/
        // synthetic spans (start_line == 0) do not pollute the map.
        let content_start_line = if block.span.is_valid() {
            block.span.start_line as u32 + 1
        } else {
            0 // sentinel: record_block_line skips pasta_line == 0 (and subsequent)
        };
        // Output code content with proper indentation
        for (offset, line) in block.content.lines().enumerate() {
            self.writeln(line)?;
            if content_start_line > 0 {
                self.record_block_line(content_start_line + offset as u32);
            }
        }
        Ok(())
    }

    /// Shared word definition generator parameterized by prefix.
    ///
    /// Generates: `{prefix}:entry("value1", "value2", ...)`
    ///
    /// The `prefix` includes the full method call syntax, e.g.:
    /// - `PASTA.create_word("key")` for global words (dot syntax)
    /// - `SCENE:create_word("key")` for scene-local words (colon syntax)
    ///
    /// Source-map wiring (Requirements 1.1, 1.3): one `.pasta` word definition with
    /// multiple key names emits one `.lua` line per name; ALL of those output lines
    /// map to the same definition `.pasta` [`Span`]. Recording happens per emitted
    /// line (following `generate_action`'s per-line delta pattern), so every line is
    /// covered, not just the last.
    fn generate_word_definition(
        &mut self,
        word: &KeyWords,
        receiver: &str,
        separator: &str,
    ) -> Result<(), TranspileError> {
        if word.words.is_empty() {
            return Ok(());
        }

        let values: Vec<String> = word
            .words
            .iter()
            .map(|w| StringLiteralizer::literalize(w))
            .collect::<Result<Vec<_>, _>>()?;
        let entry = values.join(", ");

        for name in &word.names {
            let out_line_before = self.out_line();
            self.writeln(&format!(
                "{}{}create_word({}):entry({})",
                receiver,
                separator,
                StringLiteralizer::literalize(name)?,
                entry
            ))?;
            // Map this output line to the word definition's `.pasta` span. Every name
            // line maps to the same span (Requirement 1.3).
            if self.out_line() > out_line_before {
                self.record_span(word.span);
            }
        }

        Ok(())
    }

    /// Generate global word definition (Requirement 2.1, Task 4.2).
    ///
    /// Generates: `PASTA.create_word("key"):entry("value1", "value2", ...)`
    ///
    /// Called at file level, outside of any do block.
    pub fn generate_global_word(&mut self, word: &KeyWords) -> Result<(), TranspileError> {
        self.generate_word_definition(word, "PASTA", ".")
    }

    /// Generate local word definition for a scene (Requirement 2.2, Task 4.3).
    ///
    /// Generates: `SCENE:create_word("key"):entry("value1", "value2", ...)`
    ///
    /// Called inside a local scene function, after init_scene.
    pub fn generate_local_word(&mut self, word: &KeyWords) -> Result<(), TranspileError> {
        self.generate_word_definition(word, "SCENE", ":")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_gen::source_map::SourceMapSink;
    use crate::config::LineEnding;
    use pasta_dsl::parser::{Arg, CallTarget, FnScope};

    /// Test sink capturing each `(lua_line, pasta_line)` record.
    #[derive(Default)]
    struct CapturingSink {
        records: Vec<(u32, u32)>,
    }

    impl SourceMapSink for CapturingSink {
        fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
            self.records.push((lua_line, pasta_line));
        }
    }

    fn gen_to_string<F>(f: F) -> String
    where
        F: FnOnce(&mut LuaCodeGenerator<'_, Vec<u8>>) -> Result<(), TranspileError>,
    {
        let mut output = Vec::new();
        {
            let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
            f(&mut cg).unwrap();
        }
        String::from_utf8(output).unwrap()
    }

    // ------------------------------------------------------------------
    // generate_action edge cases
    // ------------------------------------------------------------------

    /// A malformed escape sequence with no second character emits NOTHING:
    /// no bytes, no out_line advance, and no source-map record even with a
    /// sink attached (the documented empty-escape case).
    #[test]
    fn escape_with_single_char_sequence_emits_nothing() {
        let mut sink = CapturingSink::default();
        let mut output = Vec::new();
        let final_out_line;
        {
            let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
            cg.set_source_map(&mut sink);
            let action = Action::Escape {
                sequence: "@".to_string(), // no char at index 1
                span: Span::new(1, 1, 1, 2, 0, 1),
            };
            cg.generate_action(&action, "さくら").unwrap();
            final_out_line = cg.out_line();
        }
        assert!(output.is_empty(), "empty escape must emit no bytes");
        assert_eq!(final_out_line, 0, "out_line must not advance");
        assert!(sink.records.is_empty(), "no record without an emitted line");
    }

    /// SakuraScript action emits `act.{actor}:sakura_script(<literal>)`.
    #[test]
    fn sakura_script_action_emits_sakura_script_call() {
        let text = gen_to_string(|cg| {
            cg.generate_action(
                &Action::SakuraScript {
                    script: "\\s[0]".to_string(),
                    span: Span::default(),
                },
                "さくら",
            )
        });
        assert!(
            text.contains("act.さくら:sakura_script("),
            "must route through sakura_script, got: {}",
            text
        );
        assert!(text.contains("\\s[0]"), "script payload preserved: {}", text);
    }

    // ------------------------------------------------------------------
    // Continuation lines
    // ------------------------------------------------------------------

    /// A continuation line before any speaker line is an InvalidContinuation
    /// error (no actor to inherit).
    #[test]
    fn continue_action_without_prior_actor_is_invalid_continuation_error() {
        let mut output = Vec::new();
        let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
        let cont = ContinueAction {
            actions: vec![Action::Talk {
                text: "続き".to_string(),
                span: Span::default(),
            }],
            span: Span::new(2, 1, 2, 3, 5, 8),
        };
        let err = cg.generate_continue_action(&cont, &None).unwrap_err();
        assert!(
            matches!(err, TranspileError::InvalidContinuation { .. }),
            "expected InvalidContinuation, got: {:?}",
            err
        );
    }

    /// `generate_action_line` records its actor as `last_actor`, and a
    /// following continuation line inherits that speaker.
    #[test]
    fn continue_action_inherits_actor_from_preceding_action_line() {
        let text = gen_to_string(|cg| {
            let mut last_actor: Option<String> = None;
            cg.generate_action_line(
                &ActionLine {
                    actor: "うにゅう".to_string(),
                    actions: vec![Action::Talk {
                        text: "やあ".to_string(),
                        span: Span::default(),
                    }],
                    span: Span::default(),
                },
                &mut last_actor,
            )?;
            assert_eq!(last_actor.as_deref(), Some("うにゅう"));
            cg.generate_continue_action(
                &ContinueAction {
                    actions: vec![Action::Talk {
                        text: "続き".to_string(),
                        span: Span::default(),
                    }],
                    span: Span::default(),
                },
                &last_actor,
            )
        });
        assert_eq!(
            text,
            "act.うにゅう:talk(\"やあ\")\nact.うにゅう:talk(\"続き\")\n",
            "continuation must reuse the inherited speaker"
        );
    }

    // ------------------------------------------------------------------
    // generate_call_scene
    // ------------------------------------------------------------------

    fn call_scene(target: CallTarget, args: Option<Args>) -> CallScene {
        CallScene {
            target,
            args,
            span: Span::default(),
        }
    }

    /// Static call without args forwards only `table.unpack(args)`.
    #[test]
    fn call_scene_static_without_args_forwards_table_unpack_only() {
        let text = gen_to_string(|cg| {
            cg.generate_call_scene(
                &call_scene(CallTarget::Static("次シーン".to_string()), None),
                false,
            )
        });
        assert_eq!(
            text,
            "act:call(SCENE.__global_name__, \"次シーン\", {}, table.unpack(args))\n"
        );
    }

    /// `Some(args)` with an EMPTY item list behaves like no args at all
    /// (still only `table.unpack(args)` — no leading comma artifacts).
    #[test]
    fn call_scene_with_empty_args_list_matches_no_args_form() {
        let text = gen_to_string(|cg| {
            cg.generate_call_scene(
                &call_scene(CallTarget::Static("次".to_string()), Some(Args::empty())),
                false,
            )
        });
        assert_eq!(
            text,
            "act:call(SCENE.__global_name__, \"次\", {}, table.unpack(args))\n"
        );
    }

    /// Positional and keyword args are emitted in order before
    /// `table.unpack(args)`; keyword keys are dropped (value-only).
    #[test]
    fn call_scene_emits_positional_and_keyword_args_before_unpack() {
        let args = Args {
            items: vec![
                Arg::Positional(Expr::Integer(1)),
                Arg::Keyword {
                    key: "名前".to_string(),
                    value: Expr::String("さくら".to_string()),
                },
            ],
            span: Span::default(),
        };
        let text = gen_to_string(|cg| {
            cg.generate_call_scene(
                &call_scene(CallTarget::Static("次".to_string()), Some(args)),
                false,
            )
        });
        assert_eq!(
            text,
            "act:call(SCENE.__global_name__, \"次\", {}, 1, \"さくら\", table.unpack(args))\n"
        );
    }

    /// Dynamic target evaluates the expression and wraps it in `tostring(...)`.
    #[test]
    fn call_scene_dynamic_target_wraps_expr_in_tostring() {
        let text = gen_to_string(|cg| {
            cg.generate_call_scene(
                &call_scene(
                    CallTarget::Dynamic(Expr::VarRef {
                        name: "行き先".to_string(),
                        scope: VarScope::Local,
                    }),
                    None,
                ),
                false,
            )
        });
        assert_eq!(
            text,
            "act:call(SCENE.__global_name__, tostring(var.行き先), {}, table.unpack(args))\n"
        );
    }

    /// Tail call prepends `return ` (Lua TCO).
    #[test]
    fn call_scene_tail_call_prepends_return() {
        let text = gen_to_string(|cg| {
            cg.generate_call_scene(
                &call_scene(CallTarget::Static("次".to_string()), None),
                true,
            )
        });
        assert!(
            text.starts_with("return act:call("),
            "tail call must start with 'return ', got: {}",
            text
        );
    }

    // ------------------------------------------------------------------
    // Expression generation (via expression statements: VarSet name=None)
    // ------------------------------------------------------------------

    fn expr_stmt(expr: Expr) -> VarSet {
        VarSet {
            name: None,
            scope: VarScope::Local,
            value: SetValue::Expr(expr),
            span: Span::default(),
        }
    }

    /// Float, blank-string, paren, and all five binary operators render with
    /// the exact Lua spellings (` + `, ` - `, ` * `, ` / `, ` % `).
    #[test]
    fn expr_renders_float_blank_string_paren_and_all_binary_ops() {
        use pasta_dsl::parser::BinOp;
        // ((1 - 2)) * 3 / 4 % 5 + var.x  (left-nested to exercise every op)
        let expr = Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr::Binary {
                op: BinOp::Mod,
                lhs: Box::new(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(Expr::Paren(Box::new(Expr::Binary {
                            op: BinOp::Sub,
                            lhs: Box::new(Expr::Integer(1)),
                            rhs: Box::new(Expr::Integer(2)),
                        }))),
                        rhs: Box::new(Expr::Integer(3)),
                    }),
                    rhs: Box::new(Expr::Integer(4)),
                }),
                rhs: Box::new(Expr::Integer(5)),
            }),
            rhs: Box::new(Expr::VarRef {
                name: "x".to_string(),
                scope: VarScope::Local,
            }),
        };
        let text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(expr)));
        assert_eq!(text, "(1 - 2) * 3 / 4 % 5 + var.x\n");

        let float_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::Float(1.5))));
        assert_eq!(float_text, "1.5\n");

        let blank_text = gen_to_string(|cg| cg.generate_var_set(&expr_stmt(Expr::BlankString)));
        assert_eq!(blank_text, "\"\"\n");
    }

    /// Args-scope variable references convert 0-based AST index to 1-based
    /// Lua index in expression position (`Args(2)` -> `args[3]`).
    #[test]
    fn expr_args_var_ref_converts_to_one_based_lua_index() {
        let text = gen_to_string(|cg| {
            cg.generate_var_set(&expr_stmt(Expr::VarRef {
                name: "2".to_string(),
                scope: VarScope::Args(2),
            }))
        });
        assert_eq!(text, "args[3]\n");
    }

    /// Local fn call in expression position uses `act:expr_fn("name", ...)`;
    /// global fn call uses `GLOBAL.name(act, ...)`.
    #[test]
    fn expr_fn_call_local_and_global_spellings() {
        let local_text = gen_to_string(|cg| {
            cg.generate_var_set(&expr_stmt(Expr::FnCall {
                name: "時刻".to_string(),
                args: Args::empty(),
                scope: FnScope::Local,
            }))
        });
        assert_eq!(local_text, "act:expr_fn(\"時刻\")\n");

        let global_text = gen_to_string(|cg| {
            cg.generate_var_set(&expr_stmt(Expr::FnCall {
                name: "rand".to_string(),
                args: Args {
                    items: vec![Arg::Positional(Expr::Integer(6))],
                    span: Span::default(),
                },
                scope: FnScope::Global,
            }))
        });
        assert_eq!(global_text, "GLOBAL.rand(act, 6)\n");
    }

    // ------------------------------------------------------------------
    // Word definitions
    // ------------------------------------------------------------------

    fn key_words(names: &[&str], words: &[&str]) -> KeyWords {
        KeyWords {
            names: names.iter().map(|s| s.to_string()).collect(),
            words: words.iter().map(|s| s.to_string()).collect(),
            span: Span::new(3, 1, 3, 10, 30, 50),
        }
    }

    /// A word definition with an empty value list emits nothing (early
    /// return), for both global and local flavors.
    #[test]
    fn word_definition_with_no_words_emits_nothing() {
        let global = gen_to_string(|cg| cg.generate_global_word(&key_words(&["挨拶"], &[])));
        assert!(global.is_empty(), "global: got {:?}", global);
        let local = gen_to_string(|cg| cg.generate_local_word(&key_words(&["挨拶"], &[])));
        assert!(local.is_empty(), "local: got {:?}", local);
    }

    /// Global words use `PASTA.create_word` (dot), local words use
    /// `SCENE:create_word` (colon); multiple key names emit one line each
    /// with the SAME entry list.
    #[test]
    fn word_definition_prefixes_and_multi_name_lines() {
        let kw = key_words(&["挨拶", "あいさつ"], &["おはよう", "こんにちは"]);

        let global = gen_to_string(|cg| cg.generate_global_word(&kw));
        assert_eq!(
            global,
            "PASTA.create_word(\"挨拶\"):entry(\"おはよう\", \"こんにちは\")\n\
             PASTA.create_word(\"あいさつ\"):entry(\"おはよう\", \"こんにちは\")\n"
        );

        let local = gen_to_string(|cg| cg.generate_local_word(&kw));
        assert_eq!(
            local,
            "SCENE:create_word(\"挨拶\"):entry(\"おはよう\", \"こんにちは\")\n\
             SCENE:create_word(\"あいさつ\"):entry(\"おはよう\", \"こんにちは\")\n"
        );
    }

    // ------------------------------------------------------------------
    // Code blocks
    // ------------------------------------------------------------------

    /// A code block with an INVALID span (synthetic/headerless) still emits
    /// its content lines but records NO source-map entries (sentinel path).
    #[test]
    fn code_block_with_invalid_span_emits_lines_without_records() {
        let mut sink = CapturingSink::default();
        let mut output = Vec::new();
        {
            let mut cg = LuaCodeGenerator::with_line_ending(&mut output, LineEnding::Lf);
            cg.set_source_map(&mut sink);
            cg.generate_code_block(&CodeBlock {
                language: Some("lua".to_string()),
                content: "local a = 1\nlocal b = 2".to_string(),
                span: Span::default(), // invalid: end_byte == 0
            })
            .unwrap();
        }
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "local a = 1\nlocal b = 2\n",
            "content must still be emitted verbatim"
        );
        assert!(
            sink.records.is_empty(),
            "invalid span must not pollute the source map, got {:?}",
            sink.records
        );
    }
}
