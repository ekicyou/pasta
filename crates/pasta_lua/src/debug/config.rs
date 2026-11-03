//! デバッグの有効の解決（`PASTA_DEBUG` ＞ `[debug] enabled` ＞ false）。
//!
//! 読む環境変数は `PASTA_DEBUG` だけ。呼ぶのはローダの Phase 5.5 の 1 回だけ。

use crate::loader::DebugFileConfig;

/// `PASTA_DEBUG`（設定されていて読める値なら）＞ `[debug] enabled` ＞ false。
pub fn resolve_enabled(file: Option<&DebugFileConfig>) -> bool {
    let env = std::env::var("PASTA_DEBUG")
        .ok()
        .and_then(|v| parse_env_bool(&v));
    pick(env, file.map(|f| f.enabled))
}

/// 純粋な部分。
fn pick(env: Option<bool>, file: Option<bool>) -> bool {
    env.or(file).unwrap_or(false)
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

    #[test]
    fn env_wins_over_file() {
        assert!(pick(Some(true), Some(false)));
        assert!(!pick(Some(false), Some(true)));
    }

    #[test]
    fn unreadable_env_is_same_as_unset() {
        assert_eq!(parse_env_bool("maybe"), None);
        assert!(pick(parse_env_bool("maybe"), Some(true)));
        assert!(!pick(parse_env_bool("maybe"), None));
    }

    #[test]
    fn file_decides_without_env() {
        assert!(pick(None, Some(true)));
        assert!(!pick(None, Some(false)));
    }

    #[test]
    fn default_is_false() {
        assert!(!pick(None, None));
    }

    #[test]
    fn parse_env_bool_values() {
        for v in ["1", "true", "YES", " on "] {
            assert_eq!(parse_env_bool(v), Some(true), "{v}");
        }
        for v in ["0", "false", "No", "off", ""] {
            assert_eq!(parse_env_bool(v), Some(false), "{v}");
        }
    }
}
