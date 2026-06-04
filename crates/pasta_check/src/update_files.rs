//! SSP ネットワーク更新ファイル生成モジュール
//!
//! `updates.txt` を SSP 仕様に準拠して生成します。

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
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
pub(crate) fn generate_update_files(root_dir: &Path) -> io::Result<usize> {
    let entries = collect_files(root_dir)?;
    let count = entries.len();

    if entries.is_empty() {
        return Ok(0);
    }

    generate_updates_txt(root_dir, &entries)?;

    // ghost/master が存在する場合、updates.txt をそこにもコピー
    let ghost_master = root_dir.join("ghost/master");
    if ghost_master.is_dir() {
        fs::copy(
            root_dir.join("updates.txt"),
            ghost_master.join("updates.txt"),
        )?;
    }

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
        let file_type = entry.file_type()?;

        // シンボリックリンクはスキップ（リンク先を追跡しない）
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name();

        if file_type.is_dir() {
            if EXCLUDED_DIRS.iter().any(|&d| d == name) {
                continue;
            }
            collect_files_recursive(root_dir, &path, entries)?;
        } else if file_type.is_file() {
            if EXCLUDED_FILES.iter().any(|&f| f == name) {
                continue;
            }

            let relative_path = path
                .strip_prefix(root_dir)
                .map_err(|e| io::Error::other(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");

            let metadata = fs::metadata(&path)?;

            entries.push(FileEntry {
                path: relative_path,
                md5: calculate_md5(&path)?,
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
    }

    Ok(())
}

/// ファイルの MD5 ハッシュを計算
///
/// SSP 仕様準拠の非暗号学的ファイル変更検出用途。認証・署名には使用しない。
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

/// updates.txt を生成
/// フォーマット (Version 3):
///   1行目: `charset,UTF-8`
///   以降: `file,<filepath>\x01<md5>\x01size=<bytes>\x01date=<YYYY-MM-DDTHH:MM:SS>\x01<CRLF>`
fn generate_updates_txt(root_dir: &Path, entries: &[FileEntry]) -> io::Result<()> {
    let output_path = root_dir.join("updates.txt");
    let mut file = File::create(&output_path)?;

    // charset ヘッダー（UTF-8 ゴースト向け）
    file.write_all(b"charset,UTF-8\r\n")?;

    for entry in entries {
        let date = format_datetime(entry.modified);
        let record = format!(
            "file,{}\x01{}\x01size={}\x01date={}\x01\r\n",
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

    /// updates.txt の file 行は SOH (\x01) でフィールド区切りされること
    #[test]
    fn test_updates_txt_soh_delimiters() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("test.txt"), "hello").unwrap();

        generate_update_files(temp.path()).unwrap();
        let bytes = fs::read(temp.path().join("updates.txt")).unwrap();
        let content = String::from_utf8(bytes).unwrap();

        // charset 行は SOH なし
        assert!(content.starts_with("charset,UTF-8\r\n"));

        // file 行は SOH 区切り: file,<path>\x01<md5>\x01size=...\x01date=...\x01\r\n
        let file_line = content.lines().find(|l| l.starts_with("file,")).unwrap();
        let soh_count = file_line.bytes().filter(|&b| b == 0x01).count();
        assert!(
            soh_count >= 3,
            "file line should have at least 3 SOH delimiters, got {soh_count}: {file_line:?}"
        );
    }

    /// ghost/master が存在する場合、updates.txt がそこにもコピーされること
    #[test]
    fn test_updates_txt_copied_to_ghost_master() {
        let temp = TempDir::new().unwrap();

        let ghost_dir = temp.path().join("ghost/master");
        fs::create_dir_all(&ghost_dir).unwrap();
        fs::write(ghost_dir.join("descript.txt"), "desc").unwrap();

        generate_update_files(temp.path()).unwrap();

        // ルートに生成
        assert!(temp.path().join("updates.txt").exists());
        // ghost/master にもコピー
        assert!(
            ghost_dir.join("updates.txt").exists(),
            "updates.txt should be copied to ghost/master/"
        );

        // 内容が同一であること
        let root_content = fs::read_to_string(temp.path().join("updates.txt")).unwrap();
        let copy_content = fs::read_to_string(ghost_dir.join("updates.txt")).unwrap();
        assert_eq!(root_content, copy_content);
    }

    /// 回帰テスト（Req 5.2）: 自己展開先 `ghost/master/profile/pasta/pasta_scripts/`
    /// 配下のフレームワークスクリプト・`.md5` マーカーが `updates.txt` の対象外であること。
    ///
    /// `EXCLUDED_DIRS` から `"profile"` が外れると、この test は
    /// `main.lua` / `.md5` を含むエントリが生成され FAIL する。
    #[test]
    fn test_updates_txt_excludes_self_deploy_dir() {
        let temp = TempDir::new().unwrap();

        // 通常ファイル（同梱対象 = 含まれるべき）
        let ghost_master = temp.path().join("ghost/master");
        fs::create_dir_all(&ghost_master).unwrap();
        fs::write(ghost_master.join("descript.txt"), "desc").unwrap();
        fs::create_dir_all(ghost_master.join("dic")).unwrap();
        fs::write(ghost_master.join("dic/foo.pasta"), "foo").unwrap();

        // 自己展開先（profile/ 配下 = 除外領域）
        let self_deploy = ghost_master.join("profile/pasta/pasta_scripts");
        fs::create_dir_all(&self_deploy).unwrap();
        fs::write(self_deploy.join("main.lua"), "-- framework script").unwrap();
        fs::write(self_deploy.join(".md5"), "deadbeef").unwrap();

        let count = generate_update_files(temp.path()).unwrap();
        let content = fs::read_to_string(temp.path().join("updates.txt")).unwrap();

        // 通常ファイルは含まれる
        assert!(
            content.contains("file,ghost/master/descript.txt"),
            "descript.txt should be listed: {content:?}"
        );
        assert!(
            content.contains("file,ghost/master/dic/foo.pasta"),
            "dic/foo.pasta should be listed: {content:?}"
        );

        // 自己展開先（profile/ 配下）は一切含まれない
        assert!(
            !content.contains("profile"),
            "no profile/ path must appear in updates.txt: {content:?}"
        );
        assert!(
            !content.contains("main.lua"),
            "self-deploy script must not be listed: {content:?}"
        );
        assert!(
            !content.contains(".md5"),
            ".md5 marker must not be listed: {content:?}"
        );

        // エントリ数は通常ファイル 2 件のみ（自己展開先 2 件は除外）
        assert_eq!(count, 2, "only the 2 normal files should be counted");
    }

    /// サブディレクトリの updates.txt / updates2.dau もファイル一覧から除外されること
    #[test]
    fn test_collect_files_excludes_updates_in_subdirs() {
        let temp = TempDir::new().unwrap();

        let sub = temp.path().join("ghost/master");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("descript.txt"), "desc").unwrap();
        fs::write(sub.join("updates.txt"), "should be excluded").unwrap();
        fs::write(sub.join("updates2.dau"), "should be excluded").unwrap();

        let entries = collect_files(temp.path()).unwrap();

        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(
            paths.contains(&"ghost/master/descript.txt"),
            "descript.txt should be included"
        );
        assert!(
            !paths.iter().any(|p| p.contains("updates.txt")),
            "updates.txt should be excluded from listing"
        );
        assert!(
            !paths.iter().any(|p| p.contains("updates2.dau")),
            "updates2.dau should be excluded from listing"
        );
    }
}
