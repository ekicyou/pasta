//! Shared test-support helpers externalized from `dap.rs` (Task 2.2, pure
//! behavior-invariant move). The `request` builder was a cluster-shared helper
//! inside the original single `#[cfg(test)] mod tests`; it is gathered here and
//! `pub(super)`-exported so the sibling cluster modules can reach it via
//! `use super::dap_test_support::*;`. The same module path is preserved, so
//! private / `pub(crate)` reachability into production code is unchanged.
use super::*;

/// Build a minimal DAP request value.
pub(super) fn request(seq: u64, command: &str, arguments: Value) -> Value {
    json!({
        "seq": seq,
        "type": "request",
        "command": command,
        "arguments": arguments,
    })
}
