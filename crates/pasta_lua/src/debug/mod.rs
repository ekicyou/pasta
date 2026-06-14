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
//! # Wiring
//!
//! [`enable`] is the fully wired entry point: when enabled it installs the VM
//! line hook, binds the DAP transport listener, and spawns the socket-bridge /
//! event-encoder threads, returning a [`DebugHandle`] that owns the teardown.
//! See [`enable`] and [`wiring`] for the thread topology.

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
// types::{…}` re-export below; do not re-import them here.

pub(crate) mod breakpoints;
pub(crate) mod dap;
pub(crate) mod hook;
pub(crate) mod inspect;
pub(crate) mod session;
pub(crate) mod transport;
pub(crate) mod wiring;
pub mod types;

// `.pasta`↔生成 `.lua` ソースマップの consumer 側モジュール。
// 本仕様 `pasta-source-map` で本番化（常時コンパイル）した（7.3）。
pub mod source_map;
pub use types::{
    Breakpoint, FrameInfo, LineEvent, ResolvedBreakpoint, Scope, SessionCommand, SessionEvent,
    SourceRef, StopReason, ThreadId, ThreadInfo, Variable,
};

/// Loopback host the DAP listener binds to when debugging is enabled.
///
/// Debugging is local-only by design; the address is never externally routable.
const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Source presentation mode for the debug session.
///
/// Selects whether stop positions, call stacks and breakpoints are presented in
/// `.pasta` coordinates (via the source map) or in the raw generated `.lua`
/// coordinates. The default is [`SourceMode::Pasta`] (requirements 6.1). This
/// field replaces the dead `source_map_slice: bool` reserve removed in task 3.1
/// (requirements 7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceMode {
    /// Present in `.pasta` coordinates via the source map. Default (6.1).
    #[default]
    Pasta,
    /// Present in the raw generated `.lua` coordinates (6.2).
    Lua,
}

impl SourceMode {
    /// Parse a case-insensitive string (`"pasta"` / `"lua"`, surrounding
    /// whitespace ignored) into a [`SourceMode`].
    ///
    /// Any other value falls back to the default [`SourceMode::Pasta`] and emits
    /// a warning (design "Error Categories": 不正な `sourcePresentation` 値 →
    /// 既定 `pasta` へフォールバック＋警告). This keeps an invalid env / file /
    /// attach value from breaking the session.
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "pasta" => SourceMode::Pasta,
            "lua" => SourceMode::Lua,
            other => {
                tracing::warn!(
                    value = other,
                    "invalid source presentation mode; falling back to default `pasta`"
                );
                SourceMode::default()
            }
        }
    }

    /// Encode a [`SourceMode`] as a `u8` for an [`AtomicU8`]-backed shared cell.
    fn as_u8(self) -> u8 {
        match self {
            SourceMode::Pasta => 0,
            SourceMode::Lua => 1,
        }
    }

    /// Decode a `u8` produced by [`as_u8`](Self::as_u8) back to a [`SourceMode`].
    /// Any unexpected value defaults to [`SourceMode::Pasta`] (6.1).
    fn from_u8(v: u8) -> Self {
        match v {
            1 => SourceMode::Lua,
            _ => SourceMode::Pasta,
        }
    }
}

/// A shared, interior-mutable EFFECTIVE present mode for one debug session
/// (task 5.5 / requirements 6.3).
///
/// The resolved [`DebugConfig::source_mode`] (env > file > 既定) is baked at
/// [`enable`] time, BEFORE the DAP `attach` request arrives. When that `attach`
/// request carries an explicit `sourcePresentation` argument (the HIGHEST
/// precedence, design 581: `attach > env > file > 既定`), the server must apply
/// it to the CURRENT session — switching BOTH the `.pasta` source RESOLVER
/// presentation (task 5.2) AND the `.pasta`-granular STEP granularity
/// (task 5.4). Those two consumers live on DIFFERENT threads — the resolver on
/// the socket-bridge thread (it owns the [`DapAdapter`]) and the stepper on the
/// VM thread (inside the line hook) — so the effective mode is shared here.
///
/// Mirrors the established [`BreakpointSet`](crate::debug::breakpoints::BreakpointSet)
/// pattern (a cheap `Arc` clone of settable-while-running shared state): the
/// socket-bridge thread WRITES the new mode when the `attach` arg is received,
/// and the VM-thread stepper READS it per line. An [`AtomicU8`] is sufficient
/// (the value is `Copy`, a single scalar, with no compound invariant) and needs
/// no lock on the hot per-line read path.
///
/// When the `attach` request carries NO `sourcePresentation`, the cell is left
/// at the [`enable`]-time resolved mode, so the env > file > default decision
/// stands (design 581: a client default must NOT override env/file).
#[derive(Clone, Debug)]
pub(crate) struct SharedSourceMode {
    inner: Arc<std::sync::atomic::AtomicU8>,
}

