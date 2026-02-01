//! pasta_sample_ghost - サンプルゴースト配布物生成ツール
//!
//! hello-pasta ゴーストの配布物一式を生成します。
//!
//! # 使い方
//!
//! ```bash
//! # デフォルト（ghosts/hello-pasta/ に生成）
//! cargo run -p pasta_sample_ghost
//!
//! # カスタム出力先
//! cargo run -p pasta_sample_ghost -- /path/to/output
//!
//! # finalize モード（更新ファイルのみ生成）
//! cargo run -p pasta_sample_ghost -- --finalize
//! cargo run -p pasta_sample_ghost -- --finalize /path/to/output
//! ```
//!
//! # 生成されるファイル
//!
//! - install.txt, readme.txt
//! - ghost/master/descript.txt, pasta.toml
//! - ghost/master/dic/*.pasta（4ファイル）
//! - shell/master/descript.txt, surfaces.txt
//! - shell/master/surface*.png（18ファイル）
//! - updates2.dau, updates.txt（finalize モード時）
//!
//! # 注意
//!
//! pasta.dll と scripts/ は setup.bat で別途コピーされます。
//! 更新ファイル（updates2.dau, updates.txt）は --finalize オプションで生成します。

use pasta_sample_ghost::{GhostConfig, finalize_ghost, generate_ghost};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // --finalize オプションをチェック
    let finalize_mode = args.iter().any(|arg| arg == "--finalize");

    // 出力先を決定（--finalize 以外の引数を探す）
    let output_dir = get_output_dir(&args);

    if finalize_mode {
        run_finalize_mode(&output_dir)?;
    } else {
        run_generate_mode(&output_dir)?;
    }

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
    println!("Generating ghost distribution...");
    generate_ghost(output_dir, &config)?;

    // 生成されたファイルをカウント
    let file_count = count_files(output_dir);

    println!();
    println!("========================================");
    println!("  Generation Complete!");
    println!("========================================");
    println!();
    println!("  Location: {}", output_dir.display());
    println!("  Files:    {}", file_count);
    println!();
    println!("Next steps:");
    println!("  1. Run setup.bat to copy pasta.dll and scripts/");
    println!("  2. setup.bat will also generate updates2.dau and updates.txt");
    println!();

    Ok(())
}

/// finalize モード：更新ファイルのみ生成
fn run_finalize_mode(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  pasta_sample_ghost Finalize");
    println!("========================================");
    println!();
    println!("Target: {}", output_dir.display());
    println!();

    // 出力ディレクトリが存在するか確認
    if !output_dir.exists() {
        eprintln!("ERROR: Directory does not exist: {}", output_dir.display());
        eprintln!("       Run without --finalize first to generate the ghost.");
        std::process::exit(1);
    }

    // 更新ファイルを生成
    println!("Generating update files...");
    let entry_count = finalize_ghost(output_dir)?;

    println!();
    println!("========================================");
    println!("  Finalize Complete!");
    println!("========================================");
    println!();
    println!("  Location: {}", output_dir.display());
    println!("  Entries:  {} files indexed", entry_count);
    println!();
    println!("Generated files:");
    println!("  - updates2.dau (SSP binary format)");
    println!("  - updates.txt  (SSP text format)");
    println!();

    Ok(())
}

/// 出力先ディレクトリを決定する
fn get_output_dir(args: &[String]) -> PathBuf {
    // --finalize 以外の引数を探す
    for arg in args.iter().skip(1) {
        if arg != "--finalize" && !arg.starts_with('-') {
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
