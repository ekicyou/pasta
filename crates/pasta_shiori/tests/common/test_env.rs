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

    /// Send a SHIORI request with normalized text and return a structured response.
    ///
    /// The request text is normalized for convenience:
    /// - Leading/trailing newlines are trimmed
    /// - Line endings are normalized to `\r\n`
    /// - `\r\n\r\n` is appended as the SHIORI request terminator
    ///
    /// This allows writing requests as raw strings:
    /// ```ignore
    /// env.request(r#"
    /// GET SHIORI/3.0
    /// ID: OnBoot
    /// X-Pasta-Time: 2025-07-15T12:00:00Z
    /// "#)
    /// ```
    pub fn request(&mut self, text: &str) -> Result<ShioriResponse, ShioriRequestError> {
        let normalized = normalize_request(text);
        self.raw_request(&normalized)
    }

    /// Send a pre-formatted SHIORI request without normalization.
    pub fn raw_request(&mut self, text: &str) -> Result<ShioriResponse, ShioriRequestError> {
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

/// Normalize a SHIORI request string for test convenience.
///
/// - Trims leading/trailing newlines
/// - Normalizes line endings to `\r\n`
/// - Appends `\r\n\r\n` terminator
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut result = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    result.push_str("\r\n\r\n");
    result
}
