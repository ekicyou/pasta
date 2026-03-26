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

use pasta_sample_ghost::{GhostConfig, generate_ghost};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let output_dir = get_output_dir(&args);
    run_generate_mode(&output_dir)?;
    Ok(())
}

/// 通常モード：ゴースト配布物を生成
fn run_generate_mode(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  pasta_sample_ghost Generator");
    println!("========================================");
    println!();
    println!("Output: {}", output_dir.display());
    println!();

    // 設定
    let config = GhostConfig::default();

    // ゴースト生成
    println!("Generating surface images and surfaces.txt...");
    generate_ghost(output_dir, &config)?;

    // 生成されたファイルをカウント
    let file_count = count_files(output_dir);

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

/// ディレクトリ内のファイル数をカウント
fn count_files(dir: &PathBuf) -> usize {
    walkdir(dir)
}

/// 再帰的にファイル数をカウント
fn walkdir(path: &PathBuf) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += walkdir(&path);
            }
        }
    }
    count
}
