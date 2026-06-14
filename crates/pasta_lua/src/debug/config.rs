//! Runtime debug configuration resolution and the zero-cost enablement gate.
//!
//! Split out of `debug/mod.rs` (task 4.5): [`DebugConfig`] (with `resolve` /
//! `from_env` / `from_file`), the `[debug]` env/file mapping helpers, and the
//! loopback [`LOOPBACK`] const that only [`DebugConfig::resolve`] uses. The
//! parent `mod.rs` keeps `pub use config::DebugConfig;` so the public surface is
//! byte-identical.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use super::source_mode::SourceMode;
use crate::loader::{DebugFileConfig, default_debug_port};

/// Loopback host the DAP listener binds to when debugging is enabled.
///
/// Debugging is local-only by design; the address is never externally routable.
const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Runtime-resolved debug configuration and zero-cost gate.
///
/// Produced by [`DebugConfig::resolve`] (pure) or the [`DebugConfig::from_env`]
/// / [`DebugConfig::from_file`] wrappers. When `enabled` is `false`, `listen`
/// is guaranteed to be `None` (no port is opened — R5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugConfig {
    /// Whether the debug backend is active.
    pub enabled: bool,

    /// Listen address for the DAP transport. `None` when disabled (R5.5).
    pub listen: Option<SocketAddr>,

    /// Source presentation mode. Default [`SourceMode::Pasta`] (6.1).
    ///
    /// Composed in [`resolve`](Self::resolve) with precedence
    /// `DAP attach 引数 > env > pasta.toml [debug] > 既定 Pasta`.
    pub source_mode: SourceMode,

    /// Whether to additionally write the on-disk `.lua.map` sidecar (3.2).
    /// Default `false`; the in-memory source map is always the primary path.
    ///
    /// Composed in [`resolve`](Self::resolve) with precedence `env > file >
    /// default` (same convention as `enabled`/`port`).
    pub source_map_sidecar: bool,
}

impl Default for DebugConfig {
    /// The disabled, zero-cost configuration (R5.2 / R5.5).
    ///
    /// Equivalent to `DebugConfig::resolve(None, None, None, None, None, None,
    /// None, None)`: `enabled = false`, `listen = None` (no port is ever opened),
    /// `source_mode = Pasta` (6.1), `source_map_sidecar = false` (3.2). This lets
    /// every existing `RuntimeConfig` constructor stay zero-cost by deriving
    /// `RuntimeConfig::debug` from this default.
    fn default() -> Self {
        Self {
            enabled: false,
            listen: None,
            source_mode: SourceMode::Pasta,
            source_map_sidecar: false,
        }
    }
}

