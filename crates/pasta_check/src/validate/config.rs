//! pasta.toml の [loader] と [scene] を pasta.dll と同じ厳しさで読む段（2.2・2.4・2.5・2.10）。

use std::collections::BTreeMap;

use pasta_core::{SceneAliasError, SceneAliasTable};
use serde::Deserialize;
use toml::Spanned;

use super::{Finding, FindingKind, TomlSource, msg};

/// 検査の残りの段が使う設定。誤りがあった部分は既定値になる
#[derive(Debug, Clone)]
pub(crate) struct GhostConfig {
    pub pasta_patterns: Vec<String>,
    pub aliases: SceneAliasTable,
}

impl Default for GhostConfig {
    fn default() -> Self {
        GhostConfig {
            pasta_patterns: vec!["dic/**/*.pasta".to_string()],
            aliases: SceneAliasTable::builtin_default(),
        }
    }
}

/// 設定と、設定の誤りを返す。誤りがあっても既定値で検査を続ける
pub(crate) fn read_config(src: &TomlSource) -> (GhostConfig, Vec<Finding>) {
    let mut config = GhostConfig::default();
    let text = match src {
        TomlSource::Text { text } => text,
        TomlSource::Missing => {
            let f = Finding::new(
                FindingKind::ConfigMissing,
                PATH,
                None,
                None,
                msg::config_missing(),
            );
            return (config, vec![f]);
        }
        TomlSource::Unreadable { reason } => {
            let f = Finding::new(
                FindingKind::FileUnreadable,
                PATH,
                None,
                None,
                msg::file_unreadable(reason),
            );
            return (config, vec![f]);
        }
    };

    // pasta.dll（PastaConfig::parse）と同じく、まず表として読む
    if let Err(e) = toml::from_str::<toml::Table>(text) {
        let (line, column) = position(text, &e);
        let f = Finding::new(
            FindingKind::ConfigSyntax,
            PATH,
            line,
            column,
            msg::config_syntax(e.message()),
        );
        return (config, vec![f]);
    }

    let mut findings = Vec::new();
    match read_loader(text) {
        Ok(Some(patterns)) => config.pasta_patterns = patterns,
        Ok(None) => {}
        Err(found) => findings.extend(found),
    }
    match read_aliases(text) {
        Ok(Some(aliases)) => config.aliases = aliases,
        Ok(None) => {}
        Err(found) => findings.extend(found),
    }
    (config, findings)
}

pub(super) use super::input::PASTA_TOML as PATH;

/// [loader] の 4 つのキー。pasta.dll の LoaderConfig と同じ型で読む（知らないキーは無視）
#[derive(Deserialize)]
struct LoaderProbe {
    loader: Option<BTreeMap<String, Spanned<toml::Value>>>,
}

/// 書いた pasta_patterns を返す（書いていなければ None）。型の誤りはキーごとに返す
fn read_loader(text: &str) -> Result<Option<Vec<String>>, Vec<Finding>> {
    let loader = match toml::from_str::<LoaderProbe>(text) {
        Ok(probe) => probe.loader,
        Err(e) => {
            let f = Finding::new(
                FindingKind::ConfigLoaderType,
                PATH,
                position(text, &e).0,
                None,
                msg::config_loader_not_table(e.message()),
            );
            return Err(vec![f]);
        }
    };
    let Some(loader) = loader else {
        return Ok(None);
    };

    let mut findings = Vec::new();
    let mut check = |key: &str, result: Result<(), toml::de::Error>| {
        if let Err(e) = result {
            let line = line_of(text, loader[key].span().start);
            findings.push(Finding::new(
                FindingKind::ConfigLoaderType,
                PATH,
                Some(line),
                None,
                msg::config_loader_type(key, e.message()),
            ));
        }
    };
    let mut patterns = None;
    for (key, value) in &loader {
        let value = value.get_ref().clone();
        match key.as_str() {
            "pasta_patterns" => check(key, value.try_into().map(|p| patterns = Some(p))),
            "lua_search_paths" => check(key, value.try_into::<Vec<String>>().map(drop)),
            "transpiled_output_dir" => check(key, value.try_into::<String>().map(drop)),
            "debug_mode" => check(key, value.try_into::<bool>().map(drop)),
            _ => {}
        }
    }
    if findings.is_empty() {
        Ok(patterns)
    } else {
        Err(findings)
    }
}

/// pasta.dll の SceneProbe と同じ形（`[scene]` の alias 以外のキーは無視する）
#[derive(Deserialize)]
struct SceneProbe {
    scene: Option<SceneSection>,
}

#[derive(Deserialize)]
struct SceneSection {
    alias: Option<BTreeMap<String, Vec<Spanned<String>>>>,
}

