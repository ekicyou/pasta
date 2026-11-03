//! 古い環境変数を読まないことの結合テスト（debug-mcp-runtime 3.4）。
//!
//! `PASTA_DEBUG_PORT`・`PASTA_DEBUG_SOURCE_MODE`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR` を置いた
//! プロセスで有効なゴーストを読み込み、`.lua.map` が書かれず、待ち受けのログが出ないことを確かめる。
//! 環境を変えるので、ほかのテストと別のバイナリにする。`PASTA_DEBUG`・`PASTA_LOG` は
//! `common` の `#[ctor]` が消す。

mod common;

use std::path::Path;

use common::create_temp_with_pasta;
use pasta_lua::GlobalLoggerRegistry;
use pasta_lua::loader::PastaLoader;

/// 古い環境変数を置く。`#[ctor]` は `main` の前（スレッドが 1 つのとき）に走るので競合しない。
#[ctor::ctor(unsafe)]
fn set_old_debug_env() {
    unsafe {
        std::env::set_var("PASTA_DEBUG_PORT", "1");
        std::env::set_var("PASTA_DEBUG_SOURCE_MODE", "lua");
        std::env::set_var("PASTA_DEBUG_SOURCE_MAP_SIDECAR", "1");
    }
}

/// `dir` の下の `.lua.map` のファイルを集める。
fn lua_maps_under(dir: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            lua_maps_under(&path, found);
        } else if path.to_string_lossy().ends_with(".lua.map") {
            found.push(path.display().to_string());
        }
    }
}

#[test]
fn old_debug_env_vars_are_ignored() {
    assert_eq!(std::env::var("PASTA_DEBUG_PORT").as_deref(), Ok("1"));
    assert!(std::env::var("PASTA_DEBUG").is_err());

    pasta_lua::init_tracing();
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let dir = temp.path();
    std::fs::write(
        dir.join("pasta.toml"),
        "[loader]\ndebug_mode = true\n\n[debug]\nenabled = true\n",
    )
    .unwrap();

    let runtime = PastaLoader::load(dir).expect("ghost must load");
    assert!(
        runtime.debug_enabled(),
        "[debug] enabled = true must enable"
    );

    // ランタイムと登録簿の両方が手放すとログがフラッシュされる。
    drop(runtime);
    GlobalLoggerRegistry::instance().unregister(dir);
    let log = std::fs::read_to_string(dir.join("profile/pasta/logs/pasta.log")).unwrap();
    assert!(log.contains("Debug enabled"), "log must be real: {log}");
    assert!(!log.contains("listening"), "no listener log: {log}");

    let mut maps = Vec::new();
    lua_maps_under(dir, &mut maps);
    assert!(maps.is_empty(), "no .lua.map expected: {maps:?}");
}
