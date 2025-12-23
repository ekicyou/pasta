//! Code generator for transpiler2.
//!
//! This module generates Rune source code from parser2 AST nodes.

use crate::parser2::{
    Action, ActionLine, CallScene, CodeBlock, ContinueAction, Expr, GlobalSceneScope,
    LocalSceneItem, LocalSceneScope, VarScope, VarSet,
};
use crate::registry::SceneRegistry;

use super::context::TranspileContext2;
use super::error::TranspileError;

use std::io::Write;

/// Rune code generator.
///
/// Generates Rune source code from parser2 AST nodes.
pub struct CodeGenerator<'a, W: Write> {
    /// Output writer
    writer: &'a mut W,
    /// Current indentation level
    indent_level: usize,
}

impl<'a, W: Write> CodeGenerator<'a, W> {
    /// Create a new code generator.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            indent_level: 0,
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
        writeln!(self.writer, "{}", s)?;
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

    /// Generate a global scene module.
    ///
    /// Generates a Rune `pub mod` block containing the scene's functions.
    pub fn generate_global_scene(
        &mut self,
        scene: &GlobalSceneScope,
        scene_counter: usize,
        _context: &TranspileContext2,
    ) -> Result<(), TranspileError> {
        let sanitized_name = SceneRegistry::sanitize_name(&scene.name);
        let module_name = format!("{}_{}", sanitized_name, scene_counter);

        // Generate module header
        self.writeln(&format!("pub mod {} {{", module_name))?;
        self.indent();

        // Generate __start__ function or inline code if no local scenes
        if scene.local_scenes.is_empty() {
            // Empty scene - generate placeholder
            self.writeln("pub fn __start__(ctx, args) {")?;
            self.indent();
            self.writeln("// Empty scene")?;
            self.dedent();
            self.writeln("}")?;
        } else {
            // Generate local scenes
            for (idx, local_scene) in scene.local_scenes.iter().enumerate() {
                self.generate_local_scene(local_scene, idx)?;
            }
        }

        // Generate code blocks at module level (after all functions)
        for code_block in &scene.code_blocks {
            self.generate_code_block(code_block)?;
        }

        self.dedent();
        self.writeln("}")?;

        Ok(())
    }

