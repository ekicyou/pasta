//! Phase A: loader/config.rs のインラインテストを外部化
//!
//! from_str: #[cfg(test)] 除去 → pub に昇格
//! default_lua_search_paths, default_log_file_path: pub に昇格

use pasta_lua::loader::{
    LoaderConfig, LoggingConfig, LuaConfig, PastaConfig, PersistenceConfig, default_log_file_path,
    default_lua_search_paths,
};

// ============================================================================
// Task 4.4 — pasta.toml [debug] 提示モード/サイドカー供給 (requirements 6.3 / 3.2)
//
// 設計参照: design.md "SourceMode / DebugConfig"（提示モード合成 precedence
// attach > env > pasta.toml [debug] > 既定 Pasta）、"SidecarWriter Trigger"
// （source_map_sidecar を pasta.toml [debug] + env で合成、env>file>既定）。
//
// このタスクは pasta.toml `[debug]` の `present_as` / `source_map_sidecar` を
// パースし、`DebugConfig::resolve` の **ファイル由来値** として供給する配線を
// 担う（task 4.1 はそれらを None で受けていた）。
// ============================================================================

use pasta_lua::debug::{DebugConfig, SourceMode};
use pasta_lua::loader::DebugFileConfig;

/// 6.3: pasta.toml `[debug]` の `present_as = "lua"` がパースされ、env/attach 上書きが
/// 無い時に `DebugConfig::from_env` 経由で `source_mode == Lua` を解決する。
#[test]
fn debug_file_present_as_lua_resolves_to_lua_source_mode() {
    // 環境変数の汚染を避けるため、ここでは env を読まない経路（from_file）も併用する。
    let toml_str = r#"
[debug]
enabled = true
present_as = "lua"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let file: DebugFileConfig = config.debug().expect("[debug] section should parse");

    // ファイル由来の提示モードが resolve に供給され、Lua になる（env/attach 無し）。
    let resolved = DebugConfig::from_file(Some(&file));
    assert_eq!(
        resolved.source_mode,
        SourceMode::Lua,
        "6.3: pasta.toml [debug] present_as=\"lua\" must yield source_mode == Lua"
    );
}

/// 6.3: `present_as` 省略時（[debug] はあるが present_as 無し）は既定 `.pasta`。
#[test]
fn debug_file_present_as_default_is_pasta() {
    let toml_str = r#"
[debug]
enabled = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let file: DebugFileConfig = config.debug().expect("[debug] section should parse");

    let resolved = DebugConfig::from_file(Some(&file));
    assert_eq!(
        resolved.source_mode,
        SourceMode::Pasta,
        "6.1/6.3: omitted present_as must default to .pasta"
    );
}

/// 3.2: pasta.toml `[debug]` の `source_map_sidecar = true` がパースされ、resolve で
/// `source_map_sidecar == true` になる（env 無し）。既定は false。
#[test]
fn debug_file_source_map_sidecar_flag_resolves() {
    let on = PastaConfig::from_str(
        r#"
[debug]
enabled = true
source_map_sidecar = true
"#,
    )
    .unwrap();
    let file_on: DebugFileConfig = on.debug().expect("[debug] parse");
    assert!(
        DebugConfig::from_file(Some(&file_on)).source_map_sidecar,
        "3.2: source_map_sidecar=true must yield true"
    );

    let off = PastaConfig::from_str(
        r#"
[debug]
enabled = true
"#,
    )
    .unwrap();
    let file_off: DebugFileConfig = off.debug().expect("[debug] parse");
    assert!(
        !DebugConfig::from_file(Some(&file_off)).source_map_sidecar,
        "3.2: omitted source_map_sidecar must default to false"
    );
}

