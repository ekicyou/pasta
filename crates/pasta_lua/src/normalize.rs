//! Output normalization for Pasta Lua transpiler.
//!
//! Removes trailing blank lines and ends the output with exactly one newline.

/// Old-line → new-line shift map produced by output normalization.
///
/// Normalization is **deletion-only** (it removes only the trailing blank lines /
/// trailing whitespace). This map records which pre-normalize line numbers were
/// deleted so a consumer can rebase recorded pre-normalize line positions onto the
/// final `.lua` line numbers.
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
        if self.deleted.binary_search(&pre_line).is_ok() {
            return None;
        }
        let deleted_before = self.deleted.partition_point(|&d| d < pre_line) as u32;
        Some(pre_line - deleted_before)
    }
}

/// Normalize the output buffer and return both the normalized text and a
/// [`LineShift`] describing which pre-normalize lines were deleted.
///
/// # Behavior
/// 1. Normalizes CRLF to LF
/// 2. Removes trailing whitespace (spaces, tabs, newlines)
/// 3. Appends exactly one newline
///
/// See [`LineShift`] for the line-number domain and invariants.
///
/// # Preconditions
/// - `input` is the full intermediate code-generation buffer (valid UTF-8).
///
/// # Postconditions
/// - `.0` ends with exactly "\n"; blank lines anywhere before the tail
///   (including before `end`) are preserved.
/// - `.1` records exactly the lines deleted (the trailing blank lines), so
///   `map` resolves surviving lines and returns `None` for deleted ones.
///
/// # Examples
/// ```
/// use pasta_lua::normalize::normalize_output_with_shift;
///
/// assert_eq!(normalize_output_with_shift("code\n\n\n").0, "code\n");
/// assert_eq!(normalize_output_with_shift("code").0, "code\n");
/// assert_eq!(normalize_output_with_shift("a\n\nb\n\n").0, "a\n\nb\n");
/// assert_eq!(normalize_output_with_shift("    end\n\nend\n").0, "    end\n\nend\n");
/// ```
pub fn normalize_output_with_shift(input: &str) -> (String, LineShift) {
    let input_lf = input.replace("\r\n", "\n");

    // `lines[idx]` is pre-normalize line number `idx + 1` (1-based, == out_line).
    let lines: Vec<&str> = input_lf.lines().collect();

    // Only the whitespace-only lines after the last content line are deleted.
    let kept = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map_or(0, |k| k + 1);
    let deleted: Vec<u32> = (kept + 1..=lines.len()).map(|n| n as u32).collect();

    // Also trims in-line trailing whitespace on the last kept line, which is not a
    // line deletion.
    let trimmed = input_lf.trim_end_matches([' ', '\t', '\r', '\n']);

    (format!("{}\n", trimmed), LineShift { deleted })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_empty_input() {
        assert_eq!(normalize_output_with_shift("").0, "\n");
    }

    #[test]
    fn test_normalize_existing_single_newline() {
        assert_eq!(normalize_output_with_shift("code\n").0, "code\n");
    }

    #[test]
    fn test_normalize_single_extra_blank_line() {
        assert_eq!(normalize_output_with_shift("code\n\n").0, "code\n");
    }

    #[test]
    fn test_normalize_multiple_extra_blank_lines() {
        assert_eq!(normalize_output_with_shift("code\n\n\n").0, "code\n");
    }

    #[test]
    fn test_normalize_preserves_intermediate_blank_lines() {
        assert_eq!(normalize_output_with_shift("a\n\nb\n\n").0, "a\n\nb\n");
    }

    #[test]
    fn test_normalize_crlf_input() {
        assert_eq!(normalize_output_with_shift("code\r\n\r\n").0, "code\n");
    }

    #[test]
    fn test_normalize_mixed_line_endings() {
        assert_eq!(normalize_output_with_shift("a\r\nb\n\r\n\n").0, "a\nb\n");
    }

    #[test]
    fn test_normalize_trailing_whitespace_only() {
        assert_eq!(normalize_output_with_shift("code\n  \t\n").0, "code\n");
    }

    #[test]
    fn test_normalize_no_trailing_newline() {
        assert_eq!(normalize_output_with_shift("code").0, "code\n");
    }

    #[test]
    fn test_normalize_multi_line_content() {
        let input = "line1\nline2\nline3\n\n\n";
        assert_eq!(
            normalize_output_with_shift(input).0,
            "line1\nline2\nline3\n"
        );
    }

    /// Blank lines before `end` are kept (they may be Lua block
    /// content, e.g. inside a long string); every line maps to itself.
    #[test]
    fn test_normalize_keeps_blank_lines_before_end() {
        let (s, shift) = normalize_output_with_shift("a\n\nend\n");
        assert_eq!(s, "a\n\nend\n");
        assert_eq!(shift, LineShift::default());
        assert_eq!(
            normalize_output_with_shift("    end\n\n\n    end\nend\n").0,
            "    end\n\n\n    end\nend\n"
        );
    }

    // --- LineShift ---

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

    /// A whitespace-only trailing line is deleted; the content line keeps its number.
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
