//! Lua code generator for Pasta DSL.
//!
//! This module generates Lua source code from Pasta AST nodes.
//! Implements Requirements 1, 3a-3g for Lua code generation.

use pasta_core::parser::{
    Action, ActionLine, ActorScope, Args, AttrValue, CallScene, CodeBlock, ContinueAction, Expr,
    GlobalSceneScope, LocalSceneItem, LocalSceneScope, SceneActorItem, SetValue, VarScope, VarSet,
};
use pasta_core::registry::SceneRegistry;

use super::config::LineEnding;
use super::context::TranspileContext;
use super::error::TranspileError;
use super::string_literalizer::StringLiteralizer;

use std::collections::HashMap;
use std::io::Write;

/// Lua code generator.
///
/// Generates Lua source code from Pasta AST nodes.
pub struct LuaCodeGenerator<'a, W: Write> {
    /// Output writer
    writer: &'a mut W,
    /// Current indentation level
    indent_level: usize,
    /// Current module name for scene resolution
    current_module: String,
    /// Line ending style
    line_ending: LineEnding,
}

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Create a new Lua code generator.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            indent_level: 0,
            current_module: String::new(),
            line_ending: LineEnding::default(),
        }
    }

    /// Create a new Lua code generator with specified line ending.
    pub fn with_line_ending(writer: &'a mut W, line_ending: LineEnding) -> Self {
        Self {
            writer,
            indent_level: 0,
            current_module: String::new(),
            line_ending,
        }
    }

    /// Write indentation at current level.
    fn write_indent(&mut self) -> Result<(), TranspileError> {
        let indent = "    ".repeat(self.indent_level);
        write!(self.writer, "{}", indent)?;
        Ok(())
    }

    /// Write a line with current indentation.
    fn writeln(&mut self, s: &str) -> Result<(), TranspileError> {
        self.write_indent()?;
        write!(self.writer, "{}{}", s, self.line_ending.as_str())?;
        Ok(())
    }

    /// Write a blank line without indentation.
    fn write_blank_line(&mut self) -> Result<(), TranspileError> {
        write!(self.writer, "{}", self.line_ending.as_str())?;
        Ok(())
    }

    /// Write without indentation.
    fn write_raw(&mut self, s: &str) -> Result<(), TranspileError> {
        write!(self.writer, "{}", s)?;
        Ok(())
    }

    /// Increase indentation level.
    fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level.
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Write the Lua header (require statement).
    pub fn write_header(&mut self) -> Result<(), TranspileError> {
        self.writeln("local PASTA = require \"pasta\"")?;
        self.write_blank_line()?;
        Ok(())
    }

    /// Generate actor definition block (Requirement 3a).
    ///
    /// Generates:
    /// ```lua
    /// do
    ///     local ACTOR = PASTA.create_actor("アクター名")
    ///     ACTOR.属性 = [=[値]=]
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

        // Generate word definitions as actor attributes
        for word_def in &actor.words {
            // Each word definition becomes an actor attribute
            // In Pasta, actor words like ＄通常：\s[0] become ACTOR.通常 = [=[\s[0]]=]
            if let Some(first_word) = word_def.words.first() {
                let literal = StringLiteralizer::literalize_with_span(first_word, &word_def.span)?;
                self.writeln(&format!("ACTOR.{} = {}", word_def.name, literal))?;
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
        scene_counter: usize,
        _context: &TranspileContext,
        _file_attrs: &HashMap<String, AttrValue>,
    ) -> Result<(), TranspileError> {
        let sanitized_name = SceneRegistry::sanitize_name(&scene.name);
        let module_name = format!("{}{}", sanitized_name, scene_counter);

        // Store current module name for Call scene resolution
        self.current_module = module_name.clone();

        // do block for scope separation (Requirement 1)
        self.writeln("do")?;
        self.indent();

        // Create scene
        self.writeln(&format!(
            "local SCENE = PASTA.create_scene(\"{}\")",
            module_name
        ))?;
        self.write_blank_line()?;

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
            format!("__{}_{}__", sanitized, counter)
        } else {
            "__start__".to_string()
        };

        self.writeln(&format!("function SCENE.{}(ctx, ...)", fn_name))?;
        self.indent();

        // Session initialization
        self.writeln("local args = { ... }")?;

        // Generate actor initialization block for __start__ only (counter == 0)
        // Order: clear_spot -> set_spot(s) -> create_session (Requirement 1.1, 1.2)
        if counter == 0 && !actors.is_empty() {
            // clear_spot at the start of actor initialization block (Requirement 2.1)
            self.writeln("PASTA.clear_spot()")?;
            // set_spot with new format: PASTA.set_spot("name", number) (Requirement 3.1, 3.2)
            for actor in actors {
                self.writeln(&format!(
                    r#"PASTA.set_spot("{}", {})"#,
                    actor.name, actor.number
                ))?;
            }
        }

        // create_session comes after actor initialization (Requirement 1.2)
        self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
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

    /// Generate local scene items (action lines, var sets, calls).
    fn generate_local_scene_items(
        &mut self,
        items: &[LocalSceneItem],
    ) -> Result<(), TranspileError> {
        let mut last_actor: Option<String> = None;

        for item in items {
            match item {
                LocalSceneItem::VarSet(var_set) => {
                    self.generate_var_set(var_set)?;
                }
                LocalSceneItem::CallScene(call_scene) => {
                    self.generate_call_scene(call_scene)?;
                }
                LocalSceneItem::ActionLine(action_line) => {
                    self.generate_action_line(action_line, &mut last_actor)?;
                }
                LocalSceneItem::ContinueAction(continue_action) => {
                    self.generate_continue_action(continue_action, &last_actor)?;
                }
            }
        }

        Ok(())
    }

    /// Generate variable assignment (Requirement 3d).
    ///
    /// Local: `var.変数名 = 値`
    /// Global: `save.変数名 = 値`
    fn generate_var_set(&mut self, var_set: &VarSet) -> Result<(), TranspileError> {
        let var_path = match var_set.scope {
            VarScope::Local => format!("var.{}", var_set.name),
            VarScope::Global => format!("save.{}", var_set.name),
            VarScope::Args(_) => {
                // Cannot assign to scene arguments
                return Err(TranspileError::invalid_ast(
                    &var_set.span,
                    "Cannot assign to scene argument",
                ));
            }
        };

        match &var_set.value {
            SetValue::Expr(expr) => {
                self.write_indent()?;
                self.write_raw(&format!("{} = ", var_path))?;
                self.generate_expr(expr)?;
                writeln!(self.writer)?;
            }
            SetValue::WordRef { name } => {
                // Generate: var.変数名 = act:word("単語名") or save.変数名 = act:word("単語名")
                self.writeln(&format!("{} = act:word(\"{}\")", var_path, name))?;
            }
        }

        Ok(())
    }

    /// Generate scene call (Requirement 3d, 3g).
    ///
    /// Generates: `act:call("モジュール名", "ラベル名", {}, table.unpack(args))`
    fn generate_call_scene(&mut self, call_scene: &CallScene) -> Result<(), TranspileError> {
        let target = &call_scene.target;

        // Generate argument list
        let args_str = if let Some(ref args) = call_scene.args {
            let mut parts = Vec::new();
            for arg in &args.items {
                match arg {
                    pasta_core::parser::Arg::Positional(expr) => {
                        let mut buf = Vec::new();
                        self.generate_expr_to_buffer(expr, &mut buf)?;
                        parts.push(String::from_utf8(buf).unwrap_or_default());
                    }
                    pasta_core::parser::Arg::Keyword { key: _, value } => {
                        let mut buf = Vec::new();
                        self.generate_expr_to_buffer(value, &mut buf)?;
                        parts.push(String::from_utf8(buf).unwrap_or_default());
                    }
                }
            }
            if parts.is_empty() {
                "table.unpack(args)".to_string()
            } else {
                format!("{}, table.unpack(args)", parts.join(", "))
            }
        } else {
            "table.unpack(args)".to_string()
        };

        self.writeln(&format!(
            "act:call(\"{}\", \"{}\", {{}}, {})",
            self.current_module, target, args_str
        ))?;

        Ok(())
    }

    /// Generate action line (with speaker).
    fn generate_action_line(
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
    fn generate_continue_action(
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
    fn generate_action(&mut self, action: &Action, actor: &str) -> Result<(), TranspileError> {
        match action {
            Action::Talk { text, .. } => {
                // act.アクター:talk("文字列")
                let literal = StringLiteralizer::literalize(text)?;
                self.writeln(&format!("act.{}:talk({})", actor, literal))?;
            }
            Action::WordRef {
                name: word_name, ..
            } => {
                // act.アクター:word("単語名")
                self.writeln(&format!("act.{}:word(\"{}\")", actor, word_name))?;
            }
            Action::VarRef { name, scope, .. } => {
                // Variable interpolation: generate talk with concatenation
                let var_path = match scope {
                    VarScope::Local => format!("var.{}", name),
                    VarScope::Global => format!("save.{}", name),
                    VarScope::Args(index) => format!("args[{}]", index + 1), // 0-indexed to 1-indexed
                };
                self.writeln(&format!("act.{}:talk(tostring({}))", actor, var_path))?;
            }
            Action::FnCall {
                name, args, scope, ..
            } => {
                // SCENE:関数名(ctx, 引数...)
                let args_str = self.generate_args_string(args)?;
                let prefix = match scope {
                    pasta_core::parser::FnScope::Local => "SCENE:",
                    pasta_core::parser::FnScope::Global => "SCENE:", // Same for now
                };
                self.writeln(&format!(
                    "act.{}:talk(tostring({}{}(ctx{})))",
                    actor,
                    prefix,
                    name,
                    if args_str.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", args_str)
                    }
                ))?;
            }
            Action::SakuraScript { script, .. } => {
                // SakuraScript is output as talk
                let literal = StringLiteralizer::literalize(script)?;
                self.writeln(&format!("act.{}:talk({})", actor, literal))?;
            }
            Action::Escape {
                sequence: escape, ..
            } => {
                // Extract the escaped character (second char)
                if let Some(c) = escape.chars().nth(1) {
                    self.writeln(&format!("act.{}:talk(\"{}\")", actor, c))?;
                }
            }
        }

        Ok(())
    }

    /// Generate an expression.
    fn generate_expr(&mut self, expr: &Expr) -> Result<(), TranspileError> {
        match expr {
            Expr::Integer(n) => {
                write!(self.writer, "{}", n)?;
            }
            Expr::Float(f) => {
                write!(self.writer, "{}", f)?;
            }
            Expr::String(s) => {
                let literal = StringLiteralizer::literalize(s)?;
                write!(self.writer, "{}", literal)?;
            }
            Expr::BlankString => {
                write!(self.writer, "\"\"")?;
            }
            Expr::VarRef { name, scope } => {
                let var_path = match scope {
                    VarScope::Local => format!("var.{}", name),
                    VarScope::Global => format!("save.{}", name),
                    VarScope::Args(index) => format!("args[{}]", index + 1),
                };
                write!(self.writer, "{}", var_path)?;
            }
            Expr::FnCall { name, args, scope } => {
                let args_str = self.generate_args_string(args)?;
                let prefix = match scope {
                    pasta_core::parser::FnScope::Local => "SCENE.",
                    pasta_core::parser::FnScope::Global => "SCENE.",
                };
                write!(
                    self.writer,
                    "{}{}(ctx{})",
                    prefix,
                    name,
                    if args_str.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", args_str)
                    }
                )?;
            }
            Expr::Paren(inner) => {
                write!(self.writer, "(")?;
                self.generate_expr(inner)?;
                write!(self.writer, ")")?;
            }
            Expr::Binary { op, lhs, rhs } => {
                self.generate_expr(lhs)?;
                let op_str = match op {
                    pasta_core::parser::BinOp::Add => " + ",
                    pasta_core::parser::BinOp::Sub => " - ",
                    pasta_core::parser::BinOp::Mul => " * ",
                    pasta_core::parser::BinOp::Div => " / ",
                    pasta_core::parser::BinOp::Mod => " % ",
                };
                write!(self.writer, "{}", op_str)?;
                self.generate_expr(rhs)?;
            }
        }

        Ok(())
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
                let var_path = match scope {
                    VarScope::Local => format!("var.{}", name),
                    VarScope::Global => format!("save.{}", name),
                    VarScope::Args(index) => format!("args[{}]", index + 1),
                };
                write!(buf, "{}", var_path)?;
            }
            Expr::FnCall { name, args, scope } => {
                let args_str = self.generate_args_string(args)?;
                let prefix = match scope {
                    pasta_core::parser::FnScope::Local => "SCENE.",
                    pasta_core::parser::FnScope::Global => "SCENE.",
                };
                write!(
                    buf,
                    "{}{}(ctx{})",
                    prefix,
                    name,
                    if args_str.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", args_str)
                    }
                )?;
            }
            Expr::Paren(inner) => {
                write!(buf, "(")?;
                self.generate_expr_to_buffer(inner, buf)?;
                write!(buf, ")")?;
            }
            Expr::Binary { op, lhs, rhs } => {
                self.generate_expr_to_buffer(lhs, buf)?;
                let op_str = match op {
                    pasta_core::parser::BinOp::Add => " + ",
                    pasta_core::parser::BinOp::Sub => " - ",
                    pasta_core::parser::BinOp::Mul => " * ",
                    pasta_core::parser::BinOp::Div => " / ",
                    pasta_core::parser::BinOp::Mod => " % ",
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
            match arg {
                pasta_core::parser::Arg::Positional(expr) => {
                    let mut buf = Vec::new();
                    self.generate_expr_to_buffer(expr, &mut buf)?;
                    parts.push(String::from_utf8(buf).unwrap_or_default());
                }
                pasta_core::parser::Arg::Keyword { key: _, value } => {
                    let mut buf = Vec::new();
                    self.generate_expr_to_buffer(value, &mut buf)?;
                    parts.push(String::from_utf8(buf).unwrap_or_default());
                }
            }
        }
        Ok(parts.join(", "))
    }

    /// Generate code block (Requirement 3f).
    ///
    /// Outputs the code block content directly without transformation.
    pub fn generate_code_block(&mut self, block: &CodeBlock) -> Result<(), TranspileError> {
        // Output code content with proper indentation
        for line in block.content.lines() {
            self.writeln(line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_core::parser::{SetValue, Span, VarSet};

    #[allow(dead_code)]
    fn create_action_line(actor: &str, actions: Vec<Action>) -> ActionLine {
        ActionLine {
            actor: actor.to_string(),
            actions,
            span: Span::default(),
        }
    }

    #[test]
    fn test_generate_talk_action() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let action = Action::Talk {
            text: "こんにちは".to_string(),
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:talk(\"こんにちは\")"));
    }

    #[test]
    fn test_generate_word_ref_action() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let action = Action::WordRef {
            name: "挨拶".to_string(),
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:word(\"挨拶\")"));
    }

    #[test]
    fn test_generate_var_ref_local() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let action = Action::VarRef {
            name: "カウンタ".to_string(),
            scope: VarScope::Local,
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:talk(tostring(var.カウンタ))"));
    }

    #[test]
    fn test_generate_var_ref_global() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let action = Action::VarRef {
            name: "グローバル".to_string(),
            scope: VarScope::Global,
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:talk(tostring(save.グローバル))"));
    }

    #[test]
    fn test_generate_var_ref_args() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        // $0 in Pasta -> args[1] in Lua (0-indexed to 1-indexed)
        let action = Action::VarRef {
            name: "0".to_string(),
            scope: VarScope::Args(0),
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:talk(tostring(args[1]))"));
    }

    #[test]
    fn test_generate_escape_action() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let action = Action::Escape {
            sequence: "@@".to_string(),
            span: Span::default(),
        };
        codegen.generate_action(&action, "さくら").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("act.さくら:talk(\"@\")"));
    }

    #[test]
    fn test_generate_header() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        codegen.write_header().unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("local PASTA = require \"pasta\""));
    }

    // ========================================================================
    // VarSet with WordRef tests (Requirement 1.1, 1.2, 1.3, 4.1)
    // ========================================================================

    #[test]
    fn test_generate_var_set_wordref_local() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let var_set = VarSet {
            name: "場所".to_string(),
            scope: VarScope::Local,
            value: SetValue::WordRef {
                name: "場所".to_string(),
            },
            span: Span::default(),
        };
        codegen.generate_var_set(&var_set).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(
            result.contains("var.場所 = act:word(\"場所\")"),
            "Expected 'var.場所 = act:word(\"場所\")' but got: {}",
            result
        );
    }

    #[test]
    fn test_generate_var_set_wordref_global() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let var_set = VarSet {
            name: "グローバル".to_string(),
            scope: VarScope::Global,
            value: SetValue::WordRef {
                name: "単語".to_string(),
            },
            span: Span::default(),
        };
        codegen.generate_var_set(&var_set).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(
            result.contains("save.グローバル = act:word(\"単語\")"),
            "Expected 'save.グローバル = act:word(\"単語\")' but got: {}",
            result
        );
    }

    #[test]
    fn test_generate_var_set_wordref_args_error() {
        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let var_set = VarSet {
            name: "0".to_string(),
            scope: VarScope::Args(0),
            value: SetValue::WordRef {
                name: "単語".to_string(),
            },
            span: Span::default(),
        };
        let result = codegen.generate_var_set(&var_set);

        assert!(
            result.is_err(),
            "Expected error for Args scope WordRef assignment"
        );
        let err = result.unwrap_err();
        match err {
            TranspileError::InvalidAst { message, .. } => {
                assert!(
                    message.contains("Cannot assign to scene argument"),
                    "Expected 'Cannot assign to scene argument' error but got: {}",
                    message
                );
            }
            _ => panic!("Expected InvalidAst error but got: {:?}", err),
        }
    }

    #[test]
    fn test_generate_var_set_expr_still_works() {
        use pasta_core::parser::Expr;

        let mut output = Vec::new();
        let mut codegen = LuaCodeGenerator::new(&mut output);

        let var_set = VarSet {
            name: "カウンタ".to_string(),
            scope: VarScope::Local,
            value: SetValue::Expr(Expr::Integer(10)),
            span: Span::default(),
        };
        codegen.generate_var_set(&var_set).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(
            result.contains("var.カウンタ = 10"),
            "Expected 'var.カウンタ = 10' but got: {}",
            result
        );
    }
}
