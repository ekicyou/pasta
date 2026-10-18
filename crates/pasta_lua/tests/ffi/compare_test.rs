//! 元の `ffi.load` と並べた位置・文言・型の比べ（要件 6.x・9.6）。
//!
//! FFI だけを開いた素の VM（元の `ffi.load`）と、`ffi = true` のゴーストの VM に、同じチャンク名の
//! 同じ Lua を流し、`pcall` の結果の型と文言をバイトで比べる。
//! - `cannot load module '<名前>': <理由>` の文言は、`': ` までの前の部分（位置・名前）をそのまま比べ、
//!   理由は元の側を ANSI から UTF-8 にしてから比べる（6.2・6.4）。
//! - 数の引数は LuaJIT と同じく文字列にして読むので、理由は同じく UTF-8 にしてから比べる（6.2）。
//! - 名前に ANSI に無い文字を含むときは、理由は pasta が自分で見つけた原因でよい（6.3）。前の部分は
//!   同じで、名前が欠けないこと（6.5）を確かめる。

use mlua::{Lua, Value};
use pasta_lua::{Encoder, Encoding, PastaLoader};

use crate::common::NON_ANSI_DIR_NAME;
use crate::dll_fixture::{FFI_TOML, bare_ffi_lua, lock, make_ghost};

/// 比べるときのチャンク名（両方の VM で同じ）。
const CHUNK: &str = "=compare";

/// 理由の部分の比べ方。
#[derive(Clone, Copy, Debug)]
enum Reason {
    /// 元の側を ANSI から UTF-8 にしてから同じ（6.2）。
    Utf8,
    /// pasta が自分で見つけた原因でよい（6.3）。
    Own,
}

/// チャンクを流し、`pcall` の (成否, 型の名前, 文言のバイト) を返す。
fn run(lua: &Lua, code: &str) -> (bool, &'static str, Vec<u8>) {
    let (ok, e): (bool, Value) = lua
        .load(code)
        .set_name(CHUNK)
        .call(())
        .unwrap_or_else(|err| panic!("チャンクが pcall の外で失敗: {err}\n{code}"));
    let bytes = match &e {
        Value::String(s) => s.as_bytes().to_vec(),
        other => panic!("文言が文字列でない: {other:?}\n{code}"),
    };
    (ok, e.type_name(), bytes)
}

