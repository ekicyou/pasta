use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use crate::{non_unicode_name, path_error};
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// NAR ファイルを作成。戻り値は NAR ファイルのサイズ（バイト）。
pub(crate) fn create_nar(release_dir: &Path, nar_path: &Path) -> io::Result<u64> {
    // 親ディレクトリを必要に応じて作成
    if let Some(parent) = nar_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|e| path_error(e, "create", parent))?;
    }

    let write_err = |e| path_error(e, "write", nar_path);
    let file = File::create(nar_path).map_err(write_err)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut buffer = Vec::new();
    add_dir_to_zip(&mut zip, release_dir, release_dir, &options, &mut buffer)?;

    zip.finish().map_err(|e| write_err(io::Error::other(e)))?;

    let metadata = fs::metadata(nar_path).map_err(write_err)?;
    Ok(metadata.len())
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    root: &Path,
    current: &Path,
    options: &SimpleFileOptions,
    buffer: &mut Vec<u8>,
) -> io::Result<()> {
    let read_err = |e| path_error(e, "read", current);
    for entry in fs::read_dir(current).map_err(read_err)? {
        let entry = entry.map_err(read_err)?;
        let file_type = entry.file_type().map_err(read_err)?;

        // シンボリックリンクはスキップ（Req 2.1, 2.2）
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();

        if file_type.is_dir() {
            // profile/ を除外
            if entry.file_name() == "profile" {
                continue;
            }
            add_dir_to_zip(zip, root, &path, options, buffer)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| io::Error::other(e.to_string()))?
                .to_str()
                .ok_or_else(|| non_unicode_name(&path))?
                .replace('\\', "/");

            let pack_err = |e| path_error(e, "pack", &path);
            zip.start_file(&relative, *options)
                .map_err(|e| pack_err(io::Error::other(e)))?;

            buffer.clear();
            File::open(&path)
                .and_then(|mut f| f.read_to_end(buffer))
                .map_err(|e| path_error(e, "read", &path))?;
            zip.write_all(buffer).map_err(pack_err)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_nar_basic() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(release.join("ghost/master")).unwrap();
        fs::write(release.join("ghost/master/descript.txt"), "test").unwrap();
        fs::write(release.join("install.txt"), "install").unwrap();

        let nar_path = temp.path().join("out.nar");
        let size = create_nar(&release, &nar_path).unwrap();
        assert!(size > 0);
        assert!(nar_path.exists());

        // ZIP として読み取れることを確認
        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
            .collect();
        assert!(names.contains(&"ghost/master/descript.txt".to_string()));
        assert!(names.contains(&"install.txt".to_string()));
    }

    #[test]
    fn test_create_nar_excludes_profile() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(release.join("profile")).unwrap();
        fs::write(release.join("profile/user.txt"), "data").unwrap();
        fs::write(release.join("install.txt"), "install").unwrap();

        let nar_path = temp.path().join("out.nar");
        create_nar(&release, &nar_path).unwrap();

        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
            .collect();
        assert!(!names.iter().any(|n| n.contains("profile")));
        assert!(names.contains(&"install.txt".to_string()));
    }

    /// 回帰テスト（Req 5.3）: 自己展開先 `ghost/master/profile/pasta/pasta_scripts/`
    /// 配下のフレームワークスクリプト・`.md5` マーカーが `.nar`（ZIP）封入対象外であること。
    ///
    /// `profile` の除外条件が外れると、この test は ZIP に
    /// `main.lua` / `.md5` エントリが現れ FAIL する。
    #[test]
    fn test_create_nar_excludes_self_deploy_dir() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");

        // 通常ファイル（封入対象 = 含まれるべき）
        let ghost_master = release.join("ghost/master");
        fs::create_dir_all(&ghost_master).unwrap();
        fs::write(ghost_master.join("descript.txt"), "desc").unwrap();
        fs::create_dir_all(ghost_master.join("dic")).unwrap();
        fs::write(ghost_master.join("dic/foo.pasta"), "foo").unwrap();

        // 自己展開先（profile/ 配下 = 除外領域）
        let self_deploy = ghost_master.join("profile/pasta/pasta_scripts");
        fs::create_dir_all(&self_deploy).unwrap();
        fs::write(self_deploy.join("main.lua"), "-- framework script").unwrap();
        fs::write(self_deploy.join(".md5"), "deadbeef").unwrap();

        let nar_path = temp.path().join("out.nar");
        create_nar(&release, &nar_path).unwrap();

        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
            .collect();

        // 通常ファイルは封入される
        assert!(
            names.contains(&"ghost/master/descript.txt".to_string()),
            "descript.txt must be archived: {names:?}"
        );
        assert!(
            names.contains(&"ghost/master/dic/foo.pasta".to_string()),
            "dic/foo.pasta must be archived: {names:?}"
        );

        // 自己展開先（profile/ 配下）は一切封入されない
        assert!(
            !names.iter().any(|n| n.contains("profile")),
            "no profile/ entry must be archived: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.contains("main.lua")),
            "self-deploy script must not be archived: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.ends_with(".md5")),
            ".md5 marker must not be archived: {names:?}"
        );
    }

    #[test]
    fn test_create_nar_creates_parent_dir() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(&release).unwrap();
        fs::write(release.join("a.txt"), "a").unwrap();

        let nar_path = temp.path().join("nested/dir/out.nar");
        let size = create_nar(&release, &nar_path).unwrap();
        assert!(size > 0);
        assert!(nar_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_create_nar_excludes_symlinks() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(release.join("ghost")).unwrap();
        fs::write(release.join("ghost/real.txt"), "real").unwrap();

        // ファイルへのシンボリックリンク
        unix_fs::symlink(
            release.join("ghost/real.txt"),
            release.join("ghost/link.txt"),
        )
        .unwrap();
        // ディレクトリへのシンボリックリンク
        unix_fs::symlink(release.join("ghost"), release.join("linked_dir")).unwrap();

        let nar_path = temp.path().join("out.nar");
        create_nar(&release, &nar_path).unwrap();

        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
            .collect();
        // 実ファイルは含まれる
        assert!(names.contains(&"ghost/real.txt".to_string()));
        // シンボリックリンクは除外される
        assert!(!names.iter().any(|n| n.contains("link.txt")));
        assert!(!names.iter().any(|n| n.contains("linked_dir")));
    }

    /// 空の release ディレクトリでもエントリ 0 件の正当な ZIP が生成される
    #[test]
    fn test_create_nar_empty_dir() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(&release).unwrap();

        let nar_path = temp.path().join("out.nar");
        let size = create_nar(&release, &nar_path).unwrap();
        assert!(size > 0); // ZIP の end-of-central-directory 分

        let file = File::open(&nar_path).unwrap();
        let archive = zip::ZipArchive::new(file).unwrap();
        assert_eq!(archive.len(), 0);
    }

    /// release ディレクトリが存在しない場合はエラーが伝播する
    #[test]
    fn test_create_nar_missing_release_dir_errors() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("no_such_dir");
        let nar_path = temp.path().join("out.nar");
        assert!(create_nar(&release, &nar_path).is_err());
    }

    /// ZIP エントリの内容がソースファイルのバイト列と一致する（名前だけでなく中身の忠実性）
    #[test]
    fn test_create_nar_entry_content_roundtrip() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(release.join("ghost/master")).unwrap();
        let content: Vec<u8> = (0u16..2000).flat_map(|i| i.to_le_bytes()).collect();
        fs::write(release.join("ghost/master/data.bin"), &content).unwrap();

        let nar_path = temp.path().join("out.nar");
        create_nar(&release, &nar_path).unwrap();

        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut entry = archive.by_name("ghost/master/data.bin").unwrap();
        let mut extracted = Vec::new();
        entry.read_to_end(&mut extracted).unwrap();
        assert_eq!(extracted, content);
    }

    /// 「profile」という名前の通常ファイルは除外されない
    /// （profile 除外はディレクトリのみに適用される境界の文書化）
    #[test]
    fn test_create_nar_profile_named_file_is_included() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(&release).unwrap();
        fs::write(release.join("profile"), "not a directory").unwrap();

        let nar_path = temp.path().join("out.nar");
        create_nar(&release, &nar_path).unwrap();

        let file = File::open(&nar_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("profile").is_ok());
    }

    #[test]
    fn test_create_nar_overwrites_existing() {
        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(&release).unwrap();
        fs::write(release.join("a.txt"), "a").unwrap();

        let nar_path = temp.path().join("out.nar");
        fs::write(&nar_path, "old content").unwrap();

        let size = create_nar(&release, &nar_path).unwrap();
        assert!(size > 0);
        // 上書きされたファイルは ZIP として読めること
        let file = File::open(&nar_path).unwrap();
        let archive = zip::ZipArchive::new(file).unwrap();
        assert_eq!(archive.len(), 1);
    }

    /// Unicode として不正な名前のファイルは名前を化けさせず InvalidData で止める
    #[cfg(windows)]
    #[test]
    fn test_create_nar_rejects_non_unicode_name() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let temp = TempDir::new().unwrap();
        let release = temp.path().join("release");
        fs::create_dir_all(release.join("ghost")).unwrap();
        fs::write(
            release.join("ghost").join(OsString::from_wide(&[0xD800])),
            "a",
        )
        .unwrap();

        let err = create_nar(&release, &temp.path().join("out.nar")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string()
                .starts_with("file name is not valid Unicode: "),
            "{err}"
        );
    }
}