impl DebugConfig {
    /// Resolve a [`DebugConfig`] from explicit inputs (pure, deterministic).
    ///
    /// This is the single resolution point and is unit-testable without a Lua
    /// VM or process-global environment. Wrappers ([`from_env`](Self::from_env),
    /// [`from_file`](Self::from_file)) feed it real inputs.
    ///
    /// # Arguments
    /// * `file` - parsed `[debug]` section, if present in pasta.toml.
    /// * `env_enabled` - `PASTA_DEBUG` parsed to a bool, if the var was set.
    /// * `env_port` - `PASTA_DEBUG_PORT` parsed to a port, if the var was set.
    /// * `env_source_mode` - `PASTA_DEBUG_SOURCE_MODE` parsed to a [`SourceMode`],
    ///   if the var was set.
    /// * `env_sidecar` - `PASTA_DEBUG_SOURCE_MAP_SIDECAR` parsed to a bool, if the
    ///   var was set.
    /// * `file_source_mode` - the `[debug]` source-presentation mode, if present.
    /// * `file_sidecar` - the `[debug]` sidecar flag, if present.
    ///   The two `file_*` mode/sidecar values are supplied separately (not via
    ///   [`DebugFileConfig`]) because the pasta.toml loading of these fields lands
    ///   in task 4.4 (`loader/config.rs`); `resolve` only needs to ACCEPT them.
    /// * `attach_source_mode` - the DAP `attach` `sourcePresentation` override,
    ///   set ONLY when the client explicitly specifies it (task 5.5 plumbing).
    ///   A client default is NOT passed here, so it never overrides env/file.
    ///
    /// # Precedence
    /// - `enabled` / `port`: `env` (when `Some`) beats `file` beats default
    ///   (`enabled = false`, `port = 9276`). (unchanged)
    /// - `source_mode`: `attach_source_mode` beats `env_source_mode` beats
    ///   `file_source_mode` beats 既定 [`SourceMode::Pasta`] (6.1). The DAP attach
    ///   引数 wins, then env, then the pasta.toml `[debug]` value, consistent with
    ///   the env>file convention above.
    /// - `source_map_sidecar`: `env_sidecar` beats `file_sidecar` beats default
    ///   `false` (3.2; same env>file convention).
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        file: Option<&DebugFileConfig>,
        env_enabled: Option<bool>,
        env_port: Option<u16>,
        env_source_mode: Option<SourceMode>,
        env_sidecar: Option<bool>,
        file_source_mode: Option<SourceMode>,
        file_sidecar: Option<bool>,
        attach_source_mode: Option<SourceMode>,
    ) -> Self {
        let file_enabled = file.map(|f| f.enabled).unwrap_or(false);
        let file_port = file.map(|f| f.port).unwrap_or_else(default_debug_port);

        let enabled = env_enabled.unwrap_or(file_enabled);
        let port = env_port.unwrap_or(file_port);

        // R5.5: only materialise a listen address when actually enabled.
        let listen = if enabled {
            Some(SocketAddr::V4(SocketAddrV4::new(LOOPBACK, port)))
        } else {
            None
        };

        // 6.1: source presentation mode. Precedence attach > env > file > Pasta.
        let source_mode = attach_source_mode
            .or(env_source_mode)
            .or(file_source_mode)
            .unwrap_or_default();

        // 3.2: disk sidecar output. Precedence env > file > false (A1 convention).
        let source_map_sidecar = env_sidecar.or(file_sidecar).unwrap_or(false);

        Self {
            enabled,
            listen,
            source_mode,
            source_map_sidecar,
        }
    }

    /// Resolve from a file config plus the process environment.
    ///
    /// Reads `PASTA_DEBUG` / `PASTA_DEBUG_PORT` / `PASTA_DEBUG_SOURCE_MODE` /
    /// `PASTA_DEBUG_SOURCE_MAP_SIDECAR` via [`std::env`]. Prefer
    /// [`resolve`](Self::resolve) in tests to avoid global-env races.
    ///
    /// The pasta.toml `[debug]` source-mode (`present_as`) / sidecar
    /// (`source_map_sidecar`) values are loaded by `loader/config.rs` (task 4.4)
    /// and SUPPLIED here from `file`: they are fed to [`resolve`](Self::resolve)
    /// as the `file_*` inputs so the precedence becomes `env > file > 既定`
    /// (requirements 6.3 / 3.2). No DAP attach override is available at this
    /// layer (task 5.5).
    pub fn from_env(file: Option<&DebugFileConfig>) -> Self {
        let env_enabled = std::env::var("PASTA_DEBUG")
            .ok()
            .and_then(|v| parse_env_bool(&v));
        let env_port = std::env::var("PASTA_DEBUG_PORT")
            .ok()
            .and_then(|v| v.trim().parse::<u16>().ok());
        let env_source_mode = std::env::var("PASTA_DEBUG_SOURCE_MODE")
            .ok()
            .map(|v| SourceMode::parse(&v));
        let env_sidecar = std::env::var("PASTA_DEBUG_SOURCE_MAP_SIDECAR")
            .ok()
            .and_then(|v| parse_env_bool(&v));
        Self::resolve(
            file,
            env_enabled,
            env_port,
            env_source_mode,
            env_sidecar,
            file_source_mode(file), // pasta.toml [debug] present_as (task 4.4)
            file_sidecar(file),     // pasta.toml [debug] source_map_sidecar (task 4.4)
            None,                   // no DAP attach override at this layer (task 5.5)
        )
    }

    /// Resolve from a file config only, ignoring the environment.
    ///
    /// Equivalent to feeding [`resolve`](Self::resolve) the file's
    /// `present_as`→[`SourceMode`] and `source_map_sidecar` as the `file_*`
    /// inputs with no env / attach override (precedence `file > 既定`).
    pub fn from_file(file: Option<&DebugFileConfig>) -> Self {
        Self::resolve(
            file,
            None,
            None,
            None,
            None,
            file_source_mode(file),
            file_sidecar(file),
            None,
        )
    }
}

/// Map a pasta.toml `[debug]` `present_as` string to a [`SourceMode`], if the
/// key was present. `None` (key omitted) lets env/default decide; an invalid
/// value is tolerated and parsed back to the default `.pasta` via
/// [`SourceMode::parse`] (requirements 6.1 / 6.3).
fn file_source_mode(file: Option<&DebugFileConfig>) -> Option<SourceMode> {
    file.and_then(|f| f.present_as.as_deref())
        .map(SourceMode::parse)
}

/// The pasta.toml `[debug]` `source_map_sidecar` flag, supplied to `resolve`
/// only when a file config is present (3.2). When no file config is present this
/// is `None` so the env/default decides.
fn file_sidecar(file: Option<&DebugFileConfig>) -> Option<bool> {
    file.map(|f| f.source_map_sidecar)
}

/// Parse an environment variable value into a boolean.
///
/// Truthy: `1`, `true`, `yes`, `on` (case-insensitive, surrounding whitespace
/// ignored). Falsy: `0`, `false`, `no`, `off`, and the empty string. Any other
/// value yields `None` (treated as "not specified" by callers).
pub(super) fn parse_env_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" | "" => Some(false),
        _ => None,
    }
}
