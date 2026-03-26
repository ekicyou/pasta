use crate::ReleaseArgs;
use crate::copy::{copy_dir_recursive, prepare_release_dir};
use crate::nar::create_nar;
use crate::update_files::generate_update_files;
use std::io;

/// release サブコマンドを実行
pub fn execute_release(args: &ReleaseArgs) -> io::Result<()> {
    // Step 1: リリースフォルダー初期化
    println!("[1/5] Preparing release folder...");
    prepare_release_dir(&args.release)?;

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

    // Step 4: 更新ファイル生成
    println!("[4/5] Generating update files...");
    let entries = generate_update_files(&args.release)?;
    println!("  Generated updates2.dau and updates.txt ({entries} entries)");

    // Step 5: NAR 作成
    println!("[5/5] Creating NAR archive...");
    let nar_size = create_nar(&args.release, &args.nar)?;
    let nar_size_kb = nar_size as f64 / 1024.0;
    println!("  Created {} ({nar_size_kb:.1} KB)", args.nar.display());

    println!();
    println!("Release complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_execute_release_full_pipeline() {
        let temp = TempDir::new().unwrap();

        // target フォルダーを準備
        let target = temp.path().join("target_ghost");
        fs::create_dir_all(target.join("ghost/master")).unwrap();
        fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
        fs::write(target.join("install.txt"), "install").unwrap();

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
        assert!(release.join("updates2.dau").exists());
        assert!(release.join("updates.txt").exists());
        // NAR が作成されている
        assert!(nar.exists());
        assert!(nar.metadata().unwrap().len() > 0);
        // target フォルダーは変更されていない
        assert!(!target.join("updates2.dau").exists());
        assert_eq!(fs::read_to_string(target.join("install.txt")).unwrap(), "install");
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

        assert_eq!(fs::read_to_string(release.join("a.txt")).unwrap(), "overwritten");
        assert_eq!(fs::read_to_string(release.join("b.txt")).unwrap(), "new file");
        // target は変更されていない
        assert_eq!(fs::read_to_string(target.join("a.txt")).unwrap(), "original");
    }
}
