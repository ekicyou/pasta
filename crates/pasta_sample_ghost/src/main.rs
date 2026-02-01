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
//! ```
//!
//! # 生成されるファイル
//!
//! - install.txt, readme.txt
//! - ghost/master/descript.txt, pasta.toml
//! - ghost/master/dic/*.pasta（4ファイル）
//! - shell/master/descript.txt, surfaces.txt
//! - shell/master/surface*.png（18ファイル）
//!
//! # 注意
//!
//! pasta.dll と scripts/ は setup.bat で別途コピーされます。

use pasta_sample_ghost::{GhostConfig, generate_ghost};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 出力先を決定
    let output_dir = get_output_dir();

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
    generate_ghost(&output_dir, &config)?;

    // 生成されたファイルをカウント
    let file_count = count_files(&output_dir);

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
    println!("  2. Or manually copy:");
    println!("     - target/i686-pc-windows-msvc/release/pasta.dll");
    println!("     - crates/pasta_lua/scripts/");
    println!();

    Ok(())
}

/// 出力先ディレクトリを決定する
fn get_output_dir() -> PathBuf {
    // コマンドライン引数があればそれを使う
    if let Some(arg) = env::args().nth(1) {
        return PathBuf::from(arg);
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
