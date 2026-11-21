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

/// `pasta.toml` と `profile/pasta/save/save.toml` を置いて読み込み、判断と pasta.log を返す。
fn load_with_save(pasta_toml: &str, save_toml: &str) -> (bool, String) {
    pasta_lua::init_tracing();
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let dir = temp.path();
    std::fs::write(dir.join("pasta.toml"), pasta_toml).unwrap();
    write_save(dir, save_toml);

    let runtime = PastaLoader::load(dir).expect("ghost must load");
    let enabled = runtime.debug_enabled();
    // ランタイムと登録簿の両方が手放すとログがフラッシュされる。
    drop(runtime);
    GlobalLoggerRegistry::instance().unregister(dir);
    let log = std::fs::read_to_string(dir.join("profile/pasta/logs/pasta.log")).unwrap();
    (enabled, log)
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
