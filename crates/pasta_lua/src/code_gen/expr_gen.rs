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

/// Operand value kind known at generation time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StaticKind {
    /// Number literal or arithmetic result (always a number)
    Number,
    /// String literal, blank string, or concat result (always a string)
    String,
    /// Variable reference or function call (unknown until run time)
    Unknown,
}

/// One term of a binary chain: its Lua code, its warning description as a
/// string literal, and its static kind (a paren takes its content's).
struct Operand {
    code: String,
    desc: Option<String>,
    kind: StaticKind,
}

/// Static kind of an operand expression. A binary chain is a string when it
/// has a `＆` on its left spine (concat folds last), otherwise a number.
fn static_kind(expr: &Expr) -> StaticKind {
    match expr {
        Expr::Integer(_) | Expr::Float(_) => StaticKind::Number,
        Expr::String(_) | Expr::BlankString => StaticKind::String,
        Expr::Paren(inner) => static_kind(inner),
        Expr::Binary { .. } => {
            let mut terms = Vec::new();
            let mut ops = Vec::new();
            flatten_binary(expr, &mut terms, &mut ops);
            if ops.contains(&BinOp::Concat) {
                StaticKind::String
            } else {
                StaticKind::Number
            }
        }
        Expr::VarRef { .. } | Expr::FnCall { .. } | Expr::DynamicFnCall { .. } => {
            StaticKind::Unknown
        }
    }
}

/// One operation: `(左 op 右)` with a space on both sides of the operator
/// (`(10 - -3)` never becomes a `--` comment). Arithmetic passes operands that
/// are not statically numbers through `PASTA.num("op", 値[, 説明])`; concat
/// passes operands that are not statically strings through
/// `PASTA.str(値[, 説明])`. The result carries no description of its own.
fn binary_node(op: BinOp, lhs: Operand, rhs: Operand) -> Operand {
    let (sym, kind) = match op {
        BinOp::Add => ("+", StaticKind::Number),
        BinOp::Sub => ("-", StaticKind::Number),
        BinOp::Mul => ("*", StaticKind::Number),
        BinOp::Div => ("/", StaticKind::Number),
        BinOp::Mod => ("%", StaticKind::Number),
        BinOp::Concat => ("..", StaticKind::String),
    };
    let wrap = |o: Operand| {
        let desc = o.desc.map(|d| format!(", {}", d)).unwrap_or_default();
        if o.kind == kind {
            o.code
        } else if kind == StaticKind::Number {
            format!("PASTA.num(\"{}\", {}{})", sym, o.code, desc)
        } else {
            format!("PASTA.str({}{})", o.code, desc)
        }
    };
    Operand {
        code: format!("({} {} {})", wrap(lhs), sym, wrap(rhs)),
        desc: None,
        kind,
    }
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

    /// Render a binary chain as nested `(左 op 右)` operations.
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
        Ok(terms.pop().expect("binary chain folds to one term").code)
    }

    /// One operand: its Lua code, its warning description as a string
    /// literal (variable path, `@名前()`, `@*名前()`, `@$パス()`) or `None`
    /// for literals and nested operations, and its static kind.
    fn binary_operand(&self, expr: &Expr) -> Result<Operand, TranspileError> {
        Ok(Operand {
            code: self.expr_to_string(expr)?,
            desc: Self::operand_desc(expr)?
                .map(|d| StringLiteralizer::literalize(&d))
                .transpose()?,
            kind: static_kind(expr),
        })
    }

    pub(super) fn operand_desc(expr: &Expr) -> Result<Option<String>, TranspileError> {
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
