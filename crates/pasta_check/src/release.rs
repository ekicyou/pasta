use crate::ReleaseArgs;
use crate::balloon::plan_bundled_balloons;
use crate::copy::{copy_dir_recursive, prepare_release_dir};
use crate::nar::create_nar;
use crate::update_files::generate_update_files;
use std::fs;
use std::io;
use std::path::Path;

/// release サブコマンドを実行
pub(crate) fn execute_release(args: &ReleaseArgs) -> io::Result<()> {
    // Step 1: リリースフォルダー初期化
    println!("[1/5] Preparing release folder...");
    prepare_release_dir(&args.release)?;
    remove_stale_nar(&args.nar)?;

    // Step 2: target → release コピー
    println!("[2/5] Copying target files...");
    let count = copy_dir_recursive(&args.target, &args.release)?;
    println!("  Copied {count} files from {}", args.target.display());

    // Step 3: --copy 上書きコピー
    if !args.copy_dirs.is_empty() {
        println!("[3/5] Applying overlay copies...");
        for copy_dir in &args.copy_dirs {
            let c = copy_dir_recursive(copy_dir, &args.release)?;
            println!("  Copied {c} files from {}", copy_dir.display());
        }
    } else {
        println!("[3/5] Applying overlay copies... (none specified)");
    }

    // Step 4: 更新ファイル生成（判定は --copy 上書き後の配布フォルダで行う・Req 1.7）
    println!("[4/5] Generating update files...");
    let plan = plan_bundled_balloons(&args.release)?;
    let summary = generate_update_files(&args.release, &plan.dirs)?;
    println!(
        "  Generated updates.txt ({} entries)",
        summary.ghost_entries
    );
    for (dir, entries) in &summary.balloon_entries {
        println!("  Generated {dir}/updates.txt ({entries} entries)");
    }
    // 警告は止めずに段 5 へ進む（Req 7.3・7.4）
    for w in &plan.warnings {
        eprintln!("Warning: {w}");
    }

    // Step 5: NAR 作成
    println!("[5/5] Creating NAR archive...");
    // 途中で失敗したら作りかけの nar を消してから元のエラーを返す（Req 8.8）
    let nar_size = create_nar(&args.release, &args.nar).inspect_err(|_| {
        let _ = fs::remove_file(&args.nar);
    })?;
    let nar_size_kb = nar_size as f64 / 1024.0;
    println!("  Created {} ({nar_size_kb:.1} KB)", args.nar.display());

    println!();
    println!("Release complete!");

    Ok(())
}

