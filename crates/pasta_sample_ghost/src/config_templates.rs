//! 設定ファイルテンプレート生成
//!
//! ukadoc 準拠の設定ファイルと pasta.toml を生成します。
//! テンプレートファイルは `templates/` ディレクトリから読み込まれます。

use crate::{GhostConfig, GhostError};
use std::fs;
use std::path::Path;

// テンプレートファイルをコンパイル時にインクルード
const INSTALL_TXT_TEMPLATE: &str = include_str!("../templates/install.txt.template");
const GHOST_DESCRIPT_TEMPLATE: &str = include_str!("../templates/ghost_descript.txt.template");
const SHELL_DESCRIPT_TEMPLATE: &str = include_str!("../templates/shell_descript.txt.template");
const PASTA_TOML_TEMPLATE: &str = include_str!("../templates/pasta.toml.template");

/// ゴースト配布物のディレクトリ構造を生成
pub fn generate_structure(output_dir: &Path, config: &GhostConfig) -> Result<(), GhostError> {
    // ディレクトリ作成
    let ghost_master = output_dir.join("ghost/master");
    let dic_dir = ghost_master.join("dic");
    let shell_master = output_dir.join("shell/master");

    fs::create_dir_all(&dic_dir)?;
    fs::create_dir_all(&shell_master)?;

    // 各設定ファイルを生成
    fs::write(output_dir.join("install.txt"), generate_install_txt(config))?;
    fs::write(
        ghost_master.join("descript.txt"),
        generate_ghost_descript(config),
    )?;
    fs::write(ghost_master.join("pasta.toml"), generate_pasta_toml(config))?;
    fs::write(
        shell_master.join("descript.txt"),
        generate_shell_descript(config),
    )?;
    fs::write(shell_master.join("surfaces.txt"), generate_surfaces_txt())?;

    Ok(())
}

/// install.txt を生成
pub fn generate_install_txt(config: &GhostConfig) -> String {
    INSTALL_TXT_TEMPLATE.replace("{{name}}", &config.name)
}

/// ghost/master/descript.txt を生成
pub fn generate_ghost_descript(config: &GhostConfig) -> String {
    GHOST_DESCRIPT_TEMPLATE
        .replace("{{name}}", &config.name)
        .replace("{{sakura_name}}", &config.sakura_name)
        .replace("{{kero_name}}", &config.kero_name)
        .replace("{{craftman}}", &config.craftman)
        .replace("{{craftman_w}}", &config.craftman_w)
        .replace("{{shiori}}", &config.shiori)
        .replace("{{homeurl}}", &config.homeurl)
}

/// shell/master/descript.txt を生成
///
/// 仕様準拠: requirements.md Requirement 9.3
pub fn generate_shell_descript(config: &GhostConfig) -> String {
    SHELL_DESCRIPT_TEMPLATE
        .replace("{{craftman}}", &config.craftman)
        .replace("{{craftman_w}}", &config.craftman_w)
}

/// surfaces.txt を生成
pub fn generate_surfaces_txt() -> String {
    let mut content = String::from("charset,UTF-8\n\n");

    // sakura サーフェス (0-8)
    for i in 0..=8 {
        content.push_str(&format!(
            r#"surface{i}
{{
element0,overlay,surface{i}.png,0,0
}}

"#
        ));
    }

    // kero サーフェス (10-18)
    for i in 10..=18 {
        content.push_str(&format!(
            r#"surface{i}
{{
element0,overlay,surface{i}.png,0,0
}}

"#
        ));
    }

    content
}

/// pasta.toml を生成（教育的コメント付き）
///
/// 仕様準拠: requirements.md Requirement 7.1-7.4
pub fn generate_pasta_toml(config: &GhostConfig) -> String {
    PASTA_TOML_TEMPLATE
        .replace("{{name}}", &config.name)
        .replace("{{version}}", &config.version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_txt() {
        let config = GhostConfig::default();
        let content = generate_install_txt(&config);
        assert!(content.contains("type,ghost"));
        assert!(content.contains("name,hello-pasta"));
        assert!(content.contains("directory,hello-pasta"));
    }

    #[test]
    fn test_ghost_descript() {
        let config = GhostConfig::default();
        let content = generate_ghost_descript(&config);
        assert!(content.contains("charset,UTF-8"));
        assert!(content.contains("shiori,pasta.dll"));
        assert!(content.contains("sakura.name,女の子"));
        assert!(content.contains("kero.name,男の子"));
    }

    #[test]
    fn test_pasta_toml() {
        let config = GhostConfig::default();
        let content = generate_pasta_toml(&config);
        assert!(content.contains("[package]"));
        assert!(content.contains("[loader]"));
        assert!(content.contains("[ghost]"));
        assert!(content.contains("[logging]"));
        assert!(content.contains("level = \"debug\""));
        assert!(content.contains("# filter = \"debug\""));
        assert!(content.contains("talk_interval_min = 180"));
        assert!(content.contains("talk_interval_max = 300"));
        assert!(content.contains("hour_margin"));
        assert!(content.contains("教育的サンプル"));
    }

    #[test]
    fn test_surfaces_txt() {
        let content = generate_surfaces_txt();
        assert!(content.contains("surface0"));
        assert!(content.contains("surface8"));
        assert!(content.contains("surface10"));
        assert!(content.contains("surface18"));
    }
}
