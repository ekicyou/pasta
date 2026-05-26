//! SHIORI test environment wrapper.
//!
//! Provides a convenient wrapper for SHIORI integration testing:
//! fixture setup, load, request, and structured response parsing.

use super::copy_fixture_to_temp;
use super::response::{ShioriResponse, ShioriResponseError};
use pasta::error::MyError;
use pasta::{PastaShiori, Shiori};
use std::path::Path;
use tempfile::TempDir;

/// Error type for ShioriTestEnv request operations.
#[derive(Debug)]
pub enum ShioriRequestError {
    /// SHIORI processing error.
    Shiori(MyError),
    /// Response parsing error.
    Parse(ShioriResponseError),
}

impl From<MyError> for ShioriRequestError {
    fn from(e: MyError) -> Self {
        ShioriRequestError::Shiori(e)
    }
}

impl From<ShioriResponseError> for ShioriRequestError {
    fn from(e: ShioriResponseError) -> Self {
        ShioriRequestError::Parse(e)
    }
}

impl std::fmt::Display for ShioriRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shiori(e) => write!(f, "SHIORI error: {e}"),
            Self::Parse(e) => write!(f, "Response parse error: {e}"),
        }
    }
}

impl std::error::Error for ShioriRequestError {}

/// SHIORI test environment.
///
/// Manages fixture copy, SHIORI load, and request/response cycle.
/// Automatically cleans up temporary directory on drop.
pub struct ShioriTestEnv {
    shiori: PastaShiori,
    _temp_dir: TempDir,
}

impl ShioriTestEnv {
    /// Create a new test environment from a fixture.
    ///
    /// Copies fixture files to a temp directory, then loads SHIORI.
    /// Panics on failure (intended for test use).
    pub fn new(fixture: &str) -> Self {
        let temp_dir = copy_fixture_to_temp(fixture);
        let mut shiori = PastaShiori::default();
        let loaded = shiori
            .load(0, temp_dir.path().as_os_str())
            .expect("SHIORI load should not error");
        assert!(loaded, "SHIORI load should return true");

        Self {
            shiori,
            _temp_dir: temp_dir,
        }
    }

    /// Send a SHIORI request and return a structured response.
    pub fn request(&mut self, text: &str) -> Result<ShioriResponse, ShioriRequestError> {
        let raw = self.shiori.request(text)?;
        let response = ShioriResponse::parse(&raw)?;
        Ok(response)
    }

    /// Get a reference to the internal Lua runtime.
    pub fn runtime(&self) -> Option<&pasta_lua::PastaLuaRuntime> {
        self.shiori.runtime()
    }

    /// Get the path to the temporary directory.
    pub fn path(&self) -> &Path {
        self._temp_dir.path()
    }
}
