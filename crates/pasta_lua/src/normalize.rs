//! Output normalization for Pasta Lua transpiler.
//!
//! This module provides output buffer normalization to remove trailing blank lines
//! and ensure consistent EOF markers.

/// Normalize output buffer by removing trailing blank lines and blank lines before `end`.
///
/// # Behavior
/// 1. Removes blank lines immediately before `end` keywords
/// 2. Removes trailing whitespace (spaces, tabs, newlines)
/// 3. Appends exactly one newline
///
/// # Preconditions
/// - `input` is a valid UTF-8 string
///
/// # Postconditions
/// - Return value ends with exactly "\n"
/// - Content blank lines not immediately before `end` are preserved
/// - No blank line appears immediately before `end` keyword
///
/// # Examples
/// ```
/// use pasta_lua::normalize::normalize_output;
///
/// assert_eq!(normalize_output("code\n\n\n"), "code\n");
/// assert_eq!(normalize_output("code"), "code\n");
/// assert_eq!(normalize_output("a\n\nb\n\n"), "a\n\nb\n");
/// assert_eq!(normalize_output("    end\n\nend\n"), "    end\nend\n");
/// ```
pub fn normalize_output(input: &str) -> String {
    // Normalize CRLF to LF first
    let input_lf = input.replace("\r\n", "\n");
    
    // Split into lines (preserving the content, not the line endings)
    let lines: Vec<&str> = input_lf.lines().collect();
    let mut result_lines: Vec<&str> = Vec::with_capacity(lines.len());
    
    // Process lines, removing blank lines before `end`
    // Use a loop that can skip blank lines when followed by `end`
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        // Check if this is a blank line
        if trimmed.is_empty() {
            // Look ahead to find if any subsequent line starts with `end` 
            // (skipping any other blank lines)
            let mut j = i + 1;
            let mut found_end = false;
            
            // Check only the immediate next non-blank line
            while j < lines.len() {
                let next_trimmed = lines[j].trim();
                if next_trimmed.is_empty() {
                    // Another blank line, skip it too
                    j += 1;
                    continue;
                }
                // Found a non-blank line, check if it starts with `end`
                if next_trimmed == "end" {
                    found_end = true;
                }
                break;
            }
            
            if found_end {
                // Skip this blank line
                i += 1;
                continue;
            }
        }
        
        result_lines.push(line);
        i += 1;
    }
    
    // Join lines back with LF
    let processed = result_lines.join("\n");
    
    // Trim trailing whitespace (spaces, tabs, carriage returns, newlines)
    let trimmed = processed.trim_end_matches(|c| c == ' ' || c == '\t' || c == '\r' || c == '\n');
    
    // Return with exactly one newline at end
    format!("{}\n", trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Empty input should return single newline
    #[test]
    fn test_normalize_empty_input() {
        assert_eq!(normalize_output(""), "\n");
    }

    /// Test 2: Input already ending with single newline should remain unchanged
    #[test]
    fn test_normalize_existing_single_newline() {
        assert_eq!(normalize_output("code\n"), "code\n");
    }

    /// Test 3: Single extra trailing blank line should be removed
    #[test]
    fn test_normalize_single_extra_blank_line() {
        assert_eq!(normalize_output("code\n\n"), "code\n");
    }

    /// Test 4: Multiple extra trailing blank lines should be removed
    #[test]
    fn test_normalize_multiple_extra_blank_lines() {
        assert_eq!(normalize_output("code\n\n\n"), "code\n");
    }

    /// Test 5: Intermediate blank lines should be preserved
    #[test]
    fn test_normalize_preserves_intermediate_blank_lines() {
        assert_eq!(normalize_output("a\n\nb\n\n"), "a\n\nb\n");
    }

    /// Test 6: CRLF input should be normalized to LF only
    #[test]
    fn test_normalize_crlf_input() {
        assert_eq!(normalize_output("code\r\n\r\n"), "code\n");
    }

    /// Test 7: Mixed CRLF and LF - all normalized to LF
    #[test]
    fn test_normalize_mixed_line_endings() {
        // CRLF is normalized to LF
        assert_eq!(normalize_output("a\r\nb\n\r\n\n"), "a\nb\n");
    }

    /// Test 8: Trailing whitespace only
    #[test]
    fn test_normalize_trailing_whitespace_only() {
        assert_eq!(normalize_output("code\n  \t\n"), "code\n");
    }

    /// Test 9: No trailing newline in input
    #[test]
    fn test_normalize_no_trailing_newline() {
        assert_eq!(normalize_output("code"), "code\n");
    }

    /// Test 10: Multiple content lines with trailing blanks
    #[test]
    fn test_normalize_multi_line_content() {
        let input = "line1\nline2\nline3\n\n\n";
        assert_eq!(normalize_output(input), "line1\nline2\nline3\n");
    }

    /// Test 11: Blank line before `end` should be removed
    #[test]
    fn test_normalize_blank_line_before_end() {
        let input = "    end\n\nend\n";
        assert_eq!(normalize_output(input), "    end\nend\n");
    }

    /// Test 12: Multiple blank lines before `end` should be reduced to none
    #[test]
    fn test_normalize_multiple_blank_lines_before_end() {
        let input = "    end\n\n\nend\n";
        // All blank lines before end are removed
        assert_eq!(normalize_output(input), "    end\nend\n");
    }

    /// Test 13: Blank line before indented `end` should be removed (preserving indentation)
    #[test]
    fn test_normalize_blank_line_before_indented_end() {
        let input = "code\n\n    end\n";
        // Blank line before `    end` is removed, but indentation is preserved
        assert_eq!(normalize_output(input), "code\n    end\n");
    }

    /// Test 14: Real Lua block pattern
    #[test]
    fn test_normalize_lua_do_block() {
        let input = "do\n    function f()\n        return 1\n    end\n\nend\n";
        assert_eq!(normalize_output(input), "do\n    function f()\n        return 1\n    end\nend\n");
    }

    /// Test 15: Nested end blocks
    #[test]
    fn test_normalize_nested_end_blocks() {
        let input = "    end\n\n    end\n\nend\n";
        // Both blank lines before `end` are removed
        assert_eq!(normalize_output(input), "    end\n    end\nend\n");
    }
}
