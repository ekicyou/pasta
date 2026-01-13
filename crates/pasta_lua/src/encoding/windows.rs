//! Windows implementation of encoding module.
//!
//! Provides conversion between UTF-8 and Windows ANSI/OEM code pages
//! using the Windows API functions MultiByteToWideChar and WideCharToMultiByte.
//!
//! Original: https://github.com/bozaro/local-encoding-rs/blob/master/src/windows.rs

use super::{Encoder, Encoding};
use std::ffi::OsStr;
use std::io::{Error, ErrorKind, Result};
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Globalization::*;

// ============================================================================
// Code page constants
// ============================================================================

/// ANSI code page (system default for GUI applications)
const CP_ACP_VALUE: u32 = CP_ACP;
/// OEM code page (system default for console applications)
const CP_OEMCP_VALUE: u32 = CP_OEMCP;

// ============================================================================
// MultiByteToWideChar flags
// ============================================================================

/// Fail if an invalid input character is encountered.
const MB_ERR_INVALID_CHARS_FLAG: u32 = 0x0000_0008;

// ============================================================================
// WideCharToMultiByte flags
// ============================================================================

/// Convert composite characters, consisting of a base character and a nonspacing character,
/// each with different character values.
const WC_COMPOSITECHECK_FLAG: u32 = 0x0000_0200;

// ============================================================================
// Encoding implementation
// ============================================================================

impl Encoding {
    /// Get the Windows code page for this encoding.
    fn codepage(&self) -> u32 {
        match self {
            Encoding::ANSI => CP_ACP_VALUE,
            Encoding::OEM => CP_OEMCP_VALUE,
        }
    }
}

impl Encoder for Encoding {
    /// Convert from bytes (system encoding) to UTF-8 string.
    fn to_string(&self, data: &[u8]) -> Result<String> {
        multi_byte_to_wide_char(self.codepage(), MB_ERR_INVALID_CHARS_FLAG, data)
    }

    /// Convert from UTF-8 string to bytes (system encoding).
    fn to_bytes(&self, data: &str) -> Result<Vec<u8>> {
        string_to_multibyte(self.codepage(), data, None)
    }
}

// ============================================================================
// Windows API wrappers
// ============================================================================

/// Convert String to multibyte string.
///
/// # Arguments
/// * `codepage` - Code page to use in performing the conversion.
/// * `data` - Source UTF-8 string.
/// * `default_char` - Optional character to use if a character cannot be represented
///   in the specified code page.
///
/// # Returns
/// * `Ok(Vec<u8>)` - Converted multibyte string
/// * `Err` - If an invalid input character is encountered and `default_char` is `None`
fn string_to_multibyte(codepage: u32, data: &str, default_char: Option<u8>) -> Result<Vec<u8>> {
    let wstr: Vec<u16> = OsStr::new(data).encode_wide().collect();
    wide_char_to_multi_byte(
        codepage,
        WC_COMPOSITECHECK_FLAG,
        &wstr,
        default_char,
        default_char.is_none(),
    )
    .and_then(|(data, invalid)| {
        if invalid {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Can't convert some characters to multibyte charset",
            ))
        } else {
            Ok(data)
        }
    })
}

/// Wrapper for MultiByteToWideChar.
///
/// Converts a multibyte string to a wide (UTF-16) string.
///
/// See https://docs.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar
fn multi_byte_to_wide_char(codepage: u32, flags: u32, multi_byte_str: &[u8]) -> Result<String> {
    // Empty string
    if multi_byte_str.is_empty() {
        return Ok(String::new());
    }

    unsafe {
        // Get length of UTF-16 string
        let len = MultiByteToWideChar(
            codepage,
            flags,
            multi_byte_str.as_ptr() as _,
            multi_byte_str.len() as _,
            ptr::null_mut(),
            0,
        );

        if len > 0 {
            // Allocate buffer and convert to UTF-16
            // SAFETY: MultiByteToWideChar will fully initialize the buffer up to `len` elements.
            #[allow(clippy::uninit_vec)]
            let mut wstr: Vec<u16> = {
                let mut v = Vec::with_capacity(len as usize);
                v.set_len(len as usize);
                v
            };

            let len = MultiByteToWideChar(
                codepage,
                flags,
                multi_byte_str.as_ptr() as _,
                multi_byte_str.len() as _,
                wstr.as_mut_ptr(),
                len,
            );

            if len > 0 {
                return String::from_utf16(&wstr[0..(len as usize)])
                    .map_err(|e| Error::new(ErrorKind::InvalidInput, e));
            }
        }
        Err(Error::last_os_error())
    }
}

