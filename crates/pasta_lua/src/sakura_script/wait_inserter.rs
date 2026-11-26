//! Wait insertion logic for sakura script.
//!
//! Inserts `\_w[ms]` wait tags based on token types and wait values.

use std::collections::BTreeMap;

use crate::loader::TalkConfig;

use super::tokenizer::{Token, TokenKind};

/// Wait values in milliseconds.
#[derive(Debug, Clone, Default)]
pub struct WaitValues {
    /// Wait for general characters (default: 50ms)
    pub normal: i64,
    /// Wait for period characters (default: 1000ms)
    pub period: i64,
    /// Wait for comma characters (default: 500ms)
    pub comma: i64,
    /// Wait for strong emphasis characters (default: 500ms)
    pub strong: i64,
    /// Wait for leader characters (default: 200ms)
    pub leader: i64,
    /// 行末の待ち（アクター → [talk] で決めたもの）
    pub line_end: LineEndWaits,
}

/// 行末の待ちの値（書かれていなければ None）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineEndWaits {
    pub normal: Option<i64>,
    pub period: Option<i64>,
    pub comma: Option<i64>,
    pub strong: Option<i64>,
    pub leader: Option<i64>,
}

impl LineEndWaits {
    /// どれか 1 つでも書かれているか
    pub fn any(&self) -> bool {
        [
            self.normal,
            self.period,
            self.comma,
            self.strong,
            self.leader,
        ]
        .iter()
        .any(Option::is_some)
    }

    fn get(&self, kind: &TokenKind) -> Option<i64> {
        match kind {
            TokenKind::General => self.normal,
            TokenKind::Period => self.period,
            TokenKind::Comma => self.comma,
            TokenKind::Strong => self.strong,
            TokenKind::Leader => self.leader,
            _ => None,
        }
    }
}

/// テキストの中の 1 つの行末
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineEnd {
    /// 行末の字の終わりのバイト位置（テキストの頭からのバイト数）
    pub end: usize,
    /// 行末の並びのすぐ後ろに作者の `\_w[…]` が続くか
    pub author_wait: bool,
}

impl WaitValues {
    /// Create WaitValues from TalkConfig.
    pub fn from_config(config: &TalkConfig) -> Self {
        Self {
            normal: config.script_wait_normal,
            period: config.script_wait_period,
            comma: config.script_wait_comma,
            strong: config.script_wait_strong,
            leader: config.script_wait_leader,
            line_end: LineEndWaits::default(),
        }
    }

    /// 字の種類の行末の値（行末のキーが無ければ行の途中の値）。待ちを持たない種類は None
    pub fn line_end_value(&self, kind: &TokenKind) -> Option<i64> {
        self.line_end.get(kind).or(self.get_wait(kind))
    }

    /// Get wait value for a token kind.
    ///
    /// Returns None for SakuraScript and LineEndProhibited (no wait).
    pub fn get_wait(&self, kind: &TokenKind) -> Option<i64> {
        match kind {
            TokenKind::SakuraScript => None,
            TokenKind::LineEndProhibited => None,
            TokenKind::General => Some(self.normal),
            TokenKind::Period => Some(self.period),
            TokenKind::Comma => Some(self.comma),
            TokenKind::Strong => Some(self.strong),
            TokenKind::Leader => Some(self.leader),
            TokenKind::LineStartProhibited => None, // Handled specially in consecutive processing
        }
    }
}

