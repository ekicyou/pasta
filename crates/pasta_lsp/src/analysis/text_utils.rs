//! Source text utility functions for line/offset calculations.

// ============================================================================
// Public Utilities
// ============================================================================

/// Get the text of a specific line (1-based line number).
/// Returns the line content without trailing \r or \n.
pub fn get_line_text(source: &str, line: usize) -> &str {
    let (text, _) = get_line_text_and_offset(source, line);
    text
}

/// Get the byte offset where a line starts (1-based line number).
pub fn line_byte_offset(source: &str, line: usize) -> usize {
    let (_, offset) = get_line_text_and_offset(source, line);
    offset
}

/// Internal helper: split source into lines manually, handling both \n and \r\n.
/// Returns (line_text_without_eol, line_start_byte_offset) for the given 1-based line.
fn get_line_text_and_offset(source: &str, line: usize) -> (&str, usize) {
    let bytes = source.as_bytes();
    let mut current_line = 1usize;

    if line == 1 {
        // Fast path for first line
        let end = memchr_newline(bytes, 0);
        let text = strip_cr(&source[0..end]);
        return (text, 0);
    }

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            current_line += 1;
            i += 1;
            if current_line == line {
                let end = memchr_newline(bytes, i);
                let text = strip_cr(&source[i..end]);
                return (text, i);
            }
        } else {
            i += 1;
        }
    }

    // Requested line not found — return empty
    ("", source.len())
}

/// Find the next \n (or end of slice) starting from `start`.
#[inline]
fn memchr_newline(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Strip a trailing \r if present (for CRLF handling).
#[inline]
fn strip_cr(s: &str) -> &str {
    s.strip_suffix('\r').unwrap_or(s)
}

// ============================================================================
// Expression Text Scanning Helpers
// ============================================================================

/// Find a number literal (full-width or half-width digits, with optional decimal point)
/// at the start of the text (after skipping whitespace).
/// Returns (start_byte, end_byte) within text.
pub(super) fn find_number_literal(text: &str) -> Option<(usize, usize)> {
    let trimmed_start = text.len() - text.trim_start().len();
    let mut chars = text[trimmed_start..].char_indices().peekable();
    let mut started = false;
    let mut end = trimmed_start;

    while let Some(&(i, c)) = chars.peek() {
        if is_digit_char(c) || (started && (c == '.' || c == '．')) {
            started = true;
            end = trimmed_start + i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    if started {
        Some((trimmed_start, end))
    } else {
        None
    }
}

/// Check if a character is a digit (half-width or full-width).
#[inline]
fn is_digit_char(c: char) -> bool {
    c.is_ascii_digit() || ('０'..='９').contains(&c)
}

/// Find an opening parenthesis (half-width or full-width) in text.
/// Returns byte position.
pub(super) fn find_open_paren(text: &str) -> Option<usize> {
    for (i, c) in text.char_indices() {
        if c == '(' || c == '（' {
            return Some(i);
        }
    }
    None
}

/// Find matching closing parenthesis, respecting nesting.
/// `start` is the byte offset after the opening paren.
pub(super) fn find_close_paren(text: &str, start: usize) -> Option<usize> {
    let mut depth = 1i32;
    for (i, c) in text[start..].char_indices() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Get the byte length of the char at the given byte position.
#[inline]
pub(super) fn char_len_at(text: &str, byte_pos: usize) -> usize {
    text[byte_pos..].chars().next().map_or(1, |c| c.len_utf8())
}

/// Arithmetic operator characters of the grammar (`add`/`sub`/`mul`/`div`/`modulo`).
#[inline]
fn is_chain_op_char(c: char) -> bool {
    matches!(
        c,
        '+' | '＋' | '-' | '－' | '*' | '＊' | '×' | '/' | '／' | '÷' | '%' | '％'
    )
}

/// Scan an expression left to right once and return the (byte_pos, byte_len)
/// of each operator of its top-level binary chain.
///
/// An operator character counts only when it is at paren depth 0, outside a
/// string literal (`「…」` / `"…"`, any fence length), not at the start of a
/// term (so the sign of `－1` belongs to the term) and not right after
/// `＄`/`＠` (so `＄＊`, `＠＊`, `＄％` are markers).
pub(super) fn find_chain_ops(text: &str) -> Vec<(usize, usize)> {
    let mut ops = Vec::new();
    let mut depth = 0i32;
    let mut expect_term = true;
    let mut prev = ' ';
    let mut i = 0;
    while let Some(c) = text[i..].chars().next() {
        let mut next = i + c.len_utf8();
        match c {
            _ if c.is_whitespace() => {}
            '「' | '"' => {
                // String literal: the fence is the run of opening chars,
                // closed by the same number of closing chars.
                let close = if c == '「' { '」' } else { '"' };
                let fence = text[i..].chars().take_while(|&x| x == c).count();
                let after_open = i + fence * c.len_utf8();
                let closing = close.to_string().repeat(fence);
                next = text[after_open..]
                    .find(&closing)
                    .map_or(after_open, |p| after_open + p + closing.len());
            }
            '(' | '（' => depth += 1,
            ')' | '）' => depth -= 1,
            _ if depth == 0
                && !expect_term
                && !matches!(prev, '＄' | '$' | '＠' | '@')
                && is_chain_op_char(c) =>
            {
                ops.push((i, c.len_utf8()));
                expect_term = true;
                prev = c;
                i = next;
                continue;
            }
            _ => {}
        }
        if !c.is_whitespace() {
            expect_term = false;
        }
        prev = c;
        i = next;
    }
    ops
}

/// Find a dynamic reference `[＠@][＄$]` (plus `[＊*]` when `global`) followed by
/// `var_name` in text. Full-width/half-width markers may be mixed.
/// Returns (start_byte, end_byte) within text.
pub(super) fn find_dynamic_ref(text: &str, var_name: &str, global: bool) -> Option<(usize, usize)> {
    text.char_indices().find_map(|(i, _)| {
        let rest = text[i..]
            .strip_prefix(['＠', '@'])?
            .strip_prefix(['＄', '$'])?;
        let rest = if global {
            rest.strip_prefix(['＊', '*'])?
        } else {
            rest
        };
        let rest = rest.strip_prefix(var_name)?;
        Some((i, text.len() - rest.len()))
    })
}

/// Find the end of an argument in a comma/、-separated list.
/// Returns byte offset within the text where the argument ends.
pub(super) fn find_arg_end(text: &str) -> usize {
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '（' => depth += 1,
            ')' | '）' => depth -= 1,
            '、' | ',' if depth == 0 => return i,
            _ => {}
        }
    }
    text.len()
}
