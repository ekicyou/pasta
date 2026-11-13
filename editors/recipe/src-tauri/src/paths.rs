//! ghost/master の探し方・相対パスの検査・名前の検査（読み書きの範囲の唯一の門）。

use std::fs;
use std::path::{Path, PathBuf};

pub struct Found {
    /// std::path::absolute で形を揃えた ghost/master（ディスクの綴りのまま）
    pub master: PathBuf,
    /// 渡されたものが master の中のファイルなら、その相対パス（'/' 区切り）
    pub file: Option<String>,
}

/// 2.4〜2.7 の決まりで探す。見つからなければ Err（パスと理由の日本語の文）
pub fn find_master(path: &Path) -> Result<Found, String> {
    let abs =
        std::path::absolute(path).map_err(|e| format!("{} を開けません: {e}", path.display()))?;
    let meta =
        fs::metadata(&abs).map_err(|_| format!("{} を開けません: ありません", abs.display()))?;
    let dir = if meta.is_dir() {
        abs.as_path()
    } else {
        abs.parent().unwrap_or(&abs)
    };
    let master = child_dir(dir, "ghost")
        .and_then(|g| child_dir(&g, "master"))
        .or_else(|| name_is(dir, "ghost").then(|| child_dir(dir, "master")).flatten())
        // (5)(6) D 自身か、上へたどって最も近い ghost/master
        .or_else(|| {
            dir.ancestors()
                .find(|d| name_is(d, "master") && d.parent().is_some_and(|p| name_is(p, "ghost")))
                .map(Path::to_path_buf)
        })
        .ok_or_else(|| {
            format!(
                "{} を開けません: ゴーストのフォルダではありません（ghost/master が見つかりません）",
                abs.display()
            )
        })?;
    // master の中のファイルだけを開く。profile の下（11.4）や master の外はゴーストだけ開く
    let file = if meta.is_dir() {
        None
    } else {
        abs.strip_prefix(&master).ok().and_then(|rel| {
            let parts: Vec<String> = rel
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            (!parts[0].eq_ignore_ascii_case("profile")).then(|| parts.join("/"))
        })
    };
    Ok(Found { master, file })
}

/// master からの相対パスを検査して実際のパスにする。範囲の外・profile・リンクを拒む
pub fn resolve_rel(master: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.is_empty() {
        return Ok(master.to_path_buf());
    }
    let deny = |why: &str| format!("{rel}: {why}");
    let parts: Vec<&str> = rel.split('/').collect();
    for part in &parts {
        if let Some(why) = bad_name(part) {
            return Err(deny(&format!(
                "開いているゴーストの外は読み書きしません（{why}）"
            )));
        }
    }
    if parts[0].eq_ignore_ascii_case("profile") {
        return Err(deny("profile フォルダは読み書きしません"));
    }
    let mut path = master.to_path_buf();
    for part in &parts {
        path.push(part);
        // 途中・目的そのもののリンク（ジャンクションを含む）は、外へ出られるので拒む
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(deny(
                "リンクを通るので読み書きしません（開いているゴーストの外へ出られます）",
            ));
        }
    }
    Ok(path)
}

/// 作成・名前の変更の新しい名前を検査する（空・Windows で使えない・同じフォルダの重なり）
pub fn check_name(dir: &Path, name: &str, except: Option<&str>) -> Result<(), String> {
    if let Some(why) = bad_name(name) {
        return Err(format!("「{name}」は使えない名前です: {why}"));
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("{} を読めません: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let existing = entry.file_name().to_string_lossy().into_owned();
        // 名前の変更では、自分自身（大文字・小文字だけの変更）とは比べない。
        // NTFS は ASCII の外（全角の英字など）も区別しないので、大文字へ揃えて比べる
        if Some(existing.as_str()) != except && existing.to_uppercase() == name.to_uppercase() {
            return Err(format!(
                "「{name}」は使えない名前です: 同じフォルダに「{existing}」があります"
            ));
        }
    }
    Ok(())
}

/// Windows のファイル名として使えない理由。使えれば None
fn bad_name(name: &str) -> Option<&'static str> {
    const RESERVED: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];
    if name.is_empty() {
        return Some("名前が空です");
    }
    if name
        .chars()
        .any(|c| c.is_control() || r#"/\<>:"|?*"#.contains(c))
    {
        return Some("使えない文字を含みます");
    }
    if name.ends_with(['.', ' ']) {
        return Some("末尾が「.」か空白です");
    }
    // 拡張子付き（CON.txt）も予約された名前
    let stem = name.split('.').next().unwrap_or(name).trim_end();
    let numbered = |p: &str| {
        stem.len() == 4
            && stem.get(..3).is_some_and(|s| s.eq_ignore_ascii_case(p))
            && matches!(stem.as_bytes()[3], b'1'..=b'9')
    };
    if RESERVED.iter().any(|r| stem.eq_ignore_ascii_case(r)) || numbered("COM") || numbered("LPT") {
        return Some("予約されたデバイス名です");
    }
    None
}

