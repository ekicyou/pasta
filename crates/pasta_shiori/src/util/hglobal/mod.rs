#![cfg(windows)]
pub mod enc;
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
pub struct ShioriString {
    h: HGLOBAL,
    len: usize,
    has_free: bool,
}

unsafe impl Sync for ShioriString {}
unsafe impl Send for ShioriString {}

impl Drop for ShioriString {
    fn drop(&mut self) {
        if !self.has_free {
            return;
        }
        unsafe {
            GlobalFree(self.h);
        }
    }
}

impl ShioriString {
    /// HGLOBALをShioriStringにキャプチャーします。
    /// drop時にHGLOBALを開放します。
    /// shiori::load/requestのHGLOBAL受け入れに利用してください。
    pub fn capture(h: HGLOBAL, len: usize) -> ShioriString {
        ShioriString {
            h,
            len,
            has_free: true,
        }
    }

    /// &[u8]をHGLOBAL領域にコピーして返す。
    fn clone_from_slice_impl(bytes: &[u8], has_free: bool) -> ShioriString {
        let len = bytes.len();
        unsafe {
            let h = GlobalAlloc(GMEM_FIXED, len as _);
            let p = h as *mut u8;
            let dst = from_raw_parts_mut::<u8>(p, len);
            dst[..].clone_from_slice(bytes);
            ShioriString { h, len, has_free }
        }
    }

    /// HGLOBALを新たに作成し、&[u8]をShioriStringにクローンします。
    /// drop時にHGLOBALを開放しません。
    /// shiori応答の作成に利用してください。
    pub fn clone_from_slice_nofree(bytes: &[u8]) -> ShioriString {
        ShioriString::clone_from_slice_impl(bytes, false)
    }

    /// HGLOBALを新たに作成し、textをShioriStringにクローンします。
    /// drop時にHGLOBALを開放します。
    #[allow(dead_code)]
    pub fn clone_from_str<S: AsRef<str>>(text: S) -> ShioriString {
        let s = text.as_ref();
        let bytes = s.as_bytes();
        ShioriString::clone_from_slice_impl(bytes, true)
    }

    /// HGLOBALを新たに作成し、textをShioriStringにクローンします。
    /// drop時にHGLOBALを開放しません。
    #[allow(dead_code)]
    pub fn clone_from_str_nofree<'a, S: AsRef<str>>(text: S) -> ShioriString {
        let s = text.as_ref();
        let bytes = s.as_bytes();
        ShioriString::clone_from_slice_impl(bytes, false)
    }

    /// 要素を&[u8]として参照します。
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let p = self.h as *mut u8;
            from_raw_parts::<u8>(p, self.len)
        }
    }

    /// HGLOBALハンドルを取得します。
    #[allow(dead_code)]
    pub fn handle(&self) -> HGLOBAL {
        self.h
    }

    /// 領域サイズを取得します。
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 領域が空かどうかを返します。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// (HGLOBAL,len)を取得します。
    #[allow(dead_code)]
    pub fn value(&self) -> (HGLOBAL, usize) {
        (self.h, self.len)
    }

    /// 格納データを「ANSI STRING(JP環境ではSJIS)」とみなして、OsStrに変換します。
    /// MultiByteToWideChar()を利用する。
    /// SHIORI::load()文字列の取り出しに利用する。
    pub fn to_ansi_str(&self) -> MyResult<OsString> {
        let bytes = self.as_bytes();
        let s = Encoding::ANSI
            .to_string(bytes)
            .map_err(|_| MyError::EncodeAnsi)?;
        let os_str = OsString::from(s);
        Ok(os_str)
    }

    /// 格納データを「UTF-8」とみなして、strに変換する。
    /// SHIORI::request()文字列の取り出しに利用する。
    pub fn to_utf8_str(&self) -> MyResult<&str> {
        let bytes = self.as_bytes();
        Ok(str::from_utf8(bytes)?)
    }
}

#[test]
fn shiori_string_test() {
    {
        let text = "適当なShioriString";
        let src = ShioriString::clone_from_slice_nofree(text.as_bytes());
        assert_eq!(src.to_utf8_str().unwrap(), text);
        assert_eq!(src.len(), 21);

        let dst = ShioriString::capture(src.handle(), src.len());
        assert_eq!(dst.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let sjis = Encoding::ANSI.to_bytes(text).unwrap();
        assert_eq!(sjis.len(), 18);
        let src = ShioriString::clone_from_slice_nofree(&sjis[..]);
        assert_eq!(src.len(), 18);
        let src_osstr = src.to_ansi_str().unwrap();
        assert_eq!(src_osstr.len(), 21);

        let dst = ShioriString::capture(src.handle(), src.len());
        assert_eq!(src_osstr, dst.to_ansi_str().unwrap());

        let src_str = src_osstr.to_str().unwrap();
        assert_eq!(src_str, text);
    }
    {
        let text = "適当なShioriString";
        let src = ShioriString::clone_from_str(text);
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(&*string);
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(string);
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
    {
        let text = "適当なShioriString";
        let string = text.to_owned();
        let src = ShioriString::clone_from_str(&string);
        assert_eq!(src.to_utf8_str().unwrap(), text);
    }
}
