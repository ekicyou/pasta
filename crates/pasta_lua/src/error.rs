//! Error types for Pasta Lua transpiler.
//!
//! This module defines error types for the Lua code generation process.

use pasta_dsl::parser::Span;
use std::fmt;
use thiserror::Error;

/// Transpile error type for Lua code generation.
#[derive(Error, Debug)]
pub enum TranspileError {
    /// IO error during transpilation (writing to output).
    #[error("IO error during transpilation: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid AST structure.
    #[error("Invalid AST structure: {span}: {message}")]
    InvalidAst { span: SpanDisplay, message: String },

    /// Undefined scene reference.
    #[error("Undefined scene: '{name}' at {span}")]
    UndefinedScene { name: String, span: SpanDisplay },

    /// Undefined word reference.
    #[error("Undefined word: '{name}' at {span}")]
    UndefinedWord { name: String, span: SpanDisplay },

    /// Continuation action without actor.
    #[error("Continuation action without actor: {span}")]
    InvalidContinuation { span: SpanDisplay },

    /// String literal cannot be converted.
    #[error(
        "String literal cannot be converted: {span}: dangerous pattern detected in all formats"
    )]
    StringLiteralError { text: String, span: SpanDisplay },

    /// Too many local variables.
    #[error("Too many local variables in scope: {count} (max ~200) at {span}")]
    TooManyLocalVariables { count: usize, span: SpanDisplay },

    /// Unsupported feature.
    #[error("Unsupported feature: {feature} at {span}")]
    Unsupported { feature: String, span: SpanDisplay },

    /// Property reference used in expression context (not allowed).
    #[error("Property reference cannot be used in expressions; use variable assignment first")]
    PropertyInExpression,
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

    /// Create a PropertyInExpression error (span-less).
    pub fn property_in_expression() -> Self {
        TranspileError::PropertyInExpression
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
        "Unknown library: {0}. Valid libraries: std_all, std_all_unsafe, std_coroutine, std_table, std_io, std_os, std_string, std_math, std_package, std_debug, std_jit, std_ffi, std_bit, assertions, testing, env, regex, json, yaml"
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
        let err = TranspileError::IoError(std::io::Error::other("test error"));
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

    #[test]
    fn test_transpile_error_undefined_scene_display() {
        let span = Span::new(3, 5, 3, 12, 0, 7);
        let err = TranspileError::UndefinedScene {
            name: "メイン".to_string(),
            span: SpanDisplay::from(span),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Undefined scene"));
        assert!(msg.contains("'メイン'"));
        assert!(msg.contains("[L3:5-L3:12]"));
    }

    #[test]
    fn test_transpile_error_undefined_word_display() {
        let span = Span::new(7, 2, 7, 6, 0, 4);
        let err = TranspileError::UndefinedWord {
            name: "挨拶".to_string(),
            span: SpanDisplay::from(span),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Undefined word"));
        assert!(msg.contains("'挨拶'"));
        assert!(msg.contains("[L7:2-L7:6]"));
    }

    #[test]
    fn test_transpile_error_too_many_local_variables_display() {
        let span = Span::new(100, 1, 100, 1, 0, 0);
        let err = TranspileError::TooManyLocalVariables {
            count: 250,
            span: SpanDisplay::from(span),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Too many local variables"));
        assert!(msg.contains("250"));
        assert!(msg.contains("[L100:1-L100:1]"));
    }

    #[test]
    fn test_transpile_error_unsupported_constructor_display() {
        let span = Span::new(12, 3, 12, 20, 0, 17);
        let err = TranspileError::unsupported(&span, "goto labels");
        let msg = format!("{}", err);
        assert!(msg.contains("Unsupported feature"));
        assert!(msg.contains("goto labels"));
        assert!(msg.contains("[L12:3-L12:20]"));
    }

    #[test]
    fn test_transpile_error_string_literal_constructor_display() {
        let span = Span::new(4, 1, 4, 30, 0, 29);
        let err = TranspileError::string_literal_error(&span, "\\bad]==========text");
        // The Display message reports the span and the danger-pattern diagnosis.
        let msg = format!("{}", err);
        assert!(msg.contains("String literal cannot be converted"));
        assert!(msg.contains("dangerous pattern detected"));
        assert!(msg.contains("[L4:1-L4:30]"));
        // The offending text is preserved on the variant for programmatic access.
        match err {
            TranspileError::StringLiteralError { text, .. } => {
                assert_eq!(text, "\\bad]==========text");
            }
            other => panic!("expected StringLiteralError, got: {:?}", other),
        }
    }

    #[test]
    fn test_transpile_error_property_in_expression_constructor_display() {
        let err = TranspileError::property_in_expression();
        assert!(matches!(err, TranspileError::PropertyInExpression));
        let msg = format!("{}", err);
        assert!(msg.contains("Property reference cannot be used in expressions"));
        assert!(msg.contains("use variable assignment first"));
    }

    #[test]
    fn test_config_error_unknown_library_display() {
        let err = ConfigError::UnknownLibrary("std_bogus".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Unknown library: std_bogus"));
        // The message must enumerate valid library names to guide the user.
        assert!(msg.contains("std_all"));
        assert!(msg.contains("regex"));
        assert!(msg.contains("yaml"));
    }
}