/// Insert wait tags into token sequence.
///
/// # Rules
/// 1. SakuraScript and LineEndProhibited tokens: no wait
/// 2. General tokens: insert (normal - 50)ms after each cluster
/// 3. Leader tokens: insert (leader - 50)ms after each cluster
/// 4. Period/Comma/Strong/LineStartProhibited: accumulate until end of consecutive run
/// 5. At end of consecutive punctuation: insert (max_wait - 50)ms
/// 6. If calculated wait <= 0, skip insertion
///
/// # Arguments
/// * `tokens` - Token sequence
/// * `wait_values` - Wait values in milliseconds
/// * `line_ends` - Line ends in the text (empty keeps the mid-line output)
///
/// # Returns
/// String with wait tags inserted
pub fn insert_waits(tokens: &[Token], wait_values: &WaitValues, line_ends: &[LineEnd]) -> String {
    let _plan = plan_line_ends(tokens, wait_values, line_ends);
    let mut result = String::new();
    let mut pending_max_wait: Option<i64> = None;
    let mut pending_text = String::new();

    for token in tokens {
        match &token.kind {
            // Rule 1: SakuraScript - output as-is, flush pending if any
            TokenKind::SakuraScript => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                result.push_str(&token.text);
            }

            // Rule 1: LineEndProhibited - output as-is, flush pending if any
            TokenKind::LineEndProhibited => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                result.push_str(&token.text);
            }

            // Rule 2: General - output with one wait per token (cluster)
            TokenKind::General => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                append_with_wait(&mut result, &token.text, wait_values.normal);
            }

            // Rule 3: Leader - output with one wait per token (cluster)
            TokenKind::Leader => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                append_with_wait(&mut result, &token.text, wait_values.leader);
            }

            // Rules 4-5: Punctuation - accumulate and track max
            TokenKind::Period
            | TokenKind::Comma
            | TokenKind::Strong
            | TokenKind::LineStartProhibited => {
                let wait = match &token.kind {
                    TokenKind::Period => wait_values.period,
                    TokenKind::Comma => wait_values.comma,
                    TokenKind::Strong => wait_values.strong,
                    TokenKind::LineStartProhibited => {
                        // LineStartProhibited doesn't add its own wait, but extends the pending run
                        pending_max_wait.unwrap_or(0)
                    }
                    // SAFETY: The outer match arm constrains token.kind to exactly
                    // Period | Comma | Strong | LineStartProhibited. All four
                    // variants are exhaustively handled above, so this branch
                    // is provably unreachable.
                    _ => unreachable!(),
                };

                pending_text.push_str(&token.text);
                pending_max_wait = Some(pending_max_wait.map_or(wait, |m| m.max(wait)));
            }
        }
    }

    flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);

    result
}

/// 1 つの効く行末の計画
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineEndPlan {
    /// 並びの頭のトークンの添字（`す」` の `す`、句読点の並びの頭、または行末のトークン自身）
    start: usize,
    /// 行末のトークンの添字
    end: usize,
    /// 入れる値（50 を引く前）。直後に作者の待ちがあれば None
    wait: Option<i64>,
}

