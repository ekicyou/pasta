//! Source presentation mode value types for the debug session.
//!
//! Split out of `debug/mod.rs` (task 4.5): the [`SourceMode`] value enum. The
//! parent `mod.rs` keeps the `pub use source_mode::SourceMode;` re-export.

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
}
