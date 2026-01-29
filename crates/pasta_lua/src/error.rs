//! Error types for Pasta Lua transpiler.
//!
//! This module defines error types for the Lua code generation process.

use pasta_core::parser::Span;
use std::fmt;
use thiserror::Error;

/// Transpile error type for Lua code generation.
#[derive(Error, Debug)]
pub enum TranspileError {
    /// IO error during transpilation (writing to output).
    #[error("IO error during transpilation: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid AST structure.
    #[error("Invalid AST structure at {span}: {message}")]
    InvalidAst { span: SpanDisplay, message: String },

    /// Undefined scene reference.
    #[error("Undefined scene '{name}' at {span}")]
    UndefinedScene { name: String, span: SpanDisplay },

    /// Undefined word reference.
    #[error("Undefined word '{name}' at {span}")]
    UndefinedWord { name: String, span: SpanDisplay },

    /// Continuation action without actor.
    #[error("Continuation action without actor at {span}")]
    InvalidContinuation { span: SpanDisplay },

    /// String literal cannot be converted.
    #[error(
        "String literal cannot be converted at {span}: dangerous pattern detected in all formats"
    )]
    StringLiteralError { text: String, span: SpanDisplay },

    /// Too many local variables.
    #[error("Too many local variables in scope: {count} (max ~200) at {span}")]
    TooManyLocalVariables { count: usize, span: SpanDisplay },

    /// Unsupported feature.
    #[error("Unsupported feature: {feature} at {span}")]
    Unsupported { feature: String, span: SpanDisplay },
}

impl TranspileError {
    /// Create an InvalidAst error.
    pub fn invalid_ast(span: &Span, message: &str) -> Self {
        TranspileError::InvalidAst {
            span: SpanDisplay::from(*span),
            message: message.to_string(),
        }
    }

    /// Create an InvalidContinuation error.
    pub fn invalid_continuation(span: &Span) -> Self {
        TranspileError::InvalidContinuation {
            span: SpanDisplay::from(*span),
        }
    }

    /// Create a StringLiteralError.
    pub fn string_literal_error(span: &Span, text: &str) -> Self {
        TranspileError::StringLiteralError {
            text: text.to_string(),
            span: SpanDisplay::from(*span),
        }
    }

    /// Create an Unsupported error.
    pub fn unsupported(span: &Span, feature: &str) -> Self {
        TranspileError::Unsupported {
            span: SpanDisplay::from(*span),
            feature: feature.to_string(),
        }
    }
}

/// Wrapper for Span with Display implementation.
///
/// Format: `[L{start_line}:{start_col}-L{end_line}:{end_col}]`
#[derive(Debug, Clone, Copy)]
pub struct SpanDisplay {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl From<Span> for SpanDisplay {
    fn from(span: Span) -> Self {
        SpanDisplay {
            start_line: span.start_line,
            start_col: span.start_col,
            end_line: span.end_line,
            end_col: span.end_col,
        }
    }
}

impl fmt::Display for SpanDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[L{}:{}-L{}:{}]",
            self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}

/// Configuration errors for Lua library settings.
///
/// These errors occur during configuration parsing and validation,
/// particularly for the `[lua]` section's `libs` array.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Unknown library name in libs array.
    #[error(
        "Unknown library: {0}. Valid libraries: std_all, std_all_unsafe, std_coroutine, std_table, std_io, std_os, std_string, std_utf8, std_math, std_package, std_debug, assertions, testing, env, regex, json, yaml"
    )]
    UnknownLibrary(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_display_format() {
        let span = Span {
            start_line: 10,
            start_col: 5,
            end_line: 10,
            end_col: 23,
            start_byte: 0,
            end_byte: 18,
        };
        let display = SpanDisplay::from(span);
        assert_eq!(format!("{}", display), "[L10:5-L10:23]");
    }

    #[test]
    fn test_span_display_multiline() {
        let span = Span {
            start_line: 1,
            start_col: 1,
            end_line: 5,
            end_col: 10,
            start_byte: 0,
            end_byte: 50,
        };
        let display = SpanDisplay::from(span);
        assert_eq!(format!("{}", display), "[L1:1-L5:10]");
    }

    #[test]
    fn test_transpile_error_io() {
        let err =
            TranspileError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        assert!(format!("{}", err).contains("IO error"));
    }

    #[test]
    fn test_transpile_error_invalid_ast() {
        let span = Span::new(1, 1, 1, 10, 0, 10);
        let err = TranspileError::invalid_ast(&span, "test message");
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid AST structure"));
        assert!(msg.contains("[L1:1-L1:10]"));
        assert!(msg.contains("test message"));
    }

    #[test]
    fn test_transpile_error_invalid_continuation() {
        let span = Span::new(25, 1, 25, 8, 0, 8);
        let err = TranspileError::invalid_continuation(&span);
        let msg = format!("{}", err);
        assert!(msg.contains("Continuation action without actor"));
        assert!(msg.contains("[L25:1-L25:8]"));
    }
}
