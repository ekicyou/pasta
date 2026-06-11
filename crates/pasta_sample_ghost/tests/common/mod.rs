//! テスト共通ヘルパー

use std::path::{Path, PathBuf};

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

/// pasta_shiori.dll をコピー
///
/// # Arguments
/// * `dest_dir` - コピー先ディレクトリ
///
/// # Returns
/// コピー成功時は Ok(()), DLL不在時はエラー
pub fn copy_pasta_shiori_dll(dest_dir: &Path) -> Result<(), String> {
    let workspace = workspace_root();
    let dll_path = workspace
        .join("target")
        .join("i686-pc-windows-msvc")
        .join("release")
        .join("pasta_shiori.dll");

    if !dll_path.exists() {
        return Err(format!(
            "pasta_shiori.dll が見つかりません: {}\n\
             テスト実行前に以下のコマンドでビルドしてください:\n\
             cargo build --release --target i686-pc-windows-msvc -p pasta_shiori",
            dll_path.display()
        ));
    }

    std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;

    let dest_path = dest_dir.join("pasta.dll");
    std::fs::copy(&dll_path, &dest_path).map_err(|e| format!("DLL コピー失敗: {}", e))?;

    Ok(())
}
