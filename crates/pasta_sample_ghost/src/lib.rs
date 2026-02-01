//! pasta_sample_ghost - サンプルゴースト「hello-pasta」実装
//!
//! このクレートは、pasta システムの入門者向けサンプルゴーストを提供します。
//! SHIORI/3.0 プロトコルで動作するミニマルなゴーストとして、
//! インストール直後から動作可能な状態を実現します。

pub mod config_templates;
pub mod image_generator;
pub mod scripts;

use std::path::Path;
use thiserror::Error;

/// ゴースト生成時のエラー
#[derive(Debug, Error)]
pub enum GhostError {
    #[error("画像生成エラー: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("IOエラー: {0}")]
    IoError(#[from] std::io::Error),

    #[error("設定エラー: {0}")]
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

/// ゴースト配布物を生成
///
/// # Arguments
/// * `output_dir` - 出力先ディレクトリ（hello-pasta/ が作成される）
/// * `config` - ゴースト設定
///
/// # Returns
/// 成功時は Ok(()), 失敗時は GhostError
pub fn generate_ghost(output_dir: &Path, config: &GhostConfig) -> Result<(), GhostError> {
    // ディレクトリ構造を生成
    config_templates::generate_structure(output_dir, config)?;

    // シェル画像を生成
    let shell_dir = output_dir.join("shell/master");
    image_generator::generate_surfaces(&shell_dir)?;

    // pasta スクリプトを生成
    let dic_dir = output_dir.join("ghost/master/dic");
    scripts::generate_scripts(&dic_dir)?;

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
