//! ツリーの一覧・テキストの読み書き（BOM）・作成・名前の変更。
//! 削除の処理は持たない（4.17）。読み書きの範囲は paths の門を必ず通す。

use crate::paths::{check_name, resolve_rel};
use pasta_check::{ghost_files, validate::msg};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const BOM: &[u8] = b"\xEF\xBB\xBF";

/// master からの相対・'/' 区切り
#[derive(Debug, Serialize)]
pub struct TreeListing {
    pub files: Vec<String>,
    pub dirs: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct TextFile {
    pub text: String,
    pub bom: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Dir,
}

/// ghost_files::list の相対パスから、最初の要素が profile（大文字・小文字を区別しない）のものを除く
pub fn list_tree(master: &Path) -> Result<TreeListing, String> {
    let listing = ghost_files::list(master)?;
    // ghost_files は綴りが profile のものだけ飛ばすので、Profile なども範囲の門と同じく除く（4.3・11.4）
    let keep = |rel: &String| {
        !rel.split('/')
            .next()
            .unwrap_or("")
            .eq_ignore_ascii_case("profile")
    };
    Ok(TreeListing {
        files: listing
            .files
            .into_iter()
            .map(|(rel, _)| rel)
            .filter(keep)
            .collect(),
        dirs: listing.dirs.into_iter().filter(keep).collect(),
    })
}

/// BOM を外して覚える・UTF-8 でなければ Err
pub fn read_text(master: &Path, rel: &str) -> Result<TextFile, String> {
    let path = resolve_rel(master, rel)?;
    let unreadable = |why: String| format!("{} を読めません: {why}", path.display());
    let mut bytes = fs::read(&path).map_err(|e| unreadable(e.to_string()))?;
    let bom = bytes.starts_with(BOM);
    if bom {
        bytes.drain(..BOM.len());
    }
    let text = String::from_utf8(bytes).map_err(|_| unreadable(msg::file_not_utf8()))?;
    Ok(TextFile { text, bom })
}

/// bom なら BOM を前に付けて書く。改行は受け取ったまま。作業用のファイルを作らない（11.3）
pub fn write_text(master: &Path, rel: &str, text: &str, bom: bool) -> Result<(), String> {
    let path = resolve_rel(master, rel)?;
    let head: &[u8] = if bom { BOM } else { b"" };
    fs::write(&path, [head, text.as_bytes()].concat())
        .map_err(|e| format!("{} に書けません: {e}", path.display()))
}

/// 空のファイルかフォルダを作る。既にあれば失敗。新しい相対パスを返す
pub fn create_entry(
    master: &Path,
    parent_rel: &str,
    name: &str,
    kind: EntryKind,
) -> Result<String, String> {
    check_name(&resolve_rel(master, parent_rel)?, name, None)?;
    let new_rel = join(parent_rel, name);
    // 最上位の profile などは門が拒む
    let path = resolve_rel(master, &new_rel)?;
    match kind {
        EntryKind::File => fs::File::create_new(&path).map(drop),
        EntryKind::Dir => fs::create_dir(&path),
    }
    .map_err(|e| format!("{} を作れません: {e}", path.display()))?;
    Ok(new_rel)
}

/// 名前を変えて新しい相対パスを返す。既にある別のものは上書きしない
pub fn rename_entry(master: &Path, rel: &str, new_name: &str) -> Result<String, String> {
    if rel.is_empty() {
        return Err("ゴーストのフォルダ（ghost/master）そのものの名前は変えません".into());
    }
    let path = resolve_rel(master, rel)?;
    let (parent_rel, old) = rel.rsplit_once('/').unwrap_or(("", rel));
    // Windows の fs::rename は既にあるファイルを黙って置き換えるので、先に重なりを必ず拒む
    check_name(&resolve_rel(master, parent_rel)?, new_name, Some(old))?;
    let new_rel = join(parent_rel, new_name);
    let new_path = resolve_rel(master, &new_rel)?;
    fs::rename(&path, &new_path)
        .map_err(|e| format!("{} の名前を変えられません: {e}", path.display()))?;
    Ok(new_rel)
}

fn join(parent_rel: &str, name: &str) -> String {
    if parent_rel.is_empty() {
        name.to_string()
    } else {
        format!("{parent_rel}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// ghost/master に dic/a.pasta・Profile/p.txt を持つゴーストを作る
    fn ghost() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let master = tmp.path().join("ghost").join("master");
        fs::create_dir_all(master.join("dic")).unwrap();
        fs::create_dir_all(master.join("Profile")).unwrap();
        fs::write(master.join("dic").join("a.pasta"), "").unwrap();
        fs::write(master.join("Profile").join("p.txt"), "").unwrap();
        fs::write(master.join("pasta.toml"), "").unwrap();
        (tmp, master)
    }

    #[test]
    fn list_tree_excludes_top_level_profile_ignoring_case() {
        let (_tmp, master) = ghost();
        fs::create_dir_all(master.join("dic").join("profile")).unwrap();
        fs::write(master.join("dic").join("profile").join("q.pasta"), "").unwrap();
        let mut t = list_tree(&master).unwrap();
        t.files.sort();
        t.dirs.sort();
        assert_eq!(
            t.files,
            ["dic/a.pasta", "dic/profile/q.pasta", "pasta.toml"]
        );
        // 最上位でない profile は除かない
        assert_eq!(t.dirs, ["dic", "dic/profile"]);
    }

    #[test]
    fn read_write_round_trip_keeps_bytes() {
        let (_tmp, master) = ghost();
        let cases: [&[u8]; 4] = [
            b"\xEF\xBB\xBF*a\r\n  b\r\n",
            b"\xEF\xBB\xBF*a\n  b\n",
            "＊挨拶\r\n  こんにちは\r\n".as_bytes(),
            b"*a\n  b",
        ];
        for bytes in cases {
            let path = master.join("dic").join("a.pasta");
            fs::write(&path, bytes).unwrap();
            let t = read_text(&master, "dic/a.pasta").unwrap();
            assert_eq!(t.bom, bytes.starts_with(BOM));
            assert!(!t.text.starts_with('\u{FEFF}'));
            write_text(&master, "dic/a.pasta", &t.text, t.bom).unwrap();
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        // 作業用のファイルを残さない（11.3）
        assert_eq!(fs::read_dir(master.join("dic")).unwrap().count(), 1);
    }

    #[test]
    fn read_text_rejects_non_utf8_and_out_of_range() {
        let (_tmp, master) = ghost();
        fs::write(master.join("dic").join("sjis.txt"), b"\x82\xa0").unwrap();
        let e = read_text(&master, "dic/sjis.txt").unwrap_err();
        assert!(e.contains("UTF-8 として読めません"), "{e}");
        // BOM の後が UTF-8 でなくても Err
        fs::write(master.join("dic").join("b.txt"), b"\xEF\xBB\xBF\xff").unwrap();
        assert!(read_text(&master, "dic/b.txt").is_err());
        assert!(read_text(&master, "Profile/p.txt").is_err());
        assert!(read_text(&master, "../x").is_err());
        assert!(read_text(&master, "dic/none.pasta").is_err());
    }

    #[test]
    fn write_text_rejects_out_of_range_and_reports_os_reason() {
        let (tmp, master) = ghost();
        assert!(write_text(&master, "profile/p.txt", "x", false).is_err());
        assert!(write_text(&master, "../x.txt", "x", false).is_err());
        assert!(!tmp.path().join("ghost").join("x.txt").exists());
        let e = write_text(&master, "nodir/a.pasta", "x", false).unwrap_err();
        assert!(e.contains("書けません"), "{e}");
    }

    #[test]
    fn create_entry_makes_empty_file_and_dir_and_fails_if_exists() {
        let (_tmp, master) = ghost();
        assert_eq!(
            create_entry(&master, "dic", "b.pasta", EntryKind::File).unwrap(),
            "dic/b.pasta"
        );
        assert_eq!(fs::read(master.join("dic").join("b.pasta")).unwrap(), b"");
        assert_eq!(
            create_entry(&master, "", "sub", EntryKind::Dir).unwrap(),
            "sub"
        );
        assert!(master.join("sub").is_dir());
        assert!(create_entry(&master, "dic", "A.PASTA", EntryKind::File).is_err());
        assert!(create_entry(&master, "", "dic", EntryKind::Dir).is_err());
        assert!(create_entry(&master, "", "a?b", EntryKind::File).is_err());
        // 最上位の profile は作らない（ツリーに出ず開けない）
        fs::remove_dir_all(master.join("Profile")).unwrap();
        assert!(create_entry(&master, "", "PROFILE", EntryKind::Dir).is_err());
        assert!(create_entry(&master, "../..", "x", EntryKind::Dir).is_err());
    }

    #[test]
    fn rename_entry_returns_new_rel_and_never_overwrites() {
        let (_tmp, master) = ghost();
        fs::write(master.join("dic").join("b.pasta"), "B").unwrap();
        assert_eq!(
            rename_entry(&master, "dic/a.pasta", "c.pasta").unwrap(),
            "dic/c.pasta"
        );
        assert!(master.join("dic").join("c.pasta").is_file());
        assert!(!master.join("dic").join("a.pasta").exists());
        // 既にある別のもの（大文字・小文字違いも）は上書きしない
        assert!(rename_entry(&master, "dic/c.pasta", "B.PASTA").is_err());
        assert_eq!(fs::read(master.join("dic").join("b.pasta")).unwrap(), b"B");
        assert!(master.join("dic").join("c.pasta").is_file());
        // 自分の大文字・小文字だけの変更は通り、ディスクの綴りも変わる
        assert_eq!(
            rename_entry(&master, "dic/c.pasta", "C.pasta").unwrap(),
            "dic/C.pasta"
        );
        let names: Vec<String> = fs::read_dir(master.join("dic"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"C.pasta".to_string()), "{names:?}");
        // フォルダも変えられる
        assert_eq!(rename_entry(&master, "dic", "words").unwrap(), "words");
        assert!(master.join("words").join("b.pasta").is_file());
        // master そのもの・無いもの・最上位の profile へは変えない
        assert!(rename_entry(&master, "", "x").is_err());
        assert!(rename_entry(&master, "none.pasta", "x.pasta").is_err());
        fs::remove_dir_all(master.join("Profile")).unwrap();
        assert!(rename_entry(&master, "words", "profile").is_err());
        assert!(master.join("words").is_dir());
    }
}