/// --nar の位置の前回の nar を消す（失敗時に古い nar を最新と誤って配らないため）。
/// 無いときは成功とみなし、それ以外の削除エラー（フォルダ・削除できない）はそのまま返す。
fn remove_stale_nar(nar: &Path) -> io::Result<()> {
    match fs::remove_file(nar) {
        Ok(()) => {
            println!("  Removed previous {}", nar.display());
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_release_full_pipeline() {
        let temp = TempDir::new().unwrap();

        // target フォルダーを準備
        let target = temp.path().join("target_ghost");
        fs::create_dir_all(target.join("ghost/master")).unwrap();
        fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
        let install_txt = "charset,UTF-8\r\ntype,ghost\r\n";
        fs::write(target.join("install.txt"), install_txt).unwrap();

        let release = temp.path().join("release_out");
        let nar = temp.path().join("out.nar");

        let args = ReleaseArgs {
            target: target.clone(),
            release: release.clone(),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        execute_release(&args).unwrap();

        // リリースフォルダーに更新ファイルが生成されている
        assert!(!release.join("updates2.dau").exists());
        assert!(release.join("updates.txt").exists());
        // NAR が作成されている
        assert!(nar.exists());
        assert!(nar.metadata().unwrap().len() > 0);
        // target フォルダーは変更されていない
        assert!(!target.join("updates2.dau").exists());
        assert_eq!(
            fs::read_to_string(target.join("install.txt")).unwrap(),
            install_txt
        );
    }

    /// 複数 --copy 指定時は後勝ち（後のオーバーレイが前のオーバーレイを上書きする）
    #[test]
    fn test_execute_release_overlay_precedence_last_wins() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "original").unwrap();

        let overlay1 = temp.path().join("overlay1");
        fs::create_dir_all(&overlay1).unwrap();
        fs::write(overlay1.join("a.txt"), "first").unwrap();
        fs::write(overlay1.join("only1.txt"), "one").unwrap();

        let overlay2 = temp.path().join("overlay2");
        fs::create_dir_all(&overlay2).unwrap();
        fs::write(overlay2.join("a.txt"), "second").unwrap();

        let release = temp.path().join("release_out");
        let args = ReleaseArgs {
            target,
            release: release.clone(),
            nar: temp.path().join("out.nar"),
            copy_dirs: vec![overlay1, overlay2],
        };

        execute_release(&args).unwrap();

        assert_eq!(fs::read_to_string(release.join("a.txt")).unwrap(), "second");
        // 上書きされなかったオーバーレイ 1 固有のファイルは残る
        assert_eq!(
            fs::read_to_string(release.join("only1.txt")).unwrap(),
            "one"
        );
    }

    /// 既存の release フォルダーは初期化され、stale ファイルが残らない
    #[test]
    fn test_execute_release_cleans_stale_release_dir() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "fresh").unwrap();

        let release = temp.path().join("release_out");
        fs::create_dir_all(&release).unwrap();
        fs::write(release.join("stale.txt"), "stale").unwrap();

        let args = ReleaseArgs {
            target,
            release: release.clone(),
            nar: temp.path().join("out.nar"),
            copy_dirs: vec![],
        };

        execute_release(&args).unwrap();

        assert!(!release.join("stale.txt").exists());
        assert_eq!(fs::read_to_string(release.join("a.txt")).unwrap(), "fresh");
    }

    /// NAR には生成済みの updates.txt が封入される
    /// （updates.txt 生成 → NAR 作成の実行順序契約。順序が逆転すると fail する）
    #[test]
    fn test_execute_release_nar_contains_updates_txt() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(target.join("ghost/master")).unwrap();
        fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();

        let nar = temp.path().join("out.nar");
        let args = ReleaseArgs {
            target,
            release: temp.path().join("release_out"),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        execute_release(&args).unwrap();

        let file = fs::File::open(&nar).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("updates.txt").is_ok());
        assert!(archive.by_name("ghost/master/updates.txt").is_ok());
    }

    /// homeurl の無い同梱バルーンは警告だけで止まらず、nar とバルーン用 updates.txt を作る（Req 7.3）。
    /// 判定は --copy の上書き後の install.txt で行う（Req 1.7）
    #[test]
    fn test_execute_release_bundled_balloon_without_homeurl_succeeds() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(target.join("ghost/master")).unwrap();
        fs::create_dir_all(target.join("bal")).unwrap();
        fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
        fs::write(
            target.join("install.txt"),
            "charset,UTF-8\r\ntype,ghost\r\n",
        )
        .unwrap();
        fs::write(
            target.join("bal/descript.txt"),
            "charset,UTF-8\r\nname,bal\r\n",
        )
        .unwrap();
        fs::write(target.join("bal/balloons0.png"), "png").unwrap();

        let overlay = temp.path().join("overlay");
        fs::create_dir_all(&overlay).unwrap();
        fs::write(
            overlay.join("install.txt"),
            "charset,UTF-8\r\ntype,ghost\r\nballoon.directory,bal\r\n",
        )
        .unwrap();

        let release = temp.path().join("release_out");
        let nar = temp.path().join("out.nar");
        let args = ReleaseArgs {
            target,
            release: release.clone(),
            nar: nar.clone(),
            copy_dirs: vec![overlay],
        };

        execute_release(&args).unwrap();

        assert!(nar.exists());
        let bal_updates = fs::read_to_string(release.join("bal/updates.txt")).unwrap();
        assert!(bal_updates.contains("file,balloons0.png\x01"));
        // ゴースト用 updates.txt には同梱バルーンのファイルを載せない
        let ghost_updates = fs::read_to_string(release.join("updates.txt")).unwrap();
        assert!(!ghost_updates.contains("bal/"));
    }

    /// 判定エラーでは段 5 に進まず nar を作らない
    #[test]
    fn test_execute_release_balloon_plan_error_creates_no_nar() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(
            target.join("install.txt"),
            "charset,UTF-8\r\nballoon.directory,missing\r\n",
        )
        .unwrap();

        let nar = temp.path().join("out.nar");
        let args = ReleaseArgs {
            target,
            release: temp.path().join("release_out"),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        assert!(execute_release(&args).is_err());
        assert!(!nar.exists());
    }

    /// 後段で失敗しても --nar の位置に前回の nar が残らない（Req 8.8）
    #[test]
    fn test_execute_release_removes_stale_nar_on_failure() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "fresh").unwrap();

        let nar = temp.path().join("out.nar");
        fs::write(&nar, "old nar").unwrap();

        let args = ReleaseArgs {
            target,
            release: temp.path().join("release_out"),
            nar: nar.clone(),
            copy_dirs: vec![temp.path().join("no_such_overlay")],
        };

        assert!(execute_release(&args).is_err());
        assert!(!nar.exists());
    }

    /// 成功時は前回の nar が新しい nar に置き換わる（Req 8.8）
    #[test]
    fn test_execute_release_replaces_stale_nar_on_success() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "fresh").unwrap();

        let nar = temp.path().join("out.nar");
        fs::write(&nar, "old nar").unwrap();

        let args = ReleaseArgs {
            target,
            release: temp.path().join("release_out"),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        execute_release(&args).unwrap();

        let mut archive = zip::ZipArchive::new(fs::File::open(&nar).unwrap()).unwrap();
        assert!(archive.by_name("a.txt").is_ok());
    }

    /// 段 5 の nar 作成が途中で失敗しても --nar の位置に作りかけの nar が残らない（Req 8.8）。
    /// 不正なサロゲートを含む 2 つの名前は to_string_lossy で同じエントリ名になり、
    /// nar ファイル作成後の 2 件目で ZIP の重複名エラーになる。
    /// var/ は段 4（updates.txt）の対象外で、段 5 だけが読む。
    #[cfg(windows)]
    #[test]
    fn test_execute_release_removes_partial_nar_on_stage5_failure() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        let var = target.join("var");
        fs::create_dir_all(&var).unwrap();
        fs::write(target.join("a.txt"), "fresh").unwrap();
        fs::write(var.join(OsString::from_wide(&[0xD800])), "a").unwrap();
        fs::write(var.join(OsString::from_wide(&[0xD801])), "b").unwrap();

        let nar = temp.path().join("out.nar");
        let release = temp.path().join("release_out");
        let args = ReleaseArgs {
            target,
            release: release.clone(),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        assert!(execute_release(&args).is_err());
        // 段 4 までは完了している（失敗は段 5）
        assert!(release.join("updates.txt").exists());
        assert!(!nar.exists());
    }

    /// --nar がフォルダのときは段 1 で止まる（削除エラーをそのまま返す・Req 8.8）
    #[test]
    fn test_execute_release_nar_is_dir_errors_at_stage1() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "fresh").unwrap();

        let nar = temp.path().join("out.nar");
        fs::create_dir_all(&nar).unwrap();

        let release = temp.path().join("release_out");
        let args = ReleaseArgs {
            target,
            release: release.clone(),
            nar: nar.clone(),
            copy_dirs: vec![],
        };

        assert!(execute_release(&args).is_err());
        // 段 2 以降に進んでいない
        assert!(!release.join("a.txt").exists());
        assert!(nar.is_dir());
    }

    /// target が存在しない場合はエラーが伝播する
    #[test]
    fn test_execute_release_missing_target_errors() {
        let temp = TempDir::new().unwrap();
        let args = ReleaseArgs {
            target: temp.path().join("no_such_target"),
            release: temp.path().join("release_out"),
            nar: temp.path().join("out.nar"),
            copy_dirs: vec![],
        };
        assert!(execute_release(&args).is_err());
    }

    #[test]
    fn test_execute_release_with_copy() {
        let temp = TempDir::new().unwrap();

        let target = temp.path().join("target_ghost");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("a.txt"), "original").unwrap();

        let overlay = temp.path().join("overlay");
        fs::create_dir_all(&overlay).unwrap();
        fs::write(overlay.join("a.txt"), "overwritten").unwrap();
        fs::write(overlay.join("b.txt"), "new file").unwrap();

        let release = temp.path().join("release_out");
        let nar = temp.path().join("out.nar");

        let args = ReleaseArgs {
            target: target.clone(),
            release: release.clone(),
            nar,
            copy_dirs: vec![overlay],
        };

        execute_release(&args).unwrap();

        assert_eq!(
            fs::read_to_string(release.join("a.txt")).unwrap(),
            "overwritten"
        );
        assert_eq!(
            fs::read_to_string(release.join("b.txt")).unwrap(),
            "new file"
        );
        // target は変更されていない
        assert_eq!(
            fs::read_to_string(target.join("a.txt")).unwrap(),
            "original"
        );
    }
}
