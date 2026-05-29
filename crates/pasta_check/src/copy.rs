use std::fs;
use std::io;
use std::path::Path;

/// release フォルダーを削除して空ディレクトリとして再作成
pub fn prepare_release_dir(release_dir: &Path) -> io::Result<()> {
    if release_dir.exists() {
        fs::remove_dir_all(release_dir)?;
    }
    fs::create_dir_all(release_dir)
}

/// `child` が `parent` ディレクトリの配下であることを検証する。
/// パストラバーサル防御のための防御的チェック。
fn ensure_within(child: &Path, parent: &Path) -> io::Result<()> {
    if !child.starts_with(parent) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "path traversal detected: {} is outside {}",
                child.display(),
                parent.display()
            ),
        ));
    }
    Ok(())
}

/// src の内容を dst に再帰コピー。戻り値はコピーしたファイル数。
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut count = 0u64;
    copy_dir_inner(src, dst, &mut count)?;
    Ok(count)
}

fn copy_dir_inner(src: &Path, dst: &Path, count: &mut u64) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // シンボリックリンクはスキップ（Req 2.1）
        if file_type.is_symlink() {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // パストラバーサル防御: dst_pathがdstの配下であることを検証（Req 1.4）
        ensure_within(&dst_path, dst)?;

        if file_type.is_dir() {
            copy_dir_inner(&src_path, &dst_path, count)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            *count += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_prepare_release_dir_creates_new() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        prepare_release_dir(&release).unwrap();
        assert!(release.is_dir());
    }

    #[test]
    fn test_prepare_release_dir_removes_existing() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(&release).unwrap();
        fs::write(release.join("old.txt"), "old").unwrap();

        prepare_release_dir(&release).unwrap();
        assert!(release.is_dir());
        assert!(!release.join("old.txt").exists());
    }

    #[test]
    fn test_copy_dir_recursive_basic() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        // ソースにファイルを配置
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "aaa").unwrap();
        fs::write(src.join("sub/b.txt"), "bbb").unwrap();

        let count = copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(count, 2);
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "aaa");
        assert_eq!(fs::read_to_string(dst.join("sub/b.txt")).unwrap(), "bbb");
    }

    #[test]
    fn test_copy_dir_recursive_overwrites() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("a.txt"), "new").unwrap();
        fs::write(dst.join("a.txt"), "old").unwrap();

        let count = copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(count, 1);
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "new");
    }

    #[test]
    fn test_copy_does_not_modify_source() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "content").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(fs::read_to_string(src.join("a.txt")).unwrap(), "content");
    }

    #[test]
    fn test_ensure_within_accepts_child_path() {
        let temp = TempDir::new().unwrap();
        let parent = temp.path().join("dst");
        let child = parent.join("sub").join("file.txt");
        ensure_within(&child, &parent).unwrap();
    }

    #[test]
    fn test_ensure_within_rejects_outside_path() {
        let temp = TempDir::new().unwrap();
        let parent = temp.path().join("dst");
        let outside = temp.path().join("other").join("file.txt");
        let err = ensure_within(&outside, &parent).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("path traversal detected"));
    }

    #[cfg(unix)]
    #[test]
    fn test_copy_skips_symlinks() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("real.txt"), "real").unwrap();

        // src/link.txt -> real.txt (シンボリックリンク)
        unix_fs::symlink(src.join("real.txt"), src.join("link.txt")).unwrap();

        let count = copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(count, 1); // real.txt のみコピー
        assert!(dst.join("real.txt").exists());
        assert!(!dst.join("link.txt").exists()); // シンボリックリンクはスキップ
    }

    #[cfg(unix)]
    #[test]
    fn test_copy_skips_symlink_dirs() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        let external = temp.path().join("external");

        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&external).unwrap();
        fs::write(src.join("real.txt"), "real").unwrap();
        fs::write(external.join("secret.txt"), "secret").unwrap();

        // src/linked_dir -> external (ディレクトリへのシンボリックリンク)
        unix_fs::symlink(&external, src.join("linked_dir")).unwrap();

        let count = copy_dir_recursive(&src, &dst).unwrap();
        assert_eq!(count, 1); // real.txt のみ
        assert!(dst.join("real.txt").exists());
        assert!(!dst.join("linked_dir").exists()); // シンボリックリンクディレクトリもスキップ
    }
}
