//! 8-bit string converters for Windows systems.
//! original: https://github.com/bozaro/local-encoding-rs/blob/master/src/windows.rs

use super::enc::Encoder;
#[cfg(test)]
use std::ffi::OsStr;
use std::io::{Error, ErrorKind, Result};
#[cfg(test)]
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Globalization::*;

/// Fail if an invalid input character is encountered.
pub const MB_ERR_INVALID_CHARS: u32 = 0x0000_0008;
/// Convert composite characters, consisting of a base character and a nonspacing character,
/// each with different character values.
#[cfg(test)]
pub const WC_COMPOSITECHECK: u32 = 0x0000_0200;
/// Replace exceptions with the default character during conversion.
#[cfg(test)]
pub const WC_DEFAULTCHAR: u32 = 0x0000_0040;
/// Fail if an invalid input character is encountered.
#[cfg(test)]
pub const WC_ERR_INVALID_CHARS: u32 = 0x0000_0080;

/// Encoding for use WinAPI calls: MultiByteToWideChar and WideCharToMultiByte.
pub struct EncoderCodePage(pub u32);

impl Encoder for EncoderCodePage {
    ///     Convert from bytes to string.
    fn to_string(&self, data: &[u8]) -> Result<String> {
        multi_byte_to_wide_char(self.0, MB_ERR_INVALID_CHARS, data)
    }

    /// Convert from string to bytes.
    #[cfg(test)]
    fn to_bytes(&self, data: &str) -> Result<Vec<u8>> {
        string_to_multibyte(self.0, data, None)
    }
}

