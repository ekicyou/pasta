//! Output normalization for Pasta Lua transpiler.
//!
//! This module provides output buffer normalization to remove trailing blank lines
//! and ensure consistent EOF markers.

/// Normalize output buffer by removing trailing blank lines.
///
/// # Behavior
/// 1. Normalizes CRLF to LF
/// 2. Removes trailing whitespace (spaces, tabs, newlines)
/// 3. Appends exactly one newline
///
/// # Preconditions
/// - `input` is a valid UTF-8 string
///
/// # Postconditions
/// - Return value ends with exactly "\n"
/// - Blank lines anywhere before the tail (including before `end`) are preserved
///
/// # Examples
/// ```
/// use pasta_lua::normalize::normalize_output;
///
/// assert_eq!(normalize_output("code\n\n\n"), "code\n");
/// assert_eq!(normalize_output("code"), "code\n");
/// assert_eq!(normalize_output("a\n\nb\n\n"), "a\n\nb\n");
/// assert_eq!(normalize_output("    end\n\nend\n"), "    end\n\nend\n");
/// ```
pub fn normalize_output(input: &str) -> String {
    // Thin wrapper: byte-identical to `normalize_output_with_shift(input).0`
    // (design `LineShift`: 既存 normalize_output は薄いラッパで互換維持).
    normalize_output_with_shift(input).0
}

/// Old-line → new-line shift map produced by output normalization.
///
/// Normalization is **deletion-only** (it removes only the trailing blank lines /
/// trailing whitespace). This map
/// records which pre-normalize line numbers were deleted so a consumer can rebase
/// recorded pre-normalize line positions onto the final `.lua` line numbers
/// (Requirement 2.1).
///
/// # Line-number domain
/// Both the input (`pre_line`) and the output of [`map`](LineShift::map) are
/// **1-based** line numbers. The input domain is aligned with the code generator's
/// `out_line` counter, which is 1-based and counts pre-normalize buffer lines.
///
/// # Invariants
/// - Deletions only: no line is inserted, merged, or grown.
/// - [`map`](LineShift::map) is monotonically increasing over surviving lines.
/// - Every deleted line was blank (whitespace-only) and therefore has no `.pasta`
///   origin, so dropping it loses no mapping.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineShift {
    /// Deleted pre-normalize line numbers (1-based, ascending).
    deleted: Vec<u32>,
}

impl LineShift {
    /// Map a pre-normalize line number to its final `.lua` line number.
    ///
    /// Returns `None` if the line was deleted by normalization (such lines are
    /// always blank and carry no `.pasta` origin). For a surviving line, returns
    /// its 1-based line number in the final output. The result is monotonically
    /// increasing in `pre_line`.
    pub fn map(&self, pre_line: u32) -> Option<u32> {
        // Deleted lines have no final position.
        if self.deleted.binary_search(&pre_line).is_ok() {
            return None;
        }
        // The final line number is the pre-normalize number minus the count of
        // deleted lines strictly before it.
        let deleted_before = self.deleted.partition_point(|&d| d < pre_line) as u32;
        Some(pre_line - deleted_before)
    }
}

