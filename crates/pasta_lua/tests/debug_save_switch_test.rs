//! 保存変数 `デバッグ`／`debug` で開発用の働きを切り替える結合テスト。
//!
//! 一時フォルダにゴーストを置いて読み込み、判断と pasta.log の判断の行を確かめる。
//! `PASTA_DEBUG`・`PASTA_LOG` は `common` の `#[ctor]` が消す。

mod common;

use std::path::Path;

use common::create_temp_with_pasta;
use pasta_lua::GlobalLoggerRegistry;
use pasta_lua::loader::PastaLoader;

const ENABLED_LINE: &str = "Debug enabled: development events are allowed";
const DISABLED_LINE: &str = "Debug disabled: development events are refused";

/// 読み込んで判断と pasta.log を返す。
fn load_dir(dir: &Path) -> (bool, String) {
    pasta_lua::init_tracing();
    let runtime = PastaLoader::load(dir).expect("ghost must load");
    let enabled = runtime.debug_enabled();
    // ランタイムと登録簿の両方が手放すとログがフラッシュされる。
    drop(runtime);
    GlobalLoggerRegistry::instance().unregister(dir);
    let log = std::fs::read_to_string(dir.join("profile/pasta/logs/pasta.log")).unwrap();
    (enabled, log)
}

fn ghost(pasta_toml: &str) -> tempfile::TempDir {
    let temp = create_temp_with_pasta(
        "＊テスト
  ゴースト：「こんにちは」
",
    );
    std::fs::write(temp.path().join("pasta.toml"), pasta_toml).unwrap();
    temp
}

/// `pasta.toml` と `profile/pasta/save/save.toml` を置いて読み込み、判断と pasta.log を返す。
fn load_with_save(pasta_toml: &str, save_toml: &str) -> (bool, String) {
    let temp = ghost(pasta_toml);
    write_save(temp.path(), save_toml);
    load_dir(temp.path())
}

fn write_save(dir: &Path, save_toml: &str) {
    let save = dir.join("profile/pasta/save/save.toml");
    std::fs::create_dir_all(save.parent().unwrap()).unwrap();
    std::fs::write(save, save_toml).unwrap();
}

fn count(log: &str, line: &str) -> usize {
    log.lines().filter(|l| l.contains(line)).count()
}

#[test]
fn save_variable_true_enables_over_pasta_toml_false() {
    let (enabled, log) = load_with_save("[debug]\nenabled = false\n", "\"デバッグ\" = true\n");
    assert!(enabled, "save variable true must enable: {log}");
    assert_eq!(count(&log, ENABLED_LINE), 1, "{log}");
    assert_eq!(count(&log, DISABLED_LINE), 0, "{log}");
}

#[test]
fn save_variable_false_disables_over_pasta_toml_true() {
    let (enabled, log) = load_with_save("[debug]\nenabled = true\n", "\"デバッグ\" = false\n");
    assert!(!enabled, "save variable false must disable: {log}");
    assert_eq!(count(&log, DISABLED_LINE), 1, "{log}");
    assert_eq!(count(&log, ENABLED_LINE), 0, "{log}");
}

const WARN_LINE: &str = "save variable is not a boolean; ignored when deciding debug";

#[test]
fn save_variable_true_shows_scene_map_and_false_hides_it() {
    let on = ghost(
        "[debug]
enabled = false
",
    );
    write_save(
        on.path(),
        "\"デバッグ\" = true
",
    );
    let rt = PastaLoader::load(on.path()).unwrap();
    assert!(rt.debug_source_map().is_some());
    let off = ghost(
        "[debug]
enabled = true
",
    );
    write_save(
        off.path(),
        "\"デバッグ\" = false
",
    );
    let rt = PastaLoader::load(off.path()).unwrap();
    assert!(rt.debug_source_map().is_none());
    assert!(!rt.debug_enabled());
}

