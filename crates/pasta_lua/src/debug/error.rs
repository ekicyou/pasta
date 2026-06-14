//! Error type surfaced by the debug backend.
//!
//! Split out of `debug/mod.rs` (task 4.5): the [`DebugError`] enum. The parent
//! `mod.rs` keeps `pub use error::DebugError;` so the public surface is
//! byte-identical.

use thiserror::Error;

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
