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

pub use crate::loader::{DebugFileConfig, default_debug_port};

pub(crate) mod breakpoints;
pub(crate) mod dap;
pub(crate) mod hook;
pub(crate) mod inspect;
pub mod kick;
// 位置→シーン解決器（task 3.1）。DAP decode から呼ぶ口は task 4.1 で結線するため、
// それまで `resolve_and_kick`/`ResolveOutcome` は未消費（crate 内可視・dead_code 許可）。
pub(crate) mod playscene;
pub(crate) mod session;
pub(crate) mod transport;
pub mod types;
pub(crate) mod wiring;

mod config;
mod enable;
mod error;
mod handle;
mod source_mode;

// `.pasta`↔生成 `.lua` ソースマップの consumer 側モジュール。
// 本仕様 `pasta-source-map` で本番化（常時コンパイル）した（7.3）。
pub mod source_map;
pub use types::{
    Breakpoint, FrameInfo, LineEvent, ResolvedBreakpoint, Scope, SessionCommand, SessionEvent,
    SourceRef, StopReason, ThreadId, ThreadInfo, Variable,
};

pub use config::DebugConfig;
// `debug_mod_tests` (a descendant of `debug`) reaches the `pub(super)`
// `config::parse_env_bool` helper through this in-module re-export; previously the
// helper lived directly in this file. Test-only, so it stays gated.
#[cfg(test)]
use config::parse_env_bool;
pub use enable::enable;
pub use error::DebugError;
pub use handle::DebugHandle;
pub use kick::{KickRequest, KickSink};
pub(crate) use source_mode::SharedSourceMode;
pub use source_mode::SourceMode;

#[cfg(test)]
#[path = "debug_mod_tests.rs"]
mod debug_mod_tests;
