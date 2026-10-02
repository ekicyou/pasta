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

/// 段 4 の生成結果
pub(crate) struct UpdateSummary {
    /// ゴースト用 updates.txt の登録件数（0 なら生成していない。従来の戻り値と同じ意味）
    pub ghost_entries: usize,
    /// 生成したバルーン用 updates.txt の (フォルダ, 登録件数)。0 件で生成しなかったものは含めない
    pub balloon_entries: Vec<(String, usize)>,
}

/// 更新ファイルを生成する。
///
/// `balloon_dirs`: 配布フォルダからの相対パス（`/` 区切り・実在名・検証済み）。
/// ゴースト用の収集から外し、各フォルダ直下にバルーン用 updates.txt を書く。
pub(crate) fn generate_update_files(
    root_dir: &Path,
    balloon_dirs: &[String],
) -> io::Result<UpdateSummary> {
    let entries = collect_files(root_dir, balloon_dirs)?;
    let ghost_entries = entries.len();

    if !entries.is_empty() {
        generate_updates_txt(root_dir, &entries)?;

        // ghost/master が存在する場合、updates.txt をそこにもコピー
        let ghost_master = root_dir.join("ghost/master");
        if ghost_master.is_dir() {
            fs::copy(
                root_dir.join("updates.txt"),
                ghost_master.join("updates.txt"),
            )?;
        }
    }

    // バルーン用: フォルダ直下にだけ書く（ghost/master への複製はしない）
    let mut balloon_entries = Vec::new();
    for dir in balloon_dirs {
        let balloon_dir = root_dir.join(dir);
        let entries = collect_files(&balloon_dir, &[])?;
        if entries.is_empty() {
            continue;
        }
        generate_updates_txt(&balloon_dir, &entries)?;
        balloon_entries.push((dir.clone(), entries.len()));
    }

    Ok(UpdateSummary {
        ghost_entries,
        balloon_entries,
    })
}

/// ディレクトリ内のファイルを再帰的に収集
///
/// `excluded_dirs`: 基準フォルダからの相対パス（`/` 区切り）が完全一致したフォルダを丸ごと外す
/// （名前一致の既存除外に追加）
fn collect_files(root_dir: &Path, excluded_dirs: &[String]) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    collect_files_recursive(root_dir, root_dir, excluded_dirs, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Unicode として不正な名前は化けさせずに止める（UTF-8 限定の作成ツール）
fn non_unicode_name(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("file name is not valid Unicode: {}", path.display()),
    )
}