/// `DebugFileConfig` の serde 既定: present_as 省略 → None 相当（既定 Pasta へ）、
/// source_map_sidecar 省略 → false。既存 enabled/port の既定も維持。
#[test]
fn debug_file_config_serde_defaults_for_new_fields() {
    // 空 [debug]（全フィールド既定）。
    let parsed: DebugFileConfig = toml::from_str("").unwrap();
    assert!(!parsed.enabled);
    assert_eq!(parsed.port, 9276);
    // 新フィールドの既定: from_file 経由で Pasta / sidecar=false に解決される。
    let resolved = DebugConfig::from_file(Some(&parsed));
    assert_eq!(resolved.source_mode, SourceMode::Pasta);
    assert!(!resolved.source_map_sidecar);
}

#[test]
fn test_default_config() {
    let config = PastaConfig::default();
    assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
    assert_eq!(
        config.loader.lua_search_paths,
        vec![
            "profile/pasta/save/lua",
            "scripts",
            "profile/pasta/pasta_scripts",
            "profile/pasta/cache/lua",
            "scriptlibs"
        ]
    );
    assert_eq!(
        config.loader.transpiled_output_dir,
        "profile/pasta/cache/lua"
    );
    assert!(config.loader.debug_mode);
    assert!(config.custom_fields.is_empty());
}

#[test]
fn test_deserialize_minimal_config() {
    let toml_str = r#"
[loader]
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
    assert!(config.loader.debug_mode);
}

#[test]
fn test_deserialize_full_config() {
    let toml_str = r#"
[loader]
pasta_patterns = ["custom/*.pasta"]
lua_search_paths = ["lib", "src"]
transpiled_output_dir = "cache"
debug_mode = false
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert_eq!(config.loader.pasta_patterns, vec!["custom/*.pasta"]);
    assert_eq!(config.loader.lua_search_paths, vec!["lib", "src"]);
    assert_eq!(config.loader.transpiled_output_dir, "cache");
    assert!(!config.loader.debug_mode);
}

#[test]
fn test_deserialize_with_custom_fields() {
    // Note: In TOML, keys after [section] belong to that section.
    // To have top-level keys with a section, put them before the section.
    let toml_str = r#"
ghost_name = "TestGhost"
version = "1.0.0"

[loader]
debug_mode = true

[user_data]
key1 = "value1"
key2 = 42
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(config.loader.debug_mode);

    // Check custom fields (loader should be excluded)
    assert!(!config.custom_fields.contains_key("loader"));
    assert_eq!(
        config.custom_fields.get("ghost_name"),
        Some(&toml::Value::String("TestGhost".to_string()))
    );
    assert_eq!(
        config.custom_fields.get("version"),
        Some(&toml::Value::String("1.0.0".to_string()))
    );

    // Check nested table
    let user_data = config.custom_fields.get("user_data").unwrap();
    if let toml::Value::Table(t) = user_data {
        assert_eq!(
            t.get("key1"),
            Some(&toml::Value::String("value1".to_string()))
        );
        assert_eq!(t.get("key2"), Some(&toml::Value::Integer(42)));
    } else {
        panic!("Expected table for user_data");
    }
}

#[test]
fn test_deserialize_without_loader_section() {
    let toml_str = r#"
ghost_name = "NoLoaderGhost"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    // Should use defaults for loader
    assert_eq!(config.loader.pasta_patterns, vec!["dic/*/*.pasta"]);
    assert_eq!(
        config.custom_fields.get("ghost_name"),
        Some(&toml::Value::String("NoLoaderGhost".to_string()))
    );
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();
    assert_eq!(config.file_path, "profile/pasta/logs/pasta.log");
    assert_eq!(config.rotation_days, 7);
    assert_eq!(config.level, "info");
    assert!(config.filter.is_none());
}

#[test]
fn test_logging_config_to_filter_directive_with_filter() {
    // filter優先: filterが設定されている場合はfilterを返す
    let config = LoggingConfig {
        file_path: default_log_file_path(),
        rotation_days: 7,
        level: "info".to_string(),
        filter: Some("debug,pasta_shiori=trace".to_string()),
    };
    assert_eq!(config.to_filter_directive(), "debug,pasta_shiori=trace");
}

