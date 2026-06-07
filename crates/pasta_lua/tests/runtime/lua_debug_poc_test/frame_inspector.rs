//! FrameInspector: 安全 API によるフレーム情報取得（R3.1）と、FFI 経由の
//! ローカル/upvalue 取得（R3.2〜R3.4）。task 2.3 を実装する。
//!
//! 兄弟モジュール `harness_types`（`Variable` / `FrameInfo` / `build_jit_off_vm()`）
//! を再利用する。
//!
//! ## 設計準拠（design.md "FrameInspector" / requirements.md R3.1〜R3.4 / research D2）
//! - `frame_info`: **安全 API のみ**（`Debug::source` / `Debug::current_line` /
//!   `Debug::names`）でソース名・行番号・関数名を取得（R3.1）。`std_debug` には
//!   一切依存しない（サンドボックス維持・R5.3）。
//! - `inspect_locals` / `inspect_upvalues`: `Lua::exec_raw` で生 `lua_State` を取り、
//!   `lua_getstack` で停止フレームのアクティベーションレコードを再取得し、
//!   `lua_getlocal` / `lua_getupvalue` で名前と値を取得（R3.2）。基本型
//!   （number / string / boolean / table）を `lua_type` で判別し、値の `repr` を
//!   構築する（R3.2）。`std_debug` 非露出のまま成立させることが第一目標（R3.3）。
//!   取得不可・制限される種別は範囲外として `Variable.notes`（=`type_name`）へ
//!   記録し、クラッシュさせない（R3.4）。
//!
//! ## mlua 0.11.6 / mlua-sys 0.10.0（luajit52 = lua51 モジュール）API（実機検証済み）
//! - `unsafe Lua::exec_raw::<R: FromLuaMulti>(args: impl IntoLuaMulti,
//!   f: impl FnOnce(*mut ffi::lua_State)) -> Result<R>`。
//!   返り値はクロージャ終了時の**スタック上の値**（`nresults = gettop - stack_start`）
//!   から構築される。本実装は値を捕捉した `Vec` で外へ運び、スタックには何も
//!   残さない（純増 0）ため `R = ()` を用いる。
//! - `ffi::lua_getstack(L, level, *mut lua_Debug) -> c_int`（0=失敗, 非0=成功）。
//! - `ffi::lua_getinfo(L, what: *const c_char, *mut lua_Debug) -> c_int`。
//!   `"Sl"` で `source`/`what`/`currentline` を、`"f"` で関数をスタックに push。
//! - `ffi::lua_getlocal(L, *const lua_Debug, n) -> *const c_char`（n=1.. を順に。
//!   非 null なら値を**スタックに push**し名前を返す。null で終端）。
//! - `ffi::lua_getupvalue(L, funcindex, n) -> *const c_char`（同様。値を push）。
//! - `ffi::lua_type(L, idx) -> c_int` ＋ `LUA_TNUMBER/LUA_TSTRING/LUA_TBOOLEAN/LUA_TTABLE`。
//! - `ffi::lua_tonumber(L, idx) -> f64` / `lua_toboolean(L, idx) -> c_int` /
//!   `lua_tolstring(L, idx, *mut usize) -> *const c_char`。
//! - `ffi::lua_gettop(L) -> c_int` / `ffi::lua_pop(L, n)` / `ffi::lua_settop(L, idx)`。
//! - `ffi::lua_Debug` は private field（`i_ci`）を持つため `mem::zeroed()` で生成する。
//!
//! ## スタック規律（design "Error Handling": VM 破壊回避）
//! 全 `unsafe` を `exec_raw` クロージャ内に限局し、クロージャ入口で `lua_gettop` を
//! 記録、出口で `lua_settop` により**必ず同一深さへ復元**する。値を読むたびに
//! 即 `lua_pop(1)` し、関数を push した場合も最後に pop する。push/pop が対称で
//! あることを `debug_assert!` で自己検証する。
//!
//! ## R3 の最重要知見（exec_raw とフックフレームの関係・stack/state 整合）
//! mlua のフックコールバックに渡る `&Lua` は**常にメインステート**を指し
//! （`Debug::new(rawlua, 0, ar)` の `state = main_state`）、`exec_raw` も同じ
//! メインステートで動く。さらに `exec_raw` のクロージャは内部で `lua_pcall`
//! （`do_call` C 関数）越しに呼ばれるため、**level 0 は C フレーム**であり、
//! 停止対象の Lua フレームはそれより上位にある。よって固定 level を仮定せず、
//! `lua_getstack` を level 0 から走査して**最初の Lua フレーム**
//! （`lua_getinfo("S")` の `what` が `"Lua"`）を停止フレームとして採用する。
//! これにより「現フレーム ar を再取得し lua_getlocal を呼ぶ」(R3.2) を
//! exec_raw の現実的制約下で正しく満たす。
//!
//! コルーチン本体の停止フレームはコルーチン固有の `lua_State` 上にあり、メイン
//! ステートからは到達できない（R3 既知制約）。本検証はメインスレッド（トップ
//! レベル）Lua フレームでの停止に対して FFI ローカル/upvalue 取得を実証し、
//! コルーチンフレームの制約は R3.4 として記録する。

