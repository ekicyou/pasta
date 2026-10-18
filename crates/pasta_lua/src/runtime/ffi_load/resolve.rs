//! `ffi.load` に渡された名前を読むパスへ解く純粋な規則（`unsafe` を持たない）。
//!
//! ファイルシステムには std（`is_file`・`std::path::absolute`）だけで触る。
//! 8.3 名とシンボリックリンクは解かない（`fs::canonicalize` は使わない）。

use crate::encoding::{Encoder, Encoding};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// `MAX_PATH`。UTF-16 でこの長さ以上のパスにだけ長いパスの接頭辞を付ける。
const MAX_PATH: usize = 260;

/// 名前を解いた結果。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Resolved {
    /// ゴーストのフォルダ（探索のフォルダ・基点・絶対パス）にあるファイル。
    Ghost { path: PathBuf, file_name: String },
    /// 名前だけで、探索のフォルダのどこにも無い。OS の既定の探索に任せる（3.4）。
    System { file_name: String },
    /// 区切り（またはドライブの `:`）のある名前で、そのファイルが無い（3.7）。
    Missing,
}

/// パスとして扱う名前か。区切りの無い `C:foo` もドライブを指すのでパスの側に入れ、
/// 基点の下に留める（名前だけの扱いにすると OS がドライブのカレントで探してしまう）。
fn is_path(name: &str) -> bool {
    name.contains(['/', '\\', ':'])
}

/// 名前を解く。`search_dirs` は `absolute_search_paths()` の順。
pub(super) fn resolve(name: &str, base_dir: &Path, search_dirs: &[PathBuf]) -> Resolved {
    if !is_path(name) {
        let file_name = with_extension_rule(name);
        return search_dirs
            .iter()
            .filter_map(|dir| std::path::absolute(dir.join(&file_name)).ok())
            .find(|path| path.is_file())
            .map(|path| Resolved::Ghost {
                path,
                file_name: file_name.clone(),
            })
            .unwrap_or(Resolved::System { file_name });
    }

    // 末尾が区切りの名前はファイルを指さない（`lib/` を `lib.dll` と読まない）。
    if name.ends_with(['/', '\\']) {
        return Resolved::Missing;
    }
    let Ok(mut path) = std::path::absolute(join_to_base(name, base_dir)) else {
        return Resolved::Missing;
    };
    let Some(file_name) = path
        .file_name()
        .and_then(|f| f.to_str())
        .map(with_extension_rule)
    else {
        return Resolved::Missing;
    };
    path.set_file_name(&file_name);
    if path.is_file() {
        Resolved::Ghost { path, file_name }
    } else {
        Resolved::Missing
    }
}

/// 絶対パスはそのまま、それ以外は基点の後ろに `\` と名前を文字列としてつなぐ（3.5・3.6）。
///
/// `Path::join` は `\foo`・`C:foo` で基点を捨てるので使わない。
fn join_to_base(name: &str, base_dir: &Path) -> PathBuf {
    let raw = Path::new(name);
    if raw.is_absolute() {
        return raw.to_path_buf();
    }
    let mut s = base_dir.as_os_str().to_os_string();
    s.push("\\");
    s.push(name);
    PathBuf::from(s)
}

/// 最後の部分に `.` が無ければ `.dll` を足す（3.1）。
pub(super) fn with_extension_rule(name: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    if last.contains('.') {
        name.to_string()
    } else {
        format!("{name}.dll")
    }
}

/// ファイル名を ANSI のバイト列にする。往復で一致しなければ None（2.8）。
///
/// 往復で比べるので、似た文字への黙った置き換え（best-fit）も表せない扱いになる。
pub(super) fn ansi_file_name(file_name: &str) -> Option<Vec<u8>> {
    let bytes = Encoding::ANSI.to_bytes(file_name).ok()?;
    let back = Encoding::ANSI.to_string(&bytes).ok()?;
    (back == file_name).then_some(bytes)
}