impl SharedSourceMode {
    /// Construct a shared cell initialised to `mode` (the [`enable`]-time
    /// resolved mode).
    pub(crate) fn new(mode: SourceMode) -> Self {
        Self {
            inner: Arc::new(std::sync::atomic::AtomicU8::new(mode.as_u8())),
        }
    }

    /// Read the current effective present mode (VM-thread stepper / resolver).
    pub(crate) fn get(&self) -> SourceMode {
        SourceMode::from_u8(self.inner.load(Ordering::SeqCst))
    }

    /// Write a new effective present mode (socket bridge, on `attach`).
    pub(crate) fn set(&self, mode: SourceMode) {
        self.inner.store(mode.as_u8(), Ordering::SeqCst);
    }
}

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
/// # Teardown (synchronous port release, bounded)
///
/// [`Drop`] sets the shared shutdown flag and then SYNCHRONOUSLY JOINS the
/// socket-bridge thread (task 3.1): the bridge observes the flag within one
/// `POLL_INTERVAL`, returns, and drops its by-value [`Transport`], whose own
/// `Drop` joins the `serve()` listener thread — so the listening port is
/// RELEASED before this `Drop` returns. This makes a SHIORI unload free the
/// fixed DAP port deterministically before the next reload re-binds it (R1.x /
/// R2.x). The join is bounded because every downstream blocking point is an
/// interruptible `POLL_INTERVAL` poll, so teardown cannot hang. The
/// event-encoder thread (which owns no socket/port) is left DETACHED — joining
/// it while this `Drop` still holds `terminate_tx` would deadlock. The backend
/// also winds down naturally when the VM thread finishes Lua execution (the
/// session's channel ends drop, closing the encoder) or the DAP client
/// disconnects (the transport closes the inbound channel).
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

        // (3) Synchronous teardown (task 3.1, R1.1/R1.2/R1.3/R2.1/R2.2): signal
        // the socket bridge to stop FIRST (it observes the flag within one
        // `POLL_INTERVAL`, returns, and drops the `Transport`), THEN JOIN it. The
        // flag MUST be set before the join, otherwise the join would wait on a
        // bridge that was never told to stop. Joining the socket bridge waits for
        // `run_socket_bridge` to return → the by-value `Transport` is dropped →
        // `Transport::drop` synchronously joins its `serve()` listener thread
        // (tasks 2.1-2.4) → the listening port is RELEASED before this `drop`
        // returns. The join is bounded: every blocking point downstream is an
        // interruptible `POLL_INTERVAL` poll, so this never hangs (a hang would
        // wedge the test; production teardown is watchdog-free by design).
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(h) = self.socket_handle.take() {
            // Synchronous JOIN: blocks until the bridge returns → Transport drop →
            // serve join → port freed. A panicked bridge yields `Err`; ignore it,
            // teardown still completed (the thread is no longer running).
            let _ = h.join();
        }

        // The event-encoder owns no socket / port, so joining it is unnecessary
        // for releasing the port. Keep it DETACHED: this `Drop` still holds
        // `terminate_tx` (a `Sender` clone of the encoder's `event_rx`), so the
        // encoder cannot observe channel disconnect and exit until AFTER this
        // method returns and drops `terminate_tx`; joining it here would deadlock.
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
///      [`hook::install`](crate::debug::hook::install) (engine-wide `jit.off()` +
///      a coroutine-crossing `EVERY_LINE` hook) — this is the VM-thread stop
///      core; inspect/step/continue are processed in its hook loop ON THIS
///      THREAD (the `mlua::Lua` never crosses a thread, R6 / `!Send`),
///   3. a [`Transport`] bound to `cfg.listen` (the OS-assigned port is readable
///      via [`DebugHandle::local_addr`] when `listen` uses port 0),
///   4. a shared [`DapAdapter`] and two bridge threads connecting the transport
///      to the session (see [`wiring`] for the thread topology).
///
/// # Thread topology (design "Architecture" / "System Flows")
///
/// One VM host thread (the caller, owns `mlua::Lua` and the session in the hook) +
/// one socket-bridge thread (sole [`Transport`] owner: multiplexes inbound
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
/// # Source map injection (task 4.2 — `pasta-source-map`)
///
/// `source_map` is the OPTIONAL immutable shared `.pasta`↔`.lua` map (design
/// "Architecture": `Arc<SourceMap>` 不変共有). Together with `cfg.source_mode`
/// (task 4.1) it is threaded to the three `.pasta` CONSUMERS — the DAP source
/// resolver (task 5.2), the breakpoint translator (task 5.3) and the stepper
/// (task 5.4) — via this injection path: `enable → wiring → DebugSession`
/// (design 548). The map+mode REACH those points only when BOTH a map is
/// supplied AND `cfg.source_mode == SourceMode::Pasta` (design 582, requirements
/// 6.1); for `None` or [`SourceMode::Lua`] every consumer keeps its existing
/// default `.lua` behavior byte-for-byte (requirements 6.2 / 7.2). This task
/// wires the SKELETON only — the consumer LOGIC is tasks 5.x.
///
/// # Errors
/// [`DebugError::Bind`] if the DAP listener fails to bind; [`DebugError::Vm`] if
/// the hook install fails (`mlua::Error` is stringified at the boundary, it is
/// `!Send`). The disabled path never errors.
pub fn enable(
    lua: &mlua::Lua,
    cfg: &DebugConfig,
    source_map: Option<Arc<source_map::SourceMap>>,
) -> Result<Option<DebugHandle>, DebugError> {
    if !cfg.enabled {
        // Zero-cost disabled path (R5.2 / R5.3 / R5.5): no hook, no port, no
        // thread, no std_debug exposure. Leave `lua` untouched. The `source_map`
        // (if any) is simply dropped here — the disabled gate never consumes it.
        return Ok(None);
    }

    // Effective present-mode cell (task 5.5 / requirement 6.3): initialise the
    // SHARED, interior-mutable mode from the resolved `cfg.source_mode` (env >
    // file > 既定). The socket bridge flips it when a DAP `attach`
    // `sourcePresentation` arrives (highest precedence, design 581); the resolver
    // (task 5.2) and the VM-thread stepper (task 5.4) both read it, so an `attach`
    // switches BOTH for this session. One clone goes to the wiring, one to the
    // session.
    let shared_mode = SharedSourceMode::new(cfg.source_mode);

    // Gating (design 582, requirements 6.1 / 6.2 / 6.3): the `.pasta` consumers
    // (resolver / BP translator / stepper) are reached only when a map is supplied
    // AND the EFFECTIVE mode is `SourceMode::Pasta`. The mode part is now decided
    // at CONSUMPTION time (`pasta_active()` reads the shared cell) rather than
    // frozen here, because a DAP `attach` `sourcePresentation` can flip the mode
    // AFTER `enable` (it arrives later) — including Lua→Pasta, which needs the map
    // available. So the map is ALWAYS threaded when supplied; the per-consumption
    // `pasta_active()` gate (map present AND effective mode Pasta) keeps `None`/
    // `Lua`/no-attach paths byte-for-byte (7.2). Cloning the `Arc` is a refcount
    // bump (immutable shared map).
    let source_map_wiring = wiring::SourceMapWiring {
        source_map: source_map.clone(),
        source_mode: shared_mode.clone(),
    };

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
    // The session is the STEPPER consumer (task 5.4 / 5.5): thread the map plus
    // the SHARED effective mode into it. The map is threaded whenever supplied
    // (the `effective_mode == Pasta` gate is applied per line via
    // `resolve_current_pasta`), so a DAP `attach` Lua→Pasta flip can activate
    // `.pasta` stepping; `with_shared_mode` lets the socket-bridge `attach` flip
    // be observed here. With no map / `Lua` effective mode the session keeps its
    // default `.lua` granularity (7.2). The baked `source_mode` is the `attach`-
    // absent fallback (matches the env > file > 既定 resolution).
    let session = DebugSession::new(breakpoints.clone(), cmd_rx, event_tx)
        .with_source_map(source_map.clone(), cfg.source_mode)
        .with_shared_mode(Some(shared_mode.clone()));
    crate::debug::hook::install(lua, session).map_err(|e| DebugError::Vm(e.to_string()))?;

    // (4) I/O side: bind the transport (None → no port; Some → bind + accept one
    // client). A bind failure surfaces as DebugError::Bind (R3.1 / R5.5). The
    // bound addr is read NOW and stored in the handle, because the transport is
    // moved into the socket-bridge thread (it is `!Sync`, single-owner).
    let transport = Transport::start(cfg.listen).map_err(|e| {
        // 2.1/2.3 (failure warn): name the attempted bind addr + io cause, then
        // propagate `DebugError::Bind` unchanged. `cfg.listen` is `Option`, so
        // bind it (the enabled gate guarantees `Some` — R5.5 only materialises a
        // listen addr when enabled) before applying `%` (Display).
        let Some(listen) = cfg.listen else {
            unreachable!("enabled => cfg.listen is Some (R5.5)")
        };
        tracing::warn!(addr = %listen, error = %e, "debug transport bind failed");
        e
    })?;
    let local_addr = transport.local_addr();

    // 1.1/1.3/1.4/1.5 (success info): one line carrying the REAL bound loopback
    // addr (`local_addr()`'s `Some`, defensively matched). On port 0 this is the
    // OS-assigned port read back from the transport.
    if let Some(addr) = local_addr {
        tracing::info!(addr = %addr, "debug backend listening");
    }

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
        // The socket bridge owns the DapAdapter (source RESOLVER attach point,
        // task 5.2) and applies setBreakpoints (BP TRANSLATION attach point, task
        // 5.3): deliver the gated map+mode there too (task 4.2 plumbing).
        let source_map_wiring = source_map_wiring.clone();
        std::thread::spawn(move || {
            wiring::run_socket_bridge(
                transport,
                adapter,
                breakpoints,
                cmd_tx,
                out_rx,
                shutdown,
                source_map_wiring,
            );
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
#[path = "debug_mod_tests.rs"]
mod debug_mod_tests;
