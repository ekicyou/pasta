//! Rust-hosted DAP debug backend for pasta_lua (SHIORI-independent).
//!
//! This module is the single entry point and enablement gate for the debug
//! backend. It is host-agnostic: it MUST NOT import `pasta_shiori` (R6).
//!
//! # Enablement gate (R5)
//!
//! Debugging is opt-in. [`DebugConfig`] is resolved from BOTH the pasta.toml
//! `[debug]` section ([`DebugFileConfig`]) AND the environment variables
//! `PASTA_DEBUG` / `PASTA_DEBUG_PORT`. When disabled, the backend is true
//! zero-cost: [`enable`] returns `Ok(None)`, installs no VM hook, opens no
//! network port, and never exposes Lua's `debug` / `std_debug` to scripts
//! (R5.2 / R5.3 / R5.5).
//!
//! # Resolution precedence
//!
//! - `enabled`: `PASTA_DEBUG` (if set) overrides `[debug] enabled` (default `false`).
//! - `port`: `PASTA_DEBUG_PORT` (if set) overrides `[debug] port` (default `9276`).
//! - The listener address is materialised only when `enabled` is true; otherwise
//!   `listen` is `None` so no port is ever opened.
//!
//! # Incremental wiring
//!
//! This is the foundation module. [`enable`]'s transport startup is wired in
//! task 4.1; the VM hook install is wired in task 1.3. For now, the enabled
//! path returns a skeleton [`DebugHandle`] without starting a listener or
//! installing a hook.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use serde_json::Value;
use thiserror::Error;

pub use crate::loader::{DebugFileConfig, default_debug_port};

use crate::debug::breakpoints::BreakpointSet;
use crate::debug::dap::DapAdapter;
use crate::debug::session::DebugSession;
use crate::debug::transport::Transport;
// `SessionCommand` / `SessionEvent` are already in scope via the `pub use
// types::{…}` re-export above; do not re-import them here.

pub(crate) mod breakpoints;
pub(crate) mod dap;
pub(crate) mod hook;
pub(crate) mod inspect;
pub(crate) mod session;
pub(crate) mod transport;
pub(crate) mod wiring;
pub mod types;

// R4 薄い実証スライス（feature `pasta-source-map-slice`・default 無効）。
// 無効時（既定）はこのモジュールを一切コンパイル/露出しない（R4.6 ゼロコスト）。
// 本番品質のソースマップは別仕様 `pasta-source-map` の担当。
#[cfg(feature = "pasta-source-map-slice")]
pub mod source_map;
pub use types::{
    Breakpoint, FrameInfo, LineEvent, ResolvedBreakpoint, Scope, SessionCommand, SessionEvent,
    SourceRef, StopReason, ThreadId, ThreadInfo, Variable,
};

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

    /// Whether the `.pasta` source-map proof-of-concept slice is active.
    ///
    /// This is ANDed with the `pasta-source-map-slice` build feature downstream
    /// (R4). It defaults to `false` and is never enabled by this foundation task.
    pub source_map_slice: bool,
}

impl Default for DebugConfig {
    /// The disabled, zero-cost configuration (R5.2 / R5.5).
    ///
    /// Equivalent to `DebugConfig::resolve(None, None, None)`: `enabled = false`,
    /// `listen = None` (no port is ever opened), `source_map_slice = false`. This
    /// lets every existing `RuntimeConfig` constructor stay zero-cost by deriving
    /// `RuntimeConfig::debug` from this default.
    fn default() -> Self {
        Self {
            enabled: false,
            listen: None,
            source_map_slice: false,
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
    ///
    /// # Precedence
    /// `env_enabled`/`env_port` (when `Some`) override the file values, which in
    /// turn override the defaults (`enabled = false`, `port = 9276`).
    pub fn resolve(
        file: Option<&DebugFileConfig>,
        env_enabled: Option<bool>,
        env_port: Option<u16>,
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

        Self {
            enabled,
            listen,
            source_map_slice: false,
        }
    }

    /// Resolve from a file config plus the process environment.
    ///
    /// Reads `PASTA_DEBUG` / `PASTA_DEBUG_PORT` via [`std::env`]. Prefer
    /// [`resolve`](Self::resolve) in tests to avoid global-env races.
    pub fn from_env(file: Option<&DebugFileConfig>) -> Self {
        let env_enabled = std::env::var("PASTA_DEBUG")
            .ok()
            .and_then(|v| parse_env_bool(&v));
        let env_port = std::env::var("PASTA_DEBUG_PORT")
            .ok()
            .and_then(|v| v.trim().parse::<u16>().ok());
        Self::resolve(file, env_enabled, env_port)
    }

    /// Resolve from a file config only, ignoring the environment.
    ///
    /// Convenience wrapper equivalent to `resolve(file, None, None)`.
    pub fn from_file(file: Option<&DebugFileConfig>) -> Self {
        Self::resolve(file, None, None)
    }
}

/// Parse an environment variable value into a boolean.
///
/// Truthy: `1`, `true`, `yes`, `on` (case-insensitive, surrounding whitespace
/// ignored). Falsy: `0`, `false`, `no`, `off`, and the empty string. Any other
/// value yields `None` (treated as "not specified" by callers).
fn parse_env_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" | "" => Some(false),
        _ => None,
    }
}

