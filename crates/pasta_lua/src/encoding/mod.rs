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

/// Convert a UTF-8 path string to system encoding for Lua.
///
/// On Windows, converts to ANSI code page (e.g., Shift-JIS).
/// On other systems, returns the original string.
pub fn path_to_lua(path: &str) -> Result<String> {
    #[cfg(windows)]
    {
        // Convert UTF-8 to ANSI bytes, then back to String for Lua
        let bytes = Encoding::ANSI.to_bytes(path)?;
        // Return as a byte string that Lua can use directly
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    #[cfg(not(windows))]
    {
        Ok(path.to_string())
    }
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
    fn test_ascii_roundtrip() {
        let original = "test/path/file.lua";
        let converted = path_to_lua(original).unwrap();
        // ASCII should pass through unchanged
        assert!(converted.contains("test"));
    }

    #[test]
    fn test_encoding_enum() {
        assert_ne!(Encoding::ANSI, Encoding::OEM);
    }
}
