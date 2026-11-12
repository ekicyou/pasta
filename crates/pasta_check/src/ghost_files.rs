//! ゴーストの `ghost/master` からファイルの一覧と、核が選んだ辞書の中身を集める（feature `fs`）。
//!
//! 読むだけで、ファイルを作らない・書き換えない・消さない。

use crate::validate::{FileText, GhostInput, TomlSource, msg, select_dictionary_files};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// ghost/master の一覧
#[derive(Debug)]
pub struct GhostListing {
    /// (相対パス '/' 区切り, 実際のパス)。walk の順
    pub files: Vec<(String, PathBuf)>,
    /// 入ったフォルダの相対パス
    pub dirs: Vec<String>,
}

/// 再帰で一覧を作る。フォルダを読めなければ Err（理由の文）
pub fn list(master: &Path) -> Result<GhostListing, String> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    walk(master, "", &mut files, &mut dirs)?;
    Ok(GhostListing { files, dirs })
}

/// ghost/master の一覧と pasta.toml を読み、核が選んだ辞書ファイルの中身を集める
pub fn collect(master: &Path) -> Result<(GhostInput, BTreeMap<String, FileText>), String> {
    let GhostListing { files: found, dirs } = list(master)?;

    let pasta_toml = match fs::read(master.join("pasta.toml")) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => TomlSource::Missing,
        Err(e) => TomlSource::Unreadable {
            reason: e.to_string(),
        },
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => TomlSource::Text { text },
            Err(_) => TomlSource::Unreadable {
                reason: msg::file_not_utf8(),
            },
        },
    };
    let input = GhostInput {
        pasta_toml,
        entries: found.iter().map(|(rel, _)| rel.clone()).collect(),
        dirs,
    };

    let paths: BTreeMap<&str, &PathBuf> = found.iter().map(|(r, p)| (r.as_str(), p)).collect();
    let files = select_dictionary_files(&input)
        .into_iter()
        .map(|rel| {
            let text = paths.get(rel.as_str()).map_or_else(
                || FileText::Unreadable {
                    reason: "一覧にありません".into(),
                },
                |p| read_file(p),
            );
            (rel, text)
        })
        .collect();
    Ok((input, files))
}

/// 再帰でファイルの (相対パス '/' 区切り, 実際のパス) と、入ったフォルダの相対パスを集める。
/// 最上位の profile に入らず、シンボリックリンクはたどらず含めない
fn walk(
    dir: &Path,
    prefix: &str,
    out: &mut Vec<(String, PathBuf)>,
    dirs: &mut Vec<String>,
) -> Result<(), String> {
    let err = |e: io::Error| format!("{} を読めません: {e}", dir.display());
    for entry in fs::read_dir(dir).map_err(err)? {
        let entry = entry.map_err(err)?;
        let file_type = entry.file_type().map_err(err)?;
        if file_type.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if prefix.is_empty() && file_type.is_dir() && name == "profile" {
            continue;
        }
        let rel = format!("{prefix}{name}");
        if file_type.is_dir() {
            walk(&entry.path(), &format!("{rel}/"), out, dirs)?;
            dirs.push(rel);
        } else {
            out.push((rel, entry.path()));
        }
    }
    Ok(())
}

/// 中身をバイトで読み、UTF-8 で読めない・読めないを区別する（Req 3.4）
pub fn read_file(path: &Path) -> FileText {
    match fs::read(path) {
        Err(e) => FileText::Unreadable {
            reason: e.to_string(),
        },
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => FileText::Text { text },
            Err(_) => FileText::NotUtf8,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// target/ghost/master を作り、(TempDir, target, master) を返す
    fn ghost() -> (TempDir, PathBuf, PathBuf) {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("ghost_dev");
        let master = target.join("ghost").join("master");
        fs::create_dir_all(master.join("dic")).unwrap();
        (temp, target, master)
    }

    const OK_SCENE: &str = "＊挨拶
  さくら：こんにちは。
";

    /// 一覧: 再帰・'/' 区切り・最上位の profile に入らない
    #[test]
    fn test_collect_entries() {
        let (_t, _target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::create_dir_all(master.join("dic/sub")).unwrap();
        fs::write(master.join("dic/sub/a.pasta"), OK_SCENE).unwrap();
        fs::create_dir_all(master.join("profile")).unwrap();
        fs::write(master.join("profile/x.pasta"), OK_SCENE).unwrap();
        fs::create_dir_all(master.join("dic/profile")).unwrap();
        fs::write(master.join("dic/profile/b.pasta"), OK_SCENE).unwrap();
        let (input, files) = collect(&master).unwrap();
        let mut entries = input.entries.clone();
        entries.sort();
        assert_eq!(
            entries,
            vec!["dic/profile/b.pasta", "dic/sub/a.pasta", "pasta.toml"]
        );
        let mut dirs = input.dirs.clone();
        dirs.sort();
        assert_eq!(dirs, vec!["dic", "dic/profile", "dic/sub"]);
        assert!(matches!(input.pasta_toml, TomlSource::Text { .. }));
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            vec!["dic/profile/b.pasta", "dic/sub/a.pasta"]
        );
    }

    /// 一覧: 中身の無いフォルダや辞書の名前のフォルダも dirs に入る（Req 2.3）
    #[test]
    fn test_collect_dirs() {
        let (_t, _target, master) = ghost();
        fs::create_dir_all(master.join("dic/x.pasta")).unwrap();
        fs::create_dir_all(master.join("profile/sub")).unwrap();
        let (input, _) = collect(&master).unwrap();
        let mut dirs = input.dirs.clone();
        dirs.sort();
        assert_eq!(dirs, vec!["dic", "dic/x.pasta"]);
        assert!(input.entries.is_empty(), "{:?}", input.entries);
    }

    /// 中身: UTF-8 で読めないのは NotUtf8、読めないのは Unreadable。pasta.toml の有無
    #[test]
    fn test_collect_contents() {
        let (_t, _target, master) = ghost();
        fs::write(master.join("dic/bad.pasta"), [0xff, 0xfe, 0x81]).unwrap();
        // ディレクトリは fs::read で失敗する
        let (input, _) = collect(&master).unwrap();
        assert!(matches!(input.pasta_toml, TomlSource::Missing));
        assert!(matches!(
            read_file(&master.join("dic")),
            FileText::Unreadable { .. }
        ));
        assert!(matches!(
            read_file(&master.join("dic/bad.pasta")),
            FileText::NotUtf8
        ));

        fs::write(master.join("pasta.toml"), [0xff]).unwrap();
        let (input, _) = collect(&master).unwrap();
        assert!(matches!(input.pasta_toml, TomlSource::Unreadable { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn test_collect_skips_symlinks() {
        use std::os::unix::fs::symlink;
        let (t, _target, master) = ghost();
        let outside = t.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("x.pasta"), OK_SCENE).unwrap();
        symlink(&outside, master.join("dic/linked")).unwrap();
        symlink(outside.join("x.pasta"), master.join("dic/l.pasta")).unwrap();
        let (input, _) = collect(&master).unwrap();
        assert!(input.entries.is_empty(), "{:?}", input.entries);
        assert_eq!(input.dirs, vec!["dic"]);
    }
}