/// Errors surfaced by the debug backend.
///
/// `mlua::Error` is `!Send`; it is stringified at the boundary into [`Vm`]
/// (or carried as a `SessionEvent::Error` string in later tasks) so debug
/// state can cross the VM/transport thread boundary.
///
/// [`Vm`]: DebugError::Vm
#[derive(Error, Debug)]
pub enum DebugError {
    /// Failed to bind the DAP transport listener (R3.1 / R5.5).
    #[error("debug transport bind failed: {0}")]
    Bind(#[source] std::io::Error),

    /// DAP protocol framing or message error.
    #[error("debug protocol error: {0}")]
    Protocol(String),

    /// Lua VM / FFI error stringified at the boundary (`mlua::Error` is `!Send`).
    #[error("debug VM error: {0}")]
    Vm(String),

    /// The DAP client disconnected.
    #[error("debug client disconnected")]
    Disconnected,
}

/// Owner of the debug backend's bridge threads and shared state (task 4.1 full
/// wiring).
///
/// Constructed by [`enable`] when debugging is active. It holds:
/// - the bound listen address (read back from the transport so a caller using
///   port 0 can discover the OS-assigned port),
/// - a shared shutdown flag and the socket-bridge / event-encoder join handles.
///
/// The shared [`BreakpointSet`] is NOT held here: it is owned by the VM-thread
/// hook (reads) and the socket-bridge thread (writes — settable while running);
/// the handle needs no clone of it for task 4.1. (Runtime integration, task 4.2,
/// may surface it on the handle when it actually consumes it.)
///
/// The [`Transport`] itself is `!Sync` (it holds a `Receiver`), so it is owned
/// solely by the socket-bridge thread (see [`wiring`]); the handle never holds
/// it. The VM-thread line hook (installed by [`enable`] via
/// [`hook::install`](crate::debug::hook::install)) owns the [`DebugSession`] and
/// the session ends of the command/event channels; `mlua::Lua` never crosses a
/// thread (it is `!Send`).
///
/// # Teardown (no hang)
///
/// [`Drop`] is non-blocking: it sets the shared shutdown flag (the socket bridge
/// observes it within its poll interval, drops the [`Transport`], and exits) and
/// does NOT join the bridge threads. The backend also winds down naturally when
/// the VM thread finishes Lua execution (the session's channel ends drop,
/// closing the encoder) or the DAP client disconnects (the transport closes the
/// inbound channel).
pub struct DebugHandle {
    /// Resolved configuration this handle was created from.
    config: DebugConfig,
    /// The bound listen address (read from the transport at construction), or
    /// `None` when no listener was opened.
    local_addr: Option<SocketAddr>,
    /// Shared shutdown flag: setting it makes the socket bridge stop and drop
    /// the transport (non-blocking teardown).
    shutdown: Arc<AtomicBool>,
    /// Socket-bridge thread join handle (sole `Transport` owner: reads + writes).
    socket_handle: Option<JoinHandle<()>>,
    /// Event-encoder thread join handle (session events → DAP frames).
    encoder_handle: Option<JoinHandle<()>>,
    /// A clone of the session's event sender, used SOLELY to emit a final
    /// [`SessionEvent::Terminated`] on teardown (task 4.2).
    ///
    /// In the long-lived SHIORI runtime there is no per-request "execution end":
    /// the debuggee is the runtime ITSELF, so a per-request `exec()` return must
    /// NOT terminate the session (R3.5's "execution end" maps to RUNTIME
    /// TEARDOWN, not request end). On `Drop` we send `Terminated` through this
    /// clone BEFORE signalling shutdown, so the event-encoder thread can encode a
    /// DAP `terminated` frame for the socket bridge to flush to any connected
    /// client (best-effort; the encoder/bridge channels then wind down). The
    /// existing disconnect→terminated path (the session's `Disconnect` handler)
    /// remains for the client-initiated case.
    terminate_tx: mpsc::Sender<SessionEvent>,
}

impl std::fmt::Debug for DebugHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugHandle")
            .field("config", &self.config)
            .field("local_addr", &self.local_addr)
            .finish_non_exhaustive()
    }
}

