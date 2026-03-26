use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

/// NAR ファイルを作成。戻り値は NAR ファイルのサイズ（バイト）。
pub fn create_nar(release_dir: &Path, nar_path: &Path) -> io::Result<u64> {
    // 親ディレクトリを必要に応じて作成
    if let Some(parent) = nar_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(nar_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut buffer = Vec::new();
    add_dir_to_zip(&mut zip, release_dir, release_dir, &options, &mut buffer)?;

    zip.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let metadata = fs::metadata(nar_path)?;
    Ok(metadata.len())
}

fn add_dir_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    root: &Path,
    current: &Path,
    options: &SimpleFileOptions,
    buffer: &mut Vec<u8>,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // profile/ を除外
            if file_name == "profile" {
                continue;
            }
            add_dir_to_zip(zip, root, &path, options, buffer)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");

            zip.start_file(&relative, *options)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            buffer.clear();
            File::open(&path)?.read_to_end(buffer)?;
            zip.write_all(buffer)?;
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
            .map(|i| archive.by_index(i).unwrap().name().to_string())
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
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(!names.iter().any(|n| n.contains("profile")));
        assert!(names.contains(&"install.txt".to_string()));
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
}
