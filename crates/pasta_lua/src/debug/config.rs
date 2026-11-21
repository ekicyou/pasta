//! デバッグの有効の解決（`PASTA_DEBUG` ＞ 保存変数 `デバッグ`／`debug` ＞ `[debug] enabled` ＞ false）。
//!
//! 読む環境変数は `PASTA_DEBUG` だけ。呼ぶのはローダの Phase 5.5 の 1 回だけ。

use crate::loader::DebugFileConfig;

/// 保存変数の段で見る名前。この順に見て、最初の真偽値を採る。
pub(super) const SAVE_SWITCH_NAMES: [&str; 2] = ["デバッグ", "debug"];

/// `PASTA_DEBUG`（読める値なら）＞ 保存変数 `デバッグ`／`debug` ＞ `[debug] enabled` ＞ false。
///
/// `save` は `PASTA_DEBUG` で決まらないときだけ呼ぶ。読めない保存ファイルは `None`。
pub fn resolve_enabled(
    file: Option<&DebugFileConfig>,
    save: impl FnOnce() -> Option<toml::Table>,
) -> bool {
    let env = std::env::var("PASTA_DEBUG")
        .ok()
        .and_then(|v| parse_env_bool(&v));
    decide(
        env,
        || save().and_then(|doc| save_switch(&doc)),
        file.map(|f| f.enabled),
    )
}

fn decide(env: Option<bool>, save: impl FnOnce() -> Option<bool>, file: Option<bool>) -> bool {
    env.or_else(save).or(file).unwrap_or(false)
}

/// `デバッグ` → `debug` の順に見て、最初の真偽値。見た名前の真偽値でない値は警告する。
pub(super) fn save_switch(doc: &toml::Table) -> Option<bool> {
    SAVE_SWITCH_NAMES
        .iter()
        .find_map(|name| match doc.get(*name)? {
            toml::Value::Boolean(b) => Some(*b),
            other => {
                tracing::warn!(
                    name = %format!("＄＊{name}"),
                    value_type = other.type_str(),
                    "save variable is not a boolean; ignored when deciding debug"
                );
                None
            }
        })
}

/// 環境変数の値を bool に読む。
///
/// 真: `1`・`true`・`yes`・`on`、偽: `0`・`false`・`no`・`off`・空（大小・前後の空白は問わない）。
/// ほかは `None`（設定されていないのと同じ）。
pub(super) fn parse_env_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" | "" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const TRI: [Option<bool>; 3] = [Some(true), Some(false), None];

    #[test]
    fn decide_table() {
        for env in TRI {
            for save in TRI {
                for file in TRI {
                    let called = Cell::new(false);
                    let got = decide(
                        env,
                        || {
                            called.set(true);
                            save
                        },
                        file,
                    );
                    let want = env.or(save).or(file).unwrap_or(false);
                    assert_eq!(got, want, "env={env:?} save={save:?} file={file:?}");
                    assert_eq!(called.get(), env.is_none(), "env={env:?} save={save:?}");
                }
            }
        }
    }

    #[test]
    fn unreadable_env_falls_through() {
        assert!(decide(parse_env_bool("maybe"), || Some(true), Some(false)));
        assert!(!decide(parse_env_bool("maybe"), || None, None));
    }

    #[derive(Clone, Copy, Debug)]
    enum Slot {
        True,
        False,
        Missing,
        NotBool,
    }

    fn put(doc: &mut toml::Table, name: &str, slot: Slot, alt: bool) {
        let v = match slot {
            Slot::True => toml::Value::Boolean(true),
            Slot::False => toml::Value::Boolean(false),
            Slot::Missing => return,
            Slot::NotBool if alt => toml::Value::Integer(1),
            Slot::NotBool => toml::Value::String("true".into()),
        };
        doc.insert(name.into(), v);
    }

    fn as_bool(slot: Slot) -> Option<bool> {
        match slot {
            Slot::True => Some(true),
            Slot::False => Some(false),
            _ => None,
        }
    }

    #[test]
    fn save_switch_table() {
        use Slot::*;
        for ja in [True, False, Missing, NotBool] {
            for en in [True, False, Missing, NotBool] {
                for alt in [false, true] {
                    let mut doc = toml::Table::new();
                    put(&mut doc, "デバッグ", ja, alt);
                    put(&mut doc, "debug", en, !alt);
                    let want = as_bool(ja).or(as_bool(en));
                    assert_eq!(save_switch(&doc), want, "ja={ja:?} en={en:?} alt={alt}");
                }
            }
        }
    }

    #[test]
    fn save_switch_ignores_other_spellings() {
        let mut doc = toml::Table::new();
        for name in ["Debug", "DEBUG", "dbg", "デバック"] {
            doc.insert(name.into(), toml::Value::Boolean(true));
        }
        assert_eq!(save_switch(&doc), None);
        assert_eq!(SAVE_SWITCH_NAMES, ["デバッグ", "debug"]);
    }

    #[test]
    #[tracing_test::traced_test]
    fn warns_for_inspected_non_boolean() {
        let mut doc = toml::Table::new();
        doc.insert("デバッグ".into(), toml::Value::String("true".into()));
        doc.insert("debug".into(), toml::Value::Integer(1));
        assert_eq!(save_switch(&doc), None);
        logs_assert(|lines| {
            let warns: Vec<_> = lines
                .iter()
                .filter(|l| l.contains("save variable is not a boolean"))
                .collect();
            match warns.as_slice() {
                [a, b]
                    if a.contains("＄＊デバッグ")
                        && a.contains("string")
                        && b.contains("＄＊debug")
                        && b.contains("integer") =>
                {
                    Ok(())
                }
                _ => Err(format!("{warns:?}")),
            }
        });
    }

    #[test]
    #[tracing_test::traced_test]
    fn no_warning_for_uninspected_or_missing() {
        let mut doc = toml::Table::new();
        doc.insert("デバッグ".into(), toml::Value::Boolean(false));
        doc.insert("debug".into(), toml::Value::String("x".into()));
        assert_eq!(save_switch(&doc), Some(false));
        assert_eq!(save_switch(&toml::Table::new()), None);
        assert!(!logs_contain("save variable is not a boolean"));
    }

    #[test]
    fn parse_env_bool_values() {
        for v in ["1", "true", "YES", " on "] {
            assert_eq!(parse_env_bool(v), Some(true), "{v}");
        }
        for v in ["0", "false", "No", "off", ""] {
            assert_eq!(parse_env_bool(v), Some(false), "{v}");
        }
        assert_eq!(parse_env_bool("maybe"), None);
    }
}
