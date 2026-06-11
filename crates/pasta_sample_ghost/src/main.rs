//! pasta_sample_ghost - サンプルゴースト配布物生成ツール
//!
//! hello-pasta ゴーストのシェル画像を生成します。
//!
//! # 使い方
//!
//! ```bash
//! # デフォルト（ghosts/hello-pasta/ に生成）
//! cargo run -p pasta_sample_ghost
//!
//! # カスタム出力先
//! cargo run -p pasta_sample_ghost -- /path/to/output
//! ```
//!
//! # 生成されるファイル
//!
//! - shell/master/surface*.png（18ファイル）
//! - shell/master/surfaces.txt
//!
//! 辞書・設定ファイルは `ghosts/hello-pasta/` に直接 git 管理されています。
//! 更新ファイルと NAR は `pasta_check release` コマンドで生成します。

use pasta_sample_ghost::generate_ghost;
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let output_dir = get_output_dir(&args);
    run_generate_mode(&output_dir)?;
    Ok(())
}

/// 通常モード：ゴースト配布物を生成
fn run_generate_mode(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  pasta_sample_ghost Generator");
    println!("========================================");
    println!();
    println!("Output: {}", output_dir.display());
    println!();

    // ゴースト生成
    println!("Generating surface images and surfaces.txt...");
    generate_ghost(output_dir)?;

    // 生成されたファイルをカウント
    let file_count = walkdir(output_dir);

    println!();
    println!("========================================");
    println!("  Generation Complete!");
    println!("  (surface*.png + surfaces.txt only)");
    println!("========================================");
    println!();
    println!("  Location: {}", output_dir.display());
    println!("  Files:    {}", file_count);
    println!();
    println!("Next steps:");
    println!("  1. Run release.ps1 to copy pasta.dll, pasta_scripts/, and create .nar");
    println!("  2. Or run: release.ps1 -SkipDllBuild (if DLL already built)");
    println!();

    Ok(())
}

/// 出力先ディレクトリを決定する
fn get_output_dir(args: &[String]) -> PathBuf {
    for arg in args.iter().skip(1) {
        if !arg.starts_with('-') {
            return PathBuf::from(arg);
        }
    }

    // デフォルト: crate_root/ghosts/hello-pasta
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .join("ghosts")
        .join("hello-pasta")
}

/// 再帰的にファイル数をカウント
fn walkdir(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            if file_type.is_file() {
                count += 1;
            } else if file_type.is_dir() {
                count += walkdir(&path);
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::{get_output_dir, walkdir};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn get_output_dir_uses_first_non_flag_argument() {
        let args = vec![
            "pasta_sample_ghost".to_string(),
            "--some-flag".to_string(),
            "custom/output".to_string(),
            "ignored".to_string(),
        ];
        assert_eq!(get_output_dir(&args), PathBuf::from("custom/output"));
    }

    #[test]
    fn get_output_dir_defaults_to_ghosts_hello_pasta() {
        // フラグのみ（位置引数なし）の場合は CARGO_MANIFEST_DIR/ghosts/hello-pasta
        let args = vec!["pasta_sample_ghost".to_string(), "--flag-only".to_string()];
        let dir = get_output_dir(&args);
        assert!(
            dir.ends_with(Path::new("ghosts").join("hello-pasta")),
            "デフォルト出力先が ghosts/hello-pasta ではありません: {}",
            dir.display()
        );
        // 引数なし（プログラム名のみ）でも同じデフォルト
        let args = vec!["pasta_sample_ghost".to_string()];
        assert_eq!(get_output_dir(&args), dir);
    }

    #[test]
    fn walkdir_counts_nested_files() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("a/b")).unwrap();
        fs::write(temp.path().join("f1.txt"), "1").unwrap();
        fs::write(temp.path().join("a").join("f2.txt"), "2").unwrap();
        fs::write(temp.path().join("a/b").join("f3.txt"), "3").unwrap();

        // ディレクトリは数えず、ネストしたファイルのみ 3 件
        assert_eq!(walkdir(temp.path()), 3);
    }

    #[test]
    fn walkdir_returns_zero_for_missing_or_empty_dir() {
        let temp = TempDir::new().unwrap();
        assert_eq!(walkdir(&temp.path().join("does_not_exist")), 0);
        assert_eq!(walkdir(temp.path()), 0);
    }

    #[cfg(unix)]
    #[test]
    fn walkdir_skips_symlinks() {
        use std::os::unix::fs as unix_fs;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("root");
        let external = temp.path().join("external");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir_all(&external).unwrap();
        fs::write(root.join("real.txt"), "real").unwrap();
        fs::write(root.join("nested").join("nested.txt"), "nested").unwrap();
        fs::write(external.join("secret.txt"), "secret").unwrap();
        unix_fs::symlink(root.join("real.txt"), root.join("link.txt")).unwrap();
        unix_fs::symlink(&external, root.join("linked_dir")).unwrap();

        assert_eq!(walkdir(&root), 2);
    }
}
