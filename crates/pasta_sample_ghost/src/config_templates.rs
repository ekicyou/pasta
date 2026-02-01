//! 設定ファイルテンプレート生成
//!
//! ukadoc 準拠の設定ファイルと pasta.toml を生成します。

use crate::{GhostConfig, GhostError};
use std::fs;
use std::path::Path;

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
    format!(
        r#"type,ghost
name,{name}
directory,{name}
accept,
"#,
        name = config.name
    )
}

/// ghost/master/descript.txt を生成
pub fn generate_ghost_descript(config: &GhostConfig) -> String {
    format!(
        r#"charset,UTF-8
type,ghost
name,{name}
sakura.name,{sakura_name}
kero.name,{kero_name}
craftman,{craftman}
craftmanw,{craftman_w}
shiori,{shiori}
homeurl,{homeurl}
"#,
        name = config.name,
        sakura_name = config.sakura_name,
        kero_name = config.kero_name,
        craftman = config.craftman,
        craftman_w = config.craftman_w,
        shiori = config.shiori,
        homeurl = config.homeurl
    )
}

/// shell/master/descript.txt を生成
pub fn generate_shell_descript(config: &GhostConfig) -> String {
    format!(
        r#"charset,UTF-8
type,shell
name,master
craftman,{craftman}
craftmanw,{craftman_w}
sakura.balloon.offsetx,0
sakura.balloon.offsety,80
kero.balloon.offsetx,0
kero.balloon.offsety,80
"#,
        craftman = config.craftman,
        craftman_w = config.craftman_w
    )
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
pub fn generate_pasta_toml(config: &GhostConfig) -> String {
    format!(
        r#"# {name} ゴースト設定ファイル
# pasta alpha04 サンプル

# [教育的サンプル]
# [package] セクションは伺かゴーストでは省略可能です。
# install.txt/readme.txt で同様の情報を管理できます。
# 将来的な pasta_lua の汎用用途（ノベルゲーム、ツール等）では
# このセクションが必須になる可能性があります。
[package]
name = "{name}"
version = "0.1.0"
authors = ["{craftman_w}"]
description = "pasta入門用サンプルゴースト"

[loader]
# スクリプトファイルパターン
patterns = ["**/*.pasta"]
# 起動時自動ロード
auto_load = true
# デバッグモード
debug_mode = true

[logging]
# ログレベル: off, error, warn, info, debug, trace
level = "info"
# ログ出力先（ベースシェルのログフォルダ）
output = "log"

[persistence]
# 永続化ファイル名
filename = "save.lua"
# 自動保存間隔（秒）- OnCloseでも保存
auto_save_interval = 300

[lua]
# メモリ制限（MB）- 0で無制限
memory_limit = 128
# 追加モジュール検索パス
module_path = ["./scripts", "./lib"]

[ghost]
# 改行マーカー切替待機時間（秒）
spot_switch_newlines = 1.5
# ランダムトーク間隔（秒）
talk_interval_min = 60   # 1分（テスト用に短縮）
talk_interval_max = 120  # 2分（テスト用に短縮）
# 時報マージン（秒）
hour_margin = 30
"#,
        name = config.name,
        craftman_w = config.craftman_w
    )
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