/// 効く行末ごとの計画を、添字の順に決める
fn plan_line_ends(
    tokens: &[Token],
    wait_values: &WaitValues,
    line_ends: &[LineEnd],
) -> Vec<LineEndPlan> {
    if line_ends.is_empty() || !wait_values.line_end.any() {
        return Vec::new();
    }

    let token_ends: Vec<usize> = tokens
        .iter()
        .scan(0, |pos, t| {
            *pos += t.text.len();
            Some(*pos)
        })
        .collect();

    // 行末のトークンの添字 → 作者の待ちが続くか
    let mut targets: BTreeMap<usize, bool> = BTreeMap::new();
    for line_end in line_ends {
        let k = token_ends.partition_point(|&e| e < line_end.end);
        if token_ends.get(k) != Some(&line_end.end) {
            continue;
        }
        let token = &tokens[k];
        // 単独の `\` の後ろの待ちは `\\` になるので、行の途中と同じく入れない
        if matches!(
            token.kind,
            TokenKind::SakuraScript | TokenKind::LineEndProhibited
        ) || token.text == "\\"
        {
            continue;
        }
        *targets.entry(k).or_default() |= line_end.author_wait;
    }

    let mut plans = Vec::new();
    // 効く行末の次から並びが始まる
    let mut floor = 0;
    for (&k, &author_wait) in &targets {
        let (start, kinds): (usize, Vec<&TokenKind>) = match &tokens[k].kind {
            kind @ (TokenKind::General | TokenKind::Leader) => (k, vec![kind]),
            _ => {
                let mut start = k;
                while start > floor && in_run(&tokens[start - 1].kind) {
                    start -= 1;
                }
                let puncts: Vec<&TokenKind> = tokens[start..=k]
                    .iter()
                    .map(|t| &t.kind)
                    .filter(|kind| **kind != TokenKind::LineStartProhibited)
                    .collect();
                if !puncts.is_empty() {
                    (start, puncts)
                } else {
                    // 行頭禁則の字だけの並びは、直前のふつうの字・リーダーにつく
                    match start.checked_sub(1).map(|j| (j, &tokens[j].kind)) {
                        Some((j, kind @ (TokenKind::General | TokenKind::Leader)))
                            if !targets.contains_key(&j) =>
                        {
                            (j, vec![kind])
                        }
                        _ => continue,
                    }
                }
            }
        };

        if !kinds
            .iter()
            .any(|kind| wait_values.line_end.get(kind).is_some())
        {
            continue;
        }
        let value = kinds
            .iter()
            .filter_map(|kind| wait_values.line_end_value(kind))
            .max();
        plans.push(LineEndPlan {
            start,
            end: k,
            wait: if author_wait { None } else { value },
        });
        floor = k + 1;
    }
    plans
}

fn in_run(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Period | TokenKind::Comma | TokenKind::Strong | TokenKind::LineStartProhibited
    )
}

/// Flush pending punctuation with wait tag.
fn flush_pending(
    result: &mut String,
    pending_text: &mut String,
    pending_max_wait: &mut Option<i64>,
) {
    if !pending_text.is_empty() {
        result.push_str(pending_text);

        if let Some(max_wait) = pending_max_wait.take() {
            let effective_wait = max_wait - 50;
            if effective_wait > 0 {
                result.push_str(&format!(r"\_w[{}]", effective_wait));
            }
        }

        pending_text.clear();
    }
    *pending_max_wait = None;
}

