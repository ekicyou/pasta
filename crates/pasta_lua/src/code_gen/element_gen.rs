//! Element-level code generation: variables, calls, actions, code blocks, words.

use super::LuaCodeGenerator;
use super::expr_gen::{format_args_suffix, global_fn_call};
use crate::error::TranspileError;
use crate::string_literalizer::StringLiteralizer;
use pasta_dsl::parser::{
    Action, ActionLine, CallScene, CodeBlock, ContinueAction, Expr, KeyWords, SetValue, Span,
    VarScope, VarSet,
};
use std::io::Write;

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
        | Action::Escape { span, .. }
        | Action::DynamicWordRef { span, .. }
        | Action::DynamicFnCall { span, .. } => *span,
    }
}

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Resolve a VarScope to its Lua variable path (e.g., `var.x`, `save.x`, `args[1]`).
    ///
    /// Returns an error for `VarScope::Property`, which must be handled separately
    /// by callers before reaching this function.
    pub(super) fn resolve_var_path(name: &str, scope: &VarScope) -> Result<String, TranspileError> {
        match scope {
            VarScope::Local => Ok(format!("var.{}", name)),
            VarScope::Global => Ok(format!("save.{}", name)),
            VarScope::Args(index) => Ok(format!("args[{}]", index + 1)),
            VarScope::Property => Err(TranspileError::property_in_expression()),
        }
    }

    /// Runtime arguments of a dynamic reference: the variable's value and its path
    /// as a string literal (e.g. `var.x, "var.x"`). The value is passed as-is (never
    /// `tostring`-ed) so the runtime can tell nil / empty / bad types apart.
    pub(super) fn dynamic_ref_args(name: &str, scope: &VarScope) -> Result<String, TranspileError> {
        let var_path = Self::resolve_var_path(name, scope)?;
        let path_literal = StringLiteralizer::literalize(&var_path)?;
        Ok(format!("{}, {}", var_path, path_literal))
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
                    SetValue::DynamicWordRef {
                        var_name,
                        var_scope,
                    } => {
                        let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                        self.writeln(&format!("{} = act:word({})", var_path, ref_args))?;
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
                    SetValue::DynamicWordRef {
                        var_name,
                        var_scope,
                    } => {
                        let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                        self.writeln(&format!("act:word({})", ref_args))?;
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
            SetValue::DynamicWordRef {
                var_name,
                var_scope,
            } => {
                let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                self.writeln(&format!(
                    "act:set_property({}, act:word({}))",
                    name_literal, ref_args
                ))?;
            }
        }
        Ok(())
    }

    /// Generate scene call (Requirement 3d, 3g).
    ///
    /// Tail: `return act:call(SCENE.__global_name__, <key>, {}, <args>)` (Lua TCO).
    /// Mid: `act:call_restore(SCENE.__global_name__, <key>, {}, <args>)`.
    /// `<key>` is `"名前"` (static) or `act:call_key(…)` (dynamic, see [`Self::call_key`]).
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

        // Key: a string literal (static) or `act:call_key(…)` (dynamic). It is
        // left of the args, so the evaluation order is unchanged.
        let key = match &call_scene.target {
            pasta_dsl::parser::CallTarget::Static(name) => format!("\"{}\"", name),
            pasta_dsl::parser::CallTarget::Dynamic(expr) => self.call_key(expr)?,
        };
        // Tail: `return act:call` (Lua TCO). Mid: `act:call_restore` restores the
        // caller's scene after the callee returns.
        let call_head = if is_tail_call {
            "return act:call"
        } else {
            "act:call_restore"
        };
        self.writeln(&format!(
            "{}(SCENE.__global_name__, {}, {{}}, {})",
            call_head, key, args_str
        ))?;

        // Record the (out_line -> span) correspondence for the line just emitted
        // (Requirement 1.1).
        if self.out_line() > out_line_before {
            self.record_span(call_scene.span);
        }

        Ok(())
    }

    /// Key expression of a dynamic Call: `act:call_key(value, "var.x")` for a
    /// single variable reference, `act:call_key(value, nil, "@f()")` for a single
    /// function call, `act:call_key(value)` otherwise. Parentheses are looked
    /// through when classifying. The value is never `tostring`-ed here.
    fn call_key(&self, expr: &Expr) -> Result<String, TranspileError> {
        let mut inner = expr;
        while let Expr::Paren(e) = inner {
            inner = e;
        }
        if let Expr::VarRef { name, scope } = inner {
            return Ok(format!(
                "act:call_key({})",
                Self::dynamic_ref_args(name, scope)?
            ));
        }
        let value = self.expr_to_string(expr)?;
        Ok(match Self::operand_desc(expr)? {
            Some(desc) => format!(
                "act:call_key({}, nil, {})",
                value,
                StringLiteralizer::literalize(&desc)?
            ),
            None => format!("act:call_key({})", value),
        })
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
        // Actor reference by name string: act:actor_proxy("アクター")
        let actor = &format!("act:actor_proxy({})", StringLiteralizer::literalize(actor)?);
        match action {
            Action::Talk { text, .. } => {
                // act:actor_proxy("アクター"):talk("文字列")
                let literal = StringLiteralizer::literalize(text)?;
                self.writeln(&format!("{}:talk({})", actor, literal))?;
            }
            Action::WordRef {
                name: word_name, ..
            } => {
                // act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word("単語名"))
                let word_literal = StringLiteralizer::literalize(word_name)?;
                self.writeln(&format!("{}:talk({}:word({}))", actor, actor, word_literal))?;
            }
            Action::VarRef { name, scope, .. } => {
                // Variable interpolation: generate talk with concatenation
                match scope {
                    VarScope::Property => {
                        let prop_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!(
                            "{}:talk(tostring(act:get_property({})))",
                            actor, prop_literal
                        ))?;
                    }
                    _ => {
                        // SAFETY: VarScope::Property is handled in the arm above;
                        // resolve_var_path returns Err for Property as a defensive guard.
                        // Pass the value as-is: talk() renders nil as empty and warns with
                        // the variable path (2nd arg) instead of printing "nil".
                        let var_path = Self::resolve_var_path(name, scope)?;
                        let path_literal = StringLiteralizer::literalize(&var_path)?;
                        self.writeln(&format!("{}:talk({}, {})", actor, var_path, path_literal))?;
                    }
                }
            }
            Action::FnCall {
                name, args, scope, ..
            } => {
                let args_str = self.generate_args_string(args)?;
                match scope {
                    pasta_dsl::parser::FnScope::Local => {
                        // act:actor_proxy("アクター"):expr_fn("関数名", 引数...)
                        // Outer parens keep only the first return value; talk() renders nil as empty.
                        let name_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!(
                            "{}:talk(({}:expr_fn({}{})))",
                            actor,
                            actor,
                            name_literal,
                            format_args_suffix(&args_str)
                        ))?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // act:global_fn("関数名", 引数...)
                        self.writeln(&format!(
                            "{}:talk(({}))",
                            actor,
                            global_fn_call(name, &args_str)?
                        ))?;
                    }
                }
            }
            Action::SakuraScript { script, .. } => {
                // SakuraScript is output as act:actor_proxy("アクター"):sakura_script()
                let literal = StringLiteralizer::literalize(script)?;
                self.writeln(&format!("{}:sakura_script({})", actor, literal))?;
            }
            Action::Escape {
                sequence: escape, ..
            } => {
                // `\\`/`\%` talk both chars (the sakura tokenizer treats them as one
                // literal unit); `＠＠`/`＄＄` talk only the second char.
                if let Some(c) = escape.chars().nth(1) {
                    let text = if escape.starts_with('\\') {
                        escape.clone()
                    } else {
                        c.to_string()
                    };
                    let literal = StringLiteralizer::literalize(&text)?;
                    self.writeln(&format!("{}:talk({})", actor, literal))?;
                }
            }
            Action::DynamicWordRef {
                var_name,
                var_scope,
                ..
            } => {
                // act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word(var.x, "var.x"))
                let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                self.writeln(&format!("{}:talk({}:word({}))", actor, actor, ref_args))?;
            }
            Action::DynamicFnCall {
                var_name,
                var_scope,
                args,
                ..
            } => {
                // act:actor_proxy("アクター"):talk((act:actor_proxy("アクター"):expr_fn_var(var.f, "var.f", 引数...)))
                let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                let args_str = self.generate_args_string(args)?;
                self.writeln(&format!(
                    "{}:talk(({}:expr_fn_var({}{})))",
                    actor,
                    actor,
                    ref_args,
                    format_args_suffix(&args_str)
                ))?;
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
    /// Called inside the global scene's `do` block, before the scene function definitions.
    pub fn generate_local_word(&mut self, word: &KeyWords) -> Result<(), TranspileError> {
        self.generate_word_definition(word, "SCENE", ":")
    }
}

#[cfg(test)]
#[path = "element_gen_tests.rs"]
mod tests;
