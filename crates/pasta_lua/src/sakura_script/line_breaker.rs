//! BudouX-based line breaker for sakura script text.
//!
//! Inserts sakura script line break tags (`\n`) at natural Japanese word
//! boundaries while transparently preserving sakura script tags.

use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// A plain text character paired with trailing sakura script tags.
struct PlainChar<'a> {
    ch: char,
    trailing: &'a str,
}

/// Tokenized representation separating plain text from tags.
struct Tokens<'a> {
    leading: &'a str,
    chars: Vec<PlainChar<'a>>,
}

/// Tokenize input into plain characters with their trailing tags.
///
/// Uses `tag_regex` to identify sakura script tags and separates them
/// from plain text characters. Each plain character carries any tags
/// that immediately follow it.
fn tokenize_plain_chars<'a>(input: &'a str, tag_regex: &Regex) -> Tokens<'a> {
    let mut chars = Vec::new();
    let mut pos = 0;
    let mut leading_end = 0;
    let mut leading_set = false;

    // Collect all tag match ranges
    let tag_ranges: Vec<(usize, usize)> = tag_regex.find_iter(input).map(|m| (m.start(), m.end())).collect();
    let mut tag_idx = 0;

    while pos < input.len() {
        // Check if current position is inside a tag
        if tag_idx < tag_ranges.len() && pos == tag_ranges[tag_idx].0 {
            pos = tag_ranges[tag_idx].1;
            tag_idx += 1;
            continue;
        }

        // Plain character
        let remaining = &input[pos..];
        if let Some(c) = remaining.chars().next() {
            if !leading_set {
                // Everything before the first plain char is "leading"
                leading_end = pos;
                leading_set = true;
            }

            let char_end = pos + c.len_utf8();

            // Find trailing: from char_end to the next plain char (or end)
            let mut trailing_end = char_end;
            // Skip over any tags immediately after this char
            loop {
                if tag_idx < tag_ranges.len() && trailing_end == tag_ranges[tag_idx].0 {
                    trailing_end = tag_ranges[tag_idx].1;
                    tag_idx += 1;
                } else {
                    break;
                }
            }

            chars.push(PlainChar {
                ch: c,
                trailing: &input[char_end..trailing_end],
            });

            pos = trailing_end;
        } else {
            break;
        }
    }

    let leading = if leading_set {
        &input[..leading_end]
    } else {
        // No plain chars at all — entire input is tags
        input
    };

    Tokens { leading, chars }
}

