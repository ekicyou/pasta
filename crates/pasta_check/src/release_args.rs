//! release の引数の確かめ（段 1 の前の関門）。
//!
//! 読み取りだけで、何も作らず消さない。誤りの文には渡したままのパスを使い、
//! 判定のために作った正規化の形（`\\?\…`）は外に出さない。

use crate::ReleaseArgs;
use crate::path_error;
use std::fs;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};

/// a から見た b: 同じ・a が b を含む・a が b の中
#[derive(Debug, PartialEq)]
enum Relation {
    Same,
    Contains,
    Inside,
}

/// 段 1 の前に引数の関係を確かめる（読み取りだけ・何も作らない・消さない）。
/// 最初に見つかった 1 つを Err で返す。
pub(crate) fn check_release_args(args: &ReleaseArgs) -> io::Result<()> {
    let release = resolve(&args.release)?;
    let nar = resolve(&args.nar)?;
    let mut inputs = vec![("--target", &args.target, resolve(&args.target)?)];
    for copy in &args.copy_dirs {
        inputs.push(("--copy", copy, resolve(copy)?));
    }

    // --release 対 --target・各 --copy は 3 つの関係とも断る
    for (option, given, resolved) in &inputs {
        if let Some(rel) = relation(&release, resolved) {
            return Err(reject("--release", &args.release, &rel, option, given));
        }
    }
    // --nar 対 --release・--target・各 --copy は同じと中にあるを断る
    for (option, given, resolved) in iter::once(("--release", &args.release, release)).chain(inputs)
    {
        if let Some(rel @ (Relation::Same | Relation::Inside)) = relation(&nar, &resolved) {
            return Err(reject("--nar", &args.nar, &rel, option, given));
        }
    }
    Ok(())
}

/// 断る文: `--a (<A>) must not <関係> --b (<B>)`
fn reject(a_option: &str, a: &Path, rel: &Relation, b_option: &str, b: &Path) -> io::Error {
    let word = match rel {
        Relation::Same => "be the same as",
        Relation::Contains => "contain",
        Relation::Inside => "be inside",
    };
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "{a_option} ({}) must not {word} {b_option} ({})",
            a.display(),
            b.display()
        ),
    )
}

/// 比べるための形: absolute にし、在る一番近い祖先を canonicalize して、残りの部品を書かれたまま足す
fn resolve(path: &Path) -> io::Result<PathBuf> {
    let abs = std::path::absolute(path).map_err(|e| path_error(e, "read", path))?;
    for ancestor in abs.ancestors() {
        if let Ok(canonical) = fs::canonicalize(ancestor) {
            // ancestor は abs の祖先なので strip_prefix は失敗しない
            return Ok(canonical.join(abs.strip_prefix(ancestor).unwrap()));
        }
    }
    // 在らないドライブなど: 在るパスとは関係を持ちえない
    Ok(abs)
}

/// 部品ごとの比べ（名前の頭が同じ隣を取り違えない）。Windows では部品を小文字にしてから比べる
fn relation(a: &Path, b: &Path) -> Option<Relation> {
    let (a, b) = (parts(a), parts(b));
    if a == b {
        Some(Relation::Same)
    } else if b.starts_with(&a) {
        Some(Relation::Contains)
    } else if a.starts_with(&b) {
        Some(Relation::Inside)
    } else {
        None
    }
}