/// LoadLibraryExW に渡す NUL 終端の UTF-16。260 以上なら `\\?\` を付ける。
///
/// UNC（`\\server\share`）は `\\?\UNC\server\share` の形にする。既に `\\?\`・`\\.\` で
/// 始まるものはそのまま。この形は読むときにだけ使い、文言・ログには出さない（2.7）。
pub(super) fn load_path_form(path: &Path) -> Vec<u16> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    let w = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
    let mut out = if wide.len() < MAX_PATH
        || wide.starts_with(&w(r"\\?\"))
        || wide.starts_with(&w(r"\\.\"))
    {
        wide
    } else if wide.starts_with(&w(r"\\")) {
        [w(r"\\?\UNC\"), wide[2..].to_vec()].concat()
    } else {
        [w(r"\\?\"), wide].concat()
    };
    out.push(0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide_str(v: &[u16]) -> String {
        assert_eq!(v.last(), Some(&0), "NUL 終端");
        String::from_utf16(&v[..v.len() - 1]).unwrap()
    }

    #[test]
    fn extension_rule() {
        assert_eq!(with_extension_rule("foo"), "foo.dll");
        assert_eq!(with_extension_rule("foo.dll"), "foo.dll");
        assert_eq!(with_extension_rule("foo.bar"), "foo.bar");
        assert_eq!(with_extension_rule("lib/foo"), "lib/foo.dll");
        assert_eq!(with_extension_rule(r"lib\foo"), r"lib\foo.dll");
        // 区切りより前の `.` は数えない。
        assert_eq!(with_extension_rule("a.b/foo"), "a.b/foo.dll");
        assert_eq!(with_extension_rule("a.b/foo.x"), "a.b/foo.x");
    }

    #[test]
    fn name_only_searches_dirs_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let base = std::path::absolute(tmp.path()).unwrap();
        let (a, b, c) = (base.join("a"), base.join("b"), base.join("c"));
        for d in [&a, &b, &c] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(b.join("foo.dll"), b"").unwrap();
        std::fs::write(c.join("foo.dll"), b"").unwrap();
        // フォルダは見つからない扱い。
        std::fs::create_dir_all(a.join("foo.dll")).unwrap();

        let dirs = vec![a.clone(), b.clone(), c.clone()];
        assert_eq!(
            resolve("foo", &base, &dirs),
            Resolved::Ghost {
                path: b.join("foo.dll"),
                file_name: "foo.dll".into()
            }
        );
        let dirs = vec![c.clone(), b.clone()];
        assert_eq!(
            resolve("foo.dll", &base, &dirs),
            Resolved::Ghost {
                path: c.join("foo.dll"),
                file_name: "foo.dll".into()
            }
        );
        // `/` を含む探索のフォルダ（`profile/pasta/...` の形）も `\` 区切りに直る。
        let dirs = vec![base.join("x/../b")];
        assert_eq!(
            resolve("foo", &base, &dirs),
            Resolved::Ghost {
                path: b.join("foo.dll"),
                file_name: "foo.dll".into()
            }
        );
        // 無ければ OS に探させる名前（基点は探さない）。
        std::fs::write(base.join("user32.dll"), b"").unwrap();
        assert_eq!(
            resolve("user32", &base, std::slice::from_ref(&a)),
            Resolved::System {
                file_name: "user32.dll".into()
            }
        );
    }

    #[test]
    fn relative_and_absolute_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let base = std::path::absolute(tmp.path()).unwrap().join("master");
        std::fs::create_dir_all(base.join("lib")).unwrap();
        let foo = base.join("lib").join("foo.dll");
        std::fs::write(&foo, b"").unwrap();
        std::fs::write(base.join("foo.dll"), b"").unwrap();
        let ghost = |path: PathBuf| Resolved::Ghost {
            path,
            file_name: "foo.dll".into(),
        };

        // `/` と `\` のどちらも区切り。結果は `\` 区切りの絶対パス。
        assert_eq!(resolve("lib/foo", &base, &[]), ghost(foo.clone()));
        assert_eq!(resolve(r"lib\foo.dll", &base, &[]), ghost(foo.clone()));
        assert_eq!(resolve("./x/../lib/foo", &base, &[]), ghost(foo.clone()));
        // `\foo`・`C:foo` は基点の外（ドライブの根・ドライブのカレント）へ出ない。
        assert_eq!(resolve(r"\foo", &base, &[]), ghost(base.join("foo.dll")));
        assert_eq!(resolve("/lib/foo", &base, &[]), ghost(foo.clone()));
        assert_eq!(resolve("c:lib/foo", &base, &[]), Resolved::Missing);
        // 区切りの無い `C:foo` もパスの扱い。基点の下に無ければ Missing で、OS へは回さない。
        // 探索のフォルダを渡しても名前だけの扱いにならない。
        assert_eq!(
            resolve("C:foo", &base, std::slice::from_ref(&base)),
            Resolved::Missing
        );
        assert_eq!(
            resolve("C:foo.dll", &base, std::slice::from_ref(&base)),
            Resolved::Missing
        );
        // 絶対パスはそのまま。
        let abs = foo.to_str().unwrap().replace('\\', "/");
        assert_eq!(
            resolve(&abs, Path::new(r"C:\elsewhere"), &[]),
            ghost(foo.clone())
        );
        let abs_noext = abs.trim_end_matches(".dll");
        assert_eq!(
            resolve(abs_noext, Path::new(r"C:\elsewhere"), &[]),
            ghost(foo)
        );
        // 無いファイル・フォルダは Missing（OS の探索へ回さない）。
        assert_eq!(resolve("lib/bar", &base, &[]), Resolved::Missing);
        assert_eq!(resolve("lib/", &base, &[]), Resolved::Missing);
        std::fs::write(base.join("lib.dll"), b"").unwrap();
        assert_eq!(resolve("lib/", &base, &[]), Resolved::Missing);
    }

    #[test]
    fn relative_never_escapes_base() {
        // `resolve` の結果で確かめる。基点の外（探索のフォルダ・ドライブの根）に
        // 同じ名前のファイルがあっても拾わず、OS の探索へも回さない。
        let tmp = tempfile::tempdir().unwrap();
        let root = std::path::absolute(tmp.path()).unwrap();
        let base = root.join("master");
        std::fs::create_dir_all(base.join("lib")).unwrap();
        std::fs::write(root.join("foo.dll"), b"").unwrap();
        std::fs::write(base.join("lib").join("foo.dll"), b"").unwrap();
        for name in [
            r"\foo",
            "/foo",
            "C:foo",
            "c:foo.dll",
            "c:lib/foo",
            r"lib\foo",
        ] {
            match resolve(name, &base, std::slice::from_ref(&root)) {
                Resolved::Ghost { path, .. } => {
                    assert!(path.starts_with(&base), "{name} -> {}", path.display())
                }
                Resolved::Missing => {}
                r @ Resolved::System { .. } => panic!("{name} -> {r:?}"),
            }
        }

        let base = Path::new(r"C:\ghost\master");
        for name in [r"\foo", "/foo", "C:foo", "c:lib/foo", r"lib\foo"] {
            let p = std::path::absolute(join_to_base(name, base)).unwrap();
            assert!(p.starts_with(base), "{name} -> {}", p.display());
        }
        assert_eq!(
            std::path::absolute(join_to_base("C:foo", base)).unwrap(),
            Path::new(r"C:\ghost\master\C:foo")
        );
        // 絶対パス（ドライブ文字＋区切り・`\\`）は基点を使わない。
        assert_eq!(join_to_base(r"D:\x\foo", base), Path::new(r"D:\x\foo"));
        assert_eq!(
            join_to_base(r"\\srv\share\foo", base),
            Path::new(r"\\srv\share\foo")
        );
    }

    #[test]
    fn ansi_round_trip() {
        assert_eq!(ansi_file_name("foo.dll"), Some(b"foo.dll".to_vec()));
        // CI と開発機の ANSI は CP932（または UTF-8）。日本語はその範囲に入る。
        let jp = ansi_file_name("日本語.dll").expect("CP の範囲の日本語は表せる");
        assert_eq!(Encoding::ANSI.to_string(&jp).unwrap(), "日本語.dll");

        let utf8_acp = Encoding::ANSI.to_bytes("😀").ok().as_deref() == Some("😀".as_bytes());
        if utf8_acp {
            assert!(ansi_file_name("😀.dll").is_some());
        } else {
            assert_eq!(ansi_file_name("😀.dll"), None);
            // best-fit（似た文字への黙った置き換え）も表せない扱い。
            let cp932 = Encoding::ANSI.to_bytes("あ").ok() == Some(vec![0x82, 0xA0]);
            if cp932 {
                assert_eq!(ansi_file_name("é.dll"), None);
            }
        }
    }

    #[test]
    fn load_path_form_prefix_boundary() {
        // `C:\` + 名前 で総長を合わせる。
        let make = |len: usize| PathBuf::from(format!(r"C:\{}", "a".repeat(len - 3)));
        let p259 = make(259);
        assert_eq!(wide_str(&load_path_form(&p259)), p259.to_str().unwrap());
        let p260 = make(260);
        assert_eq!(
            wide_str(&load_path_form(&p260)),
            format!(r"\\?\{}", p260.to_str().unwrap())
        );
        // UNC は UNC の形。
        let unc = PathBuf::from(format!(r"\\server\share\{}", "b".repeat(300)));
        assert_eq!(
            wide_str(&load_path_form(&unc)),
            format!(r"\\?\UNC\server\share\{}", "b".repeat(300))
        );
        let short_unc = PathBuf::from(r"\\server\share\foo.dll");
        assert_eq!(
            wide_str(&load_path_form(&short_unc)),
            r"\\server\share\foo.dll"
        );
        // 既に接頭辞があれば重ねない。長さで弾かない。
        let pre = PathBuf::from(format!(r"\\?\C:\{}", "c".repeat(40000)));
        assert_eq!(wide_str(&load_path_form(&pre)), pre.to_str().unwrap());
        // ANSI に無い文字も UTF-16 のまま欠けない。
        let jp = PathBuf::from(format!(r"C:\😀日本\{}", "d".repeat(300)));
        assert_eq!(
            wide_str(&load_path_form(&jp)),
            format!(r"\\?\{}", jp.to_str().unwrap())
        );
    }
}