    /// Generate a local scene function.
    pub fn generate_local_scene(
        &mut self,
        scene: &LocalSceneScope,
        index: usize,
    ) -> Result<(), TranspileError> {
        let fn_name = if let Some(ref name) = scene.name {
            let sanitized = SceneRegistry::sanitize_name(name);
            format!("{}_{}", sanitized, index + 1)
        } else {
            "__start__".to_string()
        };

        self.writeln(&format!("pub fn {}(ctx, args) {{", fn_name))?;
        self.indent();

        // Generate local scene items
        self.generate_local_scene_items(&scene.items)?;

        // Generate code blocks inline
        for code_block in &scene.code_blocks {
            self.generate_code_block(code_block)?;
        }

        self.dedent();
        self.writeln("}")?;

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

    /// Generate variable assignment.
    fn generate_var_set(&mut self, var_set: &VarSet) -> Result<(), TranspileError> {
        let var_path = match var_set.scope {
            VarScope::Local => format!("ctx.local.{}", var_set.name),
            VarScope::Global => format!("ctx.global.{}", var_set.name),
        };

        self.write_indent()?;
        self.write_raw(&format!("{} = ", var_path))?;
        self.generate_expr(&var_set.value)?;
        writeln!(self.writer, ";")?;

        Ok(())
    }

    /// Generate scene call.
    fn generate_call_scene(&mut self, call_scene: &CallScene) -> Result<(), TranspileError> {
        self.writeln(&format!(
            "for a in pasta::call(ctx, \"{}\") {{ yield a; }}",
            call_scene.target
        ))?;
        Ok(())
    }

    /// Generate action line (with speaker).
    fn generate_action_line(
        &mut self,
        action_line: &ActionLine,
        last_actor: &mut Option<String>,
    ) -> Result<(), TranspileError> {
        // Check if we need to change speaker
        let needs_speaker_change = last_actor.as_ref() != Some(&action_line.actor);

        if needs_speaker_change {
            self.writeln(&format!("yield change_speaker(\"{}\");", action_line.actor))?;
            *last_actor = Some(action_line.actor.clone());
        }

        // Generate actions
        for action in &action_line.actions {
            self.generate_action(action)?;
        }

        Ok(())
    }

    /// Generate continue action line (without speaker).
    fn generate_continue_action(
        &mut self,
        continue_action: &ContinueAction,
        last_actor: &Option<String>,
    ) -> Result<(), TranspileError> {
        if last_actor.is_none() {
            return Err(TranspileError::invalid_continuation(&continue_action.span));
        }

        // Generate actions (speaker is inherited)
        for action in &continue_action.actions {
            self.generate_action(action)?;
        }

        Ok(())
    }

    /// Generate a single action.
    fn generate_action(&mut self, action: &Action) -> Result<(), TranspileError> {
        match action {
            Action::Talk(text) => {
                // Escape backslashes and quotes for Rune string
                let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
                self.writeln(&format!("yield Talk(\"{}\");", escaped))?;
            }
            Action::WordRef(word_name) => {
                self.writeln(&format!(
                    "yield Talk(pasta_stdlib::word(\"{}\", \"{}\", []));",
                    "", // module - empty for now
                    word_name
                ))?;
            }
            Action::VarRef { name, scope } => {
                let var_path = match scope {
                    VarScope::Local => format!("ctx.local.{}", name),
                    VarScope::Global => format!("ctx.global.{}", name),
                };
                self.writeln(&format!("yield Talk(`${{{}}}`);", var_path))?;
            }
            Action::FnCall {
                name,
                args: _,
                scope: _,
            } => {
                // For now, generate simple function call with yield
                self.writeln(&format!("for a in {}(ctx, args) {{ yield a; }}", name))?;
            }
            Action::SakuraScript(script) => {
                let escaped = script.replace('\\', "\\\\").replace('"', "\\\"");
                self.writeln(&format!("yield emit_sakura_script(\"{}\");", escaped))?;
            }
            Action::Escape(escape) => {
                // Extract second character (the escaped character)
                if let Some(c) = escape.chars().nth(1) {
                    self.writeln(&format!("yield Talk(\"{}\");", c))?;
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
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                write!(self.writer, "\"{}\"", escaped)?;
            }
            Expr::BlankString => {
                write!(self.writer, "\"\"")?;
            }
            Expr::VarRef { name, scope } => {
                let var_path = match scope {
                    VarScope::Local => format!("ctx.local.{}", name),
                    VarScope::Global => format!("ctx.global.{}", name),
                };
                write!(self.writer, "{}", var_path)?;
            }
            Expr::FnCall {
                name,
                args: _,
                scope: _,
            } => {
                write!(self.writer, "{}(ctx, args)", name)?;
            }
            Expr::Paren(inner) => {
                write!(self.writer, "(")?;
                self.generate_expr(inner)?;
                write!(self.writer, ")")?;
            }
            Expr::Binary { op, lhs, rhs } => {
                self.generate_expr(lhs)?;
                let op_str = match op {
                    crate::parser2::BinOp::Add => " + ",
                    crate::parser2::BinOp::Sub => " - ",
                    crate::parser2::BinOp::Mul => " * ",
                    crate::parser2::BinOp::Div => " / ",
                    crate::parser2::BinOp::Mod => " % ",
                };
                write!(self.writer, "{}", op_str)?;
                self.generate_expr(rhs)?;
            }
        }

        Ok(())
    }

    /// Generate code block (with indent adjustment).
    fn generate_code_block(&mut self, block: &CodeBlock) -> Result<(), TranspileError> {
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
    use crate::parser2::Span;

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
        let mut codegen = CodeGenerator::new(&mut output);

        let action = Action::Talk("こんにちは".to_string());
        codegen.generate_action(&action).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("yield Talk(\"こんにちは\");"));
    }

    #[test]
    fn test_generate_escape_action() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let action = Action::Escape("@@".to_string());
        codegen.generate_action(&action).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("yield Talk(\"@\");"));
    }

    #[test]
    fn test_generate_var_ref_action() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let action = Action::VarRef {
            name: "count".to_string(),
            scope: VarScope::Local,
        };
        codegen.generate_action(&action).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("yield Talk(`${ctx.local.count}`);"));
    }

    #[test]
    fn test_generate_action_line_with_speaker_change() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let action_line = create_action_line("sakura", vec![Action::Talk("hello".to_string())]);
        let mut last_actor = None;

        codegen
            .generate_action_line(&action_line, &mut last_actor)
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("yield change_speaker(\"sakura\");"));
        assert!(result.contains("yield Talk(\"hello\");"));
        assert_eq!(last_actor, Some("sakura".to_string()));
    }

    #[test]
    fn test_generate_action_line_same_speaker() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let action_line = create_action_line("sakura", vec![Action::Talk("hello".to_string())]);
        let mut last_actor = Some("sakura".to_string());

        codegen
            .generate_action_line(&action_line, &mut last_actor)
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        // Should not have change_speaker since actor is the same
        assert!(!result.contains("yield change_speaker"));
        assert!(result.contains("yield Talk(\"hello\");"));
    }

    #[test]
    fn test_generate_continue_action_error() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let continue_action = ContinueAction {
            actions: vec![Action::Talk("continued".to_string())],
            span: Span::new(1, 1, 1, 10),
        };
        let last_actor = None;

        let result = codegen.generate_continue_action(&continue_action, &last_actor);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_expr_integer() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        codegen.generate_expr(&Expr::Integer(42)).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_generate_expr_binary() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let expr = Expr::Binary {
            op: crate::parser2::BinOp::Add,
            lhs: Box::new(Expr::Integer(1)),
            rhs: Box::new(Expr::Integer(2)),
        };
        codegen.generate_expr(&expr).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "1 + 2");
    }

    #[test]
    fn test_generate_var_set() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let var_set = VarSet {
            name: "x".to_string(),
            scope: VarScope::Local,
            value: Expr::Integer(10),
            span: Span::default(),
        };
        codegen.generate_var_set(&var_set).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("ctx.local.x = 10;"));
    }

    #[test]
    fn test_generate_call_scene() {
        let mut output = Vec::new();
        let mut codegen = CodeGenerator::new(&mut output);

        let call_scene = CallScene {
            target: "挨拶".to_string(),
            args: None,
            span: Span::default(),
        };
        codegen.generate_call_scene(&call_scene).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("for a in pasta::call(ctx, \"挨拶\") { yield a; }"));
    }
}
