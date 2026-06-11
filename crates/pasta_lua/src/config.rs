//! Transpiler configuration for Lua code generation.
//!
//! This module provides configuration options for the transpilation process.

/// Line ending style for generated code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style line endings (LF: \n)
    Lf,
    /// Windows-style line endings (CRLF: \r\n)
    CrLf,
}

impl LineEnding {
    /// Get the line ending string.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }

    /// Get the platform-native line ending.
    #[cfg(windows)]
    pub fn native() -> Self {
        LineEnding::CrLf
    }

    /// Get the platform-native line ending.
    #[cfg(not(windows))]
    pub fn native() -> Self {
        LineEnding::Lf
    }
}

impl Default for LineEnding {
    fn default() -> Self {
        Self::native()
    }
}

/// Transpiler configuration.
#[derive(Debug, Clone)]
pub struct TranspilerConfig {
    /// Enable comment mode (include Pasta source line references)
    pub comment_mode: bool,
    /// Line ending style for the intermediate code-generation buffer.
    ///
    /// Note: output normalization converts all line endings to LF as a final
    /// step, so the transpiled bytes are always LF-only regardless of this
    /// setting. See `LuaTranspiler::transpile` post-processing.
    pub line_ending: LineEnding,
}

impl Default for TranspilerConfig {
    fn default() -> Self {
        Self {
            comment_mode: true,
            line_ending: LineEnding::default(),
        }
    }
}

impl TranspilerConfig {
    /// Create a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration with comment mode disabled.
    pub fn without_comments() -> Self {
        Self {
            comment_mode: false,
            ..Default::default()
        }
    }

    /// Set line ending style for the intermediate code-generation buffer.
    ///
    /// Note: the final transpiled output is always LF-only because output
    /// normalization converts CRLF to LF; this setting has no observable
    /// effect on the bytes written by `LuaTranspiler::transpile`.
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TranspilerConfig::default();
        assert!(config.comment_mode);
    }

    #[test]
    fn test_without_comments() {
        let config = TranspilerConfig::without_comments();
        assert!(!config.comment_mode);
    }

    #[test]
    fn test_line_ending() {
        assert_eq!(LineEnding::Lf.as_str(), "\n");
        assert_eq!(LineEnding::CrLf.as_str(), "\r\n");
    }

    #[test]
    fn test_with_line_ending() {
        let config = TranspilerConfig::new().with_line_ending(LineEnding::Lf);
        assert_eq!(config.line_ending, LineEnding::Lf);
    }

    #[test]
    fn test_line_ending_native_matches_platform() {
        #[cfg(windows)]
        assert_eq!(LineEnding::native(), LineEnding::CrLf);
        #[cfg(not(windows))]
        assert_eq!(LineEnding::native(), LineEnding::Lf);
    }

    #[test]
    fn test_line_ending_default_is_native() {
        assert_eq!(LineEnding::default(), LineEnding::native());
    }

    #[test]
    fn test_new_equals_default() {
        let a = TranspilerConfig::new();
        let b = TranspilerConfig::default();
        assert_eq!(a.comment_mode, b.comment_mode);
        assert_eq!(a.line_ending, b.line_ending);
    }

    #[test]
    fn test_without_comments_keeps_default_line_ending() {
        let config = TranspilerConfig::without_comments();
        assert_eq!(config.line_ending, LineEnding::default());
    }
}