/// dir の子のフォルダで、名前が name（大文字・小文字を区別しない）のもの。綴りはディスクのまま
fn child_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let path = e.path();
        (e.file_name().to_string_lossy().eq_ignore_ascii_case(name) && path.is_dir())
            .then_some(path)
    })
}

fn name_is(dir: &Path, name: &str) -> bool {
    dir.file_name()
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// X/Ghost/MASTER/dic/a.pasta と X/Ghost/MASTER/profile/p.txt を持つゴーストを作る
    fn ghost() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let master = tmp.path().join("X").join("Ghost").join("MASTER");
        fs::create_dir_all(master.join("dic")).unwrap();
        fs::create_dir_all(master.join("profile")).unwrap();
        fs::write(master.join("dic").join("a.pasta"), "").unwrap();
        fs::write(master.join("profile").join("p.txt"), "").unwrap();
        fs::write(tmp.path().join("X").join("readme.txt"), "").unwrap();
        let master = std::path::absolute(&master).unwrap();
        (tmp, master)
    }

    #[test]
    fn find_master_three_forms_reach_same_master() {
        let (tmp, master) = ghost();
        let x = tmp.path().join("X");
        for p in [x.clone(), x.join("Ghost"), x.join("Ghost").join("MASTER")] {
            let found = find_master(&p).unwrap();
            assert_eq!(found.master, master, "{}", p.display());
            assert_eq!(found.file, None);
        }
    }

    #[test]
    fn find_master_ignores_case_and_returns_disk_spelling() {
        let (tmp, master) = ghost();
        // 子の ghost・master を小文字で探しても、返すのはディスクの綴り（Ghost/MASTER）
        let found = find_master(&tmp.path().join("X")).unwrap();
        assert!(found.master.ends_with(Path::new("Ghost").join("MASTER")));
        assert_eq!(found.master, master);
    }

    #[test]
    fn find_master_walks_up_from_inner_folder_and_file() {
        let (_tmp, master) = ghost();
        let found = find_master(&master.join("dic")).unwrap();
        assert_eq!(found.master, master);
        assert_eq!(found.file, None);

        let found = find_master(&master.join("dic").join("a.pasta")).unwrap();
        assert_eq!(found.master, master);
        assert_eq!(found.file.as_deref(), Some("dic/a.pasta"));
    }

    #[test]
    fn find_master_opens_ghost_but_not_file_outside_master_or_in_profile() {
        let (tmp, master) = ghost();
        let found = find_master(&master.join("profile").join("p.txt")).unwrap();
        assert_eq!(found.master, master);
        assert_eq!(found.file, None);

        let found = find_master(&tmp.path().join("X").join("readme.txt")).unwrap();
        assert_eq!(found.master, master);
        assert_eq!(found.file, None);
    }

    #[test]
    fn find_master_errors_with_path_and_reason() {
        let (tmp, _master) = ghost();
        let missing = tmp.path().join("nope");
        let e = find_master(&missing).err().unwrap();
        assert!(e.contains("nope") && e.contains("ありません"), "{e}");

        let lone = tmp.path().join("lone.txt");
        fs::write(&lone, "").unwrap();
        let e = find_master(&lone).err().unwrap();
        assert!(
            e.contains("lone.txt") && e.contains("ghost/master が見つかりません"),
            "{e}"
        );

        let plain = tmp.path().join("plain");
        fs::create_dir(&plain).unwrap();
        let e = find_master(&plain).err().unwrap();
        assert!(
            e.contains("plain") && e.contains("ghost/master が見つかりません"),
            "{e}"
        );
    }

    #[test]
    fn resolve_rel_accepts_inside_paths() {
        let (_tmp, master) = ghost();
        assert_eq!(
            resolve_rel(&master, "dic/a.pasta").unwrap(),
            master.join("dic").join("a.pasta")
        );
        // まだ無いファイル（作成の前）も通す
        assert_eq!(
            resolve_rel(&master, "dic/new.pasta").unwrap(),
            master.join("dic").join("new.pasta")
        );
        assert_eq!(resolve_rel(&master, "").unwrap(), master);
    }

    #[test]
    fn resolve_rel_rejects_out_of_range() {
        let (_tmp, master) = ghost();
        for rel in [
            "../x",
            "dic/../../x",
            "./dic",
            "dic//a.pasta",
            "/dic",
            "dic\\a.pasta",
            "C:/x",
            "dic/a:b",
            "dic/a\0b",
            "dic/a.",
            "dic/a ",
            "CON",
            "dic/con.txt",
            "dic/COM1",
            "dic/lpt9.pasta",
            "Profile/p.txt",
            "PROFILE",
        ] {
            assert!(resolve_rel(&master, rel).is_err(), "{rel:?} を通した");
        }
    }

    #[test]
    fn resolve_rel_rejects_intermediate_junction() {
        let (tmp, master) = ghost();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("o.pasta"), "").unwrap();
        // ジャンクションは権限なしで作れるので飛ばさない
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(master.join("j"))
            .arg(&outside)
            .output()
            .unwrap();
        assert!(status.status.success(), "{status:?}");
        assert!(resolve_rel(&master, "j/o.pasta").is_err());
    }

    #[test]
    fn resolve_rel_rejects_intermediate_symlink() {
        let (tmp, master) = ghost();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        if let Err(e) = std::os::windows::fs::symlink_dir(&outside, master.join("s")) {
            // シンボリックリンクは開発者モードか管理者の権限が要る。作れない環境ではここで終える
            println!("シンボリックリンクを作れないので、この確かめを飛ばします: {e}");
            return;
        }
        assert!(resolve_rel(&master, "s/o.pasta").is_err());
    }

    #[test]
    fn resolve_rel_rejects_final_junction() {
        let (tmp, master) = ghost();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(master.join("dic").join("j"))
            .arg(&outside)
            .output()
            .unwrap();
        assert!(status.status.success(), "{status:?}");
        assert!(resolve_rel(&master, "dic/j").is_err());
    }

    #[test]
    fn resolve_rel_rejects_final_file_symlink() {
        let (tmp, master) = ghost();
        let outside = tmp.path().join("o.pasta");
        fs::write(&outside, "").unwrap();
        let link = master.join("dic").join("l.pasta");
        if let Err(e) = std::os::windows::fs::symlink_file(&outside, link) {
            // シンボリックリンクは開発者モードか管理者の権限が要る。作れない環境ではここで終える
            println!("シンボリックリンクを作れないので、この確かめを飛ばします: {e}");
            return;
        }
        assert!(resolve_rel(&master, "dic/l.pasta").is_err());
    }

    #[test]
    fn check_name_rejects_bad_names() {
        let (_tmp, master) = ghost();
        let dic = master.join("dic");
        for name in [
            "", "a/b", "a\\b", "a<b", "a>b", "a:b", "a\"b", "a|b", "a?b", "a*b", "a\tb", "a.",
            "a ", ".", "..", "CON", "nul.txt", "Com3",
        ] {
            assert!(check_name(&dic, name, None).is_err(), "{name:?} を通した");
        }
        assert!(check_name(&dic, "b.pasta", None).is_ok());
        // 4 バイトで 3 バイト目が文字の途中の名前（予約名の確かめで落ちない）
        assert!(check_name(&dic, "aあ", None).is_ok());
    }

    #[test]
    fn check_name_rejects_case_insensitive_collision_but_allows_own_case_change() {
        let (_tmp, master) = ghost();
        let dic = master.join("dic");
        let e = check_name(&dic, "A.PASTA", None).err().unwrap();
        assert!(e.contains("A.PASTA"), "{e}");
        assert!(check_name(&dic, "A.pasta", Some("a.pasta")).is_ok());
        // 自分以外との重なりは except があっても拒む
        fs::write(dic.join("b.pasta"), "").unwrap();
        assert!(check_name(&dic, "B.pasta", Some("a.pasta")).is_err());
    }

    #[test]
    fn check_name_collision_ignores_non_ascii_case() {
        let (_tmp, master) = ghost();
        let dic = master.join("dic");
        // NTFS は全角の英字も大文字・小文字を区別しない
        fs::write(dic.join("Ａ.pasta"), "").unwrap();
        assert!(check_name(&dic, "ａ.pasta", None).is_err());
        assert!(check_name(&dic, "ａ.pasta", Some("Ａ.pasta")).is_ok());
    }
}
