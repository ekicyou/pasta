//! Error types for the search module.

use mlua::Error as LuaError;
use thiserror::Error;

/// Error type for search operations.
#[derive(Debug, Error)]
pub enum SearchError {
    /// Scene table error from pasta_core
    #[error("Scene search error: {0}")]
    SceneTableError(#[from] pasta_core::SceneTableError),

    /// Word table error from pasta_core
    #[error("Word search error: {0}")]
    WordTableError(#[from] pasta_core::WordTableError),

    /// Invalid argument type
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Lua error
    #[error("Lua error: {0}")]
    LuaError(#[from] LuaError),
}

impl From<SearchError> for LuaError {
    fn from(err: SearchError) -> Self {
        match err {
            SearchError::LuaError(e) => e,
            other => LuaError::RuntimeError(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_argument_display() {
        let err = SearchError::InvalidArgument("bad input".to_string());
        assert_eq!(err.to_string(), "Invalid argument: bad input");
    }

    #[test]
    fn test_scene_table_error_display_prefix() {
        let err = SearchError::SceneTableError(pasta_core::SceneTableError::SceneNotFound {
            scene: "テスト".to_string(),
        });
        let msg = err.to_string();
        assert!(
            msg.starts_with("Scene search error: "),
            "unexpected message: {}",
            msg
        );
        assert!(msg.contains("テスト"), "unexpected message: {}", msg);
    }

    #[test]
    fn test_from_search_error_wraps_as_runtime_error() {
        let err = SearchError::InvalidArgument("bad input".to_string());
        let lua_err = LuaError::from(err);
        match lua_err {
            LuaError::RuntimeError(msg) => {
                assert_eq!(msg, "Invalid argument: bad input");
            }
            other => panic!("expected RuntimeError, got: {:?}", other),
        }
    }

    #[test]
    fn test_from_search_error_lua_error_passthrough() {
        // A wrapped LuaError must be returned as-is, not re-wrapped.
        let original = LuaError::RuntimeError("original lua error".to_string());
        let err = SearchError::LuaError(original);
        let lua_err = LuaError::from(err);
        match lua_err {
            LuaError::RuntimeError(msg) => {
                assert_eq!(msg, "original lua error");
            }
            other => panic!("expected RuntimeError passthrough, got: {:?}", other),
        }
    }
}