/// Insert sakura script line breaks (`\n`) into text at natural Japanese
/// word boundaries determined by budoux, respecting per-line width thresholds.
///
/// Sakura script tags in the input are transparently preserved in the output
/// at their original relative positions, but excluded from width calculations.
///
/// # Arguments
/// * `input` - Input text, possibly containing sakura script tags
/// * `widths` - Per-line width thresholds (CJK character widths).
///   `widths[0]` for line 1, `widths[1]` for line 2, last value repeats for subsequent lines.
///   Empty slice → return input unchanged.
/// * `tag_regex` - Regex matching sakura script tags (`SAKURA_TAG_PATTERN`)
/// * `model` - budoux Japanese segmentation model reference
pub fn break_lines_impl(
    input: &str,
    widths: &[usize],
    tag_regex: &Regex,
    model: &budoux::Model,
) -> String {
    if input.is_empty() || widths.is_empty() {
        return input.to_string();
    }

    // Phase 1: Tokenize into plain chars with trailing tags
    let tokens = tokenize_plain_chars(input, tag_regex);

    if tokens.chars.is_empty() {
        return input.to_string();
    }

    // Phase 2: Extract plain text and segment with budoux
    let plaintext: String = tokens.chars.iter().map(|pc| pc.ch).collect();
    let words = budoux::parse(model, &plaintext);

    // Phase 3: Determine break positions using width thresholds
    // break_positions stores char indices (in tokens.chars) where a line break
    // should be inserted BEFORE that char.
    let mut break_positions = Vec::new();
    let mut line_width: usize = 0;
    let mut line_idx: usize = 0;
    let mut char_idx: usize = 0;

    for word in &words {
        let word_width = UnicodeWidthStr::width_cjk(word.as_str());
        let threshold = widths[line_idx.min(widths.len() - 1)];

        if char_idx > 0 && line_width + word_width > threshold {
            // Word doesn't fit — insert break before this word
            // But only if the line already has content (don't break before first word)
            if line_width > 0 {
                break_positions.push(char_idx);
                line_idx += 1;
                line_width = word_width;
            } else {
                // Single word exceeds threshold — don't force-break, just continue
                line_width += word_width;
            }
        } else {
            line_width += word_width;
        }

        char_idx += word.chars().count();
    }

    // Phase 4: Reconstruct output with line breaks
    let mut result = String::with_capacity(input.len() + break_positions.len() * 2);
    result.push_str(tokens.leading);

    let mut bp_idx = 0;
    for (i, pc) in tokens.chars.iter().enumerate() {
        // Insert line break before this char if needed
        if bp_idx < break_positions.len() && i == break_positions[bp_idx] {
            result.push_str("\\n");
            bp_idx += 1;
        }

        result.push(pc.ch);
        result.push_str(pc.trailing);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tokenizer::Tokenizer;

    fn tag_regex() -> Regex {
        Regex::new(Tokenizer::SAKURA_TAG_PATTERN).unwrap()
    }

    fn model() -> &'static budoux::Model {
        budoux::models::default_japanese_model()
    }

    // --- Test 1: Plain text line breaking ---

    #[test]
    fn test_plain_japanese_text_breaks_at_word_boundary() {
        let re = tag_regex();
        let m = model();
        // "今日はいい天気ですね" — budoux should find natural boundaries.
        // We use a narrow width to force at least one break.
        let input = "今日はいい天気ですね";
        let result = break_lines_impl(input, &[6], &re, m);
        // Should contain at least one \n and preserve all original chars
        assert!(result.contains("\\n"), "Expected line break in: {}", result);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input, "Plain text should be preserved");
    }

    #[test]
    fn test_plain_text_no_break_when_fits() {
        let re = tag_regex();
        let m = model();
        let input = "短い";
        let result = break_lines_impl(input, &[20], &re, m);
        assert_eq!(result, "短い");
    }

    // --- Test 2: Sakura script tag transparency ---

    #[test]
    fn test_tags_excluded_from_width_and_preserved() {
        let re = tag_regex();
        let m = model();
        // Example from requirements: こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト
        // Plain text: これはテスト (6 CJK chars = width 12), threshold 6 → break after ~3 chars
        let input = r"こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト";
        let result = break_lines_impl(input, &[6], &re, m);
        // Should have a \n somewhere, and all \_w[50] tags should remain
        assert!(result.contains("\\n"), "Expected line break in: {}", result);
        assert_eq!(result.matches(r"\_w[50]").count(), 5, "All 5 wait tags must be preserved");
    }

    #[test]
    fn test_leading_tags_preserved() {
        let re = tag_regex();
        let m = model();
        let input = r"\h\s[0]こんにちは世界";
        let result = break_lines_impl(input, &[6], &re, m);
        assert!(result.starts_with(r"\h\s[0]"), "Leading tags should be preserved: {}", result);
    }

    // --- Test 3: Multiple width thresholds ---

    #[test]
    fn test_multiple_width_thresholds() {
        let re = tag_regex();
        let m = model();
        // Build a long string that forces multiple lines
        // Use simple characters to be predictable
        let input = "あいうえおかきくけこさしすせそたちつてと";
        let result = break_lines_impl(input, &[4, 6], &re, m);
        // Should contain line breaks
        assert!(result.contains("\\n"), "Expected line breaks: {}", result);
        // Plain text should be preserved
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input);
    }

    // --- Test 4: Empty input / empty widths ---

    #[test]
    fn test_empty_input_returns_empty() {
        let re = tag_regex();
        let m = model();
        let result = break_lines_impl("", &[10], &re, m);
        assert_eq!(result, "");
    }

    #[test]
    fn test_empty_widths_returns_input_unchanged() {
        let re = tag_regex();
        let m = model();
        let input = "テスト文字列";
        let result = break_lines_impl(input, &[], &re, m);
        assert_eq!(result, input);
    }

    // --- Test 5: Oversized single word ---

    #[test]
    fn test_single_oversized_word_no_forced_break() {
        let re = tag_regex();
        let m = model();
        // A word larger than threshold → should not be force-split
        let input = "超長い一語";
        let result = break_lines_impl(input, &[2], &re, m);
        // Plain text should be fully preserved
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input);
    }

    #[test]
    fn test_oversized_word_then_normal() {
        let re = tag_regex();
        let m = model();
        // "超超超超超超超超短い" — budoux should split, oversized first part then normal
        let input = "あいうえおかきくけこさしすせそ短い文";
        let result = break_lines_impl(input, &[4], &re, m);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input, "Plain text must be preserved");
    }

    // --- Test 6: Existing \n in input ---

    #[test]
    fn test_existing_newline_tag_preserved() {
        let re = tag_regex();
        let m = model();
        let input = r"あいう\nえおか";
        let result = break_lines_impl(input, &[20], &re, m);
        // With wide threshold, no extra breaks needed; existing \n should remain
        assert!(result.contains(r"\n"), "Existing \\n should be preserved: {}", result);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, "あいうえおか");
    }

    // --- Test 3b: Last width repeats for subsequent lines ---

    #[test]
    fn test_last_width_repeats_for_subsequent_lines() {
        let re = tag_regex();
        let m = model();
        let input = "あいうえおかきくけこさしすせそたちつてと";
        // [4, 4] と [4, 4, 4] は最後の値が同じ (4) なので同一結果になること
        // widths[line_idx.min(widths.len()-1)] の実装を直接検証する
        let result_len2 = break_lines_impl(input, &[4, 4], &re, m);
        let result_len3 = break_lines_impl(input, &[4, 4, 4], &re, m);
        assert_eq!(
            result_len2, result_len3,
            "[4,4] と [4,4,4] は等価であること（最後の値繰り返し）:\n{}\nvs\n{}",
            result_len2, result_len3
        );
    }

    #[test]
    fn test_wider_last_width_produces_fewer_breaks() {
        let re = tag_regex();
        let m = model();
        let input = "あいうえおかきくけこさしすせそたちつてと";
        // [4, 4]: 全行 width=4 → 改行数多め
        let result_narrow = break_lines_impl(input, &[4, 4], &re, m);
        // [4, 20]: 1行目 width=4、2行目以降 width=20 (繰り返し) → 改行数少なめ
        let result_wide = break_lines_impl(input, &[4, 20], &re, m);
        let breaks_narrow = result_narrow.matches("\\n").count();
        let breaks_wide = result_wide.matches("\\n").count();
        assert!(
            breaks_narrow >= breaks_wide,
            "最後の値が広い方が改行数 ≤ であること: [4,4]={} [4,20]={}",
            breaks_narrow, breaks_wide
        );
        // どちらも平文が保持されること
        let plain_narrow: String = re.replace_all(&result_narrow, "").into_owned();
        let plain_wide: String = re.replace_all(&result_wide, "").into_owned();
        assert_eq!(plain_narrow, input);
        assert_eq!(plain_wide, input);
    }

    // --- Tokenizer tests ---

    #[test]
    fn test_tokenize_plain_only() {
        let re = tag_regex();
        let tokens = tokenize_plain_chars("abc", &re);
        assert_eq!(tokens.leading, "");
        assert_eq!(tokens.chars.len(), 3);
        assert_eq!(tokens.chars[0].ch, 'a');
        assert_eq!(tokens.chars[0].trailing, "");
    }

    #[test]
    fn test_tokenize_with_tags() {
        let re = tag_regex();
        let tokens = tokenize_plain_chars(r"\hこ\_w[50]ん", &re);
        assert_eq!(tokens.leading, r"\h");
        assert_eq!(tokens.chars.len(), 2);
        assert_eq!(tokens.chars[0].ch, 'こ');
        assert_eq!(tokens.chars[0].trailing, r"\_w[50]");
        assert_eq!(tokens.chars[1].ch, 'ん');
        assert_eq!(tokens.chars[1].trailing, "");
    }

    #[test]
    fn test_tokenize_tags_only() {
        let re = tag_regex();
        let tokens = tokenize_plain_chars(r"\h\s[0]", &re);
        assert_eq!(tokens.chars.len(), 0);
    }
}
