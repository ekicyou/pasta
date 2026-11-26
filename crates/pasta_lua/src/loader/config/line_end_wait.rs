//! `[talk]` と `[actor."名前"]` の行末のキーの型の検査。

use pasta_core::LINE_END_WAIT_KEYS;
use toml::de::DeTable;

/// [talk] と [actor."名前"] の行末のキーが整数かを確かめる
pub(super) fn check(content: &str, custom_fields: &toml::Table) -> Result<(), toml::de::Error> {
    let mut places: Vec<(Vec<&str>, String, &toml::Table)> = Vec::new();
    if let Some(toml::Value::Table(talk)) = custom_fields.get("talk") {
        places.push((vec!["talk"], "[talk]".to_string(), talk));
    }
    if let Some(toml::Value::Table(actors)) = custom_fields.get("actor") {
        for (name, actor) in actors {
            if let toml::Value::Table(actor) = actor {
                places.push((vec!["actor", name], format!("[actor.\"{name}\"]"), actor));
            }
        }
    }

    let mut doc = None;
    let mut messages = Vec::new();
    for (path, place, table) in &places {
        for key in LINE_END_WAIT_KEYS {
            let Some(value) = table.get(key) else {
                continue;
            };
            if value.is_integer() {
                continue;
            }
            if doc.is_none() {
                doc = Some(DeTable::parse(content)?);
            }
            let line = value_line(content, doc.as_ref().unwrap().get_ref(), path, key);
            messages.push(format!(
                "{key} は整数（ミリ秒）で書いてください: {}が書かれています; at {place} (line {line})",
                type_label(value.type_str())
            ));
        }
    }
    if messages.is_empty() {
        Ok(())
    } else {
        Err(serde::de::Error::custom(messages.join(", ")))
    }
}

fn value_line(content: &str, doc: &DeTable<'_>, path: &[&str], key: &str) -> usize {
    let mut table = doc;
    for segment in path {
        match super::actor_font::entry(table, segment).and_then(|v| v.get_ref().as_table()) {
            Some(t) => table = t,
            None => return 0,
        }
    }
    super::actor_font::entry(table, key).map_or(0, |v| {
        content[..v.span().start.min(content.len())]
            .matches('\n')
            .count()
            + 1
    })
}

fn type_label(type_name: &str) -> &'static str {
    match type_name {
        "string" => "文字列",
        "float" => "浮動小数点数",
        "boolean" => "真偽値",
        "array" => "配列",
        "table" => "表",
        "datetime" => "日時",
        _ => "ほかの型",
    }
}
