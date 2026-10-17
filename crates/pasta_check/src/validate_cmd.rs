//! validate サブコマンドの殻: ゴーストのフォルダから一覧と中身を集め、検査の核を呼ぶ。
//!
//! 読むだけで、ファイルを作らない・書き換えない・消さない（Req 1.6）。

use pasta_check::validate::{
    FileText, GhostInput, Report, TomlSource, msg, select_dictionary_files, validate_ghost,
};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 検査の結果（フォルダの誤りは `Err` で別に返す）
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ValidateOutcome {
    Clean,
    WarningsOnly,
    HasErrors(usize),
}

impl ValidateOutcome {
    /// 誤りがあれば 1、無ければ 0（Req 7.5・7.6）
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            ValidateOutcome::HasErrors(_) => 1,
            _ => 0,
        }
    }
}

/// --target のゴーストのフォルダを確かめて検査する。フォルダの誤りは Err（理由）
///
/// `ghost/master` そのものを渡されても読み替えない（Req 1.3）。
pub(crate) fn run(target: &Path) -> Result<ValidateOutcome, String> {
    if !target.exists() {
        return Err(format!("{} がありません", target.display()));
    }
    if !target.is_dir() {
        return Err(format!("{} はフォルダではありません", target.display()));
    }
    let master = target.join("ghost").join("master");
    if !master.is_dir() {
        return Err(format!(
            "{} に ghost/master がありません（--target にはゴーストの開発フォルダを指定します）",
            target.display()
        ));
    }
    run_on_master(&master)
}

/// ghost/master を直接受けて検査し、行を標準出力に出す（release からも使う）。
/// 一覧を作れないときは Err（理由）
pub(crate) fn run_on_master(master: &Path) -> Result<ValidateOutcome, String> {
    let report = check_master(master)?;
    for line in report.lines() {
        println!("{line}");
    }
    Ok(outcome_of(&report))
}

fn check_master(master: &Path) -> Result<Report, String> {
    let (input, files) = collect(master)?;
    Ok(validate_ghost(&input, &files))
}

fn outcome_of(report: &Report) -> ValidateOutcome {
    match (report.stats.errors, report.stats.warnings) {
        (0, 0) => ValidateOutcome::Clean,
        (0, _) => ValidateOutcome::WarningsOnly,
        (n, _) => ValidateOutcome::HasErrors(n),
    }
}

/// ghost/master の一覧と pasta.toml を読み、核が選んだ辞書ファイルの中身を集める
fn collect(master: &Path) -> Result<(GhostInput, BTreeMap<String, FileText>), String> {
    let mut found = Vec::new();
    let mut dirs = Vec::new();
    walk(master, "", &mut found, &mut dirs)?;

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
fn read_file(path: &Path) -> FileText {
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
    fn ghost() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("ghost_dev");
        let master = target.join("ghost").join("master");
        fs::create_dir_all(master.join("dic")).unwrap();
        (temp, target, master)
    }

    const OK_SCENE: &str = "＊挨拶\n  さくら：こんにちは。\n";

    #[test]
    fn test_run_clean() {
        let (_t, target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::write(master.join("dic/a.pasta"), OK_SCENE).unwrap();
        assert_eq!(run(&target).unwrap(), ValidateOutcome::Clean);
        assert_eq!(ValidateOutcome::Clean.exit_code(), 0);
    }

    /// 見つからない Call は注意で、誤りは無い
    #[test]
    fn test_run_warnings_only_exit_0() {
        let (_t, target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::write(master.join("dic/a.pasta"), "＊挨拶\n  ＞どこにも無い\n").unwrap();
        let outcome = run(&target).unwrap();
        assert_eq!(outcome, ValidateOutcome::WarningsOnly);
        assert_eq!(outcome.exit_code(), 0);
    }

    /// pasta.toml が無いのは誤り（config-missing）
    #[test]
    fn test_run_missing_toml_is_error() {
        let (_t, target, master) = ghost();
        fs::write(master.join("dic/a.pasta"), OK_SCENE).unwrap();
        assert_eq!(run(&target).unwrap(), ValidateOutcome::HasErrors(1));
    }

    #[test]
    fn test_run_has_errors_exit_1() {
        let (_t, target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::write(master.join("dic/a.pasta"), "＊\n  ＠＠＠\n").unwrap();
        fs::write(master.join("dic/b.pasta"), [0xff, 0xfe, 0x00, 0x81]).unwrap();
        let outcome = run(&target).unwrap();
        assert!(matches!(outcome, ValidateOutcome::HasErrors(n) if n >= 2));
        assert_eq!(outcome.exit_code(), 1);
    }

    #[test]
    fn test_run_folder_errors() {
        let (_t, target, master) = ghost();
        // 存在しない
        let e = run(&target.join("nope")).unwrap_err();
        assert!(e.contains("nope"), "{e}");
        // ディレクトリでない
        let file = target.join("file.txt");
        fs::write(&file, "x").unwrap();
        assert!(run(&file).is_err());
        // ghost/master を含まない（ghost/master そのものも読み替えない）
        assert!(run(&master).is_err());
        assert!(run(&target.join("ghost")).is_err());
        // ghost/master がファイル
        let other = target.join("other");
        fs::create_dir_all(other.join("ghost")).unwrap();
        fs::write(other.join("ghost/master"), "x").unwrap();
        assert!(run(&other).is_err());
    }

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

    /// 読むだけ: 実行の前後で一覧が変わらない
    #[test]
    fn test_run_writes_nothing() {
        let (t, target, master) = ghost();
        fs::write(master.join("dic/a.pasta"), OK_SCENE).unwrap();
        let list = || {
            let mut v = Vec::new();
            let mut stack = vec![t.path().to_path_buf()];
            while let Some(d) = stack.pop() {
                for e in fs::read_dir(d).unwrap() {
                    let p = e.unwrap().path();
                    if p.is_dir() {
                        stack.push(p.clone());
                    }
                    v.push(p);
                }
            }
            v.sort();
            v
        };
        let before = list();
        run(&target).unwrap();
        assert_eq!(before, list());
    }
}
