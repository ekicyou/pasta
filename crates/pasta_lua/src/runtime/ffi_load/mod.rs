//! `ffi.load` の置き換え（Windows のときだけ）。
//!
//! 設置（install）・Lua のラッパ・prepare の組み立て・pasta の理由の文言を持つ。
//! 名前の規則は [`resolve`]、`unsafe` の Windows の呼び出しは [`win32`] に分ける。

mod resolve;
mod win32;
