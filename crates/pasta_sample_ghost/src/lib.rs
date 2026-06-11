//! pasta_sample_ghost - サンプルゴースト「hello-pasta」実装
//!
//! このクレートは、pasta システムの入門者向けサンプルゴーストを提供します。
//! SHIORI/3.0 プロトコルで動作するミニマルなゴーストとして、
//! インストール直後から動作可能な状態を実現します。

pub mod config_templates;
pub mod image_generator;
pub mod scripts;

use std::fs;
use std::path::Path;
use thiserror::Error;

/// ゴースト生成時のエラー
#[derive(Debug, Error)]
pub enum GhostError {
    #[error("Image generation error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// ゴースト配布物を生成（画像＋surfaces.txt のみ）
///
/// テキスト系ファイル（辞書・設定ファイル）は `ghosts/hello-pasta/` に直接配置されています。
///
/// # Arguments
/// * `output_dir` - 出力先ディレクトリ（hello-pasta/ が作成される）
///
/// # Returns
/// 成功時は Ok(()), 失敗時は GhostError
pub fn generate_ghost(output_dir: &Path) -> Result<(), GhostError> {
    // シェルディレクトリ作成（画像生成前に必要）
    let shell_dir = output_dir.join("shell/master");
    fs::create_dir_all(&shell_dir)?;

    // シェル画像を生成
    image_generator::generate_surfaces(&shell_dir)?;

    // surfaces.txt を生成
    fs::write(
        shell_dir.join("surfaces.txt"),
        config_templates::generate_surfaces_txt(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ghost_writes_surfaces_txt_matching_template() {
        let temp = tempfile::TempDir::new().unwrap();
        let ghost_root = temp.path().join("hello-pasta");

        generate_ghost(&ghost_root).unwrap();

        // 書き出された surfaces.txt がテンプレート出力と完全一致すること
        // （生成パイプラインが内容を変質させない）
        let written = fs::read_to_string(ghost_root.join("shell/master/surfaces.txt")).unwrap();
        assert_eq!(written, config_templates::generate_surfaces_txt());
    }

    #[test]
    fn generate_ghost_is_idempotent_over_existing_output() {
        // 既存出力ディレクトリへの再実行（リリース手順での再生成シナリオ）が
        // エラーなく成功し、ファイル集合が変わらないこと
        let temp = tempfile::TempDir::new().unwrap();
        let ghost_root = temp.path().join("hello-pasta");

        generate_ghost(&ghost_root).unwrap();
        let count_files =
            |dir: &Path| -> usize { fs::read_dir(dir).map(|d| d.flatten().count()).unwrap_or(0) };
        let shell_dir = ghost_root.join("shell/master");
        let first = count_files(&shell_dir);
        assert_eq!(first, 19, "初回生成は surface*.png 18 + surfaces.txt 1");

        generate_ghost(&ghost_root).unwrap();
        assert_eq!(
            count_files(&shell_dir),
            first,
            "再実行でファイル集合が変化しました"
        );
    }
}
