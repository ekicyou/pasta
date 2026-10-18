//! `[actor."名前"].font` の読み込み（ActorFontLoader）。
//!
//! 規則は `pasta_core::ActorFont` が持つ。ここでは書いた順と行番号を
//! `toml::de::DeTable` の span から取り出し、`custom_fields` の `font` を
//! 検査済みのタグ列（文字列）に置き換える。誤りは読み込みエラーにする。

use pasta_core::{ActorFont, ActorFontError, FontInput, FontValue};
use toml::Spanned;
use toml::de::{DeTable, DeValue};

/// `[actor.*.font]` を検査し、custom_fields の font を検査済みのタグ列（文字列）に置き換える
pub(super) fn apply(content: &str, custom_fields: &mut toml::Table) -> Result<(), toml::de::Error> {
    let Some(toml::Value::Table(actors)) = custom_fields.get_mut("actor") else {
        return Ok(());
    };
    let doc = DeTable::parse(content)?;
    let line = |pos: usize| content[..pos.min(content.len())].matches('\n').count() + 1;

    // `toml::Table` は名前順なので、誤りは名前順で最初のアクターで止まる。
    for (name, actor) in actors.iter_mut() {
        let Some(font) = actor.as_table_mut().and_then(|t| t.get_mut("font")) else {
            continue;
        };
        let spanned = entry(doc.get_ref(), "actor")
            .and_then(|a| a.get_ref().as_table())
            .and_then(|a| entry(a, name))
            .and_then(|a| a.get_ref().as_table())
            .and_then(|a| entry(a, "font"));
        // 値の開始位置（見つからないことは無いが、無ければ末尾扱い）
        let font_pos = spanned.map_or(content.len(), |f| f.span().start);
        let pos_of = |key: &str| {
            spanned
                .and_then(|f| f.get_ref().as_table())
                .and_then(|t| entry(t, key))
                .map_or(content.len(), |v| v.span().start)
        };

        let input = match font {
            toml::Value::Table(table) => {
                let mut entries: Vec<_> = table
                    .iter()
                    .map(|(k, v)| (pos_of(k), k.clone(), font_value(v)))
                    .collect();
                entries.sort_by_key(|(pos, ..)| *pos);
                FontInput::Table(entries.into_iter().map(|(_, k, v)| (k, v)).collect())
            }
            _ => FontInput::NotTable,
        };

        match ActorFont::from_input(name, input) {
            Ok(parsed) => *font = toml::Value::String(parsed.tags().to_string()),
            Err(errors) => {
                let messages: Vec<String> = errors
                    .iter()
                    .map(|e| match e {
                        ActorFontError::NotTable { .. } => {
                            format!("{e}; at [actor.\"{name}\".font] (line {})", line(font_pos))
                        }
                        ActorFontError::UnknownAttribute { attribute, .. }
                        | ActorFontError::InvalidValue { attribute, .. } => format!(
                            "{e}; at [actor.\"{name}\".font] {attribute} (line {})",
                            line(pos_of(attribute))
                        ),
                    })
                    .collect();
                return Err(serde::de::Error::custom(messages.join(", ")));
            }
        }
    }
    Ok(())
}

/// 表のキーを名前で引く（キーが span つきなので走査する）
fn entry<'a, 'i>(table: &'a DeTable<'i>, key: &str) -> Option<&'a Spanned<DeValue<'i>>> {
    table
        .iter()
        .find(|(k, _)| k.get_ref().as_ref() == key)
        .map(|(_, v)| v)
}

fn font_value(value: &toml::Value) -> FontValue {
    match value {
        toml::Value::String(s) => FontValue::String(s.clone()),
        toml::Value::Integer(i) => FontValue::Integer(*i),
        toml::Value::Boolean(b) => FontValue::Boolean(*b),
        other => FontValue::Other {
            type_name: other.type_str(),
        },
    }
}