/// 書いた別名表を返す（`[scene.alias]` が無ければ None）。拒むときは該当する別名の行を全部返す
fn read_aliases(text: &str) -> Result<Option<SceneAliasTable>, Vec<Finding>> {
    let probe = toml::from_str::<SceneProbe>(text).map_err(|e| {
        vec![Finding::new(
            FindingKind::ConfigSceneAlias,
            PATH,
            position(text, &e).0,
            None,
            msg::config_scene_alias_type(e.message()),
        )]
    })?;
    let Some(map) = probe.scene.and_then(|s| s.alias) else {
        return Ok(None);
    };
    let entries = map
        .iter()
        .map(|(target, aliases)| (target.clone(), aliases.iter().map(|a| a.get_ref().clone())));
    let error = match SceneAliasTable::from_entries(entries) {
        Ok(table) => return Ok(Some(table)),
        Err(e) => e,
    };

    // pasta.dll と同じ選び方で、誤りに関わる別名を探す
    let hit = |target: &str, alias: &str| match &error {
        SceneAliasError::EmptyName { .. } => target.is_empty() || alias.is_empty(),
        SceneAliasError::DuplicateAlias { alias: dup, .. } => alias == dup,
        SceneAliasError::Chain { name } => alias == name,
    };
    let message = |target: &str| match &error {
        SceneAliasError::EmptyName { .. } => msg::config_scene_alias_empty(target),
        SceneAliasError::DuplicateAlias { alias, targets } => {
            msg::config_scene_alias_duplicate(alias, targets)
        }
        SceneAliasError::Chain { name } => msg::config_scene_alias_chain(name),
    };
    let mut findings: Vec<Finding> = map
        .iter()
        .flat_map(|(target, aliases)| aliases.iter().map(move |a| (target, a)))
        .filter(|(target, a)| hit(target, a.get_ref()))
        .map(|(target, a)| {
            let line = line_of(text, a.span().start);
            Finding::new(
                FindingKind::ConfigSceneAlias,
                PATH,
                Some(line),
                None,
                message(target),
            )
        })
        .collect();
    if findings.is_empty() {
        // 別名の無い置き換え先が空の名前のときは行が取れない
        let target = match &error {
            SceneAliasError::EmptyName { target } => target.as_str(),
            _ => "",
        };
        findings.push(Finding::new(
            FindingKind::ConfigSceneAlias,
            PATH,
            None,
            None,
            message(target),
        ));
    }
    Err(findings)
}

/// toml のエラーの位置（1 始まりの行・列。列は文字で数える）
fn position(text: &str, e: &toml::de::Error) -> (Option<u32>, Option<u32>) {
    match e.span() {
        Some(span) => {
            let start = span.start.min(text.len());
            let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
            let column = text[line_start..start].chars().count() + 1;
            (Some(line_of(text, start)), Some(column as u32))
        }
        None => (None, None),
    }
}

fn line_of(text: &str, offset: usize) -> u32 {
    text[..offset].matches('\n').count() as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> GhostConfig {
        read_config(&TomlSource::Text { text: text.into() }).0
    }

    #[test]
    fn reads_patterns_and_alias_table() {
        let c = read(
            "[loader]\npasta_patterns = [\"a/*.pasta\", \"b/*.pasta\"]\n[scene.alias]\nOnBoot = [\"起動\"]\n",
        );
        assert_eq!(c.pasta_patterns, vec!["a/*.pasta", "b/*.pasta"]);
        assert_eq!(c.aliases.resolve("起動"), Some("OnBoot"));
        assert_eq!(c.aliases.resolve("会話"), None);
    }

    #[test]
    fn empty_alias_table_is_used_as_written() {
        assert!(read("[scene.alias]\n").aliases.is_empty());
    }

    #[test]
    fn defaults_when_missing_or_not_written() {
        let d = GhostConfig::default();
        for c in [
            read_config(&TomlSource::Missing).0,
            read_config(&TomlSource::Unreadable { reason: "x".into() }).0,
            read(""),
            read("[loader]\ndebug_mode = false\n"),
        ] {
            assert_eq!(c.pasta_patterns, vec!["dic/**/*.pasta"]);
            assert_eq!(c.aliases, d.aliases);
        }
    }

    #[test]
    fn errors_fall_back_to_defaults() {
        for text in [
            "a = = 1\n[loader]\npasta_patterns = [\"x/*.pasta\"]\n",
            "[loader]\npasta_patterns = [\"x/*.pasta\"]\ndebug_mode = 1\n",
            "[scene.alias]\nOnTalk = [\"会話\"]\nOnBoot = [\"会話\"]\n",
        ] {
            let c = read(text);
            assert_eq!(c.pasta_patterns, vec!["dic/**/*.pasta"], "{text}");
            assert_eq!(c.aliases, SceneAliasTable::builtin_default(), "{text}");
        }
        // 別名表だけが誤りなら、パターンは書いたとおり
        let c =
            read("[loader]\npasta_patterns = [\"x/*.pasta\"]\n[scene.alias]\nOnTalk = [\"\"]\n");
        assert_eq!(c.pasta_patterns, vec!["x/*.pasta"]);
        assert_eq!(c.aliases, SceneAliasTable::builtin_default());
    }
}