/// Wrapper for WideCharToMultiByte.
///
/// Converts a wide (UTF-16) string to a multibyte string.
///
/// See https://docs.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte
fn wide_char_to_multi_byte(
    codepage: u32,
    flags: u32,
    wide_char_str: &[u16],
    default_char: Option<u8>,
    use_default_char_flag: bool,
) -> Result<(Vec<u8>, bool)> {
    // Empty string
    if wide_char_str.is_empty() {
        return Ok((Vec::new(), false));
    }

    unsafe {
        // Get length of multibyte string
        let len = WideCharToMultiByte(
            codepage,
            flags,
            wide_char_str.as_ptr(),
            wide_char_str.len() as _,
            ptr::null_mut(),
            0,
            ptr::null(),
            ptr::null_mut(),
        );

        if len > 0 {
            // Allocate buffer and convert from UTF-16 to multibyte
            // SAFETY: WideCharToMultiByte will fully initialize the buffer up to `len` elements.
            #[allow(clippy::uninit_vec)]
            let mut astr: Vec<u8> = {
                let mut v = Vec::with_capacity(len as usize);
                v.set_len(len as usize);
                v
            };

            let default_char_ref: [i8; 1] = match default_char {
                Some(c) => [c as i8],
                None => [0],
            };
            let mut use_char_ref: [i32; 1] = [0];

            let len = WideCharToMultiByte(
                codepage,
                flags,
                wide_char_str.as_ptr(),
                wide_char_str.len() as _,
                astr.as_mut_ptr() as _,
                len,
                match default_char {
                    Some(_) => default_char_ref.as_ptr() as _,
                    None => ptr::null(),
                },
                if use_default_char_flag {
                    use_char_ref.as_mut_ptr()
                } else {
                    ptr::null_mut()
                },
            );

            if (len as usize) == astr.len() {
                return Ok((astr, use_char_ref[0] != 0));
            }
            if len > 0 {
                return Ok((astr[0..(len as usize)].to_vec(), use_char_ref[0] != 0));
            }
        }
        Err(Error::last_os_error())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_string() {
        let result = Encoding::ANSI.to_string(b"Test").unwrap();
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_ansi_to_bytes() {
        let result = Encoding::ANSI.to_bytes("Test").unwrap();
        assert_eq!(result, b"Test");
    }

    #[test]
    fn test_oem_to_string() {
        let result = Encoding::OEM.to_string(b"Test").unwrap();
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_oem_to_bytes() {
        let result = Encoding::OEM.to_bytes("Test").unwrap();
        assert_eq!(result, b"Test");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(Encoding::ANSI.to_string(b"").unwrap(), "");
        assert_eq!(Encoding::ANSI.to_bytes("").unwrap(), b"");
    }

    #[test]
    fn test_multi_byte_to_wide_char_utf8() {
        // Test UTF-8 to UTF-16 conversion using CP_UTF8
        let result =
            multi_byte_to_wide_char(CP_UTF8, MB_ERR_INVALID_CHARS_FLAG, "テスト".as_bytes())
                .unwrap();
        assert_eq!(result, "テスト");
    }

    #[test]
    fn test_wide_char_to_multi_byte_ascii() {
        let wide: Vec<u16> = "Test".encode_utf16().collect();
        let (result, _) =
            wide_char_to_multi_byte(CP_ACP_VALUE, WC_COMPOSITECHECK_FLAG, &wide, None, true)
                .unwrap();
        assert_eq!(result, b"Test");
    }
}