impl DebugHandle {
    /// The resolved [`DebugConfig`] this handle owns.
    pub fn config(&self) -> &DebugConfig {
        &self.config
    }

    /// The bound DAP listen address, or `None` when no listener is active
    /// (R3.1: the OS-assigned port is read back from the transport so a caller
    /// using port 0 can discover the concrete bound port).
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }
}

impl Drop for DebugHandle {
    fn drop(&mut self) {
        // (1) Natural-end `terminated` (task 4.2): emit a final `Terminated`
        // session event so the event-encoder thread encodes a DAP `terminated`
        // frame and the socket bridge flushes it to any connected client. In the
        // long-lived SHIORI runtime the "execution end" of R3.5 is RUNTIME
        // TEARDOWN (this Drop), NOT a per-request `exec()` return — the debuggee
        // is the runtime itself, so per-request returns deliberately do not
        // terminate the session. A send failure (encoder already gone) is ignored.
        let _ = self.terminate_tx.send(SessionEvent::Terminated);

        // (2) Give the encoder + socket bridge a brief, BOUNDED window to encode
        // and flush that frame before we tear the bridge down. The socket bridge
        // polls/drains every `wiring::POLL_INTERVAL` (5ms); a few intervals is
        // enough for the `Terminated` frame to traverse encoder → out channel →
        // socket while staying effectively non-blocking for teardown.
        std::thread::sleep(std::time::Duration::from_millis(30));

        // (3) Non-blocking teardown: signal the socket bridge to stop (it observes
        // the flag within its poll interval, drops the Transport, and exits) and
        // detach the bridge threads (do NOT join in Drop — a never-resumed,
        // never-disconnected session must not hang this Drop call).
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = self.socket_handle.take();
        let _ = self.encoder_handle.take();
    }
}

