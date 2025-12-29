//! StringLiteralizer - Lua string literal conversion.
//!
//! This module provides functionality to convert strings to optimal Lua literal format.
//! Implements Requirement 2: Lua文字列リテラル形式の標準化.

use super::error::TranspileError;
use pasta_core::parser::Span;

/// Maximum number of `=` signs to try before giving up.
const MAX_EQUALS: usize = 10;

/// String literalizer for Lua output.
///
/// Converts strings to the optimal Lua literal format based on content.
pub struct StringLiteralizer;

impl StringLiteralizer {
    /// Convert a string to Lua literal format.
    ///
    /// # Rules
    /// - Rule 1: If no escape-needing characters (`\` or `"`), use `"text"` format.
    /// - Rule 2: If escape-needing characters exist, use long string format `[=[text]=]`.
    ///
    /// # Long String `=` Count Algorithm
    /// - Start with n=0 (danger pattern = `]`)
    /// - If text contains danger pattern `]` + n `=`, increment n
    /// - Continue until no danger pattern found or n > MAX_EQUALS
    ///
    /// # Examples
    /// - `hello world` → `"hello world"`
    /// - `hello\nworld` (with backslash) → `[[hello\nworld]]`
    /// - `hello]world` → `[=[hello]world]=]`
    pub fn literalize(text: &str) -> Result<String, TranspileError> {
        Self::literalize_with_span(text, &Span::default())
    }

    /// Convert a string to Lua literal format with span information for errors.
    pub fn literalize_with_span(text: &str, span: &Span) -> Result<String, TranspileError> {
        if !Self::needs_long_string(text) {
            // Rule 1: Simple double-quoted string
            Ok(format!("\"{}\"", text))
        } else {
            // Rule 2: Long string format
            for n in 0..=MAX_EQUALS {
                if !Self::contains_danger_pattern(text, n) {
                    let equals = "=".repeat(n);
                    return Ok(format!("[{}[{}]{}]", equals, text, equals));
                }
            }
            // Could not find safe format
            Err(TranspileError::string_literal_error(span, text))
        }
    }

    /// Check if the text needs long string format.
    ///
    /// Returns true if text contains `\` or `"`.
    fn needs_long_string(text: &str) -> bool {
        text.contains('\\') || text.contains('"')
    }

    /// Check if text contains the danger pattern for given n.
    ///
    /// Danger pattern for n equals: `]` followed by n `=` signs.
    /// This is the prefix of the closing delimiter `]=...=]` (without final `]`).
    fn contains_danger_pattern(text: &str, n: usize) -> bool {
        if n == 0 {
            // For n=0, danger pattern is just `]`
            text.contains(']')
        } else {
            // For n>0, danger pattern is `]` + n `=`
            let pattern: String = format!("]{}", "=".repeat(n));
            text.contains(&pattern)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string_no_special_chars() {
        // Rule 1: No special characters
        let result = StringLiteralizer::literalize("hello world").unwrap();
        assert_eq!(result, "\"hello world\"");
    }

    #[test]
    fn test_simple_string_japanese() {
        // Rule 1: Japanese text without special chars
        let result = StringLiteralizer::literalize("こんにちは").unwrap();
        assert_eq!(result, "\"こんにちは\"");
    }

    #[test]
    fn test_long_string_with_backslash() {
        // Rule 2: Contains backslash, use long string
        let result = StringLiteralizer::literalize("\\s[0]").unwrap();
        // \s[0] contains ] so needs n=1
        assert_eq!(result, "[=[\\s[0]]=]");
    }

    #[test]
    fn test_long_string_with_quote() {
        // Rule 2: Contains quote, use long string
        let result = StringLiteralizer::literalize("hello \"world\"").unwrap();
        assert_eq!(result, "[[hello \"world\"]]");
    }

    #[test]
    fn test_long_string_with_bracket() {
        // Text contains ] but no backslash/quote - still simple
        let result = StringLiteralizer::literalize("hello]world").unwrap();
        assert_eq!(result, "\"hello]world\"");
    }

    #[test]
    fn test_long_string_with_backslash_and_bracket() {
        // Contains both backslash and bracket
        let result = StringLiteralizer::literalize("\\test]value").unwrap();
        // Contains ] so n=0 is not safe, try n=1
        assert_eq!(result, "[=[\\test]value]=]");
    }

    #[test]
    fn test_long_string_n1_pattern() {
        // Contains ]= pattern, needs n=2
        let result = StringLiteralizer::literalize("\\test]=value").unwrap();
        assert_eq!(result, "[==[\\test]=value]==]");
    }

    #[test]
    fn test_long_string_n2_pattern() {
        // Contains ]== pattern, needs n=3
        let result = StringLiteralizer::literalize("\\test]==value").unwrap();
        assert_eq!(result, "[===[\\test]==value]===]");
    }

    #[test]
    fn test_sakura_script_s0() {
        // Typical SakuraScript: \s[0]
        let result = StringLiteralizer::literalize("\\s[0]").unwrap();
        assert_eq!(result, "[=[\\s[0]]=]");
    }

    #[test]
    fn test_sakura_script_s10() {
        // Typical SakuraScript: \s[10]
        let result = StringLiteralizer::literalize("\\s[10]").unwrap();
        assert_eq!(result, "[=[\\s[10]]=]");
    }

    #[test]
    fn test_needs_long_string_with_backslash() {
        assert!(StringLiteralizer::needs_long_string("\\test"));
    }

    #[test]
    fn test_needs_long_string_with_quote() {
        assert!(StringLiteralizer::needs_long_string("hello\"world"));
    }

    #[test]
    fn test_needs_long_string_plain_text() {
        assert!(!StringLiteralizer::needs_long_string("hello world"));
    }

    #[test]
    fn test_danger_pattern_n0() {
        assert!(StringLiteralizer::contains_danger_pattern("hello]world", 0));
        assert!(!StringLiteralizer::contains_danger_pattern("hello world", 0));
    }

    #[test]
    fn test_danger_pattern_n1() {
        assert!(StringLiteralizer::contains_danger_pattern("hello]=world", 1));
        assert!(!StringLiteralizer::contains_danger_pattern("hello]world", 1));
    }

    #[test]
    fn test_danger_pattern_n2() {
        assert!(StringLiteralizer::contains_danger_pattern("hello]==world", 2));
        assert!(!StringLiteralizer::contains_danger_pattern("hello]=world", 2));
    }
}
