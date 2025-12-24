//! Transpiler2 error types.
//!
//! This module defines error types specific to transpiler2 operations.

use crate::error::{PastaError, Transpiler2Pass};
use crate::parser2::Span;
use thiserror::Error;

/// Transpiler2 error type.
///
/// This is a separate error type from PastaError to maintain
/// module independence between transpiler and transpiler2.
#[derive(Error, Debug)]
pub enum TranspileError {
    /// Invalid AST structure.
    #[error("無効なAST {location}: {message}")]
    InvalidAst {
        /// Source location string (e.g., "file.pasta:10:5")
        location: String,
        /// Error description
        message: String,
    },

    /// Undefined symbol reference.
    #[error("未定義シンボル: {symbol}")]
    UndefinedSymbol {
        /// The undefined symbol name
        symbol: String,
    },

    /// Invalid continuation action (no preceding ActionLine).
    #[error("無効な継続行 {location}: 直前にActionLineがありません")]
    InvalidContinuation {
        /// Source location string
        location: String,
    },

    /// IO error during code generation.
    #[error("IO エラー: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error (should not occur in normal operation).
    #[error("内部エラー: {0}")]
    InternalError(String),
}

impl TranspileError {
    /// Create an InvalidAst error from span information.
    pub fn invalid_ast(span: &Span, message: impl Into<String>) -> Self {
        TranspileError::InvalidAst {
            location: Self::span_to_location(span),
            message: message.into(),
        }
    }

    /// Create an UndefinedSymbol error.
    pub fn undefined_symbol(symbol: impl Into<String>) -> Self {
        TranspileError::UndefinedSymbol {
            symbol: symbol.into(),
        }
    }

    /// Create an InvalidContinuation error from span information.
    pub fn invalid_continuation(span: &Span) -> Self {
        TranspileError::InvalidContinuation {
            location: Self::span_to_location(span),
        }
    }

    /// Create an InternalError.
    pub fn internal(message: impl Into<String>) -> Self {
        TranspileError::InternalError(message.into())
    }

    /// Convert this error into PastaError with pass phase context.
    pub fn into_pasta_error(self, pass: Transpiler2Pass) -> PastaError {
        PastaError::Transpiler2Error {
            pass,
            message: self.to_string(),
        }
    }

    /// Convert a Span to a location string.
    fn span_to_location(span: &Span) -> String {
        format!("{}:{}", span.start_line, span.start_col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_ast_error() {
        let span = Span::new(10, 5, 10, 15);
        let err = TranspileError::invalid_ast(&span, "unexpected token");

        assert!(err.to_string().contains("10:5"));
        assert!(err.to_string().contains("unexpected token"));
    }

    #[test]
    fn test_undefined_symbol_error() {
        let err = TranspileError::undefined_symbol("test_scene");

        assert!(err.to_string().contains("未定義シンボル"));
        assert!(err.to_string().contains("test_scene"));
    }

    #[test]
    fn test_invalid_continuation_error() {
        let span = Span::new(20, 1, 20, 10);
        let err = TranspileError::invalid_continuation(&span);

        assert!(err.to_string().contains("20:1"));
        assert!(err.to_string().contains("ActionLine"));
    }

    #[test]
    fn test_internal_error() {
        let err = TranspileError::internal("something went wrong");

        assert!(err.to_string().contains("内部エラー"));
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: TranspileError = io_err.into();

        assert!(matches!(err, TranspileError::IoError(_)));
        assert!(err.to_string().contains("IO エラー"));
    }
}
