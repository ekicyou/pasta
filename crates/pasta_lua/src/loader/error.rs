//! Error types for Pasta Loader.
//!
//! This module defines error types for the startup sequence,
//! including file discovery, configuration loading, transpilation,
//! and runtime initialization.

use std::path::PathBuf;
use thiserror::Error;

use crate::TranspileError;

/// Loader error type for the startup sequence.
///
/// This error type covers all phases of the startup sequence:
/// - Configuration loading and parsing
/// - File system operations
/// - Pasta file parsing
/// - Transpilation
/// - Lua runtime initialization
#[derive(Debug, Error)]
pub enum LoaderError {
    /// File IO error.
    #[error("Failed to read file: '{0}': {1}")]
    Io(PathBuf, #[source] std::io::Error),

    /// Configuration file parsing error.
    #[error("Failed to parse configuration file: '{0}': {1}")]
    Config(PathBuf, #[source] toml::de::Error),

    /// Configuration file not found error.
    #[error("Configuration file not found: '{0}'")]
    ConfigNotFound(PathBuf),

    /// Pasta file parsing error.
    #[error("Failed to parse Pasta file: '{file}': {message}")]
    Parse {
        file: PathBuf,
        message: String,
        #[source]
        source: Option<pasta_dsl::ParseError>,
    },

    /// Transpilation error.
    #[error("Transpilation failed: {0}")]
    Transpile(#[from] TranspileError),

    /// Lua runtime initialization error.
    #[error("Failed to initialize Lua runtime: {0}")]
    Runtime(#[from] mlua::Error),

    /// Directory not found error.
    #[error("Startup directory not found: '{0}'")]
    DirectoryNotFound(PathBuf),

    /// Glob pattern error.
    #[error("Invalid file discovery pattern: {0}")]
    GlobPattern(#[from] glob::PatternError),

    /// Glob traversal error.
    #[error("Error during file discovery: {0}")]
    GlobTraversal(#[from] glob::GlobError),

    /// Cache directory operation error.
    #[error("Failed to prepare cache directory: {path}")]
    CacheDirectoryError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// File metadata retrieval error.
    #[error("Failed to get file metadata: {path}")]
    MetadataError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Cache file write error.
    #[error("Failed to write cache file: {path}")]
    CacheWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Invalid file name error (e.g., init.lua, init.pasta).
    #[error("Invalid file name: '{0}'")]
    InvalidFileName(PathBuf),

    /// scene_dic.lua generation error.
    #[error("Failed to generate scene_dic.lua: {reason}")]
    SceneDicGenerationError {
        reason: String,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Partial transpilation failure.
    #[error(
        "Partial transpilation failure: {succeeded} succeeded, {failed} failed [{}]",
        format_failure_paths(failures)
    )]
    PartialTranspileError {
        succeeded: usize,
        failed: usize,
        failures: Vec<TranspileFailure>,
    },
}

/// Format failure file paths into a comma-separated list for Display.
fn format_failure_paths(failures: &[TranspileFailure]) -> String {
    failures
        .iter()
        .map(|f| f.source_path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Details about a single transpile failure.
#[derive(Debug)]
pub struct TranspileFailure {
    /// Source file path that failed.
    pub source_path: PathBuf,
    /// Error message describing the failure.
    pub error: String,
}

impl LoaderError {
    /// Create an IO error with file path.
    pub fn io(path: impl Into<PathBuf>, err: std::io::Error) -> Self {
        LoaderError::Io(path.into(), err)
    }

    /// Create a config error with file path.
    pub fn config(path: impl Into<PathBuf>, err: toml::de::Error) -> Self {
        LoaderError::Config(path.into(), err)
    }

    /// Create a config not found error with file path.
    pub fn config_not_found(path: impl Into<PathBuf>) -> Self {
        LoaderError::ConfigNotFound(path.into())
    }

    /// Create a parse error with file path and message.
    pub fn parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        LoaderError::Parse {
            file: path.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a parse error with source error.
    pub fn parse_with_source(
        path: impl Into<PathBuf>,
        message: impl Into<String>,
        source: pasta_dsl::ParseError,
    ) -> Self {
        LoaderError::Parse {
            file: path.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a directory not found error.
    pub fn directory_not_found(path: impl Into<PathBuf>) -> Self {
        LoaderError::DirectoryNotFound(path.into())
    }

    /// Create an invalid file name error.
    pub fn invalid_file_name(path: impl Into<PathBuf>) -> Self {
        LoaderError::InvalidFileName(path.into())
    }

    /// Create a cache directory error.
    pub fn cache_directory(path: impl Into<PathBuf>, err: std::io::Error) -> Self {
        LoaderError::CacheDirectoryError {
            path: path.into(),
            source: err,
        }
    }

    /// Create a metadata error.
    pub fn metadata(path: impl Into<PathBuf>, err: std::io::Error) -> Self {
        LoaderError::MetadataError {
            path: path.into(),
            source: err,
        }
    }

    /// Create a cache write error.
    pub fn cache_write(path: impl Into<PathBuf>, err: std::io::Error) -> Self {
        LoaderError::CacheWriteError {
            path: path.into(),
            source: err,
        }
    }

    /// Create a scene_dic generation error.
    pub fn scene_dic_generation(reason: impl Into<String>, err: Option<std::io::Error>) -> Self {
        LoaderError::SceneDicGenerationError {
            reason: reason.into(),
            source: err,
        }
    }

    /// Create a partial transpile error.
    pub fn partial_transpile(
        succeeded: usize,
        failed: usize,
        failures: Vec<TranspileFailure>,
    ) -> Self {
        LoaderError::PartialTranspileError {
            succeeded,
            failed,
            failures,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_display() {
        let err = LoaderError::io(
            "/path/to/file.pasta",
            io::Error::new(io::ErrorKind::NotFound, "file not found"),
        );
        let msg = format!("{}", err);
        assert!(msg.contains("Failed to read file"));
        assert!(msg.contains("/path/to/file.pasta"));
    }

    #[test]
    fn test_directory_not_found_display() {
        let err = LoaderError::directory_not_found("/ghost/master");
        let msg = format!("{}", err);
        assert!(msg.contains("Startup directory not found"));
        assert!(msg.contains("/ghost/master"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = LoaderError::parse("/path/to/test.pasta", "unexpected token");
        let msg = format!("{}", err);
        assert!(msg.contains("Failed to parse Pasta file"));
        assert!(msg.contains("/path/to/test.pasta"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = LoaderError::io("/test/path", io_err);
        // Check that source() works
        assert!(std::error::Error::source(&err).is_some());
    }
}
