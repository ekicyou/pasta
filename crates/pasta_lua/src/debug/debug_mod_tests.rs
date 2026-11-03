use super::*;

// --- Resolution: pure, deterministic (no global env, no Lua VM) ---

#[test]
fn disabled_by_default_no_inputs() {
    let cfg = DebugConfig::resolve(None, None, None, None, None, None, None, None);
    assert!(!cfg.enabled, "default must be disabled");
    assert!(cfg.listen.is_none(), "disabled => no listen address (R5.5)");
}

#[test]
fn disabled_when_file_enabled_false() {
    let file = DebugFileConfig {
        enabled: false,
        port: 9276,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), None, None, None, None, None, None, None);
    assert!(!cfg.enabled);
    assert!(cfg.listen.is_none());
}

#[test]
fn enabled_via_file_default_port() {
    let file = DebugFileConfig {
        enabled: true,
        port: 9276,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), None, None, None, None, None, None, None);
    assert!(cfg.enabled);
    assert_eq!(
        cfg.listen,
        Some("127.0.0.1:9276".parse().unwrap()),
        "enabled => listen 127.0.0.1:<port> (default 9276)"
    );
}

#[test]
fn enabled_via_env_when_no_file() {
    let cfg = DebugConfig::resolve(None, Some(true), None, None, None, None, None, None);
    assert!(cfg.enabled);
    assert_eq!(cfg.listen, Some("127.0.0.1:9276".parse().unwrap()));
}

#[test]
fn file_port_overrides_default() {
    let file = DebugFileConfig {
        enabled: true,
        port: 5000,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), None, None, None, None, None, None, None);
    assert_eq!(cfg.listen, Some("127.0.0.1:5000".parse().unwrap()));
}

#[test]
fn env_port_overrides_file_port() {
    let file = DebugFileConfig {
        enabled: true,
        port: 5000,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), None, Some(7000), None, None, None, None, None);
    assert_eq!(
        cfg.listen,
        Some("127.0.0.1:7000".parse().unwrap()),
        "PASTA_DEBUG_PORT overrides [debug] port"
    );
}

#[test]
fn env_enabled_overrides_file_disabled() {
    let file = DebugFileConfig {
        enabled: false,
        port: 9276,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), Some(true), None, None, None, None, None, None);
    assert!(
        cfg.enabled,
        "PASTA_DEBUG truthy overrides [debug] enabled=false"
    );
    assert_eq!(cfg.listen, Some("127.0.0.1:9276".parse().unwrap()));
}

#[test]
fn env_disabled_overrides_file_enabled() {
    let file = DebugFileConfig {
        enabled: true,
        port: 9276,
        ..Default::default()
    };
    let cfg = DebugConfig::resolve(Some(&file), Some(false), None, None, None, None, None, None);
    assert!(
        !cfg.enabled,
        "explicit PASTA_DEBUG=false overrides [debug] enabled=true"
    );
    assert!(cfg.listen.is_none());
}

#[test]
fn env_port_only_without_enable_stays_disabled() {
    // Setting a port but never enabling must NOT open anything.
    let cfg = DebugConfig::resolve(None, None, Some(7000), None, None, None, None, None);
    assert!(!cfg.enabled);
    assert!(cfg.listen.is_none());
}

#[test]
fn parse_truthy_env_values() {
    for v in ["1", "true", "TRUE", "yes", "on", "  on  "] {
        assert_eq!(parse_env_bool(v), Some(true), "{v:?} should be truthy");
    }
    for v in ["0", "false", "no", "off", ""] {
        assert_eq!(parse_env_bool(v), Some(false), "{v:?} should be falsy");
    }
    assert_eq!(parse_env_bool("garbage"), None);
}

// --- file_source_mode: invalid [debug] present_as tolerated (6.1/6.3) ---

#[test]
fn from_file_invalid_present_as_falls_back_to_pasta() {
    // An invalid pasta.toml `present_as` value must not break resolution:
    // it parses back to the default `.pasta` (design Error line 615).
    let file = DebugFileConfig {
        present_as: Some("garbage".to_string()),
        ..Default::default()
    };
    let cfg = DebugConfig::from_file(Some(&file));
    assert_eq!(
        cfg.source_mode,
        SourceMode::Pasta,
        "invalid present_as tolerated → default .pasta"
    );
}

// --- DebugFileConfig serde defaults ---

#[test]
fn file_config_defaults() {
    let parsed: DebugFileConfig = toml::from_str("").unwrap();
    assert!(!parsed.enabled, "default enabled=false");
    assert_eq!(parsed.port, 9276, "default port=9276");
}

#[test]
fn file_config_parses_section() {
    let parsed: DebugFileConfig = toml::from_str("enabled = true\nport = 1234").unwrap();
    assert!(parsed.enabled);
    assert_eq!(parsed.port, 1234);
}

// --- SourceMode: default + string parse (6.1, design Error line 615) ---

