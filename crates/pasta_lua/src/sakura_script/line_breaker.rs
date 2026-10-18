//! BudouX-based line breaker for sakura script text.
//!
//! Inserts sakura script line break tags (`\n`) at natural Japanese word
//! boundaries while transparently preserving sakura script tags.

use regex::Regex;
use unicode_width::UnicodeWidthChar;

/// 列の状態。Lua の表 { line, width, jump } と 1 対 1 に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    /// 列の番号。最初の列が 1。
    pub line: usize,
    /// その列に積んだ字数（全角 2・半角 1。タグは数えない）。
    pub width: usize,
    /// `budoux_jump` の並びを使っているか。
    pub jump: bool,
}

impl Default for Column {
    /// 最初の列の頭・字数 0・`budoux` の並び。
    fn default() -> Self {
        Self {
            line: 1,
            width: 0,
            jump: false,
        }
    }
}

/// 改行の規則（アクターの設定から作る）。
pub struct BreakRule<'a> {
    /// `budoux` の並び。空なら改行を入れない（状態は進める）。
    pub widths: &'a [usize],
    /// `budoux_jump` の並び。空なら `\_l[…]` は字数だけを 0 にする。
    pub jump_widths: &'a [usize],
    /// budoux が入れる改行の文字列（`\n` または `\n[N]`）。
    pub newline: &'a str,
}

/// 列の動き（単位 1 つが列をどう動かすか）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnMove {
    /// `\n` ちょうど・`\n[` で始まる単位。次の列へ。
    Next,
    /// `\c` ちょうどの単位。最初の列へ。
    First,
    /// `\_l[` で始まる単位。飛ぶ。
    Jump,
}

/// 単位の文字列から列の動きを決める。列を動かさない単位は None。
///
/// 単位は `SAKURA_TAG_PATTERN` の一致 1 つ。囲み（`\_?…\_?`・`\_!…\_!`）と
/// エスケープ（`\\`）は先頭が違うので、中に `\n` などがあっても動かない（2.9）。
fn column_move(unit: &str) -> Option<ColumnMove> {
    if unit == r"\n" || unit.starts_with(r"\n[") {
        Some(ColumnMove::Next)
    } else if unit == r"\c" {
        Some(ColumnMove::First)
    } else if unit.starts_with(r"\_l[") {
        Some(ColumnMove::Jump)
    } else {
        None
    }
}

impl Column {
    /// 列の動きを当てる。どの動きも字数を 0 にする。
    /// budoux が入れた改行は `Next` を当ててから、字数に次の語のかけらの幅を入れる。
    fn apply(&mut self, m: ColumnMove, rule: &BreakRule) {
        match m {
            ColumnMove::Next => self.line += 1,
            ColumnMove::First => {
                self.line = 1;
                self.jump = false;
            }
            ColumnMove::Jump if !rule.jump_widths.is_empty() => {
                self.line = 1;
                self.jump = true;
            }
            ColumnMove::Jump => {}
        }
        self.width = 0;
    }

    /// 今の列の幅の上限。並びの最後の値を、それ以降の列で繰り返す。
    /// `rule.widths` が空でないときだけ呼ぶ。
    fn threshold(&self, rule: &BreakRule) -> usize {
        let list = if self.jump && !rule.jump_widths.is_empty() {
            rule.jump_widths
        } else {
            rule.widths
        };
        list[(self.line.max(1) - 1).min(list.len() - 1)]
    }
}

/// A plain text character paired with trailing sakura script tags.
struct PlainChar<'a> {
    ch: char,
    trailing: &'a str,
    /// `trailing` の単位の列の動き（順に）。
    moves: Vec<ColumnMove>,
}