/// `cannot load module '<名前>': ` までを前の部分、残りを理由に分ける。その形でなければ None。
fn split_reason(msg: &[u8]) -> Option<(&[u8], &[u8])> {
    const MARK: &[u8] = b"cannot load module '";
    let start = msg.windows(MARK.len()).position(|w| w == MARK)? + MARK.len();
    let sep = msg[start..].windows(3).position(|w| w == b"': ")? + start + 3;
    Some(msg.split_at(sep))
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// 1 つの場合を比べる。pasta の側の文言（UTF-8）を返す。
fn compare(orig: &Lua, pasta: &Lua, code: &str, reason: Reason) -> String {
    let (o_ok, o_ty, o_msg) = run(orig, code);
    let (p_ok, p_ty, p_msg) = run(pasta, code);
    assert!(!o_ok && !p_ok, "失敗しなかった:\n{code}");
    assert_eq!(p_ty, o_ty, "型が違う:\n{code}");

    match (split_reason(&o_msg), split_reason(&p_msg)) {
        (Some((o_head, o_reason)), Some((p_head, p_reason))) => {
            assert_eq!(
                show(p_head),
                show(o_head),
                "位置・名前の部分が違う:\n{code}"
            );
            assert_eq!(p_head, o_head, "位置・名前の部分のバイトが違う:\n{code}");
            match reason {
                Reason::Utf8 => {
                    let o_reason = Encoding::ANSI.to_string(o_reason).unwrap();
                    assert_eq!(show(p_reason), o_reason, "理由が違う:\n{code}");
                    assert_eq!(p_reason, o_reason.as_bytes());
                }
                Reason::Own => assert!(!p_reason.is_empty(), "理由が空:\n{code}"),
            }
        }
        (None, None) => assert_eq!(show(&p_msg), show(&o_msg), "文言が違う:\n{code}"),
        _ => panic!(
            "文言の形が違う:\n元: {}\npasta: {}\n{code}",
            show(&o_msg),
            show(&p_msg)
        ),
    }
    String::from_utf8(p_msg).expect("pasta の文言が UTF-8 でない")
}

/// 関数の中から呼ぶ（位置が付く）チャンク。
fn in_function(args: &str) -> String {
    format!("local function f()\n  local m = ffi.load({args})\n  return m\nend\nreturn pcall(f)\n")
}

/// `pcall(ffi.load, …)` で直接呼ぶ（位置が付かない）チャンク。
fn direct(args: &str) -> String {
    if args.is_empty() {
        "return pcall(ffi.load)\n".to_string()
    } else {
        format!("return pcall(ffi.load, {args})\n")
    }
}

#[test]
fn failures_match_original_ffi_load() {
    let _guard = lock();
    let temp = tempfile::TempDir::new().unwrap();
    let ghost = make_ghost(&temp.path().join("g"), FFI_TOML);
    let runtime = PastaLoader::load(&ghost).expect("ffi = true のゴーストを読めない");
    let pasta = runtime.lua();
    let orig = bare_ffi_lua();

    let cases: &[(&str, Reason)] = &[
        // 名前だけで無いもの（OS の既定の探索へ渡る・3.4）。
        (r#""pasta_cmp_missing""#, Reason::Utf8),
        (r#""pasta_cmp_missing", true"#, Reason::Utf8),
        // 相対パスで無いもの（pasta が自分で失敗させる・3.7）。
        (r#""sub/pasta_cmp_missing.dll""#, Reason::Utf8),
        (r#""sub\\pasta_cmp_missing""#, Reason::Utf8),
        // 文字列でない引数。`nil` は元の ffi.load へそのまま渡る。数は LuaJIT と同じく文字列にして読む。
        ("nil", Reason::Utf8),
        ("12345", Reason::Utf8),
    ];
    for &(args, reason) in cases {
        for code in [in_function(args), direct(args)] {
            compare(&orig, pasta, &code, reason);
        }
    }

    // 位置の付き方: 関数の中からは呼び出し元の行、直接の pcall では位置なし（6.1・6.4）。
    let inner = compare(
        &orig,
        pasta,
        &in_function(r#""pasta_cmp_missing""#),
        Reason::Utf8,
    );
    assert!(
        inner.starts_with("compare:2: cannot load module 'pasta_cmp_missing': "),
        "{inner}"
    );
    let outer = compare(
        &orig,
        pasta,
        &direct(r#""pasta_cmp_missing""#),
        Reason::Utf8,
    );
    assert!(
        outer.starts_with("cannot load module 'pasta_cmp_missing': "),
        "{outer}"
    );

    // 引数なしは `got no value` のまま（4.1 の Implementation Notes）。
    let no_arg = compare(&orig, pasta, &direct(""), Reason::Utf8);
    assert!(no_arg.ends_with("got no value)"), "{no_arg}");
    let nil_arg = compare(&orig, pasta, &direct("nil"), Reason::Utf8);
    assert!(nil_arg.ends_with("got nil)"), "{nil_arg}");

    // ANSI に無い文字を含む名前: 前の部分（名前）は元の側と同じバイトで、pasta の文言では欠けない（6.5）。
    for name in [
        format!("pasta_cmp_{NON_ANSI_DIR_NAME}"),
        format!("{NON_ANSI_DIR_NAME}/pasta_cmp_missing.dll"),
    ] {
        for code in [
            in_function(&format!("{name:?}")),
            direct(&format!("{name:?}")),
        ] {
            let msg = compare(&orig, pasta, &code, Reason::Own);
            assert!(
                msg.contains(&format!("cannot load module '{name}': ")),
                "{msg}"
            );
        }
    }
}
