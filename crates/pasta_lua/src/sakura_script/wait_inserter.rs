//! Wait insertion logic for sakura script.
//!
//! Inserts `\_w[ms]` wait tags based on token types and wait values.

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
        }
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
/// 2. General tokens: insert (normal - 50)ms after each character
/// 3. Leader tokens: insert (leader - 50)ms after each character
/// 4. Period/Comma/Strong/LineStartProhibited: accumulate until end of consecutive run
/// 5. At end of consecutive punctuation: insert (max_wait - 50)ms
/// 6. If calculated wait <= 0, skip insertion
///
/// # Arguments
/// * `tokens` - Token sequence
/// * `wait_values` - Wait values in milliseconds
///
/// # Returns
/// String with wait tags inserted
pub fn insert_waits(tokens: &[Token], wait_values: &WaitValues) -> String {
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

            // Rule 2: General - output with wait per character
            TokenKind::General => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                append_with_per_char_wait(&mut result, &token.text, wait_values.normal);
            }

            // Rule 3: Leader - output with wait per character
            TokenKind::Leader => {
                flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);
                append_with_per_char_wait(&mut result, &token.text, wait_values.leader);
            }

            // Rules 4-5: Punctuation - accumulate and track max
            TokenKind::Period | TokenKind::Comma | TokenKind::Strong | TokenKind::LineStartProhibited => {
                let wait = match &token.kind {
                    TokenKind::Period => wait_values.period,
                    TokenKind::Comma => wait_values.comma,
                    TokenKind::Strong => wait_values.strong,
                    TokenKind::LineStartProhibited => {
                        // LineStartProhibited doesn't add its own wait, but extends the pending run
                        pending_max_wait.unwrap_or(0)
                    }
                    _ => unreachable!(),
                };

                pending_text.push_str(&token.text);
                pending_max_wait = Some(pending_max_wait.map_or(wait, |m| m.max(wait)));
            }
        }
    }

    // Flush any remaining pending punctuation
    flush_pending(&mut result, &mut pending_text, &mut pending_max_wait);

    result
}

/// Flush pending punctuation with wait tag.
fn flush_pending(result: &mut String, pending_text: &mut String, pending_max_wait: &mut Option<i64>) {
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

/// Append text with per-character wait.
fn append_with_per_char_wait(result: &mut String, text: &str, wait_ms: i64) {
    let effective_wait = wait_ms - 50;

    for c in text.chars() {
        result.push(c);
        if effective_wait > 0 {
            result.push_str(&format!(r"\_w[{}]", effective_wait));
        }
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

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, r"\h\s[0]あ\_w[50]");
    }

    #[test]
    fn test_period_wait() {
        let tokens = vec![
            make_token(TokenKind::General, "あ"),
            make_token(TokenKind::Period, "。"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, r"…\_w[150]…\_w[150]");
    }

    #[test]
    fn test_line_end_prohibited_no_wait() {
        let tokens = vec![
            make_token(TokenKind::LineEndProhibited, "「"),
            make_token(TokenKind::General, "あ"),
        ];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, r"「あ\_w[50]");
    }

    #[test]
    fn test_zero_wait_skip() {
        let tokens = vec![make_token(TokenKind::General, "あ")];
        let wait_values = WaitValues {
            normal: 50, // effective: 0
            ..Default::default()
        };

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, r"\hあ\_w[50]。\_w[950]い\_w[50]");
    }

    #[test]
    fn test_empty_tokens() {
        let tokens: Vec<Token> = vec![];
        let wait_values = default_wait_values();

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, "");
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

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

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

        let result = insert_waits(&tokens, &wait_values);

        assert_eq!(result, r"\h\s[0]こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]");
    }
}
