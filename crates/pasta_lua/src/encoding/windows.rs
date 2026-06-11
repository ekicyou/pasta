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

/// Convert a buffer length to the `i32` expected by the Win32 conversion APIs.
///
/// Lengths above `i32::MAX` are rejected: a plain `as` cast would wrap
/// (e.g. `u32::MAX as i32 == -1`, which `MultiByteToWideChar` /
/// `WideCharToMultiByte` interpret as "null-terminated input", reading
/// past the end of the borrowed slice).
fn buffer_len_to_i32(len: usize) -> Result<i32> {
    i32::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            "Input too large for Windows code page conversion (must be < 2 GiB)",
        )
    })
}

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

    // Reject lengths the Win32 API cannot represent (prevents `as` cast wrap).
    let in_len = buffer_len_to_i32(multi_byte_str.len())?;

    // SAFETY: Calls to `MultiByteToWideChar` are safe under these conditions:
    //  1. `multi_byte_str` is a valid borrowed slice; its pointer and length are valid
    //     for the duration of each FFI call.
    //  2. The first call (output buffer = null, length = 0) only queries the required
    //     buffer size — no memory is written.
    //  3. The second call writes into `wstr`, which is allocated to exactly `len`
    //     elements as returned by the first call.  `set_len(len)` is sound because
    //     `MultiByteToWideChar` fully initializes all `len` elements on success.
    //  4. Return values are checked: if either call returns 0, we fall through to
    //     `Error::last_os_error()` instead of reading uninitialized memory.
    unsafe {
        // Get length of UTF-16 string
        let len = MultiByteToWideChar(
            codepage,
            flags,
            multi_byte_str.as_ptr() as _,
            in_len,
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
                in_len,
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

    // Reject lengths the Win32 API cannot represent (prevents `as` cast wrap).
    let in_len = buffer_len_to_i32(wide_char_str.len())?;

    // SAFETY: Calls to `WideCharToMultiByte` are safe under these conditions:
    //  1. `wide_char_str` is a valid borrowed slice; its pointer and length are valid
    //     for the duration of each FFI call.
    //  2. The first call (output buffer = null, length = 0) only queries the required
    //     buffer size — no memory is written.
    //  3. The second call writes into `astr`, which is allocated to exactly `len`
    //     elements as returned by the first call.  `set_len(len)` is sound because
    //     `WideCharToMultiByte` fully initializes all `len` elements on success.
    //  4. `default_char_ref` and `use_char_ref` are stack-local arrays whose lifetimes
    //     span the entire FFI call; null is passed when not needed.
    //  5. Return values are checked: if either call returns 0, we fall through to
    //     `Error::last_os_error()` instead of reading uninitialized memory.
    unsafe {
        // Get length of multibyte string
        let len = WideCharToMultiByte(
            codepage,
            flags,
            wide_char_str.as_ptr(),
            in_len,
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
                in_len,
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

    // ------------------------------------------------------------------
    // Locale-independent tests using explicit code pages (CP932 / CP_UTF8).
    // CP932 (Shift-JIS) conversion tables ship with every Windows install,
    // so these are deterministic regardless of the system ANSI code page.
    // ------------------------------------------------------------------

    /// CP932 code page identifier (Shift-JIS).
    const CP932: u32 = 932;

    #[test]
    fn test_multi_byte_to_wide_char_cp932_decodes_shift_jis() {
        // "日本" in Shift-JIS: 日 = 0x93 0xFA, 本 = 0x96 0x7B
        let sjis = [0x93u8, 0xFA, 0x96, 0x7B];
        let result = multi_byte_to_wide_char(CP932, MB_ERR_INVALID_CHARS_FLAG, &sjis).unwrap();
        assert_eq!(result, "日本");
    }

    #[test]
    fn test_multi_byte_to_wide_char_cp932_truncated_lead_byte_errors() {
        // 0x93 is a Shift-JIS lead byte with no trail byte: invalid sequence
        // must be rejected because MB_ERR_INVALID_CHARS is set.
        let truncated = [0x93u8];
        let result = multi_byte_to_wide_char(CP932, MB_ERR_INVALID_CHARS_FLAG, &truncated);
        assert!(result.is_err(), "truncated SJIS lead byte should fail");
    }

    #[test]
    fn test_multi_byte_to_wide_char_utf8_invalid_bytes_error() {
        // 0xFF 0xFE is never valid UTF-8.
        let invalid = [0xFFu8, 0xFE];
        let result = multi_byte_to_wide_char(CP_UTF8, MB_ERR_INVALID_CHARS_FLAG, &invalid);
        assert!(result.is_err(), "invalid UTF-8 bytes should fail");
    }

    #[test]
    fn test_string_to_multibyte_cp932_encodes_shift_jis() {
        // "日本語" in Shift-JIS: 日 = 0x93FA, 本 = 0x967B, 語 = 0x8CEA
        let result = string_to_multibyte(CP932, "日本語", None).unwrap();
        assert_eq!(result, [0x93u8, 0xFA, 0x96, 0x7B, 0x8C, 0xEA]);
    }

    #[test]
    fn test_string_to_multibyte_cp932_roundtrip() {
        let original = "パス/テスト_01";
        let bytes = string_to_multibyte(CP932, original, None).unwrap();
        let restored =
            multi_byte_to_wide_char(CP932, MB_ERR_INVALID_CHARS_FLAG, &bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn test_string_to_multibyte_unmappable_errors_without_default_char() {
        // U+20AC (€) has no mapping in CP932; with no default char the
        // conversion must report InvalidInput instead of silently degrading.
        let result = string_to_multibyte(CP932, "€", None);
        let err = result.expect_err("unmappable char should fail without default_char");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn test_string_to_multibyte_unmappable_uses_default_char() {
        // With a default char supplied, unmappable input is substituted
        // instead of failing.
        let result = string_to_multibyte(CP932, "€", Some(b'?')).unwrap();
        assert_eq!(result, b"?");
    }

    #[test]
    fn test_buffer_len_to_i32_accepts_representable_lengths() {
        // Boundary regression for the FFI length guard (hardening):
        // representable lengths pass through unchanged.
        assert_eq!(buffer_len_to_i32(0).unwrap(), 0);
        assert_eq!(buffer_len_to_i32(1).unwrap(), 1);
        assert_eq!(
            buffer_len_to_i32(i32::MAX as usize).unwrap(),
            i32::MAX,
            "i32::MAX is the largest representable Win32 buffer length"
        );
    }

    #[test]
    fn test_buffer_len_to_i32_rejects_overlong_lengths() {
        // Boundary regression for the FFI length guard (hardening):
        // lengths above i32::MAX previously wrapped via `as` cast.
        // u32::MAX is the most dangerous value — it wraps to -1, which the
        // Win32 APIs treat as "null-terminated input" (out-of-bounds read).
        let just_over = i32::MAX as usize + 1;
        let err = buffer_len_to_i32(just_over).expect_err("i32::MAX+1 must be rejected");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let wraps_to_minus_one = u32::MAX as usize;
        let err = buffer_len_to_i32(wraps_to_minus_one).expect_err("u32::MAX must be rejected");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn test_wide_char_to_multi_byte_empty_input() {
        let (bytes, used_default) =
            wide_char_to_multi_byte(CP932, WC_COMPOSITECHECK_FLAG, &[], None, true).unwrap();
        assert!(bytes.is_empty());
        assert!(!used_default);
    }
}
