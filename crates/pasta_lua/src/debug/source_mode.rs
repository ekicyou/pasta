//! Source presentation mode value types for the debug session.
//!
//! Split out of `debug/mod.rs` (task 4.5): the [`SourceMode`] value enum and the
//! shared, interior-mutable [`SharedSourceMode`] cell. The parent `mod.rs` keeps
//! the `pub use source_mode::{SourceMode, SharedSourceMode};` re-export so the
//! public surface is byte-identical.

use std::sync::Arc;
use std::sync::atomic::Ordering;

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
    pub(super) fn as_u8(self) -> u8 {
        match self {
            SourceMode::Pasta => 0,
            SourceMode::Lua => 1,
        }
    }

    /// Decode a `u8` produced by [`as_u8`](Self::as_u8) back to a [`SourceMode`].
    /// Any unexpected value defaults to [`SourceMode::Pasta`] (6.1).
    pub(super) fn from_u8(v: u8) -> Self {
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
