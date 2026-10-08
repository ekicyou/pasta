//! グローバルシーン名の別名表。
//!
//! 書いた名前が別名と完全一致したときだけ、置き換え先の名前を 1 段だけ返す。
//! 照合用の名前に揃える（`SceneRegistry::sanitize_name`）のは利用者側の後段で行う。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::error::SceneAliasError;

/// 別名 → 置き換え先 の表。`Default` は空の表。
///
/// 不変条件: 空文字列なし・別名は一意・置き換え先は別名として書かれていない。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneAliasTable {
    map: BTreeMap<String, String>,
}

impl SceneAliasTable {
    /// 空の表（別名を 1 件も持たない）。
    pub fn empty() -> Self {
        Self::default()
    }

    /// 内蔵の既定表: `OnTalk = ["会話"]` の 1 件だけ。
    pub fn builtin_default() -> Self {
        Self {
            map: BTreeMap::from([("会話".to_string(), "OnTalk".to_string())]),
        }
    }

    /// (置き換え先, 別名の列) の並びから作る。検証は空文字列 → 重複 → 連鎖の順。
    pub fn from_entries<I, A>(entries: I) -> Result<Self, SceneAliasError>
    where
        I: IntoIterator<Item = (String, A)>,
        A: IntoIterator<Item = String>,
    {
        let entries: Vec<(String, Vec<String>)> = entries
            .into_iter()
            .map(|(target, aliases)| (target, aliases.into_iter().collect()))
            .collect();

        for (target, aliases) in &entries {
            if target.is_empty() || aliases.iter().any(String::is_empty) {
                return Err(SceneAliasError::EmptyName {
                    target: target.clone(),
                });
            }
        }

        let mut map = BTreeMap::new();
        for (target, aliases) in &entries {
            for alias in aliases {
                if map.insert(alias.clone(), target.clone()).is_some() {
                    let targets = entries
                        .iter()
                        .flat_map(|(t, aliases)| {
                            aliases.iter().filter(|a| *a == alias).map(|_| t.clone())
                        })
                        .collect();
                    return Err(SceneAliasError::DuplicateAlias {
                        alias: alias.clone(),
                        targets,
                    });
                }
            }
        }

        if let Some((target, _)) = entries.iter().find(|(t, _)| map.contains_key(t)) {
            return Err(SceneAliasError::Chain {
                name: target.clone(),
            });
        }

        Ok(Self { map })
    }

    /// 書いた名前が別名に完全一致すれば置き換え先を返す。1 段だけ。
    pub fn resolve<'a>(&'a self, name: &str) -> Option<&'a str> {
        self.map.get(name).map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 置き換え先ごとに別名を並べた一覧（置き換え先・別名ともに辞書順）。
    pub fn entries(&self) -> Vec<(&str, Vec<&str>)> {
        let mut grouped: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (alias, target) in &self.map {
            grouped.entry(target).or_default().insert(alias);
        }
        grouped
            .into_iter()
            .map(|(target, aliases)| (target, aliases.into_iter().collect()))
            .collect()
    }

    /// 正準な指紋。先頭行 `scene_alias/1`、以下 `entries()` の順に 1 行ずつ、
    /// 名前は長さ接頭辞つき（`{len}:{name}`）。
    pub fn fingerprint(&self) -> String {
        let mut out = String::from("scene_alias/1\n");
        for (target, aliases) in self.entries() {
            out.push_str(&format!("{}:{}", target.len(), target));
            for alias in aliases {
                out.push_str(&format!(" {}:{}", alias.len(), alias));
            }
            out.push('\n');
        }
        out
    }
}

