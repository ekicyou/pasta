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
