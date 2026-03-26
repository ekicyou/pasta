//! SSP ネットワーク更新ファイル生成モジュール
//!
//! `updates.txt` を SSP 仕様に準拠して生成します（`updates2.dau` は将来用に保持）。

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 除外パターン（ディレクトリ）
const EXCLUDED_DIRS: &[&str] = &["profile", "var"];

/// 除外パターン（ファイル）
const EXCLUDED_FILES: &[&str] = &["updates2.dau", "updates.txt", "developer_options.txt"];

/// ファイル情報
#[derive(Debug, Clone)]
struct FileEntry {
    /// 相対パス（スラッシュ区切り）
    path: String,
    /// MD5 ハッシュ（32文字小文字16進数）
    md5: String,
    /// ファイルサイズ（バイト）
    size: u64,
    /// ファイル更新日時
    modified: SystemTime,
}

/// SystemTime を ISO 8601 形式 (UTC) に変換: `YYYY-MM-DDTHH:MM:SS`
fn format_datetime(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;
    let days = secs / 86400;

    // 1970-01-01 からの日数をグレゴリオ暦に変換
    let (year, month, day) = days_to_ymd(days as u32);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}

fn is_leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_to_ymd(mut days: u32) -> (u32, u32, u32) {
    let mut year = 1970u32;
    loop {
        let y_days = if is_leap(year) { 366 } else { 365 };
        if days < y_days {
            break;
        }
        days -= y_days;
        year += 1;
    }
    let month_days = [
        31u32,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

/// 更新ファイルを生成。戻り値は登録したファイルエントリ数。
pub fn generate_update_files(root_dir: &Path) -> io::Result<usize> {
    let entries = collect_files(root_dir)?;
    let count = entries.len();

    if entries.is_empty() {
        return Ok(0);
    }

    generate_updates_txt(root_dir, &entries)?;

    Ok(count)
}

/// ディレクトリ内のファイルを再帰的に収集
fn collect_files(root_dir: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    collect_files_recursive(root_dir, root_dir, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// 再帰的にファイルを収集
fn collect_files_recursive(
    root_dir: &Path,
    current_dir: &Path,
    entries: &mut Vec<FileEntry>,
) -> io::Result<()> {
    let read_dir = match fs::read_dir(current_dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if EXCLUDED_DIRS.contains(&file_name.as_str()) {
                continue;
            }
            collect_files_recursive(root_dir, &path, entries)?;
        } else if path.is_file() {
            if EXCLUDED_FILES.contains(&file_name.as_str()) {
                continue;
            }

            let relative_path = path
                .strip_prefix(root_dir)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");

            let md5 = calculate_md5(&path)?;
            let metadata = fs::metadata(&path)?;
            let size = metadata.len();
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            entries.push(FileEntry {
                path: relative_path,
                md5,
                size,
                modified,
            });
        }
    }

    Ok(())
}

/// ファイルの MD5 ハッシュを計算
fn calculate_md5(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        context.consume(&buffer[..bytes_read]);
    }

    let digest = context.finalize();
    Ok(format!("{:032x}", digest))
}

/// updates2.dau を生成
/// フォーマット: `<filepath><SOH><md5><SOH>size=<bytes><SOH><CRLF>`
#[allow(dead_code)]
fn generate_updates2_dau(root_dir: &Path, entries: &[FileEntry]) -> io::Result<()> {
    let output_path = root_dir.join("updates2.dau");
    let mut file = File::create(&output_path)?;

    for entry in entries {
        let record = format!(
            "{}\x01{}\x01size={}\x01\r\n",
            entry.path, entry.md5, entry.size
        );
        file.write_all(record.as_bytes())?;
    }

    Ok(())
}

/// updates.txt を生成
/// フォーマット:
///   1行目: `charset,UTF-8`
///   以降: `file,<filepath><md5>size=<bytes>date=<YYYY-MM-DDTHH:MM:SS><CRLF>`
fn generate_updates_txt(root_dir: &Path, entries: &[FileEntry]) -> io::Result<()> {
    let output_path = root_dir.join("updates.txt");
    let mut file = File::create(&output_path)?;

    // charset ヘッダー（UTF-8 ゴースト向け）
    file.write_all(b"charset,UTF-8\r\n")?;

    for entry in entries {
        let date = format_datetime(entry.modified);
        let record = format!(
            "file,{}{}size={}date={}\r\n",
            entry.path, entry.md5, entry.size, date
        );
        file.write_all(record.as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_md5() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let md5 = calculate_md5(&test_file).unwrap();
        assert_eq!(md5, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_collect_files_excludes_update_files() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("test.txt"), "content").unwrap();
        fs::write(temp.path().join("updates2.dau"), "should be excluded").unwrap();
        fs::write(temp.path().join("updates.txt"), "should be excluded").unwrap();

        let profile_dir = temp.path().join("profile");
        fs::create_dir(&profile_dir).unwrap();
        fs::write(profile_dir.join("user.txt"), "user data").unwrap();

        let entries = collect_files(temp.path()).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "test.txt");
    }

    #[test]
    fn test_generate_update_files() {
        let temp = TempDir::new().unwrap();

        let ghost_dir = temp.path().join("ghost/master");
        fs::create_dir_all(&ghost_dir).unwrap();
        fs::write(ghost_dir.join("descript.txt"), "test content").unwrap();

        let shell_dir = temp.path().join("shell/master");
        fs::create_dir_all(&shell_dir).unwrap();
        fs::write(shell_dir.join("surface0.png"), "fake png").unwrap();

        let count = generate_update_files(temp.path()).unwrap();
        assert_eq!(count, 2);

        assert!(!temp.path().join("updates2.dau").exists());
        assert!(temp.path().join("updates.txt").exists());

        let content = fs::read_to_string(temp.path().join("updates.txt")).unwrap();
        assert!(content.starts_with("charset,UTF-8\r\n"));
        assert!(content.contains("file,ghost/master/descript.txt"));
        assert!(content.contains("file,shell/master/surface0.png"));
        assert!(content.contains("size="));
        assert!(content.contains("date="));
    }

    #[test]
    fn test_format_datetime() {
        // 2026-03-15T16:12:47 UTC = 1773734000 + offset ... let's use a known epoch
        // 1970-01-01T00:00:00 = 0
        assert_eq!(format_datetime(UNIX_EPOCH), "1970-01-01T00:00:00");
        // 2024-01-01T00:00:00 UTC = 1704067200
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1704067200);
        assert_eq!(format_datetime(t), "2024-01-01T00:00:00");
    }
}