/// Tokenized representation separating plain text from tags.
struct Tokens<'a> {
    leading: &'a str,
    /// `leading` の単位の列の動き（順に）。
    leading_moves: Vec<ColumnMove>,
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
    let mut leading_moves = Vec::new();

    // Collect all tag match ranges
    let tag_ranges: Vec<(usize, usize)> = tag_regex
        .find_iter(input)
        .map(|m| (m.start(), m.end()))
        .collect();
    let mut tag_idx = 0;

    while pos < input.len() {
        // Check if current position is inside a tag
        if tag_idx < tag_ranges.len() && pos == tag_ranges[tag_idx].0 {
            leading_moves.extend(column_move(&input[pos..tag_ranges[tag_idx].1]));
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
            let mut moves = Vec::new();
            // Skip over any tags immediately after this char
            loop {
                if tag_idx < tag_ranges.len() && trailing_end == tag_ranges[tag_idx].0 {
                    moves.extend(column_move(&input[trailing_end..tag_ranges[tag_idx].1]));
                    trailing_end = tag_ranges[tag_idx].1;
                    tag_idx += 1;
                } else {
                    break;
                }
            }

            chars.push(PlainChar {
                ch: c,
                trailing: &input[char_end..trailing_end],
                moves,
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

    Tokens {
        leading,
        leading_moves,
        chars,
    }
}

/// budoux の改行を入れる。`column` を始めの状態として読み、終わりの状態に書き換える。
///
/// 先頭の単位の列の動きを当て、語を列の動きのある所でかけらに割って先頭から見る。
/// 語の頭のかけらで、列に字があり上限を超えるなら `rule.newline` を入れて次の列へ進み、
/// かけらの最後の字の後ろの単位の列の動きを順に当てる。タグの後ろのかけらは字数に足すだけ。
/// タグは字数に数えない。`rule.widths` が空なら語に分けず、入力をそのまま返して
/// 列の状態だけ進める。
pub fn break_lines_impl(
    input: &str,
    rule: &BreakRule,
    column: &mut Column,
    tag_regex: &Regex,
    parser: &budouy::Parser,
) -> String {
    column.line = column.line.max(1);
    let tokens = tokenize_plain_chars(input, tag_regex);
    for &m in &tokens.leading_moves {
        column.apply(m, rule);
    }

    let width_of = |pc: &PlainChar| UnicodeWidthChar::width_cjk(pc.ch).unwrap_or(0);

    if rule.widths.is_empty() || tokens.chars.is_empty() {
        for pc in &tokens.chars {
            column.width += width_of(pc);
            for &m in &pc.moves {
                column.apply(m, rule);
            }
        }
        return input.to_string();
    }

    // 本文の字をつないで budoux で語に分ける
    let plaintext: String = tokens.chars.iter().map(|pc| pc.ch).collect();
    let words = parser.parse(&plaintext);

    // break_positions は、その字の前に改行を入れる tokens.chars の添字
    let mut break_positions: Vec<usize> = Vec::new();
    let mut char_idx: usize = 0;

    for word in &words {
        let word_end = char_idx + word.chars().count();
        // 語を、字の後ろに列の動きがある所で「かけら」に割る
        let mut head = true;
        for frag in tokens.chars[char_idx..word_end].split_inclusive(|pc| !pc.moves.is_empty()) {
            let frag_width: usize = frag.iter().map(width_of).sum();
            // 語の頭のかけらだけ: 列に字があり、足すと上限を超えるなら前で割る（1 語で超えても割らない）。
            // タグの後ろのかけらは字数 0 の列の頭なので、足すだけ
            if head && column.width > 0 && column.width + frag_width > column.threshold(rule) {
                // 単独の `\` の直後に入れると `\\` に化けるので、`\` と語を 1 語として
                // `\` の前で割る（決定 9）。`\` は今の列に数えてあるので新しい列へ移す。
                // `\` が列の頭（その前に字が無い）なら、割ると字の無い列ができるので割らない
                let lone = char_idx
                    .checked_sub(1)
                    .map(|i| &tokens.chars[i])
                    .filter(|prev| prev.ch == '\\' && prev.trailing.is_empty());
                match lone {
                    Some(prev) if column.width > width_of(prev) => {
                        break_positions.push(char_idx - 1);
                        column.apply(ColumnMove::Next, rule);
                        column.width = width_of(prev);
                    }
                    Some(_) => {}
                    None => {
                        break_positions.push(char_idx);
                        column.apply(ColumnMove::Next, rule);
                    }
                }
            }
            head = false;
            column.width += frag_width;
            for &m in &frag[frag.len() - 1].moves {
                column.apply(m, rule);
            }
        }
        char_idx = word_end;
    }

    let mut result =
        String::with_capacity(input.len() + break_positions.len() * rule.newline.len());
    result.push_str(tokens.leading);

    let mut bp_idx = 0;
    for (i, pc) in tokens.chars.iter().enumerate() {
        if bp_idx < break_positions.len() && i == break_positions[bp_idx] {
            result.push_str(rule.newline);
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

    /// 新しい状態・`\n`・飛んだ後の並び無しで呼ぶ（今の呼び出し口と同じ包み方）。
    fn break_fresh(input: &str, widths: &[usize], re: &Regex, p: &budouy::Parser) -> String {
        break_from(input, widths, &mut Column::default(), re, p)
    }

    fn break_from(
        input: &str,
        widths: &[usize],
        column: &mut Column,
        re: &Regex,
        p: &budouy::Parser,
    ) -> String {
        let rule = BreakRule {
            widths,
            jump_widths: &[],
            newline: r"\n",
        };
        break_lines_impl(input, &rule, column, re, p)
    }

    // --- 列の状態を受け取って返す (Requirement 1.4, 2.1, 2.2, 2.3, 3.2, 6.1) ---

    fn col(line: usize, width: usize) -> Column {
        Column {
            line,
            width,
            jump: false,
        }
    }

    #[test]
    fn test_newline_tag_moves_to_next_width() {
        // 今日は|いい|天気ですね。`\n` の後は 2 列目の 20 で数える
        let mut c = Column::default();
        let out = break_from(
            r"今日は\nいい天気ですね",
            &[4, 20],
            &mut c,
            &tag_regex(),
            &parser(),
        );
        assert_eq!(out, r"今日は\nいい天気ですね");
        assert_eq!(c, col(2, 14));
    }

    #[test]
    fn test_clear_tag_moves_to_first_width() {
        // 3 列目（20）から始めても、`\c` の後は先頭の 4 で数える
        let mut c = col(3, 6);
        let out = break_from(
            r"\c今日はいい天気ですね",
            &[4, 20],
            &mut c,
            &tag_regex(),
            &parser(),
        );
        assert_eq!(out, r"\c今日は\nいい天気ですね");
        assert_eq!(c, col(2, 14));
    }

    #[test]
    fn test_jump_tag_resets_width_only() {
        // `\_l` の後は字数だけ 0。列は 1 列目のまま（幅 4）
        let mut c = Column::default();
        let out = break_from(
            r"今日は\_l[0,0]いい天気ですね",
            &[4, 20],
            &mut c,
            &tag_regex(),
            &parser(),
        );
        assert_eq!(out, r"今日は\_l[0,0]いい\n天気ですね");
        assert_eq!(c, col(2, 10));
    }

    #[test]
    fn test_empty_widths_returns_input_and_advances_column() {
        let mut c = col(1, 3);
        let input = r"\hあ\nい\_w[50]う\c\nab";
        let out = break_from(input, &[], &mut c, &tag_regex(), &parser());
        assert_eq!(out, input);
        assert_eq!(c, col(2, 2));
    }

    #[test]
    fn test_end_state_carries_into_next_call() {
        let (re, p) = (tag_regex(), parser());
        let mut c = Column::default();
        assert_eq!(break_from("今日は", &[4, 20], &mut c, &re, &p), "今日は");
        assert_eq!(c, col(1, 6));
        // 持ち越した字数 6 で 1 つ目の語（4）が 1 列目（4）に収まらないので、1 つ目の語の前で割る
        assert_eq!(
            break_from("いい天気ですね", &[4, 20], &mut c, &re, &p),
            r"\nいい天気ですね"
        );
        assert_eq!(c, col(2, 14));
    }

    /// 飛んだ後の並びと改行の文字列を指定して呼ぶ。
    fn break_rule(
        input: &str,
        widths: &[usize],
        jump_widths: &[usize],
        newline: &str,
        column: &mut Column,
    ) -> String {
        let rule = BreakRule {
            widths,
            jump_widths,
            newline,
        };
        break_lines_impl(input, &rule, column, &tag_regex(), &parser())
    }

    // --- 語の途中のタグ・持ち越し・飛んだ後の並び・改行の文字列 (Requirement 1.1, 2.5, 2.6, 3.1, 3.3, 3.4, 4.6, 4.7) ---

    #[test]
    fn test_tag_inside_word_splits_into_fragments() {
        // 「空調」は budoux が 1 語にする。`\n` の後ろの「調」は字数 0 の列の頭（前に改行を入れない）
        let mut c = Column::default();
        assert_eq!(
            break_from(r"空\n調", &[2], &mut c, &tag_regex(), &parser()),
            r"空\n調"
        );
        assert_eq!(c, col(2, 2));
        // 語の頭のかけら「空」（2）だけを足して上限 8 を判定する（語全体の 4 ではない）
        let mut c = Column::default();
        assert_eq!(
            break_from(r"今日は空\n調", &[8], &mut c, &tag_regex(), &parser()),
            r"今日は空\n調"
        );
        assert_eq!(c, col(2, 2));
    }

    #[test]
    fn test_carried_width_breaks_before_first_word() {
        // 持ち越した字数 6 で 1 つ目の語が収まらない: 先頭の単位の後、1 つ目の字の前で割る
        let mut c = col(1, 6);
        assert_eq!(
            break_from(
                r"\s[1]いい天気ですね",
                &[4, 20],
                &mut c,
                &tag_regex(),
                &parser()
            ),
            r"\s[1]\nいい天気ですね"
        );
        assert_eq!(c, col(2, 14));
        // 字数 0 なら 1 つ目の語の前で割らない
        let mut c = col(1, 0);
        assert_eq!(
            break_from(
                r"\s[1]いい天気ですね",
                &[4, 20],
                &mut c,
                &tag_regex(),
                &parser()
            ),
            r"\s[1]いい\n天気ですね"
        );
        assert_eq!(c, col(2, 10));
    }

    #[test]
    fn test_jump_widths_after_jump_tag() {
        let at = |line, width, jump| Column { line, width, jump };
        // `\_l` の後の 1 列目は 40: 36 + 4 は収まり、続く 10 で割る
        let mut c = at(1, 36, true);
        assert_eq!(
            break_rule("いい天気ですね", &[4], &[40, 28], r"\n", &mut c),
            r"いい\n天気ですね"
        );
        assert_eq!(c, at(2, 10, true));
        // 2 列目以降は 28
        let mut c = at(2, 14, true);
        assert_eq!(
            break_rule("いい天気ですね", &[4], &[40, 28], r"\n", &mut c),
            "いい天気ですね"
        );
        let mut c = at(3, 18, true);
        assert_eq!(
            break_rule("いい天気ですね", &[4], &[40, 28], r"\n", &mut c),
            r"いい\n天気ですね"
        );
        // `\_l` の後ろは飛んだ後の並び（40）で数えるので割らない
        let mut c = Column::default();
        assert_eq!(
            break_rule(
                r"今日は\_l[0,0]いい天気ですね",
                &[4],
                &[40, 28],
                r"\n",
                &mut c
            ),
            r"今日は\_l[0,0]いい天気ですね"
        );
        assert_eq!(c, at(1, 14, true));
        // `\c` で `budoux` の並び（4）に戻る
        let mut c = Column::default();
        assert_eq!(
            break_rule(
                r"\_l[0,0]今日は\cいい天気ですね",
                &[4],
                &[40, 28],
                r"\n",
                &mut c
            ),
            r"\_l[0,0]今日は\cいい\n天気ですね"
        );
        assert_eq!(c, at(2, 10, false));
        // 2 度目の `\_l` で飛んだ後の並びの先頭から使い直す
        let mut c = at(3, 20, true);
        break_rule(r"\_l[0,0]いい", &[4], &[40, 28], r"\n", &mut c);
        assert_eq!(c, at(1, 4, true));
    }

    #[test]
    fn test_newline_string_follows_rule() {
        let mut c = Column::default();
        assert_eq!(
            break_rule("今日はいい天気ですね", &[4], &[], r"\n[180]", &mut c),
            r"今日は\n[180]いい\n[180]天気ですね"
        );
        assert_eq!(c, col(3, 10));
    }

    #[test]
    fn test_break_after_lone_backslash_moves_before_it() {
        // 単独の `\` の直後に改行を入れると `\\`（エスケープ）に化けるので、`\` の前に寄せる
        let input = r"天気天気\今日は";
        let out = direct(input, &[4]);
        assert_eq!(out, r"天気天気\n\今日は");
        assert!(!out.contains(r"\\n"), "改行がエスケープに化けた: {out}");
        assert_eq!(out.replace(r"\n", ""), input);
    }

    #[test]
    fn test_break_before_lone_backslash_counts_it_in_new_column() {
        // 寄せた改行の後ろの列は `\今日は`（`\` の 1 ＋ 6）
        let mut c = Column::default();
        assert_eq!(
            break_from(r"天気天気\今日は", &[4], &mut c, &tag_regex(), &parser()),
            r"天気天気\n\今日は"
        );
        assert_eq!(c, col(2, 7));
    }

    #[test]
    fn test_lone_backslash_at_column_head_joins_next_word() {
        // budoux が `\` だけで 1 語にし、その前で割った。`\` と次の語は 1 語として扱い、
        // 改行を重ねず列も進めない（1 語で上限を超えても割らない）
        let mut c = Column::default();
        assert_eq!(
            break_from(
                r"こんにちは\「今日」",
                &[4],
                &mut c,
                &tag_regex(),
                &parser()
            ),
            r"こんに\nちは\n\「今日」"
        );
        assert_eq!(c, col(3, 9));
        // 作者の `\n` の後ろの `\` も列の頭。前に寄せると字の無い列ができるので割らない
        let mut c = Column::default();
        assert_eq!(
            break_from(r"天気天気\n\今日は", &[4], &mut c, &tag_regex(), &parser()),
            r"天気天気\n\今日は"
        );
        assert_eq!(c, col(2, 7));
    }

    #[test]
    fn test_lone_backslash_state_matches_recount_of_output() {
        // どの経路でも、終わりの状態は出力を数え直した状態（幅が空なら数えるだけ）と同じ
        let (re, p) = (tag_regex(), parser());
        let inputs = [
            r"天気天気\今日は",
            r"こんにちは\「今日」",
            r"ああ\「「",
            r"天気天気\今日は今日は",
            r"天気天気\n\今日は",
            r"\今日はいい天気",
            r"今日は\",
        ];
        for input in inputs {
            for w in 1..=8 {
                for start in [0, 3] {
                    let mut c = col(1, start);
                    let out = break_from(input, &[w], &mut c, &re, &p);
                    let mut recount = col(1, start);
                    break_from(&out, &[], &mut recount, &re, &p);
                    let ctx = format!("{input} [{w}] start {start} -> {out}");
                    assert_eq!(c, recount, "{ctx}");
                    assert!(!out.contains(r"\\n"), "{ctx}");
                    assert!(!out.contains(r"\n\n"), "{ctx}");
                    assert_eq!(out.replace(r"\n", ""), input.replace(r"\n", ""), "{ctx}");
                }
            }
        }
    }

    // --- Test 1: Plain text line breaking ---

    #[test]
    fn test_plain_japanese_text_breaks_at_word_boundary() {
        let re = tag_regex();
        let p = parser();
        // "今日はいい天気ですね" — budoux should find natural boundaries.
        // We use a narrow width to force at least one break.
        let input = "今日はいい天気ですね";
        let result = break_fresh(input, &[6], &re, &p);
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
        let result = break_fresh(input, &[20], &re, &p);
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
        let result = break_fresh(input, &[6], &re, &p);
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
        let result = break_fresh(input, &[6], &re, &p);
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
        let result = break_fresh(input, &[4, 6], &re, &p);
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
        let result = break_fresh("", &[10], &re, &p);
        assert_eq!(result, "");
    }

    #[test]
    fn test_empty_widths_returns_input_unchanged() {
        let re = tag_regex();
        let p = parser();
        let input = "テスト文字列";
        let result = break_fresh(input, &[], &re, &p);
        assert_eq!(result, input);
    }

    // --- Test 5: Oversized single word ---

    #[test]
    fn test_single_oversized_word_no_forced_break() {
        let re = tag_regex();
        let p = parser();
        // A word larger than threshold → should not be force-split
        let input = "超長い一語";
        let result = break_fresh(input, &[2], &re, &p);
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
        let result = break_fresh(input, &[4], &re, &p);
        let plain: String = re.replace_all(&result, "").into_owned();
        assert_eq!(plain, input, "Plain text must be preserved");
    }

    // --- Test 6: Existing \n in input ---

    #[test]
    fn test_existing_newline_tag_preserved() {
        let re = tag_regex();
        let p = parser();
        let input = r"あいう\nえおか";
        let result = break_fresh(input, &[20], &re, &p);
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
        let result_len2 = break_fresh(input, &[4, 4], &re, &p);
        let result_len3 = break_fresh(input, &[4, 4, 4], &re, &p);
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
        let result_narrow = break_fresh(input, &[4, 4], &re, &p);
        // [4, 20]: 1行目 width=4、2行目以降 width=20 (繰り返し) → 改行数少なめ
        let result_wide = break_fresh(input, &[4, 20], &re, &p);
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
        break_fresh(&waited, widths, tokenizer.tag_regex(), &parser())
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
        break_fresh(input, widths, &tag_regex(), &parser())
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

    // --- 列を動かす単位の見分け (Requirement 2.1, 2.2, 2.8, 2.9) ---

    #[test]
    fn test_column_move_classification() {
        let moving = [
            (r"\n", ColumnMove::Next),
            (r"\n[180]", ColumnMove::Next),
            (r"\n[half]", ColumnMove::Next),
            (r"\c", ColumnMove::First),
            (r"\_l[-295,0]", ColumnMove::Jump),
        ];
        for (unit, want) in moving {
            assert_eq!(column_move(unit), Some(want), "unit={}", unit);
        }
        let still = [
            r"\c[line,1]",
            r"\x",
            r"\x[noclear]",
            r"\_q",
            r"\_w[50]",
            r"\s[1]",
            r"\0",
            r"\p[1]",
            r"\b[2]",
            r"\\",
            r"\%",
            r"\_?\n\_?",
            r"\_!\c\_!",
        ];
        for unit in still {
            assert_eq!(column_move(unit), None, "unit={}", unit);
        }
    }

    #[test]
    fn test_tokenize_records_column_moves() {
        let re = tag_regex();
        let tokens =
            tokenize_plain_chars(r"\h\c\_l[0,0]あ\_w[50]\n[half]い\_?\n\_?う\\\_l[1,1]", &re);
        assert_eq!(
            tokens.leading_moves,
            vec![ColumnMove::First, ColumnMove::Jump]
        );
        assert_eq!(tokens.chars.len(), 3);
        assert_eq!(tokens.chars[0].moves, vec![ColumnMove::Next]);
        assert_eq!(tokens.chars[1].moves, vec![]);
        assert_eq!(tokens.chars[2].moves, vec![ColumnMove::Jump]);
    }

    #[test]
    fn test_column_apply_follows_move_table() {
        let widths = [40];
        let jump_widths = [40, 28];
        let with_jump = BreakRule {
            widths: &widths,
            jump_widths: &jump_widths,
            newline: r"\n",
        };
        let no_jump = BreakRule {
            widths: &widths,
            jump_widths: &[],
            newline: r"\n",
        };
        let at = |line, width, jump| Column { line, width, jump };

        assert_eq!(Column::default(), at(1, 0, false));

        let mut c = at(2, 7, true);
        c.apply(ColumnMove::Next, &with_jump);
        assert_eq!(c, at(3, 0, true));

        let mut c = at(3, 7, true);
        c.apply(ColumnMove::First, &with_jump);
        assert_eq!(c, at(1, 0, false));

        let mut c = at(3, 7, false);
        c.apply(ColumnMove::Jump, &with_jump);
        assert_eq!(c, at(1, 0, true));

        let mut c = at(3, 7, false);
        c.apply(ColumnMove::Jump, &no_jump);
        assert_eq!(c, at(3, 0, false));
    }

    #[test]
    fn test_column_threshold() {
        let widths = [4, 20];
        let jump_widths = [40, 28];
        let rule = BreakRule {
            widths: &widths,
            jump_widths: &jump_widths,
            newline: r"\n",
        };
        let no_jump = BreakRule {
            widths: &widths,
            jump_widths: &[],
            newline: r"\n",
        };
        let at = |line, jump| Column {
            line,
            width: 0,
            jump,
        };
        assert_eq!(at(0, false).threshold(&rule), 4);
        assert_eq!(at(1, false).threshold(&rule), 4);
        assert_eq!(at(2, false).threshold(&rule), 20);
        assert_eq!(at(9, false).threshold(&rule), 20);
        assert_eq!(at(1, true).threshold(&rule), 40);
        assert_eq!(at(5, true).threshold(&rule), 28);
        assert_eq!(at(1, true).threshold(&no_jump), 4);
    }
}