#![allow(dead_code)]

use std::os::raw::c_int;

use mlua::{Debug, Lua};

use super::harness_types::{FrameInfo, Variable};

// ---------------------------------------------------------------------------
// (A) 安全 API によるフレーム情報取得（R3.1）
// ---------------------------------------------------------------------------

/// 安全 mlua API のみで停止フレームの情報を取得する（R3.1）。
///
/// `Debug::source()`（チャンク名＝`source`、無ければ `short_src`）、
/// `Debug::current_line()`（行番号）、`Debug::names()`（関数名）を用いる。
/// `std_debug` には依存しない（R5.3）。
pub(crate) fn frame_info(debug: &Debug) -> FrameInfo {
    let src = debug.source();
    let source = src
        .source
        .as_ref()
        .map(|c| c.as_ref().to_string())
        .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
        .unwrap_or_default();
    let line = debug.current_line().unwrap_or(0) as u32;
    let func_name = debug.names().name.map(|c| c.as_ref().to_string());
    FrameInfo {
        source,
        line,
        func_name,
    }
}

// ---------------------------------------------------------------------------
// (B) FFI 値の型判別と repr 構築（unsafe 補助・exec_raw クロージャ内専用）
// ---------------------------------------------------------------------------

/// スタックトップ（idx = -1）に積まれた値の (type_name, repr) を **読むだけ**で
/// 構築する（pop は呼び出し側が行う）。
///
/// number / string / boolean / table を判別し、それ以外は範囲外種別として
/// `lua_typename` による型名を返す（R3.2 基本型判別 / R3.4 未対応種別の記録）。
///
/// # Safety
/// `L` は有効な `lua_State`、idx=-1 に少なくとも 1 値が積まれていること。
/// `lua_tolstring` は string/number に対してのみ呼ぶ（他種別では値を mutate
/// しうるため呼ばない）。本関数は **read-only**（スタック深さを変えない）。
unsafe fn read_value_at_top(l: *mut mlua::ffi::lua_State) -> (String, String) {
    unsafe {
        let t = mlua::ffi::lua_type(l, -1);
        match t {
        mlua::ffi::LUA_TNUMBER => {
            let n = mlua::ffi::lua_tonumber(l, -1);
            // 整数なら整数表記、そうでなければ浮動小数表記で repr を作る。
            let repr = if n.fract() == 0.0 && n.is_finite() {
                format!("{}", n as i64)
            } else {
                format!("{n}")
            };
            ("number".to_string(), repr)
        }
        mlua::ffi::LUA_TSTRING => {
            // string は lua_tolstring で安全に読める（既に string 型なので coerce
            // による mutate は起きない）。
            let mut len: usize = 0;
            let ptr = mlua::ffi::lua_tolstring(l, -1, &mut len as *mut usize);
            let repr = if ptr.is_null() {
                String::new()
            } else {
                let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
                String::from_utf8_lossy(bytes).into_owned()
            };
            ("string".to_string(), repr)
        }
        mlua::ffi::LUA_TBOOLEAN => {
            let b = mlua::ffi::lua_toboolean(l, -1) != 0;
            ("boolean".to_string(), b.to_string())
        }
        mlua::ffi::LUA_TTABLE => {
            // table の中身展開は範囲外（R3 PoC）。アドレスをプレースホルダ repr とする。
            let addr = mlua::ffi::lua_topointer(l, -1);
            ("table".to_string(), format!("table: {addr:p}"))
        }
        other => {
            // 範囲外種別（function / userdata / thread / nil / cdata 等）は
            // 型名のみ記録し、クラッシュさせない（R3.4）。
            let name_ptr = mlua::ffi::lua_typename(l, other);
            let type_name = if name_ptr.is_null() {
                format!("type#{other}")
            } else {
                std::ffi::CStr::from_ptr(name_ptr)
                    .to_string_lossy()
                    .into_owned()
            };
            let repr = format!("<unsupported {type_name}>");
            (type_name, repr)
        }
        }
    }
}