/// Enable the debug backend for `lua` according to `cfg` (task 4.1 full wiring).
///
/// - When `cfg.enabled == false`: returns `Ok(None)`. No VM hook is installed,
///   no port is opened, no thread is spawned, and `std_debug` is NOT exposed to
///   scripts. This is the true zero-cost path (R5.2 / R5.3 / R5.5).
/// - When `cfg.enabled == true`: builds a FULLY WIRED backend and returns
///   `Ok(Some(DebugHandle))`:
///   1. a shared [`BreakpointSet`] (settable while the VM runs),
///   2. a [`DebugSession`] over the VM-thread ends of the command/event
///      channels, installed into the line hook via
///      [`hook::install`](crate::debug::hook::install) (engine-wide `jit.off()`
///      + a coroutine-crossing `EVERY_LINE` hook) — this is the VM-thread stop
///      core; inspect/step/continue are processed in its hook loop ON THIS
///      THREAD (the `mlua::Lua` never crosses a thread, R6 / `!Send`),
///   3. a [`Transport`] bound to `cfg.listen` (the OS-assigned port is readable
///      via [`DebugHandle::local_addr`] when `listen` uses port 0),
///   4. a shared [`DapAdapter`] and two bridge threads connecting the transport
///      to the session (see [`wiring`] for the thread topology).
///
/// # Thread topology (design "Architecture" / "System Flows")
///
/// One VM host thread (the caller, owns `mlua::Lua` and the session in the hook)
/// + one socket-bridge thread (sole [`Transport`] owner: multiplexes inbound
/// socket reads and outbound socket writes, since `Transport` is `!Sync` and
/// `mpsc` has no `select`) + one event-encoder thread (session events → DAP
/// frames). The socket bridge and encoder share the [`DapAdapter`] behind an
/// `Arc<Mutex<…>>` (its `seq` + correlation table). See [`wiring`] for the full
/// topology and the inbound-poll / outbound-frame-channel structure.
///
/// # SHIORI independence (R6)
///
/// This function does not import or reference `pasta_shiori`; any pasta host (or
/// a test harness) drives it directly.
///
/// # Preconditions
/// `lua` must already be constructed on the VM thread.
///
/// # Errors
/// [`DebugError::Bind`] if the DAP listener fails to bind; [`DebugError::Vm`] if
/// the hook install fails (`mlua::Error` is stringified at the boundary, it is
/// `!Send`). The disabled path never errors.
pub fn enable(lua: &mlua::Lua, cfg: &DebugConfig) -> Result<Option<DebugHandle>, DebugError> {
    if !cfg.enabled {
        // Zero-cost disabled path (R5.2 / R5.3 / R5.5): no hook, no port, no
        // thread, no std_debug exposure. Leave `lua` untouched.
        return Ok(None);
    }

    // (1) Shared breakpoint store: one clone goes to the VM-thread hook (reads),
    // one clone to the handle / socket bridge (writes — settable while running).
    let breakpoints = BreakpointSet::new();

    // (2) Channel seam (the ONLY thing that crosses the VM/transport boundary):
    //   cmd:   controller (socket bridge) → session (VM thread)
    //   event: session (VM thread) → controller (event encoder)
    let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
    let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

    // A clone of the event sender for the handle's teardown `terminated` (task
    // 4.2). The session keeps the original `event_tx`; this clone outlives the VM
    // thread (it lives on the handle) so `Drop` can emit `Terminated` even after
    // the VM has finished executing — the event-encoder thread stays alive as long
    // as ANY `Sender` (this clone) is held, then winds down when the handle drops.
    let terminate_tx = event_tx.clone();

    // (3) The stop core: a DebugSession over the VM-thread channel ends, plugged
    // into the line hook. install() applies engine-wide jit.off() and registers
    // the coroutine-crossing EVERY_LINE hook (R1.7 / R5.2). The session is moved
    // INTO the hook closure and thereafter lives on this VM thread inside `lua`.
    let session = DebugSession::new(breakpoints.clone(), cmd_rx, event_tx);
    crate::debug::hook::install(lua, session).map_err(|e| DebugError::Vm(e.to_string()))?;

    // (4) I/O side: bind the transport (None → no port; Some → bind + accept one
    // client). A bind failure surfaces as DebugError::Bind (R3.1 / R5.5). The
    // bound addr is read NOW and stored in the handle, because the transport is
    // moved into the socket-bridge thread (it is `!Sync`, single-owner).
    let transport = Transport::start(cfg.listen)?;
    let local_addr = transport.local_addr();

    // (5) Shared DAP adapter (seq counter + per-kind FIFO request correlation),
    // mutated by BOTH the socket bridge and the event encoder → Arc<Mutex<…>>.
    let adapter: wiring::SharedAdapter = Arc::new(Mutex::new(DapAdapter::new()));

    // (6) Encoded-frame channel: the event encoder produces DAP frames; the
    // socket bridge (sole Transport owner) writes them to the socket.
    let (out_tx, out_rx) = mpsc::channel::<Value>();

    // (7) Shared shutdown flag (non-blocking teardown via the handle's Drop).
    let shutdown = Arc::new(AtomicBool::new(false));

    // (8) Socket-bridge thread: owns the Transport; multiplexes inbound decode
    // (reply / apply setBreakpoints / forward stop-context commands) and
    // outbound frame writes.
    let socket_handle = {
        let adapter = Arc::clone(&adapter);
        let breakpoints = breakpoints.clone();
        let shutdown = Arc::clone(&shutdown);
        std::thread::spawn(move || {
            wiring::run_socket_bridge(transport, adapter, breakpoints, cmd_tx, out_rx, shutdown);
        })
    };

    // (9) Event-encoder thread: session events → DAP frames → out_tx.
    let encoder_handle = {
        let adapter = Arc::clone(&adapter);
        std::thread::spawn(move || {
            wiring::run_event_encoder(adapter, event_rx, out_tx);
        })
    };

    Ok(Some(DebugHandle {
        config: cfg.clone(),
        local_addr,
        shutdown,
        socket_handle: Some(socket_handle),
        encoder_handle: Some(encoder_handle),
        terminate_tx,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Resolution: pure, deterministic (no global env, no Lua VM) ---

    #[test]
    fn disabled_by_default_no_inputs() {
        let cfg = DebugConfig::resolve(None, None, None);
        assert!(!cfg.enabled, "default must be disabled");
        assert!(cfg.listen.is_none(), "disabled => no listen address (R5.5)");
        assert!(!cfg.source_map_slice);
    }

    #[test]
    fn disabled_when_file_enabled_false() {
        let file = DebugFileConfig {
            enabled: false,
            port: 9276,
        };
        let cfg = DebugConfig::resolve(Some(&file), None, None);
        assert!(!cfg.enabled);
        assert!(cfg.listen.is_none());
    }

    #[test]
    fn enabled_via_file_default_port() {
        let file = DebugFileConfig {
            enabled: true,
            port: 9276,
        };
        let cfg = DebugConfig::resolve(Some(&file), None, None);
        assert!(cfg.enabled);
        assert_eq!(
            cfg.listen,
            Some("127.0.0.1:9276".parse().unwrap()),
            "enabled => listen 127.0.0.1:<port> (default 9276)"
        );
    }

    #[test]
    fn enabled_via_env_when_no_file() {
        let cfg = DebugConfig::resolve(None, Some(true), None);
        assert!(cfg.enabled);
        assert_eq!(cfg.listen, Some("127.0.0.1:9276".parse().unwrap()));
    }

    #[test]
    fn file_port_overrides_default() {
        let file = DebugFileConfig {
            enabled: true,
            port: 5000,
        };
        let cfg = DebugConfig::resolve(Some(&file), None, None);
        assert_eq!(cfg.listen, Some("127.0.0.1:5000".parse().unwrap()));
    }

    #[test]
    fn env_port_overrides_file_port() {
        let file = DebugFileConfig {
            enabled: true,
            port: 5000,
        };
        let cfg = DebugConfig::resolve(Some(&file), None, Some(7000));
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
        };
        let cfg = DebugConfig::resolve(Some(&file), Some(true), None);
        assert!(cfg.enabled, "PASTA_DEBUG truthy overrides [debug] enabled=false");
        assert_eq!(cfg.listen, Some("127.0.0.1:9276".parse().unwrap()));
    }

    #[test]
    fn env_disabled_overrides_file_enabled() {
        let file = DebugFileConfig {
            enabled: true,
            port: 9276,
        };
        let cfg = DebugConfig::resolve(Some(&file), Some(false), None);
        assert!(!cfg.enabled, "explicit PASTA_DEBUG=false overrides [debug] enabled=true");
        assert!(cfg.listen.is_none());
    }

    #[test]
    fn env_port_only_without_enable_stays_disabled() {
        // Setting a port but never enabling must NOT open anything.
        let cfg = DebugConfig::resolve(None, None, Some(7000));
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

    #[test]
    fn enable_disabled_returns_none_and_no_trace() {
        let lua = mlua::Lua::new();
        let cfg = DebugConfig::resolve(None, None, None);
        let handle = enable(&lua, &cfg).expect("enable must not error when disabled");
        assert!(handle.is_none(), "disabled enable() returns Ok(None) (R5.2)");

        // No std_debug exposure as a side effect of the disabled gate (R5.3).
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(debug_is_nil, "disabled gate must not expose std_debug");
    }

    #[test]
    fn enable_enabled_returns_handle() {
        // ALL_SAFE VM so the hook's engine-wide `jit.off()` is callable (the
        // backend now installs a real hook). Port 0 → OS-assigned free loopback
        // port so the test never clashes with a fixed port across parallel runs.
        let lua = unsafe {
            mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
        };
        let cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            source_map_slice: false,
        };
        let handle = enable(&lua, &cfg).expect("enable must succeed when enabled");
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
        assert!(jit_off, "enable must install the hook and apply engine-wide jit.off()");

        // Dropping the handle tears the backend down without hanging.
        drop(handle);
        lua.remove_global_hook();
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
        let parsed: DebugFileConfig =
            toml::from_str("enabled = true\nport = 1234").unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.port, 1234);
    }

    // --- DebugError discriminants ---

    #[test]
    fn debug_error_variants_display() {
        let bind = DebugError::Bind(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "in use",
        ));
        assert!(format!("{bind}").to_lowercase().contains("bind"));
        let proto = DebugError::Protocol("bad frame".into());
        assert!(format!("{proto}").contains("bad frame"));
        let vm = DebugError::Vm("lua boom".into());
        assert!(format!("{vm}").contains("lua boom"));
        let disc = DebugError::Disconnected;
        assert!(!format!("{disc}").is_empty());
    }
}