fn parts(path: &Path) -> Vec<String> {
    path.components()
        .map(|c| {
            let s = c.as_os_str().to_string_lossy();
            // NTFS と同じく、まだ無い部品も大文字と小文字を区別せずに比べる
            if cfg!(windows) {
                s.to_lowercase()
            } else {
                s.into_owned()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::execute_release;
    use tempfile::TempDir;

    /// 一時フォルダの中の dev に、ghost（目印と作者のファイル）・extra/overlay・
    /// 在る release_out（目印）・前回の out.nar を置く
    struct Fixture {
        _temp: TempDir,
        dev: PathBuf,
        ghost: PathBuf,
        overlay: PathBuf,
        release: PathBuf,
        nar: PathBuf,
    }

    fn fixture() -> Fixture {
        let temp = TempDir::new().unwrap();
        let dev = temp.path().join("dev");
        let ghost = dev.join("ghost");
        let overlay = dev.join("extra").join("overlay");
        let release = dev.join("release_out");
        let nar = dev.join("out.nar");
        for dir in [&ghost, &overlay, &release] {
            fs::create_dir_all(dir).unwrap();
            fs::write(dir.join("marker.txt"), "marker").unwrap();
        }
        fs::write(ghost.join("author.txt"), "author").unwrap();
        fs::write(&nar, "old nar").unwrap();
        Fixture {
            _temp: temp,
            dev,
            ghost,
            overlay,
            release,
            nar,
        }
    }

    fn args(release: &Path, nar: &Path, f: &Fixture) -> ReleaseArgs {
        ReleaseArgs {
            target: f.ghost.clone(),
            release: release.to_path_buf(),
            nar: nar.to_path_buf(),
            copy_dirs: vec![f.overlay.clone()],
        }
    }

    /// execute_release が渡したパスのままの文で断り、目印・作者のファイル・前回の nar が残る
    fn assert_rejected(args: &ReleaseArgs, f: &Fixture, expected: String) {
        let err = execute_release(args).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{err}");
        assert_eq!(err.to_string(), expected);
        for dir in [&f.ghost, &f.overlay, &f.release] {
            assert_eq!(
                fs::read_to_string(dir.join("marker.txt")).unwrap(),
                "marker"
            );
        }
        assert_eq!(
            fs::read_to_string(f.ghost.join("author.txt")).unwrap(),
            "author"
        );
        assert_eq!(fs::read_to_string(&f.nar).unwrap(), "old nar");
    }

    /// 再現 B: --release が --target と同じ（1.1）
    #[test]
    fn test_rejects_release_same_as_target() {
        let f = fixture();
        let a = args(&f.ghost, &f.nar, &f);
        let expected = format!(
            "--release ({}) must not be the same as --target ({})",
            f.ghost.display(),
            f.ghost.display()
        );
        assert_rejected(&a, &f, expected);
    }

    /// 再現 A: --release が --target の親（1.1）
    #[test]
    fn test_rejects_release_containing_target() {
        let f = fixture();
        let a = args(&f.dev, &f.nar, &f);
        let expected = format!(
            "--release ({}) must not contain --target ({})",
            f.dev.display(),
            f.ghost.display()
        );
        assert_rejected(&a, &f, expected);
    }

    /// 再現 D: --nar が --release の中（1.5）
    #[test]
    fn test_rejects_nar_inside_release() {
        let f = fixture();
        let nar = f.release.join("g.nar");
        let a = args(&f.release, &nar, &f);
        let expected = format!(
            "--nar ({}) must not be inside --release ({})",
            nar.display(),
            f.release.display()
        );
        assert_rejected(&a, &f, expected);
    }

    /// 再現 F: --nar が --target の中の作者のファイル（1.6）
    #[test]
    fn test_rejects_nar_author_file_inside_target() {
        let f = fixture();
        let author = f.ghost.join("author.txt");
        let a = args(&f.release, &author, &f);
        let expected = format!(
            "--nar ({}) must not be inside --target ({})",
            author.display(),
            f.ghost.display()
        );
        assert_rejected(&a, &f, expected);
    }

    /// --release と --copy の 3 つの関係（1.2・1.4）
    #[test]
    fn test_rejects_release_vs_copy() {
        let f = fixture();
        let extra = f.dev.join("extra");
        let inside = f.overlay.join("out");
        for (release, word) in [
            (&f.overlay, "be the same as"),
            (&extra, "contain"),
            (&inside, "be inside"),
        ] {
            let a = args(release, &f.nar, &f);
            let expected = format!(
                "--release ({}) must not {word} --copy ({})",
                release.display(),
                f.overlay.display()
            );
            assert_rejected(&a, &f, expected);
        }
    }

    /// --nar と --release・--target・--copy の同じ・中にある（1.5・1.6）
    #[test]
    fn test_rejects_nar_same_or_inside() {
        let f = fixture();
        let in_copy = f.overlay.join("x.nar");
        for (nar, expected) in [
            (
                &f.release,
                format!(
                    "--nar ({}) must not be the same as --release ({})",
                    f.release.display(),
                    f.release.display()
                ),
            ),
            (
                &f.ghost,
                format!(
                    "--nar ({}) must not be the same as --target ({})",
                    f.ghost.display(),
                    f.ghost.display()
                ),
            ),
            (
                &in_copy,
                format!(
                    "--nar ({}) must not be inside --copy ({})",
                    in_copy.display(),
                    f.overlay.display()
                ),
            ),
        ] {
            let a = args(&f.release, nar, &f);
            assert_rejected(&a, &f, expected);
        }
    }

    /// 断らない組: 名前の頭が同じ隣・--copy が --target の中・--nar が --release の隣（4.1）
    #[test]
    fn test_accepts_unrelated_pairs() {
        let f = fixture();
        let sub = f.ghost.join("sub");
        fs::create_dir_all(&sub).unwrap();
        let ok = [
            // ghost と ghost2
            ReleaseArgs {
                target: f.ghost.clone(),
                release: f.dev.join("ghost2"),
                nar: f.dev.join("ghost2.nar"),
                copy_dirs: vec![],
            },
            // --copy が --target の中
            ReleaseArgs {
                target: f.ghost.clone(),
                release: f.release.clone(),
                nar: f.nar.clone(),
                copy_dirs: vec![sub],
            },
            // release_out と release_out.nar
            ReleaseArgs {
                target: f.ghost.clone(),
                release: f.release.clone(),
                nar: f.dev.join("release_out.nar"),
                copy_dirs: vec![f.overlay.clone()],
            },
        ];
        for a in &ok {
            check_release_args(a).unwrap();
        }
    }

    /// 断ったとき、まだ無い --release・--nar の親を作らない（2.3）
    #[test]
    fn test_rejection_creates_no_parents() {
        let f = fixture();
        let release = f.dev.join("new").join("out");
        let nar = release.join("sub").join("x.nar");
        let a = args(&release, &nar, &f);
        let err = check_release_args(&a).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!(
                "--nar ({}) must not be inside --release ({})",
                nar.display(),
                release.display()
            )
        );
        assert!(!f.dev.join("new").exists());
    }
}
