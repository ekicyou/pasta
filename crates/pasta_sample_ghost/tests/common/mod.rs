//! テスト共通ヘルパー

use std::path::PathBuf;

/// ワークスペースルートを取得
pub fn workspace_root() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR が設定されていません");
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .and_then(|p| p.parent()) // pasta/
        .expect("ワークスペースルートが見つかりません")
        .to_path_buf()
}