/// `lua_getstack` を level 0 から走査し、最初の **Lua フレーム**の level を返す。
///
/// `exec_raw` クロージャは内部 `lua_pcall`（C 関数 `do_call`）越しに呼ばれるため
/// level 0 は C フレームになる。停止対象の Lua フレームはそれより上位にあるので、
/// `lua_getinfo("S")` の `what` が `"Lua"`（あるいは `"main"`）となる最初の level を
/// 採用する。見つからなければ `None`。
///
/// # Safety
/// `L` は有効な `lua_State`。`ar` は呼び出し側が所有する `lua_Debug`。
/// 本関数は `lua_getinfo("S")`（スタックに値を積まない what）のみ用いるため、
/// スタック深さを変えない。
unsafe fn find_first_lua_frame(
    l: *mut mlua::ffi::lua_State,
    ar: *mut mlua::ffi::lua_Debug,
) -> Option<c_int> {
    unsafe {
        let what_s = std::ffi::CString::new("S").expect("static literal");
        let mut level: c_int = 0;
        // 上限を設けて無限走査を防ぐ（通常は数フレーム以内に Lua フレームがある）。
        while level < 64 {
            if mlua::ffi::lua_getstack(l, level, ar) == 0 {
                // これ以上スタックが無い。
                return None;
            }
            if mlua::ffi::lua_getinfo(l, what_s.as_ptr(), ar) != 0 {
                let what_ptr = (*ar).what;
                if !what_ptr.is_null() {
                    let what = std::ffi::CStr::from_ptr(what_ptr).to_string_lossy();
                    // LuaJIT/Lua 5.1 では Lua 関数は "Lua"、メインチャンクは "main"。
                    if what == "Lua" || what == "main" {
                        return Some(level);
                    }
                }
            }
            level += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// (C) FFI ローカル取得（R3.2）
// ---------------------------------------------------------------------------

/// 停止フレームのローカル変数を名前＋型＋値で取得する（R3.2）。
///
/// `Lua::exec_raw` で生 `lua_State` を取り、`find_first_lua_frame` で停止対象の
/// Lua フレーム level を特定、`lua_getstack(L, level, &ar)` で ar を確定してから
/// `lua_getlocal(L, &ar, n)` を n=1,2,... と回す。各 `lua_getlocal` は非 null の
/// とき値を**スタックに push** するので、読み終えたら必ず `lua_pop(1)` で戻す。
/// 名前が `(*temporary)` 等の内部スロット名のものはスキップせず取得対象とする
/// （PoC では全ローカルを観測対象とする）。
///
/// スタック規律: クロージャ入口で `lua_gettop` を記録し、出口で `lua_settop` に
/// より同一深さへ必ず復元する（VM 破壊回避・design "Error Handling"）。
pub(crate) fn inspect_locals(lua: &Lua) -> mlua::Result<Vec<Variable>> {
    let mut out: Vec<Variable> = Vec::new();
    // exec_raw クロージャの返り値はスタック純増 0（R=()）。値は out へ捕捉する。
    unsafe {
        lua.exec_raw::<()>((), |l| {
            let top_at_entry = mlua::ffi::lua_gettop(l);

            // lua_Debug は private field を含むため zeroed で生成する。
            let mut ar: mlua::ffi::lua_Debug = std::mem::zeroed();

            if let Some(level) = find_first_lua_frame(l, &mut ar as *mut _) {
                // 採用 level で ar を確定（find_* で最後に走査した level と一致するが
                // 明示的に取り直して ar を確実に当該フレームへ向ける）。
                if mlua::ffi::lua_getstack(l, level, &mut ar as *mut _) != 0 {
                    let mut n: c_int = 1;
                    loop {
                        let name_ptr =
                            mlua::ffi::lua_getlocal(l, &ar as *const _, n);
                        if name_ptr.is_null() {
                            // これ以上ローカルは無い（値も push されていない）。
                            break;
                        }
                        // 非 null: 値が 1 つ push された。名前を読む。
                        let name = std::ffi::CStr::from_ptr(name_ptr)
                            .to_string_lossy()
                            .into_owned();
                        let (type_name, repr) = read_value_at_top(l);
                        // 値を pop してスタックを元に戻す。
                        mlua::ffi::lua_pop(l, 1);

                        out.push(Variable {
                            name,
                            type_name,
                            repr,
                        });
                        n += 1;
                    }
                }
            }

            // スタックを入口時の深さへ必ず復元（VM 破壊回避）。
            let top_at_exit = mlua::ffi::lua_gettop(l);
            debug_assert_eq!(
                top_at_entry, top_at_exit,
                "inspect_locals must keep the VM stack balanced (push/pop symmetric)"
            );
            mlua::ffi::lua_settop(l, top_at_entry);
        })?;
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// (D) FFI upvalue 取得（R3.2）
// ---------------------------------------------------------------------------

/// 指定 level のフレームで走っている関数の upvalue を名前＋型＋値で取得する（R3.2）。
///
/// `lua_getstack(L, level, &ar)` で ar を取り、`lua_getinfo(L, "f", &ar)` で当該
/// 関数を**スタックに push**（このとき関数は idx -1）、`lua_getupvalue(L, -1, n)` を
/// n=1,2,... と回す。各 `lua_getupvalue` は非 null のとき upvalue 値を push する
/// ので、読み終えたら `lua_pop(1)`。最後に関数自身も pop してスタックを復元する。
///
/// `level` 引数は呼び出し側が論理 level（停止フレーム）を指定するためのものだが、
/// `exec_raw` の C フレーム介在を吸収するため、まず `find_first_lua_frame` で得た
/// 実 level を基準（offset 0）とし、そこへ呼び出し側 `level` を加算して対象を選ぶ。
/// 通常は `level = 0`（停止フレームそのもの）を渡す。
pub(crate) fn inspect_upvalues(lua: &Lua, level: i32) -> mlua::Result<Vec<Variable>> {
    let mut out: Vec<Variable> = Vec::new();
    unsafe {
        lua.exec_raw::<()>((), |l| {
            let top_at_entry = mlua::ffi::lua_gettop(l);
            let mut ar: mlua::ffi::lua_Debug = std::mem::zeroed();

            if let Some(base) = find_first_lua_frame(l, &mut ar as *mut _) {
                let target = base + level as c_int;
                if mlua::ffi::lua_getstack(l, target, &mut ar as *mut _) != 0 {
                    let what_f = std::ffi::CString::new("f").expect("static literal");
                    // "f": 当該 level の関数をスタックに push する。
                    if mlua::ffi::lua_getinfo(l, what_f.as_ptr(), &mut ar as *mut _) != 0 {
                        let func_index = mlua::ffi::lua_gettop(l); // push された関数の絶対 idx
                        let mut n: c_int = 1;
                        loop {
                            let name_ptr = mlua::ffi::lua_getupvalue(l, func_index, n);
                            if name_ptr.is_null() {
                                break;
                            }
                            let name = std::ffi::CStr::from_ptr(name_ptr)
                                .to_string_lossy()
                                .into_owned();
                            let (type_name, repr) = read_value_at_top(l);
                            mlua::ffi::lua_pop(l, 1); // upvalue 値を pop
                            out.push(Variable {
                                name,
                                type_name,
                                repr,
                            });
                            n += 1;
                        }
                        // 関数自身を pop（func_index に積んだもの）。
                        mlua::ffi::lua_pop(l, 1);
                    }
                }
            }

            let top_at_exit = mlua::ffi::lua_gettop(l);
            debug_assert_eq!(
                top_at_entry, top_at_exit,
                "inspect_upvalues must keep the VM stack balanced (push/pop symmetric)"
            );
            mlua::ffi::lua_settop(l, top_at_entry);
        })?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_debug_poc_test::harness_types::{build_jit_off_vm, ItemOutcome};
    use mlua::{HookTriggers, VmState};
    use std::sync::{Arc, Mutex};

    /// 記録した変数集合から名前で 1 件を引く（非空虚 assert 用）。
    fn find_var<'a>(vars: &'a [Variable], name: &str) -> Option<&'a Variable> {
        vars.iter().find(|v| v.name == name)
    }

    /// R3.1 / R3.2 / R3.3:
    /// jit.off 済み VM のトップレベル Lua チャンク内に number / string / boolean /
    /// table のローカルを置き、指定行で line hook を撃つ。フック内で
    /// `inspect_locals` を呼び、取得した `Vec<Variable>` を共有バッファへ格納する。
    /// 走行後、4 基本型が**名前付き・型判別付き**で取得できていること、および
    /// `debug`（std_debug）が依然 nil（サンドボックス維持・R5.3/R3.3）を assert する。
    #[test]
    fn inspect_locals_via_ffi_basic_types() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        // 停止対象行（チャンク `@locals_chunk` の 5 行目）。4 ローカルが全て可視に
        // なった直後の行で inspect する。
        let target_line: u32 = 6;
        let captured_hook = Arc::clone(&captured);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let info = frame_info(debug);
            if info.source == "@locals_chunk" && info.line == target_line {
                // フック内（VM スレッド）で FFI ローカル取得を実行。
                let vars = inspect_locals(hook_lua)
                    .expect("inspect_locals must not error inside the hook");
                if let Ok(mut g) = captured_hook.lock() {
                    if g.is_empty() {
                        *g = vars;
                    }
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        // 1: num, 2: str, 3: flag(bool), 4: tbl(table) を宣言し、6 行目（marker）で停止。
        // marker 行までに 4 ローカルが全てスコープ内に入る。
        let chunk = "\
local num = 42
local str = 'hello'
local flag = true
local tbl = { 1, 2, 3 }
local marker = num
return marker
";
        lua.load(chunk)
            .set_name("@locals_chunk")
            .exec()
            .expect("chunk should execute");

        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "inspect_locals must capture locals at the breakpoint (R3.2). got empty"
        );

        // number: num == 42
        let num = find_var(&vars, "num")
            .unwrap_or_else(|| panic!("local 'num' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(num.type_name, "number", "num must be discriminated as number (R3.2)");
        assert_eq!(num.repr, "42", "num value must be readable as 42");

        // string: str == hello
        let s = find_var(&vars, "str")
            .unwrap_or_else(|| panic!("local 'str' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(s.type_name, "string", "str must be discriminated as string (R3.2)");
        assert_eq!(s.repr, "hello", "str value must be readable as 'hello'");

        // boolean: flag == true
        let flag = find_var(&vars, "flag")
            .unwrap_or_else(|| panic!("local 'flag' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(flag.type_name, "boolean", "flag must be discriminated as boolean (R3.2)");
        assert_eq!(flag.repr, "true", "flag value must be readable as true");

        // table: tbl is a table (repr はアドレスプレースホルダ)
        let tbl = find_var(&vars, "tbl")
            .unwrap_or_else(|| panic!("local 'tbl' must be retrieved by name. got: {:?}", *vars));
        assert_eq!(tbl.type_name, "table", "tbl must be discriminated as table (R3.2)");
        assert!(
            tbl.repr.starts_with("table:"),
            "table repr must be a readable placeholder. got: {}",
            tbl.repr
        );

        // R3.3 / R5.3: std_debug が露出していないまま inspect が成立していること。
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(
            debug_is_nil,
            "std_debug must remain unexposed during FFI inspection (sandbox preserved, R3.3/R5.3)"
        );

        println!(
            "[R3] inspect_locals via FFI obtained {} locals incl. number/string/boolean/table \
             by name; std_debug stayed unexposed (sandbox preserved).",
            vars.len()
        );
    }

    /// R3.2（upvalue 経路）:
    /// upvalue を 1 つ閉じ込めたクロージャを停止対象とし、フック内で
    /// `inspect_upvalues(lua, 0)` を呼んで upvalue が**名前付き・型判別付き**で
    /// 取得できることを assert する。
    #[test]
    fn inspect_upvalues_via_ffi() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let info = frame_info(debug);
            // クロージャ本体（`@upvalue_closure`）の任意の行で 1 度だけ inspect する。
            if info.source == "@upvalue_closure" {
                if let Ok(mut g) = captured_hook.lock() {
                    if g.is_empty() {
                        let vars = inspect_upvalues(hook_lua, 0)
                            .expect("inspect_upvalues must not error inside the hook");
                        if !vars.is_empty() {
                            *g = vars;
                        }
                    }
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        // 外側で `captured_num` を定義 → 返す内側クロージャがそれを **upvalue** として
        // 捕捉する。チャンク名 `@upvalue_closure` は内側 `function()` 本体にも継承される
        // ため、本体行でフックが撃たれたとき `frame_info().source` は `@upvalue_closure`
        // になる（そこで `inspect_upvalues` を走らせる）。
        let closure: mlua::Function = lua
            .load(
                "\
local captured_num = 7
return function()
    local doubled = captured_num * 2
    return doubled
end
",
            )
            .set_name("@upvalue_closure")
            .eval::<mlua::Function>()
            .expect("named closure factory should produce a function");

        // クロージャを呼ぶ → 本体行でフックが撃たれ、inspect_upvalues が走る。
        closure
            .call::<i64>(())
            .expect("closure call should succeed");

        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "inspect_upvalues must capture the captured upvalue (R3.2). got empty"
        );

        let up = find_var(&vars, "captured_num").unwrap_or_else(|| {
            panic!(
                "upvalue 'captured_num' must be retrieved by name. got: {:?}",
                *vars
            )
        });
        assert_eq!(
            up.type_name, "number",
            "captured upvalue must be discriminated as number (R3.2)"
        );
        assert_eq!(up.repr, "7", "captured upvalue value must be readable as 7");

        println!(
            "[R3] inspect_upvalues via FFI obtained upvalue '{}' = {} ({}) by name.",
            up.name, up.repr, up.type_name
        );
    }

    /// R3.1（任意・補強）: 安全 API のみで停止フレームの source / line / func_name を
    /// 取得できること。既知の行で停止し、`frame_info` の値を assert する。
    #[test]
    fn frame_info_via_safe_api() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let captured: Arc<Mutex<Option<FrameInfo>>> = Arc::new(Mutex::new(None));
        let captured_hook = Arc::clone(&captured);
        let target_line: u32 = 2;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |_lua, debug| {
            let info = frame_info(debug);
            if info.source == "@frame_info_chunk" && info.line == target_line {
                if let Ok(mut g) = captured_hook.lock() {
                    if g.is_none() {
                        *g = Some(info);
                    }
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        let chunk = "\
local a = 1
local b = a + 1
return b
";
        lua.load(chunk)
            .set_name("@frame_info_chunk")
            .exec()
            .expect("chunk should execute");

        lua.remove_global_hook();

        let info = captured
            .lock()
            .unwrap()
            .clone()
            .expect("frame_info must be captured at the target line (R3.1)");
        assert_eq!(
            info.source, "@frame_info_chunk",
            "frame_info must report the chunk source via safe API (R3.1)"
        );
        assert_eq!(
            info.line, target_line,
            "frame_info must report the current line via safe API (R3.1)"
        );

        println!(
            "[R3] frame_info via safe API: source={} line={} func_name={:?}",
            info.source, info.line, info.func_name
        );
    }

    /// R3.4（FFI 失敗/制限時に取得可能範囲と制約・回避策を記録）:
    /// 停止フレームのローカルに **基本型（number）と非対応種別（function / nil）** を
    /// 同居させ、`inspect_locals` が:
    /// 1. **クラッシュせず**（パニック/longjmp なし）、
    /// 2. 基本型（number）は従来どおり名前＋型＋値で取得でき、
    /// 3. 非対応種別（function/nil）は**範囲外として `type_name` に記録**
    ///    （`read_value_at_top` の other 分岐で `lua_typename` 由来の型名＋
    ///    `<unsupported ...>` repr）、
    /// 4. スタックは破壊されない（`debug_assert` のスタック均衡＋走行後も VM 健全）
    /// ことを実証する。これにより R3.4「取得可能な範囲と制約を記録」を満たす。
    ///
    /// ## 設計準拠（design.md "FrameInspector" Implementation Notes / research R3.4）
    /// `FrameInspector` は `lua_type` で安全側に分類し、未対応種別を範囲外として
    /// 記録する方針。本テストはその**グレースフルな記録**を assert する（種別で
    /// クラッシュしない・基本型は引き続き取得できる）。回避策（デバッグ時限定の
    /// `std_debug` 露出など）は `ItemOutcome` / research へ記録する制約である。
    #[test]
    fn inspect_unsupported_type_recorded() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let captured: Arc<Mutex<Vec<Variable>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_hook = Arc::clone(&captured);
        // 全ローカルが可視になった直後の marker 行で 1 度だけ inspect する。
        let target_line: u32 = 5;

        lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
            let info = frame_info(debug);
            if info.source == "@unsupported_chunk" && info.line == target_line {
                // 非対応種別が混在しても inspect_locals は Err/panic にならない（R3.4）。
                let vars = inspect_locals(hook_lua)
                    .expect("inspect_locals must NOT crash on unsupported kinds (R3.4)");
                if let Ok(mut g) = captured_hook.lock() {
                    if g.is_empty() {
                        *g = vars;
                    }
                }
            }
            Ok(VmState::Continue)
        })
        .expect("set_global_hook should succeed");

        // ローカル:
        //   basic … number（取得可能な基本型）
        //   fnval … function（非対応種別: 範囲外として記録）
        //   nilval … nil（非対応種別: 範囲外として記録）
        // marker 行（5 行目）で停止し inspect する。
        let chunk = "\
local basic = 42
local fnval = function() return 1 end
local nilval = nil
local marker = basic
return marker
";
        lua.load(chunk)
            .set_name("@unsupported_chunk")
            .exec()
            .expect("chunk should execute even though unsupported-typed locals are present (R3.4)");

        lua.remove_global_hook();

        let vars = captured.lock().unwrap();
        assert!(
            !vars.is_empty(),
            "inspect_locals must capture locals at the breakpoint (R3.4). got empty"
        );

        // (1) 基本型（number）は引き続き名前＋型＋値で取得できる。
        let basic = find_var(&vars, "basic").unwrap_or_else(|| {
            panic!(
                "basic-typed local 'basic' must still be obtained alongside unsupported kinds (R3.4). got: {:?}",
                *vars
            )
        });
        assert_eq!(
            basic.type_name, "number",
            "the basic type must still be discriminated as number (R3.4)"
        );
        assert_eq!(basic.repr, "42", "the basic value must be readable as 42 (R3.4)");

        // (2) 非対応種別（function）は範囲外として type_name に記録される。
        let fnval = find_var(&vars, "fnval").unwrap_or_else(|| {
            panic!(
                "function-typed local 'fnval' must be RECORDED (out-of-scope), not dropped (R3.4). got: {:?}",
                *vars
            )
        });
        assert_eq!(
            fnval.type_name, "function",
            "an unsupported kind must be recorded by its lua_type name 'function' (R3.4)"
        );
        assert!(
            fnval.repr.starts_with("<unsupported"),
            "an unsupported kind must carry an out-of-scope repr placeholder (R3.4): {:?}",
            fnval.repr
        );

        // (3) nil も範囲外として記録される（クラッシュさせない）。
        let nilval = find_var(&vars, "nilval").unwrap_or_else(|| {
            panic!(
                "nil-typed local 'nilval' must be RECORDED as out-of-scope (R3.4). got: {:?}",
                *vars
            )
        });
        assert_eq!(
            nilval.type_name, "nil",
            "a nil local must be recorded by its lua_type name 'nil' (R3.4)"
        );
        assert!(
            nilval.repr.starts_with("<unsupported"),
            "nil must carry an out-of-scope repr placeholder (R3.4): {:?}",
            nilval.repr
        );

        // (4) スタックが破壊されていないこと: 走行後も VM が健全に評価できる
        //     （inspect_locals の入口/出口で lua_settop により均衡が保たれている）。
        let sane: i64 = lua
            .load("return 1 + 2")
            .eval()
            .expect("VM must remain usable after inspecting unsupported kinds (stack not corrupted, R3.4)");
        assert_eq!(sane, 3, "VM stack must remain balanced after R3.4 inspection");

        // R3.4「取得可能な範囲と制約を記録」: 制約と回避策を ItemOutcome として残す。
        let outcome = ItemOutcome {
            id: "R3".to_string(),
            // 取得可能範囲（基本型）は成立しており、非対応種別はグレースフルに記録
            // できているため、R3 全体としては passed（制約付き GO 方向）。制約は notes へ。
            passed: true,
            method: "FFI inspect_locals + lua_type classification (unsupported kinds recorded out-of-scope)".to_string(),
            notes: "constraint: function/userdata/thread/nil are out-of-scope for value repr \
                    (recorded by type_name with '<unsupported ...>' placeholder, no crash, stack balanced); \
                    workaround: debug-only std_debug exposure or per-kind expansion in pasta-vscode-lua-debug"
                .to_string(),
        };
        assert!(
            outcome.notes.contains("unsupported"),
            "R3.4 outcome must document the unsupported-kind constraint and workaround"
        );

        println!(
            "[R3.4] inspect recorded unsupported kinds gracefully (basic=number kept; \
             fnval={} nilval={}); stack balanced, VM still usable. notes: {}",
            fnval.type_name, nilval.type_name, outcome.notes
        );
    }
}
