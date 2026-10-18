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
    let tag_ranges: Vec<(usize, usize)> = tag_regex
        .find_iter(input)
        .map(|m| (m.start(), m.end()))
        .collect();
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
/// * `parser` - budoux Japanese segmentation parser reference
pub fn break_lines_impl(
    input: &str,
    widths: &[usize],
    tag_regex: &Regex,
    parser: &budouy::Parser,
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
    let words = parser.parse(&plaintext);

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
    use super::super::tokenizer::Tokenizer;
    use super::*;

    fn tag_regex() -> Regex {
        Regex::new(Tokenizer::SAKURA_TAG_PATTERN).unwrap()
    }

    fn parser() -> budouy::Parser {
        budouy::model::load_default_japanese_parser()
    }

    // --- Test 1: Plain text line breaking ---

    #[test]
    fn test_plain_japanese_text_breaks_at_word_boundary() {
        let re = tag_regex();
        let p = parser();
        // "今日はいい天気ですね" — budoux should find natural boundaries.
        // We use a narrow width to force at least one break.
        let input = "今日はいい天気ですね";
        let result = break_lines_impl(input, &[6], &re, &p);
        // Should contain at least one \n and preserve all original chars
        assert!(result.contains("\\n"), "Expected line break in: {}", result);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input, "Plain text should be preserved");
    }

    #[test]
    fn test_plain_text_no_break_when_fits() {
        let re = tag_regex();
        let p = parser();
        let input = "短い";
        let result = break_lines_impl(input, &[20], &re, &p);
        assert_eq!(result, "短い");
    }

    // --- Test 2: Sakura script tag transparency ---

    #[test]
    fn test_tags_excluded_from_width_and_preserved() {
        let re = tag_regex();
        let p = parser();
        // Example from requirements: こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト
        // Plain text: これはテスト (6 CJK chars = width 12), threshold 6 → break after ~3 chars
        let input = r"こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト";
        let result = break_lines_impl(input, &[6], &re, &p);
        // Should have a \n somewhere, and all \_w[50] tags should remain
        assert!(result.contains("\\n"), "Expected line break in: {}", result);
        assert_eq!(
            result.matches(r"\_w[50]").count(),
            5,
            "All 5 wait tags must be preserved"
        );
    }

    #[test]
    fn test_leading_tags_preserved() {
        let re = tag_regex();
        let p = parser();
        let input = r"\h\s[0]こんにちは世界";
        let result = break_lines_impl(input, &[6], &re, &p);
        assert!(
            result.starts_with(r"\h\s[0]"),
            "Leading tags should be preserved: {}",
            result
        );
    }

    // --- Test 3: Multiple width thresholds ---

    #[test]
    fn test_multiple_width_thresholds() {
        let re = tag_regex();
        let p = parser();
        // Build a long string that forces multiple lines
        // Use simple characters to be predictable
        let input = "私は今日学校へ行って友達と一緒に勉強をしました";
        let result = break_lines_impl(input, &[4, 6], &re, &p);
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
        let p = parser();
        let result = break_lines_impl("", &[10], &re, &p);
        assert_eq!(result, "");
    }

    #[test]
    fn test_empty_widths_returns_input_unchanged() {
        let re = tag_regex();
        let p = parser();
        let input = "テスト文字列";
        let result = break_lines_impl(input, &[], &re, &p);
        assert_eq!(result, input);
    }

    // --- Test 5: Oversized single word ---

    #[test]
    fn test_single_oversized_word_no_forced_break() {
        let re = tag_regex();
        let p = parser();
        // A word larger than threshold → should not be force-split
        let input = "超長い一語";
        let result = break_lines_impl(input, &[2], &re, &p);
        // Plain text should be fully preserved
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input);
    }

    #[test]
    fn test_oversized_word_then_normal() {
        let re = tag_regex();
        let p = parser();
        // "超超超超超超超超短い" — budoux should split, oversized first part then normal
        let input = "あいうえおかきくけこさしすせそ短い文";
        let result = break_lines_impl(input, &[4], &re, &p);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input, "Plain text must be preserved");
    }

    // --- Test 6: Existing \n in input ---

    #[test]
    fn test_existing_newline_tag_preserved() {
        let re = tag_regex();
        let p = parser();
        let input = r"あいう\nえおか";
        let result = break_lines_impl(input, &[20], &re, &p);
        // With wide threshold, no extra breaks needed; existing \n should remain
        assert!(
            result.contains(r"\n"),
            "Existing \\n should be preserved: {}",
            result
        );
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, "あいうえおか");
    }

    // --- Test 3b: Last width repeats for subsequent lines ---

    #[test]
    fn test_last_width_repeats_for_subsequent_lines() {
        let re = tag_regex();
        let p = parser();
        let input = "私は今日学校へ行って友達と一緒に勉強をしました";
        // [4, 4] と [4, 4, 4] は最後の値が同じ (4) なので同一結果になること
        // widths[line_idx.min(widths.len()-1)] の実装を直接検証する
        let result_len2 = break_lines_impl(input, &[4, 4], &re, &p);
        let result_len3 = break_lines_impl(input, &[4, 4, 4], &re, &p);
        assert_eq!(
            result_len2, result_len3,
            "[4,4] と [4,4,4] は等価であること（最後の値繰り返し）:\n{}\nvs\n{}",
            result_len2, result_len3
        );
    }

    #[test]
    fn test_wider_last_width_produces_fewer_breaks() {
        let re = tag_regex();
        let p = parser();
        let input = "私は今日学校へ行って友達と一緒に勉強をしました";
        // [4, 4]: 全行 width=4 → 改行数多め
        let result_narrow = break_lines_impl(input, &[4, 4], &re, &p);
        // [4, 20]: 1行目 width=4、2行目以降 width=20 (繰り返し) → 改行数少なめ
        let result_wide = break_lines_impl(input, &[4, 20], &re, &p);
        let breaks_narrow = result_narrow.matches("\\n").count();
        let breaks_wide = result_wide.matches("\\n").count();
        assert!(
            breaks_narrow >= breaks_wide,
            "最後の値が広い方が改行数 ≤ であること: [4,4]={} [4,20]={}",
            breaks_narrow,
            breaks_wide
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

    // --- `\\`（エスケープされた `\`）は割れない (Requirement 4.5) ---

    /// `talk_to_script` と同じ順（トークナイズ → ウェイト挿入 → 改行推定）で組み立てる
    fn pipeline(input: &str, widths: &[usize]) -> String {
        use super::super::wait_inserter::{WaitValues, insert_waits};
        let tokenizer = Tokenizer::new(&crate::loader::TalkConfig::default()).unwrap();
        let waits = WaitValues {
            normal: 100,
            period: 1000,
            comma: 500,
            strong: 500,
            leader: 200,
        };
        let waited = insert_waits(&tokenizer.tokenize(input), &waits);
        break_lines_impl(&waited, widths, tokenizer.tag_regex(), &parser())
    }

    /// さくらスクリプトの規約どおり左から読み、表示される単位を返す。
    /// `\\` は 1 単位、`\_w[…]` と `\n` は読み捨て、それ以外は 1 文字ずつ。
    /// 検査対象の正規表現に依存しない独立した読み手。
    fn displayed_units(s: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = s;
        while let Some(c) = rest.chars().next() {
            if let Some(after) = rest.strip_prefix(r"\\") {
                out.push(r"\\".to_string());
                rest = after;
            } else if let Some(after) = rest.strip_prefix(r"\n") {
                rest = after;
            } else if let Some(after) = rest.strip_prefix(r"\_w[") {
                rest = &after[after.find(']').unwrap() + 1..];
            } else {
                out.push(c.to_string());
                rest = &rest[c.len_utf8()..];
            }
        }
        out
    }

    #[test]
    fn test_escaped_backslash_gets_no_wait_inside() {
        // ウェイトのみ（改行推定なし）。`\\` の 2 文字の間・直後にウェイトが入らない
        assert_eq!(
            pipeline(r"ab\\cd", &[]),
            r"a\_w[50]b\_w[50]\\c\_w[50]d\_w[50]"
        );
        assert_eq!(pipeline(r"a\\\\", &[]), r"a\_w[50]\\\\");
        assert_eq!(pipeline(r"a\\\n", &[]), r"a\_w[50]\\\n");
    }

    #[test]
    fn test_escaped_backslash_not_split_with_waits_and_line_breaks() {
        let input = r"C:\\new\\s[0]フォルダ\\\\の中に\\あるファイルを今日は開きます\\";
        for width in 1..=12 {
            let result = pipeline(input, &[width]);
            assert_eq!(
                displayed_units(&result),
                displayed_units(input),
                "width={} で `\\\\` が割れた: {}",
                width,
                result
            );
        }
        // 改行推定が実際に働いていること（狭い幅では改行が足される）
        assert_ne!(pipeline(input, &[1]), pipeline(input, &[]));
    }

    // --- 特性化: 列を動かすタグを含まない入力の今の改行の位置 (Requirement 6.2・6.4・7.2) ---

    /// 特性化テストで使う幅。期待値の並びはこの順。
    const CHARACTERIZATION_WIDTHS: [&[usize]; 4] = [&[4], &[4, 8], &[6], &[10, 12]];

    fn direct(input: &str, widths: &[usize]) -> String {
        break_lines_impl(input, widths, &tag_regex(), &parser())
    }

    /// 数え方を直す前の出力を文字列のまま固定する。直した後も同じ文字列になること。
    /// 入力は今の単体テストの入力・ウェイトつき（`pipeline`）・囲み（`\_?…\_?`・`\_!…\_!`）と
    /// エスケープ（`\\`）つき。囲みの中の `\n`・`\c`・`\_l` は列を動かすタグではない。
    #[test]
    fn test_characterization_without_column_moving_tags() {
        type Convert = fn(&str, &[usize]) -> String;
        let cases: [(Convert, &str, [&str; 4]); 12] = [
            (
                direct,
                r"今日はいい天気ですね",
                [
                    r"今日は\nいい\n天気ですね",
                    r"今日は\nいい\n天気ですね",
                    r"今日は\nいい\n天気ですね",
                    r"今日はいい\n天気ですね",
                ],
            ),
            (direct, r"短い", [r"短い", r"短い", r"短い", r"短い"]),
            (
                direct,
                r"こ\_w[50]れ\_w[50]は\_w[50]テ\_w[50]ス\_w[50]ト",
                [
                    r"こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト",
                    r"こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト",
                    r"こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト",
                    r"こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト",
                ],
            ),
            (
                direct,
                r"\h\s[0]こんにちは世界",
                [
                    r"\h\s[0]こんに\nちは\n世界",
                    r"\h\s[0]こんに\nちは世界",
                    r"\h\s[0]こんに\nちは\n世界",
                    r"\h\s[0]こんにちは\n世界",
                ],
            ),
            (
                direct,
                r"私は今日学校へ行って友達と一緒に勉強をしました",
                [
                    r"私は\n今日学校へ\n行って\n友達と\n一緒に\n勉強を\nしました",
                    r"私は\n今日学校へ\n行って\n友達と\n一緒に\n勉強を\nしました",
                    r"私は\n今日学校へ\n行って\n友達と\n一緒に\n勉強を\nしました",
                    r"私は\n今日学校へ\n行って友達と\n一緒に勉強を\nしました",
                ],
            ),
            (
                direct,
                r"超長い一語",
                [
                    r"超長い\n一語",
                    r"超長い\n一語",
                    r"超長い\n一語",
                    r"超長い一語",
                ],
            ),
            (
                direct,
                r"あいうえおかきくけこさしすせそ短い文",
                [
                    r"あいうえおかきくけこさしすせそ短い\n文",
                    r"あいうえおかきくけこさしすせそ短い\n文",
                    r"あいうえおかきくけこさしすせそ短い\n文",
                    r"あいうえおかきくけこさしすせそ短い\n文",
                ],
            ),
            (
                direct,
                r"C:\\new\\s[0]フォルダ\\\\の中に\\あるファイルを今日は開きます\\",
                [
                    r"C:\\new\\s[0]フォルダ\\\\の\n中に\\\nある\nファイルを\n今日は\n開きます\\",
                    r"C:\\new\\s[0]フォルダ\\\\の\n中に\\ある\nファイルを\n今日は\n開きます\\",
                    r"C:\\new\\s[0]フォルダ\\\\の\n中に\\\nある\nファイルを\n今日は\n開きます\\",
                    r"C:\\new\\s[0]フォルダ\\\\の\n中に\\ある\nファイルを\n今日は\n開きます\\",
                ],
            ),
            (
                direct,
                r"今日は\_?\n\c\_l[0,0]\_?いい天気ですね\_!\n\_!そうですね",
                [
                    r"今日は\_?\n\c\_l[0,0]\_?\nいい\n天気ですね\_!\n\_!そうですね",
                    r"今日は\_?\n\c\_l[0,0]\_?\nいい\n天気ですね\_!\n\_!そうですね",
                    r"今日は\_?\n\c\_l[0,0]\_?\nいい\n天気ですね\_!\n\_!そうですね",
                    r"今日は\_?\n\c\_l[0,0]\_?いい\n天気ですね\_!\n\_!そうですね",
                ],
            ),
            (
                pipeline,
                r"「今日は、いい天気ですね。」と彼は言った",
                [
                    r"「今\_w[50]日\_w[50]は\_w[50]、\_w[450]\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]。」\_w[950]と\_w[50]\n彼\_w[50]は\_w[50]\n言\_w[50]っ\_w[50]た\_w[50]",
                    r"「今\_w[50]日\_w[50]は\_w[50]、\_w[450]\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]。」\_w[950]と\_w[50]\n彼\_w[50]は\_w[50]\n言\_w[50]っ\_w[50]た\_w[50]",
                    r"「今\_w[50]日\_w[50]は\_w[50]、\_w[450]\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]。」\_w[950]と\_w[50]\n彼\_w[50]は\_w[50]\n言\_w[50]っ\_w[50]た\_w[50]",
                    r"「今\_w[50]日\_w[50]は\_w[50]、\_w[450]\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]。」\_w[950]と\_w[50]\n彼\_w[50]は\_w[50]言\_w[50]っ\_w[50]た\_w[50]",
                ],
            ),
            (
                pipeline,
                r"C:\\new\\s[0]フォルダ\\\\の中に\\あるファイルを今日は開きます\\",
                [
                    r"C\_w[50]:\\n\_w[50]e\_w[50]w\_w[50]\\s\_w[50][0\_w[50]]フ\_w[50]ォ\_w[50]ル\_w[50]ダ\_w[50]\\\\の\_w[50]\n中\_w[50]に\_w[50]\\\nあ\_w[50]る\_w[50]\nフ\_w[50]ァ\_w[50]イ\_w[50]ル\_w[50]を\_w[50]\n今\_w[50]日\_w[50]は\_w[50]\n開\_w[50]き\_w[50]ま\_w[50]す\_w[50]\\",
                    r"C\_w[50]:\\n\_w[50]e\_w[50]w\_w[50]\\s\_w[50][0\_w[50]]フ\_w[50]ォ\_w[50]ル\_w[50]ダ\_w[50]\\\\の\_w[50]\n中\_w[50]に\_w[50]\\あ\_w[50]る\_w[50]\nフ\_w[50]ァ\_w[50]イ\_w[50]ル\_w[50]を\_w[50]\n今\_w[50]日\_w[50]は\_w[50]\n開\_w[50]き\_w[50]ま\_w[50]す\_w[50]\\",
                    r"C\_w[50]:\\n\_w[50]e\_w[50]w\_w[50]\\s\_w[50][0\_w[50]]フ\_w[50]ォ\_w[50]ル\_w[50]ダ\_w[50]\\\\の\_w[50]\n中\_w[50]に\_w[50]\\\nあ\_w[50]る\_w[50]\nフ\_w[50]ァ\_w[50]イ\_w[50]ル\_w[50]を\_w[50]\n今\_w[50]日\_w[50]は\_w[50]\n開\_w[50]き\_w[50]ま\_w[50]す\_w[50]\\",
                    r"C\_w[50]:\\n\_w[50]e\_w[50]w\_w[50]\\s\_w[50][0\_w[50]]フ\_w[50]ォ\_w[50]ル\_w[50]ダ\_w[50]\\\\の\_w[50]\n中\_w[50]に\_w[50]\\あ\_w[50]る\_w[50]\nフ\_w[50]ァ\_w[50]イ\_w[50]ル\_w[50]を\_w[50]\n今\_w[50]日\_w[50]は\_w[50]\n開\_w[50]き\_w[50]ま\_w[50]す\_w[50]\\",
                ],
            ),
            (
                pipeline,
                r"今日は\_?\n\c\_l[0,0]\_?いい天気ですね\_!\n\_!そうですね",
                [
                    r"今\_w[50]日\_w[50]は\_w[50]\_?\n\c\_l[0,0]\_?\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]\_!\n\_!そ\_w[50]う\_w[50]で\_w[50]す\_w[50]ね\_w[50]",
                    r"今\_w[50]日\_w[50]は\_w[50]\_?\n\c\_l[0,0]\_?\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]\_!\n\_!そ\_w[50]う\_w[50]で\_w[50]す\_w[50]ね\_w[50]",
                    r"今\_w[50]日\_w[50]は\_w[50]\_?\n\c\_l[0,0]\_?\nい\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]\_!\n\_!そ\_w[50]う\_w[50]で\_w[50]す\_w[50]ね\_w[50]",
                    r"今\_w[50]日\_w[50]は\_w[50]\_?\n\c\_l[0,0]\_?い\_w[50]い\_w[50]\n天\_w[50]気\_w[50]で\_w[50]す\_w[50]ね\_w[50]\_!\n\_!そ\_w[50]う\_w[50]で\_w[50]す\_w[50]ね\_w[50]",
                ],
            ),
        ];
        for (convert, input, expected) in cases {
            for (widths, want) in CHARACTERIZATION_WIDTHS.iter().zip(expected) {
                assert_eq!(
                    convert(input, widths),
                    want,
                    "input={} widths={:?}",
                    input,
                    widths
                );
            }
        }
    }
}
