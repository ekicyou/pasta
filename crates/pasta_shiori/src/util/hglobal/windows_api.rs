//! ANSI のバイト列を UTF-16 経由で文字列にする（MultiByteToWideChar）。
//! original: https://github.com/bozaro/local-encoding-rs/blob/master/src/windows.rs

use std::io::{Error, ErrorKind, Result};
use std::ptr;
use windows_sys::Win32::Globalization::*;

/// Fail if an invalid input character is encountered.
pub(crate) const MB_ERR_INVALID_CHARS: u32 = 0x0000_0008;

/// Convert a Rust buffer length to the `i32` the Win32 conversion APIs take.
/// 3.37 (G3): lengths above `i32::MAX` are rejected — a plain `as` cast would
/// wrap (e.g. `u32::MAX as i32 == -1`, which `MultiByteToWideChar`
/// interprets as "null-terminated input", reading past
/// the end of the borrowed slice). Same defect class as pasta_lua 3.11.
fn buffer_len_to_i32(len: usize) -> Result<i32> {
    i32::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            "Input too large for Windows code page conversion (must be < 2 GiB)",
        )
    })
}

/// Wrapper for MultiByteToWideChar.
///
/// See https://msdn.microsoft.com/en-us/library/windows/desktop/dd319072(v=vs.85).aspx
/// for more details.
pub(crate) fn multi_byte_to_wide_char(
    codepage: u32,
    flags: u32,
    multi_byte_str: &[u8],
) -> Result<String> {
    // Empty string
    if multi_byte_str.is_empty() {
        return Ok(String::new());
    }
    // Reject lengths the Win32 API cannot represent (prevents `as` cast wrap).
    let in_len = buffer_len_to_i32(multi_byte_str.len())?;
    // SAFETY: MultiByteToWideChar is called with valid inputs:
    // - codepage and flags are caller-provided API parameters
    // - multi_byte_str.as_ptr() is valid for multi_byte_str.len() bytes (Rust slice guarantee)
    // - in_len exactly matches the slice length (guarded above)
    // - Empty input is handled above before entering the unsafe block
    // First call with null output gets required buffer length;
    // second call fills the pre-allocated Vec.
    unsafe {
        let len = MultiByteToWideChar(
            codepage,
            flags,
            multi_byte_str.as_ptr() as _,
            in_len,
            ptr::null_mut(),
            0,
        );
        if len > 0 {
            let mut wstr: Vec<u16> = vec![0u16; len as usize];
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

// ----------------------------------------------------------------------
// 3.37 (G3): boundary regression for the FFI length guard.
// Direct >2GiB inputs cannot be allocated in a test, so the guard is pinned
// at the helper level (the production function feeds every length through
// it). RED evidence: with the guard mutated back to `Ok(len as i32)`, the
// rejection test fails (u32::MAX wraps to -1 == "null-terminated input").
// ----------------------------------------------------------------------

#[test]
fn buffer_len_to_i32_accepts_representable_lengths() {
    assert_eq!(buffer_len_to_i32(0).unwrap(), 0);
    assert_eq!(buffer_len_to_i32(1).unwrap(), 1);
    assert_eq!(
        buffer_len_to_i32(i32::MAX as usize).unwrap(),
        i32::MAX,
        "i32::MAX is the largest representable Win32 buffer length"
    );
}

#[test]
fn buffer_len_to_i32_rejects_overlong_lengths() {
    let just_over = i32::MAX as usize + 1;
    let err = buffer_len_to_i32(just_over).expect_err("i32::MAX+1 must be rejected");
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    // u32::MAX is the most dangerous value — it wraps to -1, which the
    // Win32 APIs treat as "null-terminated input" (out-of-bounds read).
    let wraps_to_minus_one = u32::MAX as usize;
    let err = buffer_len_to_i32(wraps_to_minus_one).expect_err("u32::MAX must be rejected");
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
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
