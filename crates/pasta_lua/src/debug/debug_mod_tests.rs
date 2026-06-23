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

// --- enable() gate ---

#[tracing_test::traced_test]
#[test]
fn enable_disabled_returns_none_and_no_trace() {
    let lua = mlua::Lua::new();
    let cfg = DebugConfig::resolve(None, None, None, None, None, None, None, None);
    let handle = enable(&lua, &cfg, None, None).expect("enable must not error when disabled");
    assert!(
        handle.is_none(),
        "disabled enable() returns Ok(None) (R5.2)"
    );

    // No std_debug exposure as a side effect of the disabled gate (R5.3).
    let debug_is_nil: bool = lua
        .load("return debug == nil")
        .eval()
        .expect("eval should succeed");
    assert!(debug_is_nil, "disabled gate must not expose std_debug");

    // 3.1 (無効時は無言): the disabled gate is the true zero-cost path — it
    // opens no port and binds nothing, so NEITHER the success `info` NOR the
    // failure `warn` must ever be emitted. Verifying both negatives here makes
    // the previously-unchecked "no_trace" name effective and completes the
    // output/no-output matrix (design Testing Strategy item 2).
    assert!(
        !logs_contain("debug backend listening"),
        "disabled enable() must emit no listening info (3.1)"
    );
    assert!(
        !logs_contain("debug transport bind failed"),
        "disabled enable() must emit no bind-failure warn (3.1)"
    );
}

#[test]
fn enable_enabled_returns_handle() {
    // ALL_SAFE VM so the hook's engine-wide `jit.off()` is callable (the
    // backend now installs a real hook). Port 0 → OS-assigned free loopback
    // port so the test never clashes with a fixed port across parallel runs.
    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let cfg = DebugConfig {
        enabled: true,
        listen: Some("127.0.0.1:0".parse().unwrap()),
        ..Default::default()
    };
    let handle = enable(&lua, &cfg, None, None).expect("enable must succeed when enabled");
    let handle = handle.expect("enabled enable() returns Ok(Some(DebugHandle))");

    // The handle echoes the config it was built from.
    assert_eq!(handle.config().listen, cfg.listen);

    // The transport bound a concrete loopback port (R3.1): readable back even
    // though the request used port 0.
    let addr = handle
        .local_addr()
        .expect("enabled handle must expose a bound addr (R3.1)");
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_ne!(addr.port(), 0, "OS must assign a concrete port");

    // The hook was installed: engine-wide jit.off() took effect (R5.2/R5.4).
    let jit_off: bool = lua
        .load("return (jit.status() == false)")
        .eval()
        .expect("jit.status() must be callable on an ALL_SAFE VM");
    assert!(
        jit_off,
        "enable must install the hook and apply engine-wide jit.off()"
    );

    // Dropping the handle tears the backend down without hanging.
    drop(handle);
    lua.remove_global_hook();
}

#[test]
fn unload_synchronously_frees_port_for_plain_rebind() {
    // R1.1/R1.2/R1.3/R2.1/R2.2: `DebugHandle::drop` must JOIN the socket
    // bridge (not detach), so the bridge returns → `Transport` drops →
    // `serve()` join releases the listening port BEFORE drop returns. We
    // prove the port is freed synchronously by immediately re-binding it with
    // a PLAIN `TcpListener::bind` (NO SO_REUSEADDR / NO socket2) — a masking
    // -aware rebind. With the pre-3.1 detached bridge, drop returns while the
    // bridge is still winding down asynchronously, so this plain rebind races
    // the still-open listener and fails with AddrInUse (10048 on Windows).
    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let cfg = DebugConfig {
        enabled: true,
        listen: Some("127.0.0.1:0".parse().unwrap()),
        ..Default::default()
    };
    let handle = enable(&lua, &cfg, None, None)
        .expect("enable must succeed when enabled")
        .expect("enabled enable() returns Ok(Some(DebugHandle))");

    // The OS-assigned loopback port the backend is listening on. NO client
    // connects — the serve listener is parked in its interruptible accept.
    let port = handle
        .local_addr()
        .expect("enabled handle must expose a bound addr (R3.1)")
        .port();
    assert_ne!(port, 0, "OS must assign a concrete port");

    // Synchronous teardown: drop must block until the bridge joins → Transport
    // drops → serve join → listener dropped → port released.
    drop(handle);

    // Immediately rebind the SAME port with a PLAIN listener (no SO_REUSEADDR).
    // This succeeds only if the previous listener was fully released by the
    // time `drop` returned — i.e. teardown was synchronous (R2.1/R2.2).
    let rebind = std::net::TcpListener::bind(("127.0.0.1", port));
    assert!(
        rebind.is_ok(),
        "plain rebind of port {port} must succeed after synchronous unload \
         (got {:?}); a failure proves the listener was still open (detached \
         teardown / AddrInUse 10048)",
        rebind.as_ref().err()
    );
    drop(rebind);

    lua.remove_global_hook();
}

