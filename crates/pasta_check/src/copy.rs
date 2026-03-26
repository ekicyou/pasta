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
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
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
}
