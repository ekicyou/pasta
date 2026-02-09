//! Error types for Pasta Core registry layer.
//!
//! This module defines registry-related errors (SceneTableError, WordTableError).
//! Parse-related errors (ParseError) are defined in the `pasta_dsl` crate.

use std::collections::HashMap;
use thiserror::Error;

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
