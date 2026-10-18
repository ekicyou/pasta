//! Actor font (display style): `[actor."name"].font` in pasta.toml.
//!
//! Validates the attributes and turns them into `\f[attribute,value]` tags in the
//! order they were written. Knows nothing about TOML or Lua: callers convert their
//! values into [`FontValue`] and pass them in written order.

use crate::error::{ActorFontError, FontValueProblem};

/// Accepted attributes (Requirement 1.2). This order is the canonical list, not the output order.
pub const FONT_ATTRIBUTES: [&str; 12] = [
    "color",
    "name",
    "height",
    "bold",
    "italic",
    "underline",
    "strike",
    "sub",
    "sup",
    "outline",
    "shadowcolor",
    "shadowstyle",
];

/// One attribute value, converted by the caller from its source (e.g. a TOML value).
#[derive(Debug, Clone, PartialEq)]
pub enum FontValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    /// Float, array, table, datetime. `type_name` is for the message ("float" etc.).
    Other {
        type_name: &'static str,
    },
}

/// The `font` entry of one actor.
#[derive(Debug, Clone, PartialEq)]
pub enum FontInput {
    /// `(attribute, value)` pairs in written order.
    Table(Vec<(String, FontValue)>),
    /// Not a table, e.g. `font = "red"`.
    NotTable,
}

/// A validated actor font: the tag string to emit on speaker switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorFont {
    tags: String,
}

impl ActorFont {
    /// Validate `input`; on error, every error in the table is returned in input order.
    pub fn from_input(actor: &str, input: FontInput) -> Result<ActorFont, Vec<ActorFontError>> {
        let FontInput::Table(entries) = input else {
            return Err(vec![ActorFontError::NotTable {
                actor: actor.to_string(),
            }]);
        };
        let mut tags = String::new();
        let mut errors = Vec::new();
        for (attribute, value) in entries {
            if !FONT_ATTRIBUTES.contains(&attribute.as_str()) {
                errors.push(ActorFontError::UnknownAttribute {
                    actor: actor.to_string(),
                    attribute,
                });
                continue;
            }
            let text = match value {
                FontValue::String(s) if s.is_empty() => Err(FontValueProblem::Empty),
                FontValue::String(s) if s.contains([']', '\\']) => Err(FontValueProblem::BreaksTag),
                FontValue::String(s) => Ok(s),
                FontValue::Integer(n) => Ok(n.to_string()),
                FontValue::Boolean(b) => Ok(if b { "1" } else { "0" }.to_string()),
                FontValue::Other { type_name } => Err(FontValueProblem::WrongType { type_name }),
            };
            match text {
                Ok(text) => tags.push_str(&format!("\\f[{attribute},{text}]")),
                Err(problem) => errors.push(ActorFontError::InvalidValue {
                    actor: actor.to_string(),
                    attribute,
                    problem,
                }),
            }
        }
        if errors.is_empty() {
            Ok(ActorFont { tags })
        } else {
            Err(errors)
        }
    }

