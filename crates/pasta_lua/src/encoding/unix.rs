//! Unix/POSIX implementation of encoding module.
//!
//! On non-Windows systems, UTF-8 is the standard encoding.
//! This module provides passthrough implementations.

use std::io::Result;

/// ANSI → UTF-8 (UTF-8 として検める)。
pub fn ansi_to_utf8(data: &[u8]) -> Result<String> {
    String::from_utf8(data.to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// UTF-8 → ANSI (UTF-8 のまま)。
pub fn utf8_to_ansi(data: &str) -> Result<Vec<u8>> {
    Ok(data.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_string() {
        let result = ansi_to_utf8(b"Hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_ansi_to_bytes() {
        let result = utf8_to_ansi("Hello").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_utf8_roundtrip() {
        let original = "日本語テスト";
        let bytes = utf8_to_ansi(original).unwrap();
        let restored = ansi_to_utf8(&bytes).unwrap();
        assert_eq!(original, restored);
    }
}
