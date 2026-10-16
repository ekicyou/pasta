//! 検査の核の入力。JS からも同じ形で渡せるよう、キーは camelCase、列挙はタグ `kind` で表す。

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhostInput {
    /// ghost/master/pasta.toml の状態
    pub pasta_toml: TomlSource,
    /// ghost/master の配下のファイルの相対パス（'/' 区切り）。順序は問わない
    pub entries: Vec<String>,
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
