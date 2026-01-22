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
    #[error("ファイル '{0}' の読み込みに失敗しました: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    /// Configuration file parsing error.
    #[error("設定ファイル '{0}' の解析に失敗しました: {1}")]
    Config(PathBuf, #[source] toml::de::Error),

    /// Configuration file not found error.
    #[error("設定ファイル '{0}' が見つかりません")]
    ConfigNotFound(PathBuf),

    /// Pasta file parsing error.
    #[error("Pastaファイル '{file}' のパースに失敗しました: {message}")]
    Parse {
        file: PathBuf,
        message: String,
        #[source]
        source: Option<pasta_core::error::ParseError>,
    },

    /// Transpilation error.
    #[error("トランスパイルに失敗しました")]
    Transpile(#[from] TranspileError),

    /// Lua runtime initialization error.
    #[error("Luaランタイムの初期化に失敗しました: {0}")]
    Runtime(#[from] mlua::Error),

    /// Directory not found error.
    #[error("起動ディレクトリ '{0}' が存在しません")]
    DirectoryNotFound(PathBuf),

    /// Glob pattern error.
    #[error("ファイル探索パターンが不正です: {0}")]
    GlobPattern(#[from] glob::PatternError),

    /// Glob traversal error.
    #[error("ファイル探索中にエラーが発生しました: {0}")]
    GlobTraversal(#[from] glob::GlobError),

    /// Cache directory operation error.
    #[error("キャッシュディレクトリの操作に失敗しました: {path}")]
    CacheDirectoryError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// File metadata retrieval error.
    #[error("ファイルメタデータの取得に失敗しました: {path}")]
    MetadataError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Cache file write error.
    #[error("キャッシュファイルの書き込みに失敗しました: {path}")]
    CacheWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// scene_dic.lua generation error.
    #[error("scene_dic.lua の生成に失敗しました: {reason}")]
    SceneDicGenerationError {
        reason: String,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Partial transpilation failure.
    #[error("トランスパイル部分失敗: {succeeded}件成功, {failed}件失敗")]
    PartialTranspileError {
        succeeded: usize,
        failed: usize,
        failures: Vec<TranspileFailure>,
    },
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
        source: pasta_core::error::ParseError,
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
        assert!(msg.contains("ファイル"));
        assert!(msg.contains("/path/to/file.pasta"));
    }

    #[test]
    fn test_directory_not_found_display() {
        let err = LoaderError::directory_not_found("/ghost/master");
        let msg = format!("{}", err);
        assert!(msg.contains("起動ディレクトリ"));
        assert!(msg.contains("/ghost/master"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = LoaderError::parse("/path/to/test.pasta", "unexpected token");
        let msg = format!("{}", err);
        assert!(msg.contains("Pastaファイル"));
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
