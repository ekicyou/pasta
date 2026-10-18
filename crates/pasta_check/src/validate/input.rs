//! 検査の核の入力。JS からも同じ形で渡せるよう、キーは camelCase、列挙はタグ `kind` で表す。

use serde::Deserialize;

/// 報告での pasta.toml のパス（config と report の両方が使うので入力の側に置く）
pub(super) const PASTA_TOML: &str = "pasta.toml";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhostInput {
    /// ghost/master/pasta.toml の状態
    pub pasta_toml: TomlSource,
    /// ghost/master の配下のファイルの相対パス（'/' 区切り）。順序は問わない
    pub entries: Vec<String>,
    /// ghost/master の配下のフォルダの相対パス（'/' 区切り・末尾の '/' なし・ghost/master
    /// そのものは入れない）。順序は問わない。必須で既定値を持たない
    pub dirs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TomlSource {
    Missing,
    Text { text: String },
    Unreadable { reason: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FileText {
    /// UTF-8 として読めた本文（BOM は残したまま渡す。parse_str が外す）
    Text {
        text: String,
    },
    NotUtf8,
    Unreadable {
        reason: String,
    },
}
