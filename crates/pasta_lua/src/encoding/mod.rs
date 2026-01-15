//! Encoding conversion module for Windows ANSI code page support.
//!
//! This module provides utilities for converting between UTF-8 strings and
//! Windows ANSI code page encodings. This is necessary because Lua on Windows
//! uses ANSI APIs for file system access, requiring path strings to be
//! converted from UTF-8 to the system code page (e.g., Shift-JIS/CP932 for Japanese).
//!
//! On non-Windows systems, this module provides passthrough implementations
//! that simply return the original UTF-8 strings.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
#[allow(unused_imports)]
pub use self::windows::*;

#[cfg(not(windows))]
mod unix;

#[cfg(not(windows))]
#[allow(unused_imports)]
pub use self::unix::*;

use std::io::Result;

/// Converter between Rust strings and system multibyte encoding.
pub trait Encoder {
    /// Convert from bytes (system encoding) to UTF-8 string.
    fn to_string(&self, data: &[u8]) -> Result<String>;

    /// Convert from UTF-8 string to bytes (system encoding).
    fn to_bytes(&self, data: &str) -> Result<Vec<u8>>;
}

/// Text conversion encoding type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// ANSI code page (CP_ACP on Windows, UTF-8 on other systems).
    /// Use for file system operations, GUI text, registry, etc.
    ANSI,
    /// OEM code page (CP_OEMCP on Windows, UTF-8 on other systems).
    /// Use for console output only.
    OEM,
}

/// Convert a UTF-8 string to ANSI bytes (system encoding).
///
/// On Windows, converts to ANSI code page (e.g., Shift-JIS/CP932 for Japanese locale).
/// On other systems, returns the original UTF-8 bytes.
///
/// # Arguments
/// * `s` - UTF-8 string to convert
///
/// # Returns
/// * `Ok(Vec<u8>)` - Converted byte vector
/// * `Err(std::io::Error)` - If encoding conversion fails (Windows only)
///
/// # Example
/// ```rust,ignore
/// use pasta_lua::encoding::to_ansi_bytes;
///
/// let bytes = to_ansi_bytes("hello").unwrap();
/// assert_eq!(bytes, b"hello");
/// ```
#[cfg(windows)]
pub fn to_ansi_bytes(s: &str) -> Result<Vec<u8>> {
    Encoding::ANSI.to_bytes(s)
}

/// Convert a UTF-8 string to ANSI bytes (system encoding).
///
/// On Unix systems, returns the original UTF-8 bytes unchanged.
///
/// # Arguments
/// * `s` - UTF-8 string to convert
///
/// # Returns
/// * `Ok(Vec<u8>)` - UTF-8 bytes
#[cfg(not(windows))]
pub fn to_ansi_bytes(s: &str) -> Result<Vec<u8>> {
    Ok(s.as_bytes().to_vec())
}

/// Convert a path string from Lua (system encoding) to UTF-8.
///
/// On Windows, converts from ANSI code page to UTF-8.
/// On other systems, returns the original string.
pub fn path_from_lua(path: &str) -> Result<String> {
    #[cfg(windows)]
    {
        let bytes = path.as_bytes();
        Encoding::ANSI.to_string(bytes)
    }

    #[cfg(not(windows))]
    {
        Ok(path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ansi_bytes_ascii() {
        // ASCII should pass through unchanged on all platforms
        let result = to_ansi_bytes("test/path/file.lua").unwrap();
        assert_eq!(result, b"test/path/file.lua");
    }

    #[test]
    fn test_to_ansi_bytes_empty() {
        // Empty string should return empty bytes
        let result = to_ansi_bytes("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_encoding_enum() {
        assert_ne!(Encoding::ANSI, Encoding::OEM);
    }

    #[cfg(windows)]
    #[test]
    fn test_to_ansi_bytes_japanese() {
        // Japanese characters should be converted to ANSI (Shift-JIS/CP932)
        let result = to_ansi_bytes("日本語パス").unwrap();
        // On Japanese Windows, this should be convertible
        assert!(!result.is_empty());
        // The result should not be the same as UTF-8 bytes
        let utf8_bytes = "日本語パス".as_bytes();
        // ANSI encoding will be different from UTF-8 for Japanese
        assert_ne!(result, utf8_bytes);
    }

    #[cfg(windows)]
    #[test]
    fn test_to_ansi_bytes_roundtrip() {
        // Roundtrip test: UTF-8 -> ANSI -> UTF-8
        let original = "日本語テスト";
        let ansi_bytes = to_ansi_bytes(original).unwrap();
        let restored = Encoding::ANSI.to_string(&ansi_bytes).unwrap();
        assert_eq!(restored, original);
    }
}