#[test]
fn enable_bind_failure_surfaces_debug_error_bind() {
    // Occupy a concrete loopback port so the backend's bind must fail.
    let blocker = std::net::TcpListener::bind("127.0.0.1:0").expect("test listener must bind");
    let taken = blocker.local_addr().expect("bound addr");

    // ALL_SAFE VM: the hook (installed BEFORE the transport bind) needs jit.
    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let cfg = DebugConfig {
        enabled: true,
        listen: Some(taken),
        ..Default::default()
    };

    // R3.1 / R5.5: a bind failure is surfaced as DebugError::Bind, not a
    // panic and not a silently disabled backend.
    let err = enable(&lua, &cfg, None, None).expect_err("bind to an occupied port must fail");
    assert!(
        matches!(err, DebugError::Bind(_)),
        "expected DebugError::Bind, got: {err:?}"
    );
    assert!(
        format!("{err}").to_lowercase().contains("bind"),
        "Bind display names the failure: {err}"
    );

    // Clean up the hook the failed enable() left installed (the install
    // step precedes the bind; the test VM is dropped right after anyway).
    lua.remove_global_hook();
    drop(blocker);
}

// --- enable() startup logging (task 1.1 / requirements 1, 2, 3) ---

#[tracing_test::traced_test]
#[test]
fn enable_enabled_emits_listening_info() {
    // 1.1/1.3/1.4: enabling the backend emits a single `info` carrying the
    // real bound loopback addr. ALL_SAFE so the hook's `jit.off()` works;
    // port 0 → OS-assigned free loopback port (env-independent, no clash).
    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let cfg = DebugConfig {
        enabled: true,
        listen: Some("127.0.0.1:0".parse().unwrap()),
        ..Default::default()
    };
    let handle = enable(&lua, &cfg, None, None)
        .expect("enable must succeed when enabled")
        .expect("enabled enable() returns Some(handle)");

    // The fixed identifying message is emitted (1.3) at `info` (1.2)...
    assert!(
        logs_contain("debug backend listening"),
        "enable() must emit the listening info (1.1/1.3)"
    );
    // ...and carries the real bound loopback host:port (1.4/1.5).
    let port = handle.local_addr().expect("bound addr").port();
    assert!(
        logs_contain(&format!("addr=127.0.0.1:{port}")),
        "listening info must carry the real bound addr (1.4/1.5)"
    );

    drop(handle);
    lua.remove_global_hook();
}

#[tracing_test::traced_test]
#[test]
fn enable_bind_failure_emits_warn_and_no_info() {
    // 2.1/2.2/2.3: a bind failure emits a `warn` naming the attempted addr,
    // and NO listening `info` is emitted. Occupy a concrete loopback port so
    // the backend's bind must fail.
    let blocker = std::net::TcpListener::bind("127.0.0.1:0").expect("test listener must bind");
    let taken = blocker.local_addr().expect("bound addr");

    let lua =
        unsafe { mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()) };
    let cfg = DebugConfig {
        enabled: true,
        listen: Some(taken),
        ..Default::default()
    };

    let err = enable(&lua, &cfg, None, None).expect_err("bind to an occupied port must fail");
    assert!(
        matches!(err, DebugError::Bind(_)),
        "expected Bind, got {err:?}"
    );

    // 2.1/2.3: a warn names the bind failure...
    assert!(
        logs_contain("debug transport bind failed"),
        "bind failure must emit a warn (2.1)"
    );
    // 2.2: ...and no listening info is emitted on the failure path.
    assert!(
        !logs_contain("debug backend listening"),
        "no listening info must be emitted when the bind fails (2.2)"
    );

    lua.remove_global_hook();
    drop(blocker);
}

// --- SharedSourceMode: shared effective-mode cell (task 5.5 / 6.3) ---

#[test]
fn shared_source_mode_get_set_round_trip() {
    // The cell is initialised to the enable-time resolved mode...
    let cell = SharedSourceMode::new(SourceMode::Pasta);
    assert_eq!(cell.get(), SourceMode::Pasta);

    // ...a clone shares the SAME underlying cell (Arc semantics: the
    // socket-bridge writer and the VM-thread reader observe one value)...
    let reader = cell.clone();
    cell.set(SourceMode::Lua);
    assert_eq!(reader.get(), SourceMode::Lua, "clone observes the write");

    // ...and the flip is reversible (attach can switch Lua→Pasta too).
    cell.set(SourceMode::Pasta);
    assert_eq!(reader.get(), SourceMode::Pasta);
}

#[test]
fn source_mode_u8_codec_round_trips_and_defends_unknown() {
    // as_u8 / from_u8 round-trip for both variants.
    assert_eq!(
        SourceMode::from_u8(SourceMode::Pasta.as_u8()),
        SourceMode::Pasta
    );
    assert_eq!(
        SourceMode::from_u8(SourceMode::Lua.as_u8()),
        SourceMode::Lua
    );
    // Defensive default: any unknown byte decodes to Pasta (6.1).
    assert_eq!(SourceMode::from_u8(42), SourceMode::Pasta);
    assert_eq!(SourceMode::from_u8(u8::MAX), SourceMode::Pasta);
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

// --- DebugError discriminants ---

#[test]
fn debug_error_variants_display() {
    let bind = DebugError::Bind(std::io::Error::new(std::io::ErrorKind::AddrInUse, "in use"));
    assert!(format!("{bind}").to_lowercase().contains("bind"));
    let proto = DebugError::Protocol("bad frame".into());
    assert!(format!("{proto}").contains("bad frame"));
    let vm = DebugError::Vm("lua boom".into());
    assert!(format!("{vm}").contains("lua boom"));
    let disc = DebugError::Disconnected;
    assert!(!format!("{disc}").is_empty());
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