/// Append one token (one cluster) followed by one wait.
fn append_with_wait(result: &mut String, text: &str, wait_ms: i64) {
    let effective_wait = wait_ms - 50;
    result.push_str(text);
    // Units never reach here, so a lone `\` starts no unit. A wait right after it
    // would form `\\` (an escape), so its wait goes after the next cluster.
    if effective_wait > 0 && text != "\\" {
        result.push_str(&format!(r"\_w[{}]", effective_wait));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(kind: TokenKind, text: &str) -> Token {
        Token::new(kind, text)
    }

    fn default_wait_values() -> WaitValues {
        WaitValues {
            normal: 100,  // effective: 50
            period: 1000, // effective: 950
            comma: 500,   // effective: 450
            strong: 500,  // effective: 450
            leader: 200,  // effective: 150
            line_end: LineEndWaits::default(),
        }
    }

    #[test]
    fn test_general_text_wait() {
        let tokens = vec![
            make_token(TokenKind::General, "こ"),
            make_token(TokenKind::General, "ん"),
            make_token(TokenKind::General, "に"),
            make_token(TokenKind::General, "ち"),
            make_token(TokenKind::General, "は"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]");
    }

    #[test]
    fn test_sakura_script_no_wait() {
        let tokens = vec![
            make_token(TokenKind::SakuraScript, r"\h"),
            make_token(TokenKind::SakuraScript, r"\s[0]"),
            make_token(TokenKind::General, "あ"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"\h\s[0]あ\_w[50]");
    }

    #[test]
    fn test_period_wait() {
        let tokens = vec![
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::Period, "。"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"あ\_w[50]。\_w[950]");
    }

    #[test]
    fn test_consecutive_punctuation_max_wait() {
        // 」」」！？。、 -> max is period(1000), effective: 950
        let tokens = vec![
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::Strong, "！"),
            make_token(TokenKind::Strong, "？"),
            make_token(TokenKind::Period, "。"),
            make_token(TokenKind::Comma, "、"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        // All consecutive punctuation is output together with max wait
        assert_eq!(result, r"」」」！？。、\_w[950]");
    }

    #[test]
    fn test_leader_per_char_wait() {
        let tokens = vec![
            make_token(TokenKind::Leader, "…"),
            make_token(TokenKind::Leader, "…"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"…\_w[150]…\_w[150]");
    }

    #[test]
    fn test_line_end_prohibited_no_wait() {
        let tokens = vec![
            make_token(TokenKind::LineEndProhibited, "「"),
            make_token(TokenKind::General, "あ"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"「あ\_w[50]");
    }

    #[test]
    fn test_zero_wait_skip() {
        let tokens = vec![make_token(TokenKind::General, "あ")];
        let wait_values = WaitValues {
            normal: 50, // effective: 0
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values, &[]);

        // No wait tag when effective wait is 0
        assert_eq!(result, "あ");
    }

    #[test]
    fn test_negative_wait_skip() {
        let tokens = vec![make_token(TokenKind::General, "あ")];
        let wait_values = WaitValues {
            normal: 30, // effective: -20
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values, &[]);

        // No wait tag when effective wait is negative
        assert_eq!(result, "あ");
    }

    #[test]
    fn test_mixed_text() {
        let tokens = vec![
            make_token(TokenKind::SakuraScript, r"\h"),
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::Period, "。"),
            make_token(TokenKind::General, "い"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"\hあ\_w[50]。\_w[950]い\_w[50]");
    }

    #[test]
    fn test_empty_tokens() {
        let tokens: Vec<Token> = vec![];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, "");
    }

    #[test]
    fn test_multi_codepoint_general_token_one_wait() {
        // 🙆‍♀️ (4 code points) and ✌️ (2) are one cluster each: one wait after each
        let tokens = vec![
            make_token(TokenKind::General, "🙆\u{200D}♀\u{FE0F}"),
            make_token(TokenKind::General, "✌\u{FE0F}"),
        ];

        let result = insert_waits(&tokens, &default_wait_values(), &[]);

        assert_eq!(result, "🙆\u{200D}♀\u{FE0F}\\_w[50]✌\u{FE0F}\\_w[50]");
    }

    #[test]
    fn test_multi_codepoint_leader_token_one_wait() {
        let tokens = vec![make_token(TokenKind::Leader, "…\u{0301}")];

        let result = insert_waits(&tokens, &default_wait_values(), &[]);

        assert_eq!(result, "…\u{0301}\\_w[150]");
    }

    #[test]
    fn test_lone_backslash_token_no_wait() {
        // A wait right after a lone `\` would form `\\`; it goes after the next cluster
        let tokens = vec![
            make_token(TokenKind::General, "\\"),
            make_token(TokenKind::General, "あ"),
        ];

        let result = insert_waits(&tokens, &default_wait_values(), &[]);

        assert_eq!(result, "\\あ\\_w[50]");
    }

    #[test]
    fn test_backslash_with_combining_mark_gets_wait() {
        // `\` + U+0301 is not a lone `\`: its wait does not follow `\` directly
        let tokens = vec![make_token(TokenKind::General, "\\\u{0301}")];

        let result = insert_waits(&tokens, &default_wait_values(), &[]);

        assert_eq!(result, "\\\u{0301}\\_w[50]");
    }

    #[test]
    fn test_requirement_7_1_example() {
        // When input is 」」」！？。、 with script_wait_period=1000
        // Output should be 」」」！？。、\_w[950]
        let tokens = vec![
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::LineStartProhibited, "」"),
            make_token(TokenKind::Strong, "！"),
            make_token(TokenKind::Strong, "？"),
            make_token(TokenKind::Period, "。"),
            make_token(TokenKind::Comma, "、"),
        ];
        let wait_values = WaitValues {
            period: 1000,
            comma: 500,
            strong: 500,
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"」」」！？。、\_w[950]");
    }

    #[test]
    fn test_requirement_7_2_example() {
        // When input is こんにちは with script_wait_normal=100
        // Output should have \_w[50] after each character
        let tokens = vec![
            make_token(TokenKind::General, "こ"),
            make_token(TokenKind::General, "ん"),
            make_token(TokenKind::General, "に"),
            make_token(TokenKind::General, "ち"),
            make_token(TokenKind::General, "は"),
        ];
        let wait_values = WaitValues {
            normal: 100,
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(result, r"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]");
    }

    #[test]
    fn test_requirement_7_3_example() {
        // Sakura script tag \h\s[0] should be preserved
        let tokens = vec![
            make_token(TokenKind::SakuraScript, r"\h"),
            make_token(TokenKind::SakuraScript, r"\s[0]"),
            make_token(TokenKind::General, "こ"),
            make_token(TokenKind::General, "ん"),
            make_token(TokenKind::General, "に"),
            make_token(TokenKind::General, "ち"),
            make_token(TokenKind::General, "は"),
        ];
        let wait_values = WaitValues {
            normal: 100,
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values, &[]);

        assert_eq!(
            result,
            r"\h\s[0]こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]"
        );
    }

    #[test]
    fn test_line_end_value_falls_back_to_mid_line() {
        let w = default_wait_values();
        assert_eq!(w.line_end_value(&TokenKind::General), Some(100));
        assert_eq!(w.line_end_value(&TokenKind::Period), Some(1000));
        assert_eq!(w.line_end_value(&TokenKind::Comma), Some(500));
        assert_eq!(w.line_end_value(&TokenKind::Strong), Some(500));
        assert_eq!(w.line_end_value(&TokenKind::Leader), Some(200));
    }

    #[test]
    fn test_line_end_value_uses_line_end_key() {
        let w = WaitValues {
            line_end: LineEndWaits {
                period: Some(0),
                leader: Some(900),
                ..Default::default()
            },
            ..default_wait_values()
        };
        assert_eq!(w.line_end_value(&TokenKind::Period), Some(0));
        assert_eq!(w.line_end_value(&TokenKind::Leader), Some(900));
        assert_eq!(w.line_end_value(&TokenKind::General), Some(100));
    }

    #[test]
    fn test_line_end_value_none_without_wait() {
        let w = default_wait_values();
        assert_eq!(w.line_end_value(&TokenKind::SakuraScript), None);
        assert_eq!(w.line_end_value(&TokenKind::LineEndProhibited), None);
        assert_eq!(w.line_end_value(&TokenKind::LineStartProhibited), None);
    }

    #[test]
    fn test_line_end_waits_any() {
        assert!(!LineEndWaits::default().any());
        for w in [
            LineEndWaits {
                normal: Some(1),
                ..Default::default()
            },
            LineEndWaits {
                period: Some(1),
                ..Default::default()
            },
            LineEndWaits {
                comma: Some(1),
                ..Default::default()
            },
            LineEndWaits {
                strong: Some(1),
                ..Default::default()
            },
            LineEndWaits {
                leader: Some(0),
                ..Default::default()
            },
        ] {
            assert!(w.any());
        }
    }

    fn eol(line_end: LineEndWaits) -> WaitValues {
        WaitValues {
            line_end,
            ..default_wait_values()
        }
    }

    fn strong_eol(v: i64) -> WaitValues {
        eol(LineEndWaits {
            strong: Some(v),
            ..Default::default()
        })
    }

    fn normal_eol(v: i64) -> WaitValues {
        eol(LineEndWaits {
            normal: Some(v),
            ..Default::default()
        })
    }

    fn at(end: usize) -> LineEnd {
        LineEnd {
            end,
            author_wait: false,
        }
    }

    fn plan(start: usize, end: usize, wait: Option<i64>) -> LineEndPlan {
        LineEndPlan { start, end, wait }
    }

    /// 雨？
    fn ame() -> Vec<Token> {
        vec![
            make_token(TokenKind::General, "雨"),
            make_token(TokenKind::Strong, "？"),
        ]
    }

    #[test]
    fn plan_strong_line_end() {
        let p = plan_line_ends(&ame(), &strong_eol(1200), &[at(6)]);
        assert_eq!(p, vec![plan(1, 1, Some(1200))]);
    }

    #[test]
    fn plan_empty_without_line_end_key() {
        assert!(plan_line_ends(&ame(), &default_wait_values(), &[at(6)]).is_empty());
    }

    #[test]
    fn plan_empty_without_line_ends() {
        assert!(plan_line_ends(&ame(), &strong_eol(1200), &[]).is_empty());
    }

    #[test]
    fn plan_empty_when_key_does_not_apply() {
        // 「？」の行末に、ふつうの字の行末のキーは効かない
        assert!(plan_line_ends(&ame(), &normal_eol(1250), &[at(6)]).is_empty());
    }

    #[test]
    fn plan_ignores_end_off_token_boundary() {
        assert!(plan_line_ends(&ame(), &strong_eol(1200), &[at(4)]).is_empty());
        assert!(plan_line_ends(&ame(), &strong_eol(1200), &[at(7)]).is_empty());
    }

    #[test]
    fn plan_ignores_line_end_prohibited_and_tags() {
        let tokens = vec![
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::LineEndProhibited, "「"),
        ];
        assert!(plan_line_ends(&tokens, &normal_eol(1250), &[at(6)]).is_empty());

        let tokens = vec![
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::SakuraScript, r"\\"),
        ];
        assert!(plan_line_ends(&tokens, &normal_eol(1250), &[at(5)]).is_empty());

        let tokens = vec![
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::General, "\\"),
        ];
        assert!(plan_line_ends(&tokens, &normal_eol(1250), &[at(4)]).is_empty());
    }

    #[test]
    fn plan_run_takes_longest_value() {
        // …ね？」 : 並びは ？」
        let tokens = vec![
            make_token(TokenKind::General, "ね"),
            make_token(TokenKind::Strong, "？"),
            make_token(TokenKind::LineStartProhibited, "」"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &strong_eol(1200), &[at(9)]),
            vec![plan(1, 2, Some(1200))]
        );
        // 行末の値が行の途中の値より短くても、行末の値を使う
        assert_eq!(
            plan_line_ends(&tokens, &strong_eol(300), &[at(9)]),
            vec![plan(1, 2, Some(300))]
        );

        // ！？
        let tokens = vec![
            make_token(TokenKind::Strong, "！"),
            make_token(TokenKind::Strong, "？"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &strong_eol(1200), &[at(6)]),
            vec![plan(0, 1, Some(1200))]
        );

        // ？。 : 書かれていない句点は行の途中の値（1000）
        let tokens = vec![
            make_token(TokenKind::General, "雨"),
            make_token(TokenKind::Strong, "？"),
            make_token(TokenKind::Period, "。"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &strong_eol(1200), &[at(9)]),
            vec![plan(1, 2, Some(1200))]
        );
        assert_eq!(
            plan_line_ends(&tokens, &strong_eol(300), &[at(9)]),
            vec![plan(1, 2, Some(1000))]
        );
    }

    #[test]
    fn plan_general_and_leader_line_end() {
        let tokens = vec![
            make_token(TokenKind::General, "ち"),
            make_token(TokenKind::General, "は"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &normal_eol(1250), &[at(6)]),
            vec![plan(1, 1, Some(1250))]
        );

        let tokens = vec![
            make_token(TokenKind::General, "た"),
            make_token(TokenKind::Leader, "‥"),
            make_token(TokenKind::Leader, "‥"),
        ];
        let w = eol(LineEndWaits {
            leader: Some(1250),
            ..Default::default()
        });
        assert_eq!(
            plan_line_ends(&tokens, &w, &[at(9)]),
            vec![plan(2, 2, Some(1250))]
        );
    }

    #[test]
    fn plan_line_start_prohibited_after_general() {
        // す」 : 並びは す」、字の種類は す
        let tokens = vec![
            make_token(TokenKind::General, "す"),
            make_token(TokenKind::LineStartProhibited, "」"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &normal_eol(1250), &[at(6)]),
            vec![plan(0, 1, Some(1250))]
        );
        assert!(plan_line_ends(&tokens, &strong_eol(1200), &[at(6)]).is_empty());
    }

    #[test]
    fn plan_line_start_prohibited_without_char_kind() {
        for head in [
            make_token(TokenKind::SakuraScript, r"\s[5]"),
            make_token(TokenKind::LineEndProhibited, "「"),
        ] {
            let len = head.text.len();
            let tokens = vec![head, make_token(TokenKind::LineStartProhibited, "」")];
            assert!(plan_line_ends(&tokens, &normal_eol(1250), &[at(len + 3)]).is_empty());
        }
        let tokens = vec![make_token(TokenKind::LineStartProhibited, "」")];
        assert!(plan_line_ends(&tokens, &normal_eol(1250), &[at(3)]).is_empty());

        // す が別の行末なら、」 は す につかない
        let tokens = vec![
            make_token(TokenKind::General, "す"),
            make_token(TokenKind::LineStartProhibited, "」"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &normal_eol(1250), &[at(3), at(6)]),
            vec![plan(0, 0, Some(1250))]
        );
    }

    #[test]
    fn plan_author_wait_has_no_value() {
        let p = plan_line_ends(
            &ame(),
            &strong_eol(1200),
            &[LineEnd {
                end: 6,
                author_wait: true,
            }],
        );
        assert_eq!(p, vec![plan(1, 1, None)]);
    }

    #[test]
    fn plan_merges_same_line_end_token() {
        let p = plan_line_ends(
            &ame(),
            &strong_eol(1200),
            &[
                at(6),
                LineEnd {
                    end: 6,
                    author_wait: true,
                },
                at(6),
            ],
        );
        assert_eq!(p, vec![plan(1, 1, None)]);
    }

    #[test]
    fn plan_keeps_short_values() {
        // 50 と 0 も計画に残す（50 を引いて 0 以下なら出力で入れない）
        let p = plan_line_ends(&ame(), &strong_eol(50), &[at(6)]);
        assert_eq!(p, vec![plan(1, 1, Some(50))]);
        let p = plan_line_ends(&ame(), &strong_eol(0), &[at(6)]);
        assert_eq!(p, vec![plan(1, 1, Some(0))]);
    }

    #[test]
    fn plan_run_stops_at_effective_line_end() {
        // 雨。（行末）。（行末）: 次の行の句点は前の行の並びにつながらない
        let tokens = vec![
            make_token(TokenKind::General, "雨"),
            make_token(TokenKind::Period, "。"),
            make_token(TokenKind::Period, "。"),
        ];
        let w = eol(LineEndWaits {
            period: Some(1200),
            ..Default::default()
        });
        assert_eq!(
            plan_line_ends(&tokens, &w, &[at(9), at(6)]),
            vec![plan(1, 1, Some(1200)), plan(2, 2, Some(1200))]
        );

        // 効かない行末（！）では並びは切れない
        let tokens = vec![
            make_token(TokenKind::Strong, "！"),
            make_token(TokenKind::Period, "。"),
        ];
        assert_eq!(
            plan_line_ends(&tokens, &w, &[at(3), at(6)]),
            vec![plan(0, 1, Some(1200))]
        );
    }
}
