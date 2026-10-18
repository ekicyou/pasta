//! pasta.toml の [loader] と [scene] を pasta.dll と同じ厳しさで読む段（2.2・2.4・2.5・2.10）。

use std::collections::BTreeMap;

use pasta_core::{
    ActorFont, ActorFontError, FontInput, FontValue, FontValueProblem, SceneAliasError,
    SceneAliasTable,
};
use serde::Deserialize;
use toml::Spanned;
use toml::de::{DeTable, DeValue};

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
    // 装いはシーンの集まりに関わらないので GhostConfig は変えない
    findings.extend(read_actor_fonts(text));
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

/// pasta.dll（loader/config/actor_font.rs）と同じ取り出し方で `[actor.*.font]` を読み、
/// 誤りを 1 つずつ返す。行は属性の値の行（表でないときは font の値の行）
fn read_actor_fonts(text: &str) -> Vec<Finding> {
    let (Ok(table), Ok(doc)) = (toml::from_str::<toml::Table>(text), DeTable::parse(text)) else {
        return Vec::new();
    };
    let Some(toml::Value::Table(actors)) = table.get("actor") else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (name, actor) in actors {
        let Some(font) = actor.as_table().and_then(|t| t.get("font")) else {
            continue;
        };
        let spanned = entry(doc.get_ref(), "actor")
            .and_then(|a| a.get_ref().as_table())
            .and_then(|a| entry(a, name))
            .and_then(|a| a.get_ref().as_table())
            .and_then(|a| entry(a, "font"));
        let font_pos = spanned.map_or(text.len(), |f| f.span().start);
        let pos_of = |key: &str| {
            spanned
                .and_then(|f| f.get_ref().as_table())
                .and_then(|t| entry(t, key))
                .map_or(text.len(), |v| v.span().start)
        };
        let input = match font {
            toml::Value::Table(t) => {
                let mut entries: Vec<_> = t
                    .iter()
                    .map(|(k, v)| (pos_of(k), k.clone(), font_value(v)))
                    .collect();
                entries.sort_by_key(|(pos, ..)| *pos);
                FontInput::Table(entries.into_iter().map(|(_, k, v)| (k, v)).collect())
            }
            _ => FontInput::NotTable,
        };
        let Err(errors) = ActorFont::from_input(name, input) else {
            continue;
        };
        for e in errors {
            let (pos, message) = match &e {
                ActorFontError::NotTable { .. } => {
                    (font_pos, msg::config_actor_font_not_table(name))
                }
                ActorFontError::UnknownAttribute { attribute, .. } => (
                    pos_of(attribute),
                    msg::config_actor_font_unknown(name, attribute),
                ),
                ActorFontError::InvalidValue {
                    attribute, problem, ..
                } => (
                    pos_of(attribute),
                    match problem {
                        FontValueProblem::BreaksTag => {
                            msg::config_actor_font_breaks_tag(name, attribute)
                        }
                        FontValueProblem::Empty => {
                            msg::config_actor_font_value(name, attribute, "空の文字列")
                        }
                        FontValueProblem::WrongType { type_name } => {
                            msg::config_actor_font_value(name, attribute, type_label(type_name))
                        }
                    },
                ),
            };
            let line = line_of(text, pos.min(text.len()));
            findings.push(Finding::new(
                FindingKind::ConfigActorFont,
                PATH,
                Some(line),
                None,
                message,
            ));
        }
    }
    findings
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

/// toml の型の名前（`type_str`）を文言の言い方に
fn type_label(type_name: &str) -> &'static str {
    match type_name {
        "float" => "浮動小数点数",
        "array" => "配列",
        "table" => "表",
        "datetime" => "日時",
        _ => "ほかの型",
    }
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