    /// e.g. `\f[color,90,110,140]\f[bold,1]`; `""` when there are no attributes.
    pub fn tags(&self) -> &str {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> FontValue {
        FontValue::String(v.to_string())
    }

    fn table(entries: Vec<(&str, FontValue)>) -> FontInput {
        FontInput::Table(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn tags(entries: Vec<(&str, FontValue)>) -> String {
        ActorFont::from_input("おじさん", table(entries))
            .unwrap()
            .tags()
            .to_string()
    }

    fn errors(input: FontInput) -> Vec<ActorFontError> {
        ActorFont::from_input("おじさん", input).unwrap_err()
    }

    fn invalid(attribute: &str, problem: FontValueProblem) -> ActorFontError {
        ActorFontError::InvalidValue {
            actor: "おじさん".to_string(),
            attribute: attribute.to_string(),
            problem,
        }
    }

    #[test]
    fn all_12_attributes_in_written_order() {
        // Written order is the reverse of FONT_ATTRIBUTES: output must follow the input.
        let entries: Vec<(&str, FontValue)> =
            FONT_ATTRIBUTES.iter().rev().map(|a| (*a, s("v"))).collect();
        let expected: String = FONT_ATTRIBUTES
            .iter()
            .rev()
            .map(|a| format!("\\f[{a},v]"))
            .collect();
        assert_eq!(tags(entries), expected);
    }

    #[test]
    fn value_formats() {
        assert_eq!(
            tags(vec![
                ("color", s("90,110,140")),
                ("bold", FontValue::Boolean(true)),
                ("italic", FontValue::Boolean(false)),
                ("height", FontValue::Integer(-12)),
                ("name", s("#5a6e8c")),
            ]),
            "\\f[color,90,110,140]\\f[bold,1]\\f[italic,0]\\f[height,-12]\\f[name,#5a6e8c]"
        );
    }

    #[test]
    fn content_is_not_checked() {
        // Requirement 7.6: SSP interprets the value.
        assert_eq!(
            tags(vec![
                ("color", s("not a color")),
                ("height", FontValue::Integer(99999))
            ]),
            "\\f[color,not a color]\\f[height,99999]"
        );
    }

    #[test]
    fn empty_table_is_empty_tags() {
        assert_eq!(tags(vec![]), "");
    }

    #[test]
    fn attribute_names_are_case_sensitive() {
        assert_eq!(
            errors(table(vec![
                ("Color", s("red")),
                ("BOLD", FontValue::Boolean(true))
            ])),
            vec![
                ActorFontError::UnknownAttribute {
                    actor: "おじさん".to_string(),
                    attribute: "Color".to_string()
                },
                ActorFontError::UnknownAttribute {
                    actor: "おじさん".to_string(),
                    attribute: "BOLD".to_string()
                },
            ]
        );
    }

    #[test]
    fn excluded_attributes_are_unknown() {
        for a in [
            "align",
            "valign",
            "cursorcolor",
            "anchorcolor",
            "default",
            "disable",
        ] {
            assert!(matches!(
                errors(table(vec![(a, s("x"))])).as_slice(),
                [ActorFontError::UnknownAttribute { .. }]
            ));
        }
    }

    #[test]
    fn wrong_types() {
        assert_eq!(
            errors(table(vec![
                ("height", FontValue::Other { type_name: "float" }),
                ("color", FontValue::Other { type_name: "array" }),
                ("name", FontValue::Other { type_name: "table" }),
            ])),
            vec![
                invalid("height", FontValueProblem::WrongType { type_name: "float" }),
                invalid("color", FontValueProblem::WrongType { type_name: "array" }),
                invalid("name", FontValueProblem::WrongType { type_name: "table" }),
            ]
        );
    }

    #[test]
    fn empty_string_and_tag_breaking_chars() {
        assert_eq!(
            errors(table(vec![
                ("color", s("")),
                ("name", s("a]b")),
                ("shadowcolor", s("a\\b")),
            ])),
            vec![
                invalid("color", FontValueProblem::Empty),
                invalid("name", FontValueProblem::BreaksTag),
                invalid("shadowcolor", FontValueProblem::BreaksTag),
            ]
        );
    }

    #[test]
    fn all_errors_collected_in_input_order() {
        assert_eq!(
            errors(table(vec![
                ("color", s("red")),
                ("colour", s("red")),
                ("bold", s("")),
                ("height", FontValue::Integer(20)),
            ])),
            vec![
                ActorFontError::UnknownAttribute {
                    actor: "おじさん".to_string(),
                    attribute: "colour".to_string()
                },
                invalid("bold", FontValueProblem::Empty),
            ]
        );
    }

    #[test]
    fn not_a_table() {
        assert_eq!(
            errors(FontInput::NotTable),
            vec![ActorFontError::NotTable {
                actor: "おじさん".to_string()
            }]
        );
    }

    #[test]
    fn messages_name_actor_and_attribute() {
        let msgs: Vec<String> = errors(table(vec![
            ("Color", s("red")),
            ("height", FontValue::Other { type_name: "float" }),
            ("name", s("")),
            ("sub", s("]")),
        ]))
        .iter()
        .map(|e| e.to_string())
        .collect();
        assert_eq!(
            msgs,
            vec![
                "actor font: actor 'おじさん' has unknown attribute 'Color'",
                "actor font: actor 'おじさん' attribute 'height': value must be a string, integer or boolean (got float)",
                "actor font: actor 'おじさん' attribute 'name': value must not be an empty string",
                "actor font: actor 'おじさん' attribute 'sub': value must not contain ']' or '\\'",
            ]
        );
        assert_eq!(
            errors(FontInput::NotTable)[0].to_string(),
            "actor font: font of actor 'おじさん' must be a table"
        );
    }
}
