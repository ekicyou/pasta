//! Error types for Pasta DSL parsing layer.
//!
//! This module defines parse-related errors that are language-independent.
//! These error types are specific to DSL parsing and do not depend on
//! any backend or registry implementation.

use thiserror::Error;

/// Parse error with source location information.
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    /// Syntax error at a specific location.
    #[error("Parse error: {file}:{line}:{column}: {message}")]
    SyntaxError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },

    /// IO error (file reading).
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}
