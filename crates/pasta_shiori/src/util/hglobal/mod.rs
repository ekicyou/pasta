#![cfg(windows)]
pub(crate) mod enc;
mod windows_api;

use self::enc::{Encoder, Encoding};
use crate::error::{MyError, MyResult};
use std::ffi::OsString;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::str;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Memory::*;

const GMEM_FIXED: u32 = 0;

/// HGLOBALを文字列にキャプチャーします。
#[derive(Debug, PartialEq)]
pub(crate) struct ShioriString {
    h: HGLOBAL,
    len: usize,
    has_free: bool,
}

// SAFETY: ShioriString owns a single HGLOBAL handle. The handle is either:
// - captured from the SHIORI host (has_free=true, will be freed on drop), or
// - allocated by clone_from_slice_impl (has_free controlled by caller).
// Transfer between threads is safe because ShioriString has exclusive ownership
// of the HGLOBAL and does not alias it.
unsafe impl Sync for ShioriString {}
unsafe impl Send for ShioriString {}

impl Drop for ShioriString {
    fn drop(&mut self) {
        if !self.has_free {
            return;
        }
        // SAFETY: self.h was allocated via GlobalAlloc (in clone_from_slice_impl)
        // or captured from the SHIORI host. has_free=true guarantees this instance
        // owns the handle and is responsible for freeing it. Drop runs at most once.
        unsafe {
            GlobalFree(self.h);
        }
    }
}

impl ShioriString {
    /// HGLOBALをShioriStringにキャプチャーします。
    /// drop時にHGLOBALを開放します。
    /// shiori::load/requestのHGLOBAL受け入れに利用してください。
    pub(crate) fn capture(h: HGLOBAL, len: usize) -> ShioriString {
        ShioriString {
            h,
            len,
            has_free: true,
        }
    }

    /// GMEM_FIXEDブロックを確保し、失敗（null）を明示エラーで返します。
    /// 3.37 (G3): GlobalAlloc は失敗時に null を返す（プロセス終了はしない）。
    /// 旧実装は null を `from_raw_parts_mut` へ渡しており未定義動作だった。
    /// エラーは既存の `MyError::Script` に相乗りする（panic_to_error と同じ
    /// 技法）: pasta_shiori は crates.io 公開済みで `MyError` は
    /// non_exhaustive でないため、変種追加は semver-major（R3.5 違反）。
    fn alloc_global(len: usize) -> MyResult<HGLOBAL> {
        // SAFETY: GlobalAlloc has no preconditions; the null (failure)
        // return is checked before any pointer use.
        let h = unsafe { GlobalAlloc(GMEM_FIXED, len as _) };
        if h.is_null() {
            Err(MyError::Script {
                message: "Global memory allocation error".into(),
            })
        } else {
            Ok(h)
        }
    }

    /// &[u8]をHGLOBAL領域にコピーして返す。
    fn clone_from_slice_impl(bytes: &[u8], has_free: bool) -> MyResult<ShioriString> {
        let len = bytes.len();
        let h = Self::alloc_global(len)?;
        // SAFETY: from_raw_parts_mut is valid because:
        // - h is non-null (checked in alloc_global) and points to a block of
        //   exactly `len` bytes
        // - clone_from_slice copies exactly `len` bytes from the source
        unsafe {
            let p = h as *mut u8;
            let dst = from_raw_parts_mut::<u8>(p, len);
            dst[..].clone_from_slice(bytes);
        }
        Ok(ShioriString { h, len, has_free })
    }

    /// HGLOBALを新たに作成し、&[u8]をShioriStringにクローンします。
    /// drop時にHGLOBALを開放しません。
    /// shiori応答の作成に利用してください。
    #[cfg(test)]
    pub fn clone_from_slice_nofree(bytes: &[u8]) -> MyResult<ShioriString> {
        ShioriString::clone_from_slice_impl(bytes, false)
    }

    /// HGLOBALを新たに作成し、textをShioriStringにクローンします。
    /// drop時にHGLOBALを開放します。
    #[cfg(test)]
    pub fn clone_from_str<S: AsRef<str>>(text: S) -> MyResult<ShioriString> {
        let s = text.as_ref();
        let bytes = s.as_bytes();
        ShioriString::clone_from_slice_impl(bytes, true)
    }

    /// HGLOBALを新たに作成し、textをShioriStringにクローンします。
    /// drop時にHGLOBALを開放しません。
    pub(crate) fn clone_from_str_nofree<S: AsRef<str>>(text: S) -> MyResult<ShioriString> {
        let s = text.as_ref();
        let bytes = s.as_bytes();
        ShioriString::clone_from_slice_impl(bytes, false)
    }