#[test]
fn test_logging_config_to_filter_directive_with_level_only() {
    // filterなし: levelを返す
    let config = LoggingConfig {
        file_path: default_log_file_path(),
        rotation_days: 7,
        level: "warn".to_string(),
        filter: None,
    };
    assert_eq!(config.to_filter_directive(), "warn");
}

#[test]
fn test_logging_config_to_filter_directive_default() {
    // デフォルト: "info"
    let config = LoggingConfig::default();
    assert_eq!(config.to_filter_directive(), "info");
}

#[test]
fn test_logging_config_from_toml() {
    let toml_str = r#"
[loader]
debug_mode = true

[logging]
file_path = "profile/custom/logs/my.log"
rotation_days = 14
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let logging = config.logging().expect("logging section should exist");
    assert_eq!(logging.file_path, "profile/custom/logs/my.log");
    assert_eq!(logging.rotation_days, 14);
    assert_eq!(logging.level, "info"); // default
    assert!(logging.filter.is_none());
}

#[test]
fn test_logging_config_from_toml_with_level() {
    let toml_str = r#"
[logging]
level = "info"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let logging = config.logging().expect("logging section should exist");
    assert_eq!(logging.level, "info");
    assert!(logging.filter.is_none());
    assert_eq!(logging.to_filter_directive(), "info");
}

#[test]
fn test_logging_config_from_toml_with_filter() {
    let toml_str = r#"
[logging]
level = "info"
filter = "debug,pasta_shiori=warn,pasta_lua::runtime=trace"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let logging = config.logging().expect("logging section should exist");
    assert_eq!(logging.level, "info");
    assert_eq!(
        logging.filter,
        Some("debug,pasta_shiori=warn,pasta_lua::runtime=trace".to_string())
    );
    // filter優先
    assert_eq!(
        logging.to_filter_directive(),
        "debug,pasta_shiori=warn,pasta_lua::runtime=trace"
    );
}

#[test]
fn test_logging_config_defaults_when_partial() {
    let toml_str = r#"
[logging]
file_path = "profile/pasta/logs/custom.log"
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let logging = config.logging().expect("logging section should exist");
    assert_eq!(logging.file_path, "profile/pasta/logs/custom.log");
    assert_eq!(logging.rotation_days, 7); // default
}

#[test]
fn test_logging_config_none_when_missing() {
    let toml_str = r#"
[loader]
debug_mode = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(config.logging().is_none());
}

#[test]
fn test_persistence_config_default() {
    let config = PersistenceConfig::default();
    assert!(!config.obfuscate);
    assert_eq!(config.file_path, "profile/pasta/save/save.json");
    assert!(!config.debug_mode);
}

#[test]
fn test_persistence_config_from_toml() {
    let toml_str = r#"
[persistence]
obfuscate = true
file_path = "profile/custom/save.dat"
debug_mode = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let persistence = config
        .persistence()
        .expect("persistence section should exist");
    assert!(persistence.obfuscate);
    assert_eq!(persistence.file_path, "profile/custom/save.dat");
    assert!(persistence.debug_mode);
}

#[test]
fn test_persistence_config_defaults_when_partial() {
    let toml_str = r#"
[persistence]
obfuscate = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let persistence = config
        .persistence()
        .expect("persistence section should exist");
    assert!(persistence.obfuscate);
    assert_eq!(persistence.file_path, "profile/pasta/save/save.json"); // default
    assert!(!persistence.debug_mode); // default
}

#[test]
fn test_persistence_config_none_when_missing() {
    let toml_str = r#"
[loader]
debug_mode = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(config.persistence().is_none());
}