/// Convert String to 8-bit string.
///
/// * `codepage`     - Code page to use in performing the conversion. This parameter can be set to
///   the value of any code page that is installed or available in the operating
///   system.
/// * `data`         - Source string.
/// * `default_char` - Optional character for replace to use if a character cannot be represented
///   in the specified code page.
///
/// Returns `Err` if an invalid input character is encountered and `default_char` is `None`.
#[cfg(test)]
pub fn string_to_multibyte(codepage: u32, data: &str, default_char: Option<u8>) -> Result<Vec<u8>> {
    let wstr: Vec<u16> = OsStr::new(data).encode_wide().collect();
    wide_char_to_multi_byte(
        codepage,
        WC_COMPOSITECHECK,
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
/// See https://msdn.microsoft.com/en-us/library/windows/desktop/dd319072(v=vs.85).aspx
/// for more details.
pub fn multi_byte_to_wide_char(codepage: u32, flags: u32, multi_byte_str: &[u8]) -> Result<String> {
    // Empty string
    if multi_byte_str.is_empty() {
        return Ok(String::new());
    }
    // SAFETY: MultiByteToWideChar is called with valid inputs:
    // - codepage and flags are caller-provided API parameters
    // - multi_byte_str.as_ptr() is valid for multi_byte_str.len() bytes (Rust slice guarantee)
    // - Empty input is handled above before entering the unsafe block
    // First call with null output gets required buffer length;
    // second call fills the pre-allocated Vec.
    unsafe {
        let len = MultiByteToWideChar(
            codepage,
            flags,
            multi_byte_str.as_ptr() as _,
            multi_byte_str.len() as _,
            ptr::null_mut(),
            0,
        );
        if len > 0 {
            // Convert to UTF-16
            // SAFETY: set_len is safe here because MultiByteToWideChar with a non-null
            // output buffer will fully initialize exactly `len` u16 elements.
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
/// See https://msdn.microsoft.com/ru-ru/library/windows/desktop/dd374130(v=vs.85).aspx
/// for more details.
#[cfg(test)]
pub fn wide_char_to_multi_byte(
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
    // SAFETY: WideCharToMultiByte is called with valid inputs:
    // - codepage and flags are caller-provided API parameters
    // - wide_char_str.as_ptr() is valid for wide_char_str.len() u16 elements (Rust slice guarantee)
    // - Empty input is handled above before entering the unsafe block
    // First call with null output gets required buffer length;
    // second call fills the pre-allocated Vec.
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
            // Convert from UTF-16 to multibyte
            // SAFETY: set_len is safe here because WideCharToMultiByte with a non-null
            // output buffer will fully initialize up to `len` bytes.
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

#[test]
fn multi_byte_to_wide_char_empty() {
    assert_eq!(
        multi_byte_to_wide_char(CP_ACP, MB_ERR_INVALID_CHARS, b"").unwrap(),
        ""
    );
}

#[test]
fn multi_byte_to_wide_char_ascii() {
    assert_eq!(
        multi_byte_to_wide_char(CP_ACP, MB_ERR_INVALID_CHARS, b"Test").unwrap(),
        "Test"
    );
}

#[test]
fn multi_byte_to_wide_char_utf8() {
    assert_eq!(
        multi_byte_to_wide_char(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            b"\xD0\xA2\xD0\xB5\xD1\x81\xD1\x82"
        )
        .unwrap(),
        "Тест"
    );
}

#[test]
fn multi_byte_to_wide_char_invalid() {
    assert!(multi_byte_to_wide_char(CP_UTF8, MB_ERR_INVALID_CHARS, b"Test\xC0").is_err());
}

#[test]
fn wide_char_to_multi_byte_empty() {
    assert_eq!(
        wide_char_to_multi_byte(CP_UTF8, WC_ERR_INVALID_CHARS, &[], None, false).unwrap(),
        (b"".to_vec(), false)
    );
}

#[test]
fn wide_char_to_multi_byte_ascii() {
    assert_eq!(
        wide_char_to_multi_byte(
            CP_ACP,
            WC_COMPOSITECHECK,
            &[0x0054, 0x0065, 0x0073, 0x0074],
            None,
            true
        )
        .unwrap(),
        (b"Test".to_vec(), false)
    );
}

#[test]
fn wide_char_to_multi_byte_utf8() {
    assert_eq!(
        wide_char_to_multi_byte(CP_UTF8, WC_ERR_INVALID_CHARS, &[0x6F22], None, false).unwrap(),
        (b"\xE6\xBC\xA2".to_vec(), false)
    );
}

#[test]
fn wide_char_to_multi_byte_replace() {
    assert_eq!(
        wide_char_to_multi_byte(
            CP_ACP,
            WC_DEFAULTCHAR | WC_COMPOSITECHECK,
            &[0x0054, 0x0065, 0x0073, 0x0074, 0xFFFF, 0x0029],
            Some(b':'),
            true
        )
        .unwrap(),
        (b"Test:)".to_vec(), true)
    );
}

#[test]
fn wide_char_to_multi_byte_invalid() {
    assert_eq!(
        wide_char_to_multi_byte(CP_ACP, WC_COMPOSITECHECK, &[0xFFFF], Some(b':'), true).unwrap(),
        (b":".to_vec(), true)
    );
    assert_eq!(
        wide_char_to_multi_byte(CP_ACP, WC_COMPOSITECHECK, &[0x0020], Some(b':'), true).unwrap(),
        (b" ".to_vec(), false)
    );
}

#[cfg(test)]
mod tests {
    use super::super::Encoder;
    use super::*;

    #[test]
    fn cp1251_to_string_test() {
        assert_eq!(
            EncoderCodePage(1251)
                .to_string(b"\xD2\xE5\xF1\xF2")
                .unwrap(),
            "Тест"
        );
    }
    #[test]
    fn string_to_cp1251_test() {
        assert_eq!(
            EncoderCodePage(1251).to_bytes("Тест").unwrap(),
            b"\xD2\xE5\xF1\xF2"
        );
    }

    #[test]
    fn cp866_to_string_test() {
        assert_eq!(
            EncoderCodePage(866).to_string(b"\x92\xA5\xE1\xE2").unwrap(),
            "Тест"
        );
    }

    #[test]
    fn string_to_cp866_test() {
        assert_eq!(
            EncoderCodePage(866).to_bytes("Тест").unwrap(),
            b"\x92\xA5\xE1\xE2"
        );
    }
}
