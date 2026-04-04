//! Element-level code generation: variables, calls, actions, expressions, words.

use super::LuaCodeGenerator;
use crate::error::TranspileError;
use crate::string_literalizer::StringLiteralizer;
use pasta_dsl::parser::{
    Action, ActionLine, Args, CallScene, CodeBlock, ContinueAction, Expr, KeyWords, SetValue,
    VarScope, VarSet,
};
use std::io::Write;

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Generate variable assignment (Requirement 3d).
    ///
    /// Local: `var.変数名 = 値`
    /// Global: `save.変数名 = 値`
    pub fn generate_var_set(&mut self, var_set: &VarSet) -> Result<(), TranspileError> {
        match &var_set.name {
            Some(name) => {
                let var_path = match var_set.scope {
                    VarScope::Local => format!("var.{}", name),
                    VarScope::Global => format!("save.{}", name),
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
                        writeln!(self.writer)?;
                    }
                    SetValue::WordRef { name } => {
                        let word_literal = StringLiteralizer::literalize(name)?;
                        self.writeln(&format!("act:word({})", word_literal))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate scene call (Requirement 3d, 3g).
    ///
    /// Generates: `act:call("モジュール名", "ラベル名", {}, table.unpack(args))`
    ///
    /// When `is_tail_call` is true, prepends `return` to enable Lua TCO.
    pub(super) fn generate_call_scene(
        &mut self,
        call_scene: &CallScene,
        is_tail_call: bool,
    ) -> Result<(), TranspileError> {
        // Generate argument list
        let args_str = if let Some(ref args) = call_scene.args {
            let mut parts = Vec::new();
            for arg in &args.items {
                match arg {
                    pasta_dsl::parser::Arg::Positional(expr) => {
                        let mut buf = Vec::new();
                        self.generate_expr_to_buffer(expr, &mut buf)?;
                        parts.push(String::from_utf8(buf).unwrap_or_default());
                    }
                    pasta_dsl::parser::Arg::Keyword { key: _, value } => {
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

        // Generate call statement based on target type
        let call_stmt = match &call_scene.target {
            pasta_dsl::parser::CallTarget::Static(name) => {
                format!(
                    "act:call(SCENE.__global_name__, \"{}\", {{}}, {})",
                    name, args_str
                )
            }
            pasta_dsl::parser::CallTarget::Dynamic(expr) => {
                let mut buf = Vec::new();
                self.generate_expr_to_buffer(expr, &mut buf)?;
                let expr_str = String::from_utf8(buf).unwrap_or_default();
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
    pub fn generate_action(&mut self, action: &Action, actor: &str) -> Result<(), TranspileError> {
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
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
                        ))?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // GLOBAL.関数名(act, 引数...)
                        self.writeln(&format!(
                            "act.{}:talk(tostring(GLOBAL.{}(act{})))",
                            actor,
                            name,
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
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
                match scope {
                    pasta_dsl::parser::FnScope::Local => {
                        // act:expr_fn("関数名", 引数...)
                        let name_literal = StringLiteralizer::literalize(name)?;
                        write!(
                            self.writer,
                            "act:expr_fn({}{})",
                            name_literal,
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
                        )?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // GLOBAL.関数名(act, 引数...)
                        write!(
                            self.writer,
                            "GLOBAL.{}(act{})",
                            name,
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
                        )?;
                    }
                }
            }
            Expr::Paren(inner) => {
                write!(self.writer, "(")?;
                self.generate_expr(inner)?;
                write!(self.writer, ")")?;
            }
            Expr::Binary { op, lhs, rhs } => {
                self.generate_expr(lhs)?;
                let op_str = match op {
                    pasta_dsl::parser::BinOp::Add => " + ",
                    pasta_dsl::parser::BinOp::Sub => " - ",
                    pasta_dsl::parser::BinOp::Mul => " * ",
                    pasta_dsl::parser::BinOp::Div => " / ",
                    pasta_dsl::parser::BinOp::Mod => " % ",
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
                match scope {
                    pasta_dsl::parser::FnScope::Local => {
                        // act:expr_fn("関数名", 引数...)
                        let name_literal = StringLiteralizer::literalize(name)?;
                        write!(
                            buf,
                            "act:expr_fn({}{})",
                            name_literal,
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
                        )?;
                    }
                    pasta_dsl::parser::FnScope::Global => {
                        // GLOBAL.関数名(act, 引数...)
                        write!(
                            buf,
                            "GLOBAL.{}(act{})",
                            name,
                            if args_str.is_empty() {
                                String::new()
                            } else {
                                format!(", {}", args_str)
                            }
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
            match arg {
                pasta_dsl::parser::Arg::Positional(expr) => {
                    let mut buf = Vec::new();
                    self.generate_expr_to_buffer(expr, &mut buf)?;
                    parts.push(String::from_utf8(buf).unwrap_or_default());
                }
                pasta_dsl::parser::Arg::Keyword { key: _, value } => {
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

    /// Generate global word definition (Requirement 2.1, Task 4.2).
    ///
    /// Generates: `PASTA.create_word("key"):entry("value1", "value2", ...)`
    ///
    /// Called at file level, outside of any do block.
    pub fn generate_global_word(&mut self, word: &KeyWords) -> Result<(), TranspileError> {
        if word.words.is_empty() {
            return Ok(());
        }

        // Generate entry values as string literals
        let values: Vec<String> = word
            .words
            .iter()
            .map(|w| StringLiteralizer::literalize(w))
            .collect::<Result<Vec<_>, _>>()?;
        let entry = values.join(", ");

        for name in &word.names {
            self.writeln(&format!(
                "PASTA.create_word({}):entry({})",
                StringLiteralizer::literalize(name)?,
                entry
            ))?;
        }

        Ok(())
    }

    /// Generate local word definition for a scene (Requirement 2.2, Task 4.3).
    ///
    /// Generates: `SCENE:create_word("key"):entry("value1", "value2", ...)`
    ///
    /// Called inside a local scene function, after init_scene.
    pub fn generate_local_word(&mut self, word: &KeyWords) -> Result<(), TranspileError> {
        if word.words.is_empty() {
            return Ok(());
        }

        // Generate entry values as string literals
        let values: Vec<String> = word
            .words
            .iter()
            .map(|w| StringLiteralizer::literalize(w))
            .collect::<Result<Vec<_>, _>>()?;
        let entry = values.join(", ");

        for name in &word.names {
            self.writeln(&format!(
                "SCENE:create_word({}):entry({})",
                StringLiteralizer::literalize(name)?,
                entry
            ))?;
        }

        Ok(())
    }
}
