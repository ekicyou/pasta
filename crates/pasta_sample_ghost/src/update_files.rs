//! SSP ネットワーク更新ファイル生成モジュール
//!
//! `updates2.dau` および `updates.txt` を SSP 仕様に準拠して生成します。
//!
//! # 仕様
//!
//! ## updates2.dau
//! - エンコーディング: Shift_JIS (CP932)
//! - 改行: CRLF (0x0D 0x0A)
//! - フォーマット: `<filepath><SOH><md5><SOH>size=<bytes><SOH><CRLF>`
//! - 区切り文字: SOH (0x01)
//!
//! ## updates.txt
//! - エンコーディング: Shift_JIS または UTF-8
//! - 改行: CRLF
//! - フォーマット: `file,<filepath><md5>size=<bytes><CRLF>`
//!
//! # 除外対象
//! - `profile/` ディレクトリ
//! - `var/` ディレクトリ
//! - `updates2.dau` 自身
//! - `updates.txt` 自身
//! - `developer_options.txt`

use encoding_rs::SHIFT_JIS;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// 除外パターン（ディレクトリ）
const EXCLUDED_DIRS: &[&str] = &["profile", "var"];

/// 除外パターン（ファイル）
const EXCLUDED_FILES: &[&str] = &["updates2.dau", "updates.txt", "developer_options.txt"];

/// ファイル情報
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// 相対パス（スラッシュ区切り）
    pub path: String,
    /// MD5 ハッシュ（32文字小文字16進数）
    pub md5: String,
    /// ファイルサイズ（バイト）
    pub size: u64,
}

/// 更新ファイルを生成
///
/// # Arguments
/// * `root_dir` - ゴーストルートディレクトリ
///
/// # Returns
/// 生成したファイル数
pub fn generate_update_files(root_dir: &Path) -> std::io::Result<usize> {
    // ファイル一覧を収集
    let entries = collect_files(root_dir)?;
    let count = entries.len();

    if entries.is_empty() {
        return Ok(0);
    }

    // updates2.dau を生成
    generate_updates2_dau(root_dir, &entries)?;

    // updates.txt を生成
    generate_updates_txt(root_dir, &entries)?;

    Ok(count)
}

/// ディレクトリ内のファイルを再帰的に収集
fn collect_files(root_dir: &Path) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    collect_files_recursive(root_dir, root_dir, &mut entries)?;

    // パスでソート（一貫した順序のため）
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(entries)
}

/// 再帰的にファイルを収集
fn collect_files_recursive(
    root_dir: &Path,
    current_dir: &Path,
    entries: &mut Vec<FileEntry>,
) -> std::io::Result<()> {
    let read_dir = match fs::read_dir(current_dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()), // ディレクトリが読めない場合はスキップ
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // 除外ディレクトリをスキップ
            if EXCLUDED_DIRS.contains(&file_name.as_str()) {
                continue;
            }
            collect_files_recursive(root_dir, &path, entries)?;
        } else if path.is_file() {
            // 除外ファイルをスキップ
            if EXCLUDED_FILES.contains(&file_name.as_str()) {
                continue;
            }

            // 相対パスを計算（スラッシュ区切り）
            let relative_path = path
                .strip_prefix(root_dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");

            // MD5 ハッシュを計算
            let md5 = calculate_md5(&path)?;

            // ファイルサイズを取得
            let metadata = fs::metadata(&path)?;
            let size = metadata.len();

            entries.push(FileEntry {
                path: relative_path,
                md5,
                size,
            });
        }
    }

    Ok(())
}

/// ファイルの MD5 ハッシュを計算
fn calculate_md5(path: &Path) -> std::io::Result<String> {
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

    let digest = context.compute();
    Ok(format!("{:032x}", digest))
}

/// updates2.dau を生成
///
/// フォーマット: `<filepath><SOH><md5><SOH>size=<bytes><SOH><CRLF>`
fn generate_updates2_dau(root_dir: &Path, entries: &[FileEntry]) -> std::io::Result<()> {
    let output_path = root_dir.join("updates2.dau");
    let mut file = File::create(&output_path)?;

    for entry in entries {
        // レコードを構築（UTF-8）
        let record = format!(
            "{}\x01{}\x01size={}\x01\r\n",
            entry.path, entry.md5, entry.size
        );

        // Shift_JIS にエンコード
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&record);

        if had_errors {
            // エンコードできない文字がある場合は UTF-8 のままで書き込み
            // （SSP は UTF-8 もサポートしているため）
            file.write_all(record.as_bytes())?;
        } else {
            file.write_all(&encoded)?;
        }
    }

    Ok(())
}

/// updates.txt を生成
///
/// フォーマット: `file,<filepath><md5>size=<bytes><CRLF>`
fn generate_updates_txt(root_dir: &Path, entries: &[FileEntry]) -> std::io::Result<()> {
    let output_path = root_dir.join("updates.txt");
    let mut file = File::create(&output_path)?;

    for entry in entries {
        // レコードを構築（UTF-8）
        // 注意: filepath と md5 の間に区切り文字なし（仕様通り）
        let record = format!("file,{}{}size={}\r\n", entry.path, entry.md5, entry.size);

        // Shift_JIS にエンコード
        let (encoded, _, had_errors) = SHIFT_JIS.encode(&record);

        if had_errors {
            // エンコードできない文字がある場合は UTF-8 のまま
            file.write_all(record.as_bytes())?;
        } else {
            file.write_all(&encoded)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_md5() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let md5 = calculate_md5(&test_file).unwrap();
        // "Hello, World!" の MD5 ハッシュ
        assert_eq!(md5, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_collect_files_excludes_update_files() {
        let temp = TempDir::new().unwrap();

        // テストファイルを作成
        fs::write(temp.path().join("test.txt"), "content").unwrap();
        fs::write(temp.path().join("updates2.dau"), "should be excluded").unwrap();
        fs::write(temp.path().join("updates.txt"), "should be excluded").unwrap();

        // profile/ ディレクトリを作成
        let profile_dir = temp.path().join("profile");
        fs::create_dir(&profile_dir).unwrap();
        fs::write(profile_dir.join("user.txt"), "user data").unwrap();

        let entries = collect_files(temp.path()).unwrap();

        // test.txt のみ含まれるはず
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "test.txt");
    }

    #[test]
    fn test_generate_update_files() {
        let temp = TempDir::new().unwrap();

        // テストファイルを作成
        let ghost_dir = temp.path().join("ghost/master");
        fs::create_dir_all(&ghost_dir).unwrap();
        fs::write(ghost_dir.join("descript.txt"), "test content").unwrap();

        let shell_dir = temp.path().join("shell/master");
        fs::create_dir_all(&shell_dir).unwrap();
        fs::write(shell_dir.join("surface0.png"), "fake png").unwrap();

        // 更新ファイルを生成
        let count = generate_update_files(temp.path()).unwrap();
        assert_eq!(count, 2);

        // ファイルが作成されたか確認
        assert!(temp.path().join("updates2.dau").exists());
        assert!(temp.path().join("updates.txt").exists());

        // updates.txt の内容を確認
        let content = fs::read_to_string(temp.path().join("updates.txt")).unwrap();
        assert!(content.contains("file,ghost/master/descript.txt"));
        assert!(content.contains("file,shell/master/surface0.png"));
        assert!(content.contains("size="));
    }
}
