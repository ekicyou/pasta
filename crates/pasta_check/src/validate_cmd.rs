//! validate サブコマンドの殻: ゴーストのフォルダから一覧と中身を集め、検査の核を呼ぶ。
//!
//! 読むだけで、ファイルを作らない・書き換えない・消さない（Req 1.6）。

use pasta_check::ghost_files::collect;
use pasta_check::validate::{Report, derived_homeurl_finding, validate_ghost};
use std::fs;
use std::path::Path;

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
    let report = validate_ghost(&input, &files);
    // 無い・読めない・フォルダの descript.txt は注意を出さず、結果も変えない（Req 8.4）
    let Some(f) = fs::read(master.join("descript.txt"))
        .ok()
        .and_then(|bytes| derived_homeurl_finding(&bytes))
    else {
        return Ok(report);
    };
    let Report {
        mut findings,
        stats,
    } = report;
    findings.push(f);
    Ok(Report::new(findings, stats))
}

fn outcome_of(report: &Report) -> ValidateOutcome {
    match (report.stats.errors, report.stats.warnings) {
        (0, 0) => ValidateOutcome::Clean,
        (0, _) => ValidateOutcome::WarningsOnly,
        (n, _) => ValidateOutcome::HasErrors(n),
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

    const DERIVED: &str = "charset,UTF-8
name,my-ghost
homeurl,https://ekicyou.github.io/pasta/hello-pasta/
";

    /// 派生ゴーストの homeurl が hello-pasta の置き場所を指すと注意だけで終了コード 0（Req 8.3・8.7）
    #[test]
    fn test_run_derived_homeurl_warns_exit_0() {
        let (_t, target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::write(master.join("dic/a.pasta"), OK_SCENE).unwrap();
        fs::write(master.join("descript.txt"), DERIVED).unwrap();
        let report = check_master(&master).unwrap();
        assert!(
            report.findings.iter().any(|f| f.path == "descript.txt"
                && f.code == pasta_check::validate::FindingKind::HelloPastaHomeurl),
            "{:?}",
            report.findings
        );
        assert_eq!(report.stats.warnings, 1);
        let outcome = run(&target).unwrap();
        assert_eq!(outcome, ValidateOutcome::WarningsOnly);
        assert_eq!(outcome.exit_code(), 0);
    }

    /// hello-pasta 自身・descript.txt が無い・フォルダのときは注意を出さない（Req 8.4）
    #[test]
    fn test_run_homeurl_no_warning_cases() {
        let (_t, target, master) = ghost();
        fs::write(master.join("pasta.toml"), "").unwrap();
        fs::write(master.join("dic/a.pasta"), OK_SCENE).unwrap();
        assert_eq!(run(&target).unwrap(), ValidateOutcome::Clean);
        fs::create_dir(master.join("descript.txt")).unwrap();
        assert_eq!(run(&target).unwrap(), ValidateOutcome::Clean);
        fs::remove_dir(master.join("descript.txt")).unwrap();
        fs::write(
            master.join("descript.txt"),
            DERIVED.replace("name,my-ghost", "name,hello-pasta"),
        )
        .unwrap();
        assert_eq!(run(&target).unwrap(), ValidateOutcome::Clean);
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
