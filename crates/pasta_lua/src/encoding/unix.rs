//! Unix/POSIX implementation of encoding module.
//!
//! On non-Windows systems, UTF-8 is the standard encoding.
//! This module provides passthrough implementations.

use super::{Encoder, Encoding};
use std::io::Result;

impl Encoder for Encoding {
    /// Convert from bytes to string (UTF-8 passthrough).
    fn to_string(&self, data: &[u8]) -> Result<String> {
        String::from_utf8(data.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Convert from string to bytes (UTF-8 passthrough).
    fn to_bytes(&self, data: &str) -> Result<Vec<u8>> {
        Ok(data.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_string() {
        let result = Encoding::ANSI.to_string(b"Hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_ansi_to_bytes() {
        let result = Encoding::ANSI.to_bytes("Hello").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_oem_to_string() {
        let result = Encoding::OEM.to_string(b"Hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_oem_to_bytes() {
        let result = Encoding::OEM.to_bytes("Hello").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_utf8_roundtrip() {
        let original = "日本語テスト";
        let bytes = Encoding::ANSI.to_bytes(original).unwrap();
        let restored = Encoding::ANSI.to_string(&bytes).unwrap();
        assert_eq!(original, restored);
    }
}
