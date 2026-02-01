//! サンプルゴースト画像生成ツール
//!
//! `cargo run --bin generate-surfaces` でシェル画像を生成します。

use pasta_sample_ghost::image_generator;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // クレートルートからの相対パス
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let shell_dir = PathBuf::from(&manifest_dir)
        .join("ghosts")
        .join("hello-pasta")
        .join("shell")
        .join("master");

    println!("Generating surfaces to: {}", shell_dir.display());

    // ディレクトリが存在しない場合は作成
    std::fs::create_dir_all(&shell_dir)?;

    // サーフェス画像を生成
    image_generator::generate_surfaces(&shell_dir)?;

    println!("Done! Generated 18 surface images.");
    Ok(())
}