/// 再帰的にファイルを収集
fn collect_files_recursive(
    root_dir: &Path,
    current_dir: &Path,
    excluded_dirs: &[String],
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

        // 基準フォルダからの相対パス（スラッシュ区切り。read_dir の実在名から作る）
        let relative_path = path
            .strip_prefix(root_dir)
            .map_err(|e| io::Error::other(e.to_string()))?
            .to_str()
            .ok_or_else(|| non_unicode_name(&path))?
            .replace('\\', "/");

        if file_type.is_dir() {
            if EXCLUDED_DIRS.iter().any(|&d| d == name) || excluded_dirs.contains(&relative_path) {
                continue;
            }
            collect_files_recursive(root_dir, &path, excluded_dirs, entries)?;
        } else if file_type.is_file() {
            if EXCLUDED_FILES.iter().any(|&f| f == name) {
                continue;
            }

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

        let entries = collect_files(temp.path(), &[]).unwrap();

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

        let count = generate_update_files(temp.path(), &[])
            .unwrap()
            .ghost_entries;
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

    /// days_to_ymd のうるう年分岐: うるう日当日・400年規則・100年規則（非うるう）・年末境界
    #[test]
    fn test_format_datetime_leap_year_branches() {
        // 2024-02-29T12:34:56 (通常のうるう年のうるう日)
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_709_210_096);
        assert_eq!(format_datetime(t), "2024-02-29T12:34:56");

        // 2000-02-29T00:00:00 (400 で割り切れる年はうるう年)
        let t = UNIX_EPOCH + std::time::Duration::from_secs(951_782_400);
        assert_eq!(format_datetime(t), "2000-02-29T00:00:00");

        // 2100-03-01T00:00:00 (100 で割り切れ 400 で割り切れない年は非うるう年。
        // 2100 をうるう年と誤判定すると 2100-02-29 になる)
        let t = UNIX_EPOCH + std::time::Duration::from_secs(4_107_542_400);
        assert_eq!(format_datetime(t), "2100-03-01T00:00:00");

        // 2023-12-31T23:59:59 (年末・12月末日の繰り上がり境界)
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_199);
        assert_eq!(format_datetime(t), "2023-12-31T23:59:59");
    }

    /// 対象ファイルが 0 件の場合は早期 return し updates.txt を生成しない
    #[test]
    fn test_generate_update_files_empty_dir_creates_nothing() {
        let temp = TempDir::new().unwrap();
        let count = generate_update_files(temp.path(), &[])
            .unwrap()
            .ghost_entries;
        assert_eq!(count, 0);
        assert!(!temp.path().join("updates.txt").exists());
    }

    /// root が存在しない場合も collect_files が空集合へフォールバックし Ok(0)
    #[test]
    fn test_generate_update_files_nonexistent_root_returns_zero() {
        let temp = TempDir::new().unwrap();
        let missing = temp.path().join("no_such_dir");
        let count = generate_update_files(&missing, &[]).unwrap().ghost_entries;
        assert_eq!(count, 0);
    }

    /// EXCLUDED_DIRS の "var" ディレクトリが除外されること（profile は既存テストで担保）
    #[test]
    fn test_collect_files_excludes_var_dir() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("keep.txt"), "keep").unwrap();
        let var_dir = temp.path().join("var");
        fs::create_dir(&var_dir).unwrap();
        fs::write(var_dir.join("state.txt"), "state").unwrap();

        let entries = collect_files(temp.path(), &[]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["keep.txt"]);
    }

    /// EXCLUDED_FILES の developer_options.txt が除外されること
    #[test]
    fn test_collect_files_excludes_developer_options() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("keep.txt"), "keep").unwrap();
        fs::write(temp.path().join("developer_options.txt"), "dev only").unwrap();

        let entries = collect_files(temp.path(), &[]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["keep.txt"]);
    }

    /// updates.txt のエントリはパスの辞書順でソートされる（決定的出力）
    /// `pkg.txt` と `pkg/inner.txt` は '.' (0x2E) < '/' (0x2F) により
    /// `pkg.txt` が先となる — ディレクトリ走査順（`pkg/` ディレクトリが先）とは
    /// 一致しないため、`collect_files` の sort を除去するとこの test は fail する
    #[test]
    fn test_updates_txt_entries_sorted_by_path() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("zeta.txt"), "z").unwrap();
        fs::write(temp.path().join("alpha.txt"), "a").unwrap();
        let sub = temp.path().join("pkg");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("inner.txt"), "i").unwrap();
        fs::write(temp.path().join("pkg.txt"), "p").unwrap();

        generate_update_files(temp.path(), &[]).unwrap();
        let content = fs::read_to_string(temp.path().join("updates.txt")).unwrap();
        let file_paths: Vec<&str> = content
            .lines()
            .filter(|l| l.starts_with("file,"))
            .map(|l| {
                l.strip_prefix("file,")
                    .unwrap()
                    .split('\x01')
                    .next()
                    .unwrap()
            })
            .collect();
        assert_eq!(
            file_paths,
            vec!["alpha.txt", "pkg.txt", "pkg/inner.txt", "zeta.txt"]
        );
    }

    /// 空ファイルの MD5（既知ベクトル）
    #[test]
    fn test_calculate_md5_empty_file() {
        let temp = TempDir::new().unwrap();
        let empty = temp.path().join("empty.bin");
        fs::write(&empty, b"").unwrap();
        assert_eq!(
            calculate_md5(&empty).unwrap(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
    }

    /// 8192 バイトのバッファ境界をまたぐストリーミング読みが一括計算と一致すること
    /// （チャンク分割の継ぎ目にバグが入ると fail する）
    #[test]
    fn test_calculate_md5_multi_chunk_matches_one_shot() {
        let temp = TempDir::new().unwrap();
        let big = temp.path().join("big.bin");
        let data: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        fs::write(&big, &data).unwrap();

        let expected = format!("{:032x}", md5::compute(&data));
        assert_eq!(calculate_md5(&big).unwrap(), expected);
    }

    /// updates.txt の file 行は SOH (\x01) でフィールド区切りされること
    #[test]
    fn test_updates_txt_soh_delimiters() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("test.txt"), "hello").unwrap();

        generate_update_files(temp.path(), &[]).unwrap();
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

        generate_update_files(temp.path(), &[]).unwrap();

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

        let count = generate_update_files(temp.path(), &[])
            .unwrap()
            .ghost_entries;
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

    /// `date=` の値（次の SOH まで）を `<DATE>` に伏せる
    fn mask_dates(s: &str) -> String {
        let mut out = String::new();
        let mut rest = s;
        while let Some(i) = rest.find("date=") {
            out.push_str(&rest[..i + "date=".len()]);
            out.push_str("<DATE>");
            let after = &rest[i + "date=".len()..];
            rest = &after[after.find('\x01').expect("date= の後に SOH が必要")..];
        }
        out.push_str(rest);
        out
    }

    /// 特性化テスト（Req 8.1・8.4）: 同梱バルーンの無い固定フィクスチャについて、
    /// ルートと `ghost/master` の updates.txt の全バイトが `date=` の値を除き期待値と一致する。
    ///
    /// 除外の仕組みを変える前に置き、後方互換が崩れたら落ちるようにする。
    #[test]
    fn test_updates_txt_characterization_without_balloon() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // 対象ファイル（入れ子のフォルダを含む）
        fs::write(root.join("readme.txt"), "readme").unwrap();
        let ghost_master = root.join("ghost/master");
        fs::create_dir_all(ghost_master.join("dic/sub")).unwrap();
        fs::write(ghost_master.join("descript.txt"), "desc").unwrap();
        fs::write(ghost_master.join("dic/sub/foo.pasta"), "foo").unwrap();
        fs::create_dir_all(root.join("shell/master")).unwrap();
        fs::write(root.join("shell/master/surface0.png"), "png").unwrap();

        // 既存の除外対象（ルートと ghost/master の双方）
        for dir in [root, ghost_master.as_path()] {
            fs::create_dir_all(dir.join("profile/pasta")).unwrap();
            fs::write(dir.join("profile/pasta/save.lua"), "save").unwrap();
            fs::create_dir_all(dir.join("var")).unwrap();
            fs::write(dir.join("var/state.txt"), "state").unwrap();
            fs::write(dir.join("updates2.dau"), "stale dau").unwrap();
            fs::write(dir.join("developer_options.txt"), "dev only").unwrap();
            fs::write(dir.join("updates.txt"), "stale updates").unwrap();
        }

        let count = generate_update_files(root, &[]).unwrap().ghost_entries;
        assert_eq!(count, 4);

        let expected = concat!(
            "charset,UTF-8\r\n",
            "file,ghost/master/descript.txt\x011dee80c7d5ab2c1c90aa8d2f7dd47256\x01size=4\x01date=<DATE>\x01\r\n",
            "file,ghost/master/dic/sub/foo.pasta\x01acbd18db4cc2f85cedef654fccc4a4d8\x01size=3\x01date=<DATE>\x01\r\n",
            "file,readme.txt\x013905d7917f2b3429490b01cfb60d8f5b\x01size=6\x01date=<DATE>\x01\r\n",
            "file,shell/master/surface0.png\x01bff139fa05ac583f685a523ab3d110a0\x01size=3\x01date=<DATE>\x01\r\n",
        );
        for path in [root.join("updates.txt"), ghost_master.join("updates.txt")] {
            let content = String::from_utf8(fs::read(&path).unwrap()).unwrap();
            assert_eq!(mask_dates(&content), expected, "{}", path.display());
        }
    }

    /// 相対パス完全一致の除外（Req 3.1・3.3・3.4）: `bal` を外すとルート直下の `bal/` だけが
    /// 一覧から消え、`ghost/master/bal/` の同名フォルダとそれ以外のファイルは従来どおり載る。
    /// 階層付きの指定（`skin/bal2`）も同じく相対パスで一致したフォルダだけを外す。
    #[test]
    fn test_collect_files_excludes_relative_dirs() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::write(root.join("install.txt"), "inst").unwrap();
        fs::create_dir_all(root.join("bal/sub")).unwrap();
        fs::write(root.join("bal/descript.txt"), "bal").unwrap();
        fs::write(root.join("bal/sub/arrow0.png"), "png").unwrap();
        fs::create_dir_all(root.join("ghost/master/bal")).unwrap();
        fs::write(root.join("ghost/master/bal/keep.txt"), "keep").unwrap();
        fs::create_dir_all(root.join("skin/bal2")).unwrap();
        fs::write(root.join("skin/bal2/descript.txt"), "bal2").unwrap();
        fs::write(root.join("skin/other.txt"), "other").unwrap();
        fs::create_dir_all(root.join("bal2")).unwrap();
        fs::write(root.join("bal2/keep.txt"), "keep").unwrap();

        let excluded = vec!["bal".to_string(), "skin/bal2".to_string()];
        let entries = collect_files(root, &excluded).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "bal2/keep.txt",
                "ghost/master/bal/keep.txt",
                "install.txt",
                "skin/other.txt",
            ]
        );
    }

    /// `updates.txt` の file 行を (パス, md5, size) に分解する
    fn parse_file_lines(content: &str) -> Vec<(String, String, u64)> {
        content
            .lines()
            .filter_map(|l| l.strip_prefix("file,"))
            .map(|l| {
                let f: Vec<&str> = l.split('\x01').collect();
                let size = f[2].strip_prefix("size=").unwrap().parse().unwrap();
                (f[0].to_string(), f[1].to_string(), size)
            })
            .collect()
    }

    /// バルーン用 updates.txt（Req 3.2・4.1〜4.8・5.1・5.2）:
    /// - `bal/updates.txt` の各行はバルーン基準の相対パスで、自分自身・`profile/`・`var/`・
    ///   `updates2.dau`・`developer_options.txt` を含まず、md5・size が実ファイルと一致する
    /// - 既存の `bal/updates.txt` は置き換わり、`bal/ghost/master/` があっても複製しない
    /// - 0 件のバルーン（`empty`）には書かず、要約にも含めない
    /// - ルートと `ghost/master` の updates.txt は同内容で、同梱バルーン配下の行を含まない
    #[test]
    fn test_generate_update_files_writes_balloon_updates() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let ghost_master = root.join("ghost/master");
        fs::create_dir_all(&ghost_master).unwrap();
        fs::write(ghost_master.join("descript.txt"), "desc").unwrap();
        fs::write(root.join("install.txt"), "inst").unwrap();

        let bal = root.join("bal");
        fs::create_dir_all(bal.join("sub")).unwrap();
        fs::write(bal.join("descript.txt"), "bal desc\r\n").unwrap();
        fs::write(bal.join("sub/arrow0.png"), "png bytes").unwrap();
        fs::write(bal.join("updates.txt"), "stale from target").unwrap();
        fs::write(bal.join("updates2.dau"), "dau").unwrap();
        fs::write(bal.join("developer_options.txt"), "dev").unwrap();
        fs::create_dir_all(bal.join("profile")).unwrap();
        fs::write(bal.join("profile/p.txt"), "p").unwrap();
        fs::create_dir_all(bal.join("var")).unwrap();
        fs::write(bal.join("var/v.txt"), "v").unwrap();
        fs::create_dir_all(bal.join("ghost/master")).unwrap();
        fs::write(bal.join("ghost/master/x.txt"), "x").unwrap();

        // 除外規則を適用すると 0 件になるバルーン
        let empty = root.join("empty");
        fs::create_dir_all(empty.join("profile")).unwrap();
        fs::write(empty.join("profile/p.txt"), "p").unwrap();
        fs::write(empty.join("updates2.dau"), "dau").unwrap();

        let dirs = vec!["bal".to_string(), "empty".to_string()];
        let summary = generate_update_files(root, &dirs).unwrap();

        assert_eq!(summary.ghost_entries, 2);
        assert_eq!(summary.balloon_entries, vec![("bal".to_string(), 3)]);

        // バルーン用
        let bal_content = fs::read_to_string(bal.join("updates.txt")).unwrap();
        assert!(bal_content.starts_with("charset,UTF-8\r\n"));
        let lines = parse_file_lines(&bal_content);
        let paths: Vec<&str> = lines.iter().map(|(p, _, _)| p.as_str()).collect();
        assert_eq!(
            paths,
            vec!["descript.txt", "ghost/master/x.txt", "sub/arrow0.png"]
        );
        for (p, md5, size) in &lines {
            let bytes = fs::read(bal.join(p)).unwrap();
            assert_eq!(md5, &format!("{:032x}", md5::compute(&bytes)), "{p}");
            assert_eq!(*size, bytes.len() as u64, "{p}");
        }
        assert!(!bal.join("ghost/master/updates.txt").exists());
        assert!(!empty.join("updates.txt").exists());

        // ゴースト用（ルートと ghost/master が同内容・バルーン配下を含まない）
        let root_content = fs::read_to_string(root.join("updates.txt")).unwrap();
        let copy_content = fs::read_to_string(ghost_master.join("updates.txt")).unwrap();
        assert_eq!(root_content, copy_content);
        let ghost_paths: Vec<String> = parse_file_lines(&root_content)
            .into_iter()
            .map(|(p, _, _)| p)
            .collect();
        assert_eq!(
            ghost_paths,
            vec!["ghost/master/descript.txt", "install.txt"]
        );
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

        let entries = collect_files(temp.path(), &[]).unwrap();

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

    /// Unicode として不正な名前のファイル・フォルダは名前を化けさせず InvalidData で止める
    #[cfg(windows)]
    #[test]
    fn test_generate_update_files_rejects_non_unicode_names() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let bad = OsString::from_wide(&[0xD800]);

        // ファイル名
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(&bad), "a").unwrap();
        let err = generate_update_files(temp.path(), &[]).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string()
                .starts_with("file name is not valid Unicode: "),
            "{err}"
        );

        // フォルダ名（中のファイルが正常な名前でも止める）
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(&bad)).unwrap();
        fs::write(temp.path().join(&bad).join("ok.txt"), "a").unwrap();
        let err = generate_update_files(temp.path(), &[]).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(!temp.path().join("updates.txt").exists());
    }
}