#[test]
fn test_persistence_effective_file_path() {
    // Non-obfuscated: keep original path
    let config = PersistenceConfig::default();
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.json");

    // Obfuscated with .json: change to .dat
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/save.json".to_string(),
        debug_mode: false,
    };
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");

    // Obfuscated with .dat: keep as-is
    let config = PersistenceConfig {
        obfuscate: true,
        file_path: "profile/pasta/save/save.dat".to_string(),
        debug_mode: false,
    };
    assert_eq!(config.effective_file_path(), "profile/pasta/save/save.dat");
}

// ========================================
// LuaConfig tests
// ========================================

#[test]
fn test_lua_config_default() {
    let config = LuaConfig::default();
    assert_eq!(
        config.libs,
        vec!["std_all", "assertions", "testing", "regex", "json", "yaml"]
    );
}

#[test]
fn test_lua_config_from_toml() {
    let toml_str = r#"
[lua]
libs = ["std_string", "std_table", "testing"]
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let lua = config.lua().expect("lua section should exist");
    assert_eq!(lua.libs, vec!["std_string", "std_table", "testing"]);
}

#[test]
fn test_lua_config_with_subtraction() {
    let toml_str = r#"
[lua]
libs = ["std_all", "-std_debug", "testing"]
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let lua = config.lua().expect("lua section should exist");
    assert_eq!(lua.libs, vec!["std_all", "-std_debug", "testing"]);
}

#[test]
fn test_lua_config_empty_array() {
    let toml_str = r#"
[lua]
libs = []
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let lua = config.lua().expect("lua section should exist");
    assert!(lua.libs.is_empty());
}

#[test]
fn test_lua_config_defaults_when_libs_omitted() {
    let toml_str = r#"
[lua]
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    let lua = config.lua().expect("lua section should exist");
    // libs should use default when omitted
    assert_eq!(
        lua.libs,
        vec!["std_all", "assertions", "testing", "regex", "json", "yaml"]
    );
}

#[test]
fn test_lua_config_none_when_section_missing() {
    let toml_str = r#"
[loader]
debug_mode = true
"#;
    let config = PastaConfig::from_str(toml_str).unwrap();
    assert!(config.lua().is_none());
}

// ========================================
// Lua Search Path tests (lua-module-path-resolution spec)
// ========================================

#[test]
fn test_default_lua_search_paths_contains_user_scripts() {
    // Requirement 1.2: scripts should be at priority 2 (second position)
    let paths = default_lua_search_paths();
    assert_eq!(paths.len(), 5, "Should have 5 search paths");
    assert_eq!(
        paths,
        vec![
            "profile/pasta/save/lua",
            "scripts",
            "profile/pasta/pasta_scripts",
            "profile/pasta/cache/lua",
            "scriptlibs",
        ],
        "Search paths should be in correct priority order"
    );
}

#[test]
fn test_default_lua_search_paths_user_scripts_priority() {
    // Requirement 1.3: scripts (index 1) should come before profile/pasta/pasta_scripts (index 2)
    let paths = default_lua_search_paths();
    let scripts_pos = paths.iter().position(|p| p == "scripts");
    let pasta_scripts_pos = paths.iter().position(|p| p == "profile/pasta/pasta_scripts");
    assert!(scripts_pos.is_some(), "scripts should be in search paths");
    assert!(
        pasta_scripts_pos.is_some(),
        "profile/pasta/pasta_scripts should be in search paths"
    );
    assert!(
        scripts_pos.unwrap() < pasta_scripts_pos.unwrap(),
        "scripts should come before profile/pasta/pasta_scripts for override functionality"
    );
}

#[test]
fn test_loader_config_default_includes_user_scripts() {
    // Verify LoaderConfig::default() uses the new search paths
    let config = LoaderConfig::default();
    assert!(
        config.lua_search_paths.contains(&"scripts".to_string()),
        "Default LoaderConfig should include scripts"
    );
    assert!(
        config
            .lua_search_paths
            .contains(&"profile/pasta/pasta_scripts".to_string()),
        "Default LoaderConfig should include profile/pasta/pasta_scripts"
    );
}
