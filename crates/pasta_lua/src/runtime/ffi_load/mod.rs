//! `ffi.load` の置き換え（Windows のときだけ）。
//!
//! 設置（install）・Lua のラッパ・prepare の組み立て・pasta の理由の文言を持つ。
//! 名前の規則は [`resolve`]、`unsafe` の Windows の呼び出しは [`win32`] に分ける。
//!
//! 形は `searcher.rs` と同じく「Rust は値を返すだけ、送出は Lua のラッパ」。
//! Rust の `Err` を返すと `pcall` が受け取る値が文字列でなくなるため（6.4）。

mod resolve;
mod win32;

use crate::loader::LoaderContext;
use mlua::{Function, Lua, Table, Value};
use resolve::{Resolved, ansi_file_name, load_path_form, resolve};
use std::path::{Path, PathBuf};
use win32::{DllPins, free, load_ghost_dll, load_system_dll, module_named, system_message};
use windows_sys::Win32::Foundation::ERROR_MOD_NOT_FOUND;

/// `ffi.load` を包む Lua のラッパ。元の関数は末尾呼び出しで呼ぶので、元の関数が投げる文言の
/// 位置は作者のコードの位置になる。pasta の失敗は `error(msg, 2)` でラッパの呼び出し元の位置を付ける。
/// 引数は `...` のまま渡す（引数なしの呼び出しに `nil` を足すと、`got no value` の文言が変わる）。
/// 数の名前は LuaJIT（`lj_lib_checkstr`）と同じく `tostring` で文字列にして読む（同じ
/// `lj_strfmt_number` なので同じ文字列）。理由を UTF-8 にするため（6.2）。ほかの型と UTF-8 で
/// ない名前は元の関数へ同じ引数で渡す。
const WRAPPER_CHUNK: &str = r#"
local orig, prepare = ...
local type, tostring, select, error = type, tostring, select, error
return function(...)
  local name = ...
  if type(name) == "number" then
    name = tostring(name)
  end
  if type(name) == "string" then
    local pass, reason = prepare(name)
    if reason then
      error("cannot load module '" .. name .. "': " .. reason, 2)
    end
    if pass then
      return orig(pass, select(2, ...))
    end
  end
  return orig(...)
end
"#;

/// ファイル名が ANSI で表せない（2.8）。
fn reason_not_ansi(file_name: &str) -> String {
    format!("the file name '{file_name}' cannot be represented in the ANSI code page")
}

/// 同名の別 DLL が読み込み済み（5.2）。
fn reason_collision(file_name: &str) -> String {
    format!("a different DLL named '{file_name}' is already loaded in this process")
}

/// `package.loaded.ffi` があれば `ffi.load` を置き換え、`DllPins` を VM に持たせる。
/// FFI を開いていない VM では何もしない（8.1）。
///
/// `ffi` の表の `load` の 1 つだけを書き換え、ほかの FFI の機能と `package.*` には触らない
/// （1.6・8.3）。`load` が関数でなければ起動の失敗にする。
pub(crate) fn install(lua: &Lua, ctx: &LoaderContext) -> mlua::Result<()> {
    let loaded: Table = lua.globals().get::<Table>("package")?.get("loaded")?;
    let Value::Table(ffi) = loaded.get::<Value>("ffi")? else {
        return Ok(());
    };
    let Value::Function(orig) = ffi.raw_get::<Value>("load")? else {
        return Err(mlua::Error::runtime(
            "unexpected 'ffi' library: 'ffi.load' is not a function",
        ));
    };

    // DLL の参照は app data に置く。`lua_close` の後に落ちるので、`__gc`・`ffi.gc` の後始末が
    // すべて済んでから解放される（7.2）。
    lua.set_app_data(DllPins::default());

    let base_dir = ctx.base_dir.clone();
    let search_dirs = ctx.absolute_search_paths();
    let prepare = lua.create_function(move |lua, name: mlua::String| {
        prepare(lua, &name, &base_dir, &search_dirs)
    })?;
    let wrapper: Function = lua
        .load(WRAPPER_CHUNK)
        .set_name("=pasta_ffi_load")
        .into_function()?
        .call((orig, prepare))?;
    ffi.raw_set("load", wrapper)?;
    tracing::debug!("Installed ffi.load wrapper");
    Ok(())
}

/// ラッパから呼ぶ。Lua のエラーは投げない。
/// 戻り値: (Some(pass), None) = 元の ffi.load に pass を渡す
///         (None, Some(reason)) = ラッパが error(msg, 2) で送出する
///         (None, None) = UTF-8 でない名前。元の ffi.load に同じ引数で渡す
fn prepare(
    lua: &Lua,
    name: &mlua::String,
    base_dir: &Path,
    search_dirs: &[PathBuf],
) -> mlua::Result<(Option<mlua::String>, Option<String>)> {
    let Ok(name) = name.to_str() else {
        return Ok((None, None));
    };
    let name: &str = &name;
    let fail = |reason: String| Ok((None, Some(reason)));

    // 規則で解く。区切りのある名前のファイルが無ければ、そのパスを読んで失敗したときと同じ理由（3.7）。
    let (path, file_name) = match resolve(name, base_dir, search_dirs) {
        Resolved::Ghost { path, file_name } => (Some(path), file_name),
        Resolved::System { file_name } => (None, file_name),
        Resolved::Missing => return fail(system_message(ERROR_MOD_NOT_FOUND)),
    };
    // 元の ffi.load（LoadLibraryExA）に渡すファイル名は ANSI で表せること（2.8）。
    let Some(ansi) = ansi_file_name(&file_name) else {
        return fail(reason_not_ansi(&file_name));
    };

    // 先読み。理由は UTF-8 の Windows の文言（6.2）。
    let wide_file: Vec<u16> = file_name.encode_utf16().chain([0]).collect();
    let loaded = match &path {
        Some(path) => load_ghost_dll(&load_path_form(path)),
        None => load_system_dll(&wide_file),
    };
    let module = match loaded {
        Ok(m) => m,
        Err(code) => return fail(system_message(code)),
    };
    // ゴーストのファイルは、元の ffi.load がファイル名で開いたときに今回のものが返ること（5.1）。
    if path.is_some() && module_named(&wide_file) != Some(module) {
        free(module);
        return fail(reason_collision(&file_name));
    }

    lua.app_data_mut::<DllPins>()
        .ok_or_else(|| mlua::Error::runtime("ffi.load wrapper: DLL pins are missing"))?
        .pin(module);
    tracing::debug!(name, path = ?path, "Preloaded DLL for ffi.load");
    Ok((Some(lua.create_string(&ansi)?), None))
}