#[test]
fn non_boolean_japanese_name_warns_once_and_falls_to_debug_name() {
    let (enabled, log) = load_with_save(
        "",
        "\"デバッグ\" = \"true\"
debug = true
",
    );
    assert!(enabled, "{log}");
    let warns: Vec<_> = log.lines().filter(|l| l.contains(WARN_LINE)).collect();
    assert_eq!(warns.len(), 1, "{log}");
    assert!(warns[0].contains("＄＊デバッグ"), "{log}");
}

#[test]
fn unseen_debug_name_does_not_warn() {
    let (enabled, log) = load_with_save(
        "",
        "\"デバッグ\" = true
debug = \"x\"
",
    );
    assert!(enabled, "{log}");
    assert_eq!(count(&log, WARN_LINE), 0, "{log}");
}

#[test]
fn obfuscated_dat_is_read() {
    let temp = ghost(
        "[persistence]
obfuscate = true
",
    );
    let mut doc = toml::Table::new();
    doc.insert("デバッグ".into(), toml::Value::Boolean(true));
    pasta_lua::runtime::persistence::save_to_file(
        &doc,
        &temp.path().join("profile/pasta/save/save.dat"),
        true,
    )
    .unwrap();
    let (enabled, log) = load_dir(temp.path());
    assert!(enabled, "{log}");
    assert_eq!(count(&log, ENABLED_LINE), 1, "{log}");
}

#[test]
fn decision_line_appears_at_warn_log_level() {
    let (enabled, log) = load_with_save(
        "[logging]
level = \"warn\"
",
        "\"デバッグ\" = true
",
    );
    assert!(enabled, "{log}");
    assert_eq!(count(&log, ENABLED_LINE), 1, "{log}");
}

#[test]
fn missing_save_file_follows_pasta_toml() {
    for (toml_text, want, line) in [
        (
            "[debug]
enabled = true
",
            true,
            ENABLED_LINE,
        ),
        (
            "[debug]
enabled = false
",
            false,
            DISABLED_LINE,
        ),
        ("", false, DISABLED_LINE),
    ] {
        let temp = ghost(toml_text);
        let (enabled, log) = load_dir(temp.path());
        assert_eq!(enabled, want, "{log}");
        assert_eq!(count(&log, line), 1, "{log}");
    }
}

#[test]
fn corrupt_save_file_follows_pasta_toml_and_loads() {
    let (enabled, log) = load_with_save(
        "[debug]
enabled = true
",
        "not = [valid
",
    );
    assert!(enabled, "{log}");
    assert_eq!(count(&log, ENABLED_LINE), 1, "{log}");
    let (enabled, log) = load_with_save(
        "[debug]
enabled = false
",
        "not = [valid
",
    );
    assert!(!enabled, "{log}");
}

fn cache_files(dir: &Path) -> std::collections::BTreeMap<String, String> {
    let root = dir.join("profile/pasta/cache/lua");
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![root.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(&root).unwrap().display().to_string();
                // scene_dic.lua の生成時刻の行は読み込みごとに変わる。
                let text = std::fs::read_to_string(&p).unwrap();
                let body: Vec<_> = text
                    .lines()
                    .filter(|l| !l.starts_with("-- Generated at:"))
                    .collect();
                out.insert(
                    rel,
                    body.join(
                        "
",
                    ),
                );
            }
        }
    }
    out
}

#[test]
fn generated_lua_cache_is_identical_enabled_and_disabled() {
    let on = ghost("");
    write_save(
        on.path(),
        "\"デバッグ\" = true
",
    );
    let (enabled, _) = load_dir(on.path());
    assert!(enabled);
    let off = ghost("");
    write_save(
        off.path(),
        "\"デバッグ\" = false
",
    );
    let (enabled, _) = load_dir(off.path());
    assert!(!enabled);
    let (a, b) = (cache_files(on.path()), cache_files(off.path()));
    assert!(!a.is_empty());
    assert_eq!(a, b);
}
