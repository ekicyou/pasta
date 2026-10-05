//! Expression code generation: values, function calls, arguments, and binary chains.

use super::LuaCodeGenerator;
use crate::error::TranspileError;
use crate::string_literalizer::StringLiteralizer;
use pasta_dsl::parser::{Args, BinOp, Expr};
use std::io::Write;

/// Format optional arguments suffix: empty when no args, otherwise ", arg1, arg2, ..."
pub(super) fn format_args_suffix(args_str: &str) -> String {
    if args_str.is_empty() {
        String::new()
    } else {
        format!(", {}", args_str)
    }
}

/// `＠＊名前（…）` call: `act:global_fn("名前", 引数...)` (name as a string literal).
pub(super) fn global_fn_call(name: &str, args_str: &str) -> Result<String, TranspileError> {
    Ok(format!(
        "act:global_fn({}{})",
        StringLiteralizer::literalize(name)?,
        format_args_suffix(args_str)
    ))
}

/// Flatten a left-assoc binary chain into terms and operators. Only the left
/// spine is unrolled; a right-hand operand stays one term.
fn flatten_binary<'e>(expr: &'e Expr, terms: &mut Vec<&'e Expr>, ops: &mut Vec<BinOp>) {
    if let Expr::Binary { op, lhs, rhs } = expr {
        flatten_binary(lhs, terms, ops);
        ops.push(*op);
        terms.push(rhs);
    } else {
        terms.push(expr);
    }
}

/// Fold height of a binary operator: higher folds first, each level left to right.
fn precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Mul | BinOp::Div | BinOp::Mod => 2,
        BinOp::Add | BinOp::Sub => 1,
        BinOp::Concat => 0,
    }
}

/// One operation: `act:arith("op", 左, 右, 左の説明, 右の説明)` or
/// `act:concat(左, 右, 左の説明, 右の説明)`. Descriptions are omitted when
/// both are absent, and `nil` fills a missing left one. The result is a
/// nested operation, so it carries no description of its own.
fn binary_node(
    op: BinOp,
    (lhs, lhs_desc): (String, Option<String>),
    (rhs, rhs_desc): (String, Option<String>),
) -> (String, Option<String>) {
    let call = match op {
        BinOp::Add => "act:arith(\"+\", ",
        BinOp::Sub => "act:arith(\"-\", ",
        BinOp::Mul => "act:arith(\"*\", ",
        BinOp::Div => "act:arith(\"/\", ",
        BinOp::Mod => "act:arith(\"%\", ",
        BinOp::Concat => "act:concat(",
    };
    let descs = match (lhs_desc, rhs_desc) {
        (None, None) => String::new(),
        (Some(l), None) => format!(", {}", l),
        (None, Some(r)) => format!(", nil, {}", r),
        (Some(l), Some(r)) => format!(", {}, {}", l, r),
    };
    (format!("{}{}, {}{})", call, lhs, rhs, descs), None)
}

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Generate an expression (delegates to `generate_expr_to_buffer`).
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> Result<(), TranspileError> {
        let mut buf = Vec::new();
        self.generate_expr_to_buffer(expr, &mut buf)?;
        self.writer.write_all(&buf)?;
        Ok(())
    }

    /// Render an expression to a `String` (helper over `generate_expr_to_buffer`).
    pub(super) fn expr_to_string(&self, expr: &Expr) -> Result<String, TranspileError> {
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
                        // act:global_fn("関数名", 引数...)
                        write!(buf, "{}", global_fn_call(name, &args_str)?)?;
                    }
                }
            }
            Expr::Paren(inner) => {
                write!(buf, "(")?;
                self.generate_expr_to_buffer(inner, buf)?;
                write!(buf, ")")?;
            }
            Expr::DynamicFnCall {
                var_name,
                var_scope,
                args,
            } => {
                // act:expr_fn_var(var.f, "var.f", 引数...)
                let ref_args = Self::dynamic_ref_args(var_name, var_scope)?;
                let args_str = self.generate_args_string(args)?;
                write!(
                    buf,
                    "act:expr_fn_var({}{})",
                    ref_args,
                    format_args_suffix(&args_str)
                )?;
            }
            Expr::Binary { .. } => {
                write!(buf, "{}", self.binary_to_string(expr)?)?;
            }
        }

        Ok(())
    }

    /// Render a binary chain as nested `act:arith` / `act:concat` calls.
    ///
    /// The parser builds a precedence-less left-assoc tree (`1＋2＆3` is
    /// `(1＋2)＆3`), so the chain is flattened back to terms and operators and
    /// regrouped by `precedence`: the highest level folds first, each level
    /// left to right (`＊／％`, then `＋－`, then `＆`). A `Paren` is one term;
    /// a right-hand `Binary` (never produced by the parser) is also one term.
    fn binary_to_string(&self, expr: &Expr) -> Result<String, TranspileError> {
        let mut terms = Vec::new();
        let mut ops = Vec::new();
        flatten_binary(expr, &mut terms, &mut ops);

        let mut terms = terms
            .into_iter()
            .map(|t| self.binary_operand(t))
            .collect::<Result<Vec<_>, _>>()?;
        while let Some(level) = ops.iter().map(|&op| precedence(op)).max() {
            let mut rest = terms.into_iter();
            let mut folded = vec![rest.next().expect("binary chain has a first term")];
            let mut lower_ops = Vec::new();
            for (op, rhs) in ops.into_iter().zip(rest) {
                if precedence(op) == level {
                    let lhs = folded.pop().expect("folded is never empty");
                    folded.push(binary_node(op, lhs, rhs));
                } else {
                    lower_ops.push(op);
                    folded.push(rhs);
                }
            }
            terms = folded;
            ops = lower_ops;
        }
        let (code, _) = terms.pop().expect("binary chain folds to one term");
        Ok(code)
    }

    /// One operand: its Lua code and its warning description as a string
    /// literal (variable path, `@名前()`, `@*名前()`, `@$パス()`), or `None`
    /// for literals and nested operations.
    fn binary_operand(&self, expr: &Expr) -> Result<(String, Option<String>), TranspileError> {
        let code = self.expr_to_string(expr)?;
        let desc = Self::operand_desc(expr)?
            .map(|d| StringLiteralizer::literalize(&d))
            .transpose()?;
        Ok((code, desc))
    }

    fn operand_desc(expr: &Expr) -> Result<Option<String>, TranspileError> {
        Ok(match expr {
            Expr::VarRef { name, scope } => Some(Self::resolve_var_path(name, scope)?),
            Expr::FnCall {
                name,
                scope: pasta_dsl::parser::FnScope::Local,
                ..
            } => Some(format!("@{}()", name)),
            Expr::FnCall {
                name,
                scope: pasta_dsl::parser::FnScope::Global,
                ..
            } => Some(format!("@*{}()", name)),
            Expr::DynamicFnCall {
                var_name,
                var_scope,
                ..
            } => Some(format!(
                "@${}()",
                Self::resolve_var_path(var_name, var_scope)?
            )),
            Expr::Paren(inner) => Self::operand_desc(inner)?,
            _ => None,
        })
    }

    /// Generate arguments as a string.
    pub(super) fn generate_args_string(&self, args: &Args) -> Result<String, TranspileError> {
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
}

#[cfg(test)]
#[path = "expr_gen_tests.rs"]
mod tests;
