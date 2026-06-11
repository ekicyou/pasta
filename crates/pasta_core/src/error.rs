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
    #[error("No matching scene: '{scene}' with filters {filters:?}")]
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
    #[error("No more scenes: '{search_key}' with filters {filters:?}")]
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
    #[error("Word not found: @{key}")]
    WordNotFound { key: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_table_error_display() {
        // Error message formats are part of the public contract (surfaced in logs)
        assert_eq!(
            SceneTableError::SceneNotFound {
                scene: "会話".to_string()
            }
            .to_string(),
            "Scene not found: 会話"
        );
        assert_eq!(
            SceneTableError::InvalidScene {
                scene: "".to_string()
            }
            .to_string(),
            "Invalid scene name: ''"
        );
        assert_eq!(
            SceneTableError::RandomSelectionFailed.to_string(),
            "Random selection failed"
        );
        assert_eq!(
            SceneTableError::DuplicateSceneName {
                name: "会話".to_string()
            }
            .to_string(),
            "Duplicate scene name: 会話"
        );
        assert_eq!(
            SceneTableError::DuplicateScenePath {
                fn_name: "会話_1::__start__".to_string()
            }
            .to_string(),
            "Duplicate scene path: 会話_1::__start__"
        );
    }

    #[test]
    fn test_scene_table_error_display_with_filters() {
        let mut filters = HashMap::new();
        filters.insert("季節".to_string(), "春".to_string());

        let msg = SceneTableError::NoMatchingScene {
            scene: "挨拶".to_string(),
            filters: filters.clone(),
        }
        .to_string();
        assert!(msg.starts_with("No matching scene: '挨拶' with filters"));
        assert!(msg.contains("季節"));

        let msg = SceneTableError::NoMoreScenes {
            search_key: "挨拶".to_string(),
            filters,
        }
        .to_string();
        assert!(msg.starts_with("No more scenes: '挨拶' with filters"));
    }

    #[test]
    fn test_word_table_error_display() {
        assert_eq!(
            WordTableError::WordNotFound {
                key: "場所".to_string()
            }
            .to_string(),
            "Word not found: @場所"
        );
    }
}
