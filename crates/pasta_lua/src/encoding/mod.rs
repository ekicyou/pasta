//! Encoding conversion module for Windows ANSI code page support.
//!
//! This module provides utilities for converting between UTF-8 strings and
//! Windows ANSI code page encodings. This is necessary because Lua on Windows
//! uses ANSI APIs for file system access, requiring path strings to be
//! converted from UTF-8 to the system code page (e.g., Shift-JIS/CP932 for Japanese).
//!
//! On non-Windows systems, this module provides passthrough implementations
//! that simply return the original UTF-8 strings.

// The platform modules only provide `Encoder` trait impls for `Encoding`,
// so declaring the module is sufficient — no re-export is needed.
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_bytes_ascii() {
        // ASCII should pass through unchanged on all platforms
        let result = Encoding::ANSI.to_bytes("test/path/file.lua").unwrap();
        assert_eq!(result, b"test/path/file.lua");
    }

    #[test]
    fn test_ansi_to_bytes_empty() {
        // Empty string should return empty bytes
        let result = Encoding::ANSI.to_bytes("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_encoding_enum() {
        assert_ne!(Encoding::ANSI, Encoding::OEM);
    }

    #[test]
    fn test_ansi_japanese_roundtrip() {
        // Roundtrip test: UTF-8 -> ANSI -> UTF-8 (CP932 or UTF-8 code page;
        // locale-independent CP932 byte checks live in windows.rs).
        let original = "日本語パス/テスト";
        let ansi_bytes = Encoding::ANSI.to_bytes(original).unwrap();
        assert!(!ansi_bytes.is_empty());
        let restored = Encoding::ANSI.to_string(&ansi_bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_ansi_to_string_ascii_and_empty() {
        // ASCII is identical in ANSI and UTF-8 on every platform.
        let result = Encoding::ANSI
            .to_string(b"ghost/master/scripts/main.lua")
            .unwrap();
        assert_eq!(result, "ghost/master/scripts/main.lua");
        assert_eq!(Encoding::ANSI.to_string(b"").unwrap(), "");
    }
}
