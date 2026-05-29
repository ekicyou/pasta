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

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// ゴースト設定
#[derive(Debug, Clone)]
pub struct GhostConfig {
    /// ゴースト名
    pub name: String,
    /// バージョン
    pub version: String,
    /// sakura キャラクター名
    pub sakura_name: String,
    /// kero キャラクター名
    pub kero_name: String,
    /// 作者ID
    pub craftman: String,
    /// 作者名（日本語）
    pub craftman_w: String,
    /// SHIORI DLL名
    pub shiori: String,
    /// ホームURL
    pub homeurl: String,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            name: "hello-pasta".to_string(),
            version: "1.0.0".to_string(),
            sakura_name: "女の子".to_string(),
            kero_name: "男の子".to_string(),
            craftman: "ekicyou".to_string(),
            craftman_w: "どっとステーション駅長".to_string(),
            shiori: "pasta.dll".to_string(),
            homeurl: "https://github.com/ekicyou/pasta".to_string(),
        }
    }
}

/// ゴースト配布物を生成（画像＋surfaces.txt のみ）
///
/// テキスト系ファイル（辞書・設定ファイル）は `ghosts/hello-pasta/` に直接配置されています。
///
/// # Arguments
/// * `output_dir` - 出力先ディレクトリ（hello-pasta/ が作成される）
/// * `_config` - ゴースト設定（API 互換性のため保持）
///
/// # Returns
/// 成功時は Ok(()), 失敗時は GhostError
pub fn generate_ghost(output_dir: &Path, _config: &GhostConfig) -> Result<(), GhostError> {
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
    fn test_default_config() {
        let config = GhostConfig::default();
        assert_eq!(config.name, "hello-pasta");
        assert_eq!(config.sakura_name, "女の子");
        assert_eq!(config.kero_name, "男の子");
        assert_eq!(config.shiori, "pasta.dll");
    }
}