/// 例: `OnBoot <- 起動; OnTalk <- 会話, 雑談`。空なら `(empty)`。
impl fmt::Display for SceneAliasTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("(empty)");
        }
        let lines: Vec<String> = self
            .entries()
            .into_iter()
            .map(|(target, aliases)| format!("{target} <- {}", aliases.join(", ")))
            .collect();
        f.write_str(&lines.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SceneAliasError;

    fn table(entries: &[(&str, &[&str])]) -> Result<SceneAliasTable, SceneAliasError> {
        SceneAliasTable::from_entries(entries.iter().map(|(t, aliases)| {
            (
                t.to_string(),
                aliases.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            )
        }))
    }

    #[test]
    fn test_builtin_default_resolves_only_exact_alias() {
        let t = SceneAliasTable::builtin_default();
        assert_eq!(t.resolve("会話"), Some("OnTalk"));
        // 完全一致のみ: 前方一致・照合用の名前に揃えた後だけ等しい名前・置き換え先は置き換えない
        for name in [
            "会話・朝",
            "会話朝",
            "会話_",
            "会話 ",
            "OnTalk",
            "",
            "OnHour",
        ] {
            assert_eq!(t.resolve(name), None, "{name}");
        }
        // 既定表は OnTalk への 1 件だけ（OnHour への別名は含めない）
        assert_eq!(t.entries(), vec![("OnTalk", vec!["会話"])]);
    }

    #[test]
    fn test_empty_table() {
        let t = SceneAliasTable::empty();
        assert!(t.is_empty());
        assert_eq!(t, SceneAliasTable::default());
        assert_eq!(t.resolve("会話"), None);
        assert!(t.entries().is_empty());
        assert_eq!(t.to_string(), "(empty)");
        assert!(!SceneAliasTable::builtin_default().is_empty());
    }

    #[test]
    fn test_from_entries_author_table() {
        let t = table(&[("OnTalk", &["会話", "雑談"]), ("OnBoot", &["起動"])]).unwrap();
        assert_eq!(t.resolve("会話"), Some("OnTalk"));
        assert_eq!(t.resolve("雑談"), Some("OnTalk"));
        assert_eq!(t.resolve("起動"), Some("OnBoot"));
        assert_eq!(t.resolve("OnBoot"), None);
        // 照合用の名前に揃えた後だけ等しい名前は置き換えない
        let t2 = table(&[("OnTalk", &["会話・朝"])]).unwrap();
        assert_eq!(t2.resolve("会話・朝"), Some("OnTalk"));
        assert_eq!(t2.resolve("会話_朝"), None);
        assert_eq!(
            t.entries(),
            vec![("OnBoot", vec!["起動"]), ("OnTalk", vec!["会話", "雑談"])]
        );
        assert_eq!(t.to_string(), "OnBoot <- 起動; OnTalk <- 会話, 雑談");
    }

    #[test]
    fn test_from_entries_rejects_empty_name() {
        assert_eq!(
            table(&[("OnTalk", &["会話", ""])]),
            Err(SceneAliasError::EmptyName {
                target: "OnTalk".into()
            })
        );
        assert_eq!(
            table(&[("", &["会話"])]),
            Err(SceneAliasError::EmptyName { target: "".into() })
        );
    }

    #[test]
    fn test_from_entries_rejects_duplicate_alias() {
        // 配列内
        assert_eq!(
            table(&[("OnTalk", &["会話", "会話"])]),
            Err(SceneAliasError::DuplicateAlias {
                alias: "会話".into(),
                targets: vec!["OnTalk".into(), "OnTalk".into()],
            })
        );
        // 配列間
        assert_eq!(
            table(&[("OnTalk", &["会話"]), ("OnBoot", &["会話"])]),
            Err(SceneAliasError::DuplicateAlias {
                alias: "会話".into(),
                targets: vec!["OnTalk".into(), "OnBoot".into()],
            })
        );
    }

    #[test]
    fn test_from_entries_rejects_chain() {
        // 自己参照
        assert_eq!(
            table(&[("OnTalk", &["OnTalk"])]),
            Err(SceneAliasError::Chain {
                name: "OnTalk".into()
            })
        );
        // 置き換え先が別名としても書かれている
        assert_eq!(
            table(&[("OnTalk", &["会話"]), ("会話", &["雑談"])]),
            Err(SceneAliasError::Chain {
                name: "会話".into()
            })
        );
    }

    #[test]
    fn test_validation_order_empty_then_duplicate_then_chain() {
        assert!(matches!(
            table(&[("OnTalk", &["OnTalk", "会話", "会話", ""])]),
            Err(SceneAliasError::EmptyName { .. })
        ));
        assert!(matches!(
            table(&[("OnTalk", &["OnTalk", "会話", "会話"])]),
            Err(SceneAliasError::DuplicateAlias { .. })
        ));
    }

    #[test]
    fn test_fingerprint_is_stable_regardless_of_input_order() {
        let a = table(&[("OnTalk", &["会話", "雑談"]), ("OnBoot", &["起動"])]).unwrap();
        let b = table(&[("OnBoot", &["起動"]), ("OnTalk", &["雑談", "会話"])]).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_eq!(
            SceneAliasTable::builtin_default().fingerprint(),
            table(&[("OnTalk", &["会話"])]).unwrap().fingerprint()
        );
    }

    #[test]
    fn test_fingerprint_distinguishes_tables() {
        let default = SceneAliasTable::builtin_default().fingerprint();
        let empty = SceneAliasTable::empty().fingerprint();
        let two = table(&[("OnTalk", &["会話", "雑談"])])
            .unwrap()
            .fingerprint();
        assert_ne!(default, empty);
        assert_ne!(default, two);
        assert_ne!(empty, two);
        assert!(empty.starts_with("scene_alias/1"));
        // 長さ接頭辞で区切りの曖昧さを無くす
        let x = table(&[("A", &["B, C"])]).unwrap().fingerprint();
        let y = table(&[("A", &["B", "C"])]).unwrap().fingerprint();
        assert_ne!(x, y);
    }
}