/// Normalize the output buffer and return both the normalized text and a
/// [`LineShift`] describing which pre-normalize lines were deleted.
///
/// The returned string is **byte-identical** to [`normalize_output`] for the same
/// input; that function is a thin wrapper around `.0` of this one. See [`LineShift`]
/// for the line-number domain and invariants (Requirement 2.1).
///
/// # Preconditions
/// - `input` is the full intermediate code-generation buffer (valid UTF-8).
///
/// # Postconditions
/// - `.0` equals the legacy `normalize_output(input)` byte-for-byte.
/// - `.1` records exactly the lines deleted (the trailing blank lines), so
///   `map` resolves surviving lines and returns `None` for deleted ones.
pub fn normalize_output_with_shift(input: &str) -> (String, LineShift) {
    // Normalize CRLF to LF first
    let input_lf = input.replace("\r\n", "\n");

    // Split into lines (preserving the content, not the line endings).
    // `lines[idx]` is pre-normalize line number `idx + 1` (1-based, == out_line).
    let lines: Vec<&str> = input_lf.lines().collect();

    // Every line survives except the trailing whitespace-only lines: the last line
    // that contains any non-whitespace character is kept, everything after it is
    // deleted (all lines when none is kept). Ascending by construction.
    let kept = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map_or(0, |k| k + 1);
    let deleted: Vec<u32> = (kept + 1..=lines.len()).map(|n| n as u32).collect();

    // Trim trailing whitespace (spaces, tabs, carriage returns, newlines). This
    // removes the trailing blank lines and trims in-line trailing whitespace on
    // the last kept line (NOT a line deletion — it does not change the count).
    let trimmed = input_lf.trim_end_matches([' ', '\t', '\r', '\n']);

    // Return with exactly one newline at end
    (format!("{}\n", trimmed), LineShift { deleted })
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

    /// Test 11: Blank lines before `end` are kept (they may be Lua block
    /// content, e.g. inside a long string); every line maps to itself.
    #[test]
    fn test_normalize_keeps_blank_lines_before_end() {
        let (s, shift) = normalize_output_with_shift("a\n\nend\n");
        assert_eq!(s, "a\n\nend\n");
        assert_eq!(shift, LineShift::default());
        assert_eq!(
            normalize_output("    end\n\n\n    end\nend\n"),
            "    end\n\n\n    end\nend\n"
        );
    }

    // --- LineShift tests (Requirement 2.1) ---

    /// The shift wrapper must produce byte-identical output to `normalize_output`
    /// for every legacy fixture (byte-compat invariant, design line 385).
    #[test]
    fn test_with_shift_byte_identical_to_legacy() {
        let inputs = [
            "",
            "code\n",
            "code\n\n",
            "code\n\n\n",
            "a\n\nb\n\n",
            "code\r\n\r\n",
            "a\r\nb\n\r\n\n",
            "code\n  \t\n",
            "code",
            "line1\nline2\nline3\n\n\n",
            "    end\n\nend\n",
            "    end\n\n\nend\n",
            "code\n\n    end\n",
            "do\n    function f()\n        return 1\n    end\n\nend\n",
            "    end\n\n    end\n\nend\n",
        ];
        for input in inputs {
            let (s, _shift) = normalize_output_with_shift(input);
            assert_eq!(
                s,
                normalize_output(input),
                "byte mismatch for input {input:?}"
            );
        }
    }

    /// Trailing blank lines are deleted (whole-line) and map to None; in-line
    /// trailing whitespace trim on the last content line is NOT a line deletion.
    #[test]
    fn test_shift_trailing_blank_lines_mapping() {
        // pre-normalize (1-based):
        //   1: "line1"
        //   2: "line2"
        //   3: ""       <- deleted (trailing blank)
        //   4: ""       <- deleted (trailing blank)
        let input = "line1\nline2\n\n\n";
        let (s, shift) = normalize_output_with_shift(input);
        assert_eq!(s, "line1\nline2\n");

        assert_eq!(shift.map(1), Some(1));
        assert_eq!(shift.map(2), Some(2));
        assert_eq!(shift.map(3), None);
        assert_eq!(shift.map(4), None);
    }

    /// In-line trailing whitespace on a content line trims characters but does NOT
    /// remove the line, so the line number is unchanged (no deletion recorded).
    #[test]
    fn test_shift_inline_trailing_whitespace_not_a_deletion() {
        // pre-normalize (1-based):
        //   1: "code"
        //   2: "  \t"   <- whitespace-only trailing line -> deleted
        let input = "code\n  \t\n";
        let (s, shift) = normalize_output_with_shift(input);
        assert_eq!(s, "code\n");

        assert_eq!(shift.map(1), Some(1)); // content line unshifted
        assert_eq!(shift.map(2), None); // trailing whitespace line deleted
    }

    /// A blank line in the body (even right before `end`) is kept; only the
    /// trailing blanks at the tail are deleted.
    #[test]
    fn test_shift_body_blank_kept_trailing_deleted() {
        // pre-normalize (1-based):
        //   1: "do"
        //   2: "    f()"
        //   3: ""        <- kept (body blank before end)
        //   4: "end"
        //   5: ""        <- deleted (trailing blank)
        //   6: ""        <- deleted (trailing blank)
        let input = "do\n    f()\n\nend\n\n\n";
        let (s, shift) = normalize_output_with_shift(input);
        assert_eq!(s, "do\n    f()\n\nend\n");

        let survivors: Vec<u32> = (1..=6).filter_map(|p| shift.map(p)).collect();
        assert_eq!(survivors, vec![1, 2, 3, 4]);
        assert_eq!(shift.map(5), None);
        assert_eq!(shift.map(6), None);
    }

    /// Empty input: the producer emits no lines, so there are zero recorded
    /// pre-normalize lines and nothing is deleted. Output is a lone newline.
    /// (`map` is only defined over recorded pre-lines `1..=lines.len()`; for empty
    /// input that range is empty, so no `map` query is in-domain.)
    #[test]
    fn test_shift_empty_input() {
        let (s, shift) = normalize_output_with_shift("");
        assert_eq!(s, "\n");
        assert_eq!(shift, LineShift::default()); // no deletions recorded
    }

    /// A buffer that is entirely blank lines: every emitted line is whitespace and
    /// trims away, so all recorded pre-lines are deleted and map to None.
    #[test]
    fn test_shift_all_blank_lines_deleted() {
        // pre-normalize (1-based): 1:"" 2:"  " 3:"\t"  -> all trailing blanks
        let input = "\n  \n\t\n";
        let (s, shift) = normalize_output_with_shift(input);
        assert_eq!(s, "\n");
        assert_eq!(shift.map(1), None);
        assert_eq!(shift.map(2), None);
        assert_eq!(shift.map(3), None);
    }

    /// No deletions: every line maps to itself.
    #[test]
    fn test_shift_no_deletions_identity() {
        let input = "a\n\nb\nc\n";
        let (s, shift) = normalize_output_with_shift(input);
        assert_eq!(s, "a\n\nb\nc\n");
        assert_eq!(shift.map(1), Some(1));
        assert_eq!(shift.map(2), Some(2)); // intermediate blank preserved
        assert_eq!(shift.map(3), Some(3));
        assert_eq!(shift.map(4), Some(4));
    }
}
