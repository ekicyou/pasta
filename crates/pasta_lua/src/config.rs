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
    /// Line ending style for generated code
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

    /// Set line ending style.
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
}
