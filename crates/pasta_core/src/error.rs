//! Error types for Pasta Core registry layer.
//!
//! This module defines registry-related errors (SceneTableError, WordTableError,
//! SceneAliasError, ActorFontError).
//! Parse-related errors (ParseError) are defined in the `pasta_dsl` crate.

use thiserror::Error;

/// Scene table lookup errors.
#[derive(Error, Debug, Clone)]
pub enum SceneTableError {
    /// Scene not found.
    #[error("Scene not found: {scene}")]
    SceneNotFound { scene: String },

    /// 名前に合うシーンはあるが、条件を満たす候補が 1 つも無い
    #[error("No matching scene: '{scene}'")]
    NoMatchingScene { scene: String },

    /// Invalid scene name.
    #[error("Invalid scene name: '{scene}'")]
    InvalidScene { scene: String },

    /// 同じ名前のシーンに同じ `＆順番` の番号が 2 つ以上ある（読み込みエラー）
    #[error("`＆順番` の番号 {order} が同じ名前のシーン `{scene}` に 2 つ以上ある: {fn_names:?}")]
    DuplicateOrder {
        scene: String,
        order: i64,
        fn_names: Vec<String>,
    },
}

/// Word table lookup errors.
#[derive(Error, Debug, Clone)]
pub enum WordTableError {
    /// Word not found.
    #[error("Word not found: @{key}")]
    WordNotFound { key: String },
}

/// Scene alias table construction errors (`[scene.alias]` in pasta.toml).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SceneAliasError {
    #[error("scene alias: empty name is not allowed (target '{target}')")]
    EmptyName { target: String },

    #[error("scene alias: alias '{alias}' is defined more than once (targets: {targets:?})")]
    DuplicateAlias { alias: String, targets: Vec<String> },

    #[error(
        "scene alias: '{name}' is both a target and an alias (chained aliases are not allowed)"
    )]
    Chain { name: String },

    #[error(
        "scene alias: alias '{alias}' of '{target}' conflicts with the default alias of '{default_target}' (write {default_target} = [] to remove the default)"
    )]
    ConflictsWithDefault {
        alias: String,
        target: String,
        default_target: String,
    },

    #[error(
        "scene alias: target '{target}' is a default alias of '{default_target}' (write {default_target} = [] to remove the default)"
    )]
    TargetIsDefaultAlias {
        target: String,
        default_target: String,
    },
}

/// What is wrong with one `font` attribute value (`[actor."name"].font` in pasta.toml).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FontValueProblem {
    #[error("value must be a string, integer or boolean (got {type_name})")]
    WrongType { type_name: &'static str },

    #[error("value must not be an empty string")]
    Empty,

    #[error("value must not contain ']' or '\\'")]
    BreaksTag,
}

/// Actor font (display style) errors (`[actor."name"].font` in pasta.toml).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ActorFontError {
    #[error("actor font: font of actor '{actor}' must be a table")]
    NotTable { actor: String },

    #[error("actor font: actor '{actor}' has unknown attribute '{attribute}'")]
    UnknownAttribute { actor: String, attribute: String },

    #[error("actor font: actor '{actor}' attribute '{attribute}': {problem}")]
    InvalidValue {
        actor: String,
        attribute: String,
        problem: FontValueProblem,
    },
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
    }

    #[test]
    fn test_scene_table_error_display_no_matching_scene() {
        assert_eq!(
            SceneTableError::NoMatchingScene {
                scene: "挨拶".to_string()
            }
            .to_string(),
            "No matching scene: '挨拶'"
        );
    }

    #[test]
    fn test_scene_table_error_display_duplicate_order() {
        assert_eq!(
            SceneTableError::DuplicateOrder {
                scene: "OnTalk".to_string(),
                order: 10,
                fn_names: vec![
                    "OnTalk_1::__start__".to_string(),
                    "OnTalk_2::__start__".to_string()
                ],
            }
            .to_string(),
            r#"`＆順番` の番号 10 が同じ名前のシーン `OnTalk` に 2 つ以上ある: ["OnTalk_1::__start__", "OnTalk_2::__start__"]"#
        );
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
