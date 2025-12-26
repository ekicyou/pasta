//! Error types for Pasta Core parsing layer.
//!
//! This module defines parse-related errors that are language-independent.
//! Runtime errors (Rune-specific) are defined in pasta_rune.

use std::collections::HashMap;
use thiserror::Error;

/// Result type alias for parse operations.
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Parse error with source location information.
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    /// Syntax error at a specific location.
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    SyntaxError {
        file: String,
        line: usize,
        column: usize,
        message: String,
    },

    /// Pest parser error.
    #[error("Pest parse error: {0}")]
    PestError(String),

    /// IO error (file reading).
    #[error("IO error: {0}")]
    IoError(String),

    /// Multiple parse errors accumulated.
    #[error("Multiple parse errors ({} errors)", .errors.len())]
    MultipleErrors { errors: Vec<ParseErrorInfo> },
}

/// Individual parse error information for MultipleErrors.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorInfo {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl ParseError {
    /// Create a new syntax error with source location.
    pub fn syntax_error(
        file: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        ParseError::SyntaxError {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a new pest parse error.
    pub fn pest_error(message: impl Into<String>) -> Self {
        ParseError::PestError(message.into())
    }

    /// Create a new IO error from a string message.
    pub fn io_error(message: impl Into<String>) -> Self {
        ParseError::IoError(message.into())
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

/// Result type alias for scene table operations.
pub type SceneTableResult<T> = std::result::Result<T, SceneTableError>;

/// Scene table lookup errors.
#[derive(Error, Debug, Clone)]
pub enum SceneTableError {
    /// Scene not found.
    #[error("Scene not found: {scene}")]
    SceneNotFound { scene: String },

    /// No matching scene with filters.
    #[error("No matching scene for '{scene}' with filters {filters:?}")]
    NoMatchingScene {
        scene: String,
        filters: HashMap<String, String>,
    },

    /// Invalid scene name.
    #[error("Invalid scene name: '{scene}'")]
    InvalidScene { scene: String },

    /// Random selection failed.
    #[error("Random selection failed")]
    RandomSelectionFailed,

    /// Duplicate scene name.
    #[error("Duplicate scene name: {name}")]
    DuplicateSceneName { name: String },

    /// Duplicate scene path (fn_name).
    #[error("Duplicate scene path: {fn_name}")]
    DuplicateScenePath { fn_name: String },

    /// No more scenes available.
    #[error("No more scenes for '{search_key}' with filters {filters:?}")]
    NoMoreScenes {
        search_key: String,
        filters: HashMap<String, String>,
    },
}

/// Result type alias for word table operations.
pub type WordTableResult<T> = std::result::Result<T, WordTableError>;

/// Word table lookup errors.
#[derive(Error, Debug, Clone)]
pub enum WordTableError {
    /// Word not found.
    #[error("単語定義 @{key} が見つかりません")]
    WordNotFound { key: String },
}
