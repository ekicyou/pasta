//! Build script for pasta_sample_ghost
//!
//! このビルドスクリプトは最小限の処理のみを行います。
//! ゴースト配布物の生成は `cargo run -p pasta_sample_ghost` または
//! `setup.bat` で行ってください。

use std::env;
use std::path::Path;

fn main() {
    // ビルドスクリプト再実行トリガー
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/");
    println!("cargo::rerun-if-changed=templates/");

    // pasta_shiori のソース変更を監視
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to get workspace root");

    let pasta_shiori_src = workspace_root.join("crates/pasta_shiori/src");
    if pasta_shiori_src.exists() {
        println!("cargo::rerun-if-changed={}", pasta_shiori_src.display());
    }

    // ghosts/ の存在チェック（警告のみ）
    let ghosts_dir = Path::new(&manifest_dir).join("ghosts").join("hello-pasta");
    if !ghosts_dir.exists() {
        eprintln!();
        eprintln!("========================================");
        eprintln!("  ghosts/hello-pasta/ not found!");
        eprintln!("========================================");
        eprintln!();
        eprintln!("  To generate the ghost distribution, run:");
        eprintln!();
        eprintln!("    cd crates/pasta_sample_ghost");
        eprintln!("    .\\setup.bat");
        eprintln!();
        eprintln!("  Or manually:");
        eprintln!();
        eprintln!("    cargo run -p pasta_sample_ghost");
        eprintln!();
    }
}