#[test]
fn source_mode_default_is_pasta() {
    // 6.1: 既定の提示モードは `.pasta`。
    assert_eq!(SourceMode::default(), SourceMode::Pasta);
}

#[test]
fn source_mode_parse_case_insensitive() {
    assert_eq!(SourceMode::parse("pasta"), SourceMode::Pasta);
    assert_eq!(SourceMode::parse("lua"), SourceMode::Lua);
    assert_eq!(SourceMode::parse("PASTA"), SourceMode::Pasta);
    assert_eq!(SourceMode::parse("Lua"), SourceMode::Lua);
    assert_eq!(SourceMode::parse("  pasta  "), SourceMode::Pasta);
}

#[test]
fn source_mode_parse_invalid_falls_back_to_pasta() {
    // design Error line 615: 不正な値 → 既定 `pasta` へフォールバック。
    assert_eq!(SourceMode::parse("garbage"), SourceMode::Pasta);
    assert_eq!(SourceMode::parse(""), SourceMode::Pasta);
}

// --- DebugConfig: new field defaults (6.1, 3.2) ---
//
// resolve signature:
//   resolve(file, env_enabled, env_port,
//           env_source_mode, env_sidecar,
//           file_source_mode, file_sidecar, attach_source_mode)

#[test]
fn default_source_mode_is_pasta_and_sidecar_false() {
    // 6.1: 既定 source_mode == Pasta; 3.2: 既定 sidecar == false.
    let cfg = DebugConfig::resolve(None, None, None, None, None, None, None, None);
    assert_eq!(
        cfg.source_mode,
        SourceMode::Pasta,
        "6.1: default present mode is .pasta"
    );
    assert!(!cfg.source_map_sidecar, "3.2: sidecar disabled by default");

    // The struct Default mirrors the no-input resolve (zero-cost config).
    let d = DebugConfig::default();
    assert_eq!(d.source_mode, SourceMode::Pasta);
    assert!(!d.source_map_sidecar);
}

// --- DebugConfig::resolve: source_mode precedence attach > env > file > default ---

#[test]
fn source_mode_file_overrides_default() {
    // file Lua, no env, no attach => Lua (file beats default Pasta).
    let cfg = DebugConfig::resolve(
        None,
        None,
        None,
        None,                  // env source_mode
        None,                  // env sidecar
        Some(SourceMode::Lua), // file source_mode
        None,                  // file sidecar
        None,                  // attach source_mode
    );
    assert_eq!(cfg.source_mode, SourceMode::Lua, "file overrides default");
}

#[test]
fn source_mode_env_overrides_file() {
    // file Pasta, env Lua => Lua (env beats file), matching enabled/port env>file.
    let cfg = DebugConfig::resolve(
        None,
        None,
        None,
        Some(SourceMode::Lua),   // env source_mode
        None,                    // env sidecar
        Some(SourceMode::Pasta), // file source_mode
        None,                    // file sidecar
        None,                    // attach
    );
    assert_eq!(cfg.source_mode, SourceMode::Lua, "env overrides file");
}

#[test]
fn source_mode_attach_overrides_env() {
    // attach Lua beats env Pasta beats file Pasta (DAP attach 引数 > env > file).
    let cfg = DebugConfig::resolve(
        None,
        None,
        None,
        Some(SourceMode::Pasta), // env
        None,                    // env sidecar
        Some(SourceMode::Pasta), // file
        None,                    // file sidecar
        Some(SourceMode::Lua),   // attach
    );
    assert_eq!(cfg.source_mode, SourceMode::Lua, "attach overrides env");
}

// --- DebugConfig::resolve: source_map_sidecar precedence env > file > default ---

#[test]
fn sidecar_file_overrides_default() {
    // file_sidecar=true, no env => true (file beats default false).
    let cfg = DebugConfig::resolve(None, None, None, None, None, None, Some(true), None);
    assert!(
        cfg.source_map_sidecar,
        "file sidecar=true overrides default false"
    );
}

#[test]
fn sidecar_env_overrides_file() {
    // env false beats file true; and env true beats file false.
    // env false, file none:
    let off = DebugConfig::resolve(None, None, None, None, Some(false), None, None, None)
        .source_map_sidecar;
    // file true alone:
    let file_on = DebugConfig::resolve(None, None, None, None, None, None, Some(true), None)
        .source_map_sidecar;
    // env false over file true:
    let env_off_over_file_on =
        DebugConfig::resolve(None, None, None, None, Some(false), None, Some(true), None)
            .source_map_sidecar;
    // env true over file false:
    let env_on_over_file_off =
        DebugConfig::resolve(None, None, None, None, Some(true), None, Some(false), None)
            .source_map_sidecar;
    assert!(!off);
    assert!(file_on);
    assert!(
        !env_off_over_file_on,
        "PASTA_DEBUG_SOURCE_MAP_SIDECAR=false overrides file true"
    );
    assert!(
        env_on_over_file_off,
        "PASTA_DEBUG_SOURCE_MAP_SIDECAR=true overrides file false"
    );
}
