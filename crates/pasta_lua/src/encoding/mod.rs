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
pub use windows::{ansi_to_utf8, utf8_to_ansi};

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::{ansi_to_utf8, utf8_to_ansi};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_bytes_ascii() {
        // ASCII should pass through unchanged on all platforms
        let result = utf8_to_ansi("test/path/file.lua").unwrap();
        assert_eq!(result, b"test/path/file.lua");
    }

    #[test]
    fn test_ansi_to_bytes_empty() {
        // Empty string should return empty bytes
        let result = utf8_to_ansi("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ansi_japanese_roundtrip() {
        // Roundtrip test: UTF-8 -> ANSI -> UTF-8 (CP932 or UTF-8 code page;
        // locale-independent CP932 byte checks live in windows.rs).
        let original = "日本語パス/テスト";
        let ansi_bytes = utf8_to_ansi(original).unwrap();
        assert!(!ansi_bytes.is_empty());
        let restored = ansi_to_utf8(&ansi_bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_ansi_to_string_ascii_and_empty() {
        // ASCII is identical in ANSI and UTF-8 on every platform.
        let result = ansi_to_utf8(b"ghost/master/scripts/main.lua").unwrap();
        assert_eq!(result, "ghost/master/scripts/main.lua");
        assert_eq!(ansi_to_utf8(b"").unwrap(), "");
    }
}