    /// 要素を&[u8]として参照します。
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY: self.h and self.len were set together at construction time
        // (either via capture or clone_from_slice_impl), so the pointer is valid
        // for `self.len` bytes. The borrow lifetime is tied to &self.
        unsafe {
            let p = self.h as *mut u8;
            from_raw_parts::<u8>(p, self.len)
        }
    }

    /// HGLOBALハンドルを取得します。
    #[cfg(test)]
    pub fn handle(&self) -> HGLOBAL {
        self.h
    }

    /// 領域サイズを取得します。
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 領域が空かどうかを返します。
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// (HGLOBAL,len)を取得します。
    pub(crate) fn value(&self) -> (HGLOBAL, usize) {
        (self.h, self.len)
    }

    /// 格納データを「ANSI STRING(JP環境ではSJIS)」とみなして、OsStrに変換します。
    /// MultiByteToWideChar()を利用する。
    /// SHIORI::load()文字列の取り出しに利用する。
    pub(crate) fn to_ansi_str(&self) -> MyResult<OsString> {
        let bytes = self.as_bytes();
        let s = Encoding::ANSI
            .to_string(bytes)
            .map_err(|_| MyError::EncodeAnsi)?;
        let os_str = OsString::from(s);
        Ok(os_str)
    }

    /// 格納データを「UTF-8」とみなして、strに変換する。
    /// SHIORI::request()文字列の取り出しに利用する。
    pub(crate) fn to_utf8_str(&self) -> MyResult<&str> {
        let bytes = self.as_bytes();
        Ok(str::from_utf8(bytes)?)
    }
}

/// 3.37 (G3): GlobalAlloc failure must surface as an explicit Err, never as
/// a null pointer flowing into `from_raw_parts_mut` (undefined behavior).
/// A near-address-space-sized request makes GlobalAlloc fail deterministically.
/// The error rides on the existing `Script` variant (adding a variant to the
/// published non-`non_exhaustive` `MyError` would be semver-major).
/// RED evidence: with the null check removed (mutation), alloc_global returns
/// Ok(null) and this assertion fails.
#[test]
fn alloc_global_failure_returns_explicit_error() {
    match ShioriString::alloc_global(usize::MAX >> 1) {
        Err(MyError::Script { ref message }) => {
            assert_eq!(message, "Global memory allocation error");
        }
        other => panic!("Expected Err(Script {{ .. }}), got {other:?}"),
    }

    // Boundary: zero-length allocation is valid (empty SHIORI response).
    let h = ShioriString::alloc_global(0).expect("zero-length GlobalAlloc succeeds");
    drop(ShioriString::capture(h, 0));
}

/// G1 (3.35): to_utf8_str error path was untested — invalid UTF-8 bytes from
/// the host must surface as MyError::EncodeUtf8, not panic.
#[test]
fn to_utf8_str_invalid_bytes_returns_encode_utf8_error() {
    // 0xC0 0xAF is an over-long encoding — always invalid UTF-8.
    let src = ShioriString::clone_from_slice_impl(b"\xC0\xAF", true).unwrap();
    match src.to_utf8_str() {
        Err(MyError::EncodeUtf8(_)) => {}
        other => panic!("Expected Err(EncodeUtf8), got {other:?}"),
    }
}

/// G1 (3.35): zero-length boundary. A Lua script returning "" produces an
/// empty response payload through clone_from_str_nofree, so the empty
/// allocation path is reachable in production.
#[test]
fn empty_payload_roundtrip() {
    let src = ShioriString::clone_from_slice_impl(b"", true).unwrap();
    assert_eq!(src.len(), 0);
    assert!(src.is_empty());
    assert_eq!(src.to_utf8_str().unwrap(), "");
}

#[test]
fn shiori_string_test() {
    {
        let text = "適当なShioriString";
        let src = ShioriString::clone_from_slice_nofree(text.as_bytes()).unwrap();
        assert_eq!(src.to_utf8_str().unwrap(), text);
        assert_eq!(src.len(), 21);

        let dst = ShioriString::capture(src.handle(), src.len());
        assert_eq!(dst.to_utf8_str().unwrap(), text);
    }
    {
        // 3.39 (G5): this block asserts CP932 (Shift_JIS) byte lengths, which
        // only hold when the system ANSI code page is 932 (Japanese). On any
        // other ACP, Encoding::ANSI.to_bytes cannot represent the kanji and
        // errors, so the SJIS round-trip is skipped to keep the test
        // locale-independent (3.35 follow-up). The production decode path
        // (to_ansi_str) is locale-faithful by design: it always uses CP_ACP.
        let acp = unsafe { windows_sys::Win32::Globalization::GetACP() };
        if acp == 932 {
            let text = "適当なShioriString";
            let sjis = Encoding::ANSI.to_bytes(text).unwrap();
            assert_eq!(sjis.len(), 18);
            let src = ShioriString::clone_from_slice_nofree(&sjis[..]).unwrap();
            assert_eq!(src.len(), 18);
            let src_osstr = src.to_ansi_str().unwrap();
            assert_eq!(src_osstr.len(), 21);

            let dst = ShioriString::capture(src.handle(), src.len());
            assert_eq!(src_osstr, dst.to_ansi_str().unwrap());

            let src_str = src_osstr.to_str().unwrap();
            assert_eq!(src_str, text);
        } else {
            eprintln!("shiori_string_test: skipping SJIS round-trip block (ACP {acp} != 932)");
        }
    }
    {
        let text = "適当なShioriString";
        let src = ShioriString::clone_from_str(text).unwrap();
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(&*string).unwrap();
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(string).unwrap();
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(&string).unwrap();
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
}
