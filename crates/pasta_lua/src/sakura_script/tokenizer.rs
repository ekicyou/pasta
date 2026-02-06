//! Tokenizer for sakura script wait insertion.
//!
//! Breaks input text into tokens by character type for wait insertion.

use crate::loader::TalkConfig;
use regex::Regex;
use std::collections::HashSet;

/// Token kind for character classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Sakura script tag (e.g., `\h`, `\s[0]`, `\_w[500]`)
    SakuraScript,
    /// Period characters (。．. etc.)
    Period,
    /// Comma characters (、，, etc.)
    Comma,
    /// Strong emphasis characters (！？!? etc.)
    Strong,
    /// Leader characters (・‥… etc.)
    Leader,
    /// Line start prohibited characters (行頭禁則)
    LineStartProhibited,
    /// Line end prohibited characters (行末禁則)
    LineEndProhibited,
    /// General characters (all others)
    General,
}

/// Token representing a piece of text with its kind.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

/// Character sets for token classification.
#[derive(Debug, Clone)]
pub struct CharSets {
    pub period: HashSet<char>,
    pub comma: HashSet<char>,
    pub strong: HashSet<char>,
    pub leader: HashSet<char>,
    pub line_start_prohibited: HashSet<char>,
    pub line_end_prohibited: HashSet<char>,
}

impl CharSets {
    /// Create CharSets from TalkConfig.
    pub fn from_config(config: &TalkConfig) -> Self {
        Self {
            period: config.chars_period.chars().collect(),
            comma: config.chars_comma.chars().collect(),
            strong: config.chars_strong.chars().collect(),
            leader: config.chars_leader.chars().collect(),
            line_start_prohibited: config.chars_line_start_prohibited.chars().collect(),
            line_end_prohibited: config.chars_line_end_prohibited.chars().collect(),
        }
    }

    /// Classify a character into TokenKind.
    ///
    /// Priority order (first match wins):
    /// 1. Period
    /// 2. Comma
    /// 3. Strong
    /// 4. Leader
    /// 5. LineStartProhibited
    /// 6. LineEndProhibited
    /// 7. General (fallback)
    pub fn classify(&self, c: char) -> TokenKind {
        if self.period.contains(&c) {
            TokenKind::Period
        } else if self.comma.contains(&c) {
            TokenKind::Comma
        } else if self.strong.contains(&c) {
            TokenKind::Strong
        } else if self.leader.contains(&c) {
            TokenKind::Leader
        } else if self.line_start_prohibited.contains(&c) {
            TokenKind::LineStartProhibited
        } else if self.line_end_prohibited.contains(&c) {
            TokenKind::LineEndProhibited
        } else {
            TokenKind::General
        }
    }
}

/// Tokenizer for sakura script text.
///
/// Tokenizes input text by detecting sakura script tags first (priority),
/// then classifying individual characters by their type.
pub struct Tokenizer {
    sakura_tag_regex: Regex,
    char_sets: CharSets,
}

impl Tokenizer {
    /// Sakura script tag pattern.
    /// Matches: \tag or \tag[param]
    /// Examples: \h, \s[0], \_w[500], \![open,inputbox], \-, \+, \*, \_?, \&[ID]
    const SAKURA_TAG_PATTERN: &'static str = r"\\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?";
    /// Create a new Tokenizer from TalkConfig.
    ///
    /// # Arguments
    /// * `config` - TalkConfig for character sets
    ///
    /// # Returns
    /// * `Ok(Tokenizer)` - Successfully created tokenizer
    /// * `Err(regex::Error)` - Failed to compile regex
    pub fn new(config: &TalkConfig) -> Result<Self, regex::Error> {
        let sakura_tag_regex = Regex::new(Self::SAKURA_TAG_PATTERN)?;
        let char_sets = CharSets::from_config(config);

        Ok(Self {
            sakura_tag_regex,
            char_sets,
        })
    }

    /// Tokenize input text.
    ///
    /// Sakura script tags are matched first (highest priority).
    /// Other characters are classified individually.
    ///
    /// # Arguments
    /// * `input` - Input text to tokenize
    ///
    /// # Returns
    /// Vector of tokens in input order
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < input.len() {
            // Check for sakura script tag (starts with \)
            if bytes[pos] == b'\\' {
                if let Some(mat) = self.sakura_tag_regex.find(&input[pos..]) {
                    if mat.start() == 0 {
                        // Found sakura script tag at current position
                        tokens.push(Token::new(TokenKind::SakuraScript, mat.as_str()));
                        pos += mat.len();
                        continue;
                    }
                }
            }

            // Process single character
            let remaining = &input[pos..];
            if let Some(c) = remaining.chars().next() {
                let kind = self.char_sets.classify(c);
                tokens.push(Token::new(kind, c.to_string()));
                pos += c.len_utf8();
            } else {
                break;
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TalkConfig {
        TalkConfig::default()
    }

    #[test]
    fn test_tokenize_general_text() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("こんにちは");

        assert_eq!(tokens.len(), 5);
        for token in &tokens {
            assert_eq!(token.kind, TokenKind::General);
        }
    }

    #[test]
    fn test_tokenize_sakura_script_tag() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\h\s[0]こんにちは");

        assert_eq!(tokens.len(), 7); // \h, \s[0], こ, ん, に, ち, は
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\h");
        assert_eq!(tokens[1].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[1].text, r"\s[0]");
    }

    #[test]
    fn test_tokenize_period() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("。．.");

        assert_eq!(tokens.len(), 3);
        for token in &tokens {
            assert_eq!(token.kind, TokenKind::Period);
        }
    }

    #[test]
    fn test_tokenize_comma() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("、，,");

        assert_eq!(tokens.len(), 3);
        for token in &tokens {
            assert_eq!(token.kind, TokenKind::Comma);
        }
    }

    #[test]
    fn test_tokenize_strong() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("！？!?");

        assert_eq!(tokens.len(), 4);
        for token in &tokens {
            assert_eq!(token.kind, TokenKind::Strong);
        }
    }

    #[test]
    fn test_tokenize_leader() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("・‥…");

        assert_eq!(tokens.len(), 3);
        for token in &tokens {
            assert_eq!(token.kind, TokenKind::Leader);
        }
    }

    #[test]
    fn test_tokenize_line_prohibited() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();

        // Line start prohibited
        let tokens = tokenizer.tokenize("」』");
        assert_eq!(tokens[0].kind, TokenKind::LineStartProhibited);
        assert_eq!(tokens[1].kind, TokenKind::LineStartProhibited);

        // Line end prohibited
        let tokens = tokenizer.tokenize("「『");
        assert_eq!(tokens[0].kind, TokenKind::LineEndProhibited);
        assert_eq!(tokens[1].kind, TokenKind::LineEndProhibited);
    }

    #[test]
    fn test_tokenize_mixed() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\hこんにちは。");

        assert_eq!(tokens.len(), 7); // \h, こ, ん, に, ち, は, 。
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[6].kind, TokenKind::Period);
    }

    #[test]
    fn test_tokenize_complex_tag() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\_w[500]テスト");

        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\_w[500]");
    }

    #[test]
    fn test_tokenize_unicode_preservation() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("が"); // Single hiragana with dakuten

        // Note: 'が' is a single char in NFC form
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "が");
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("");

        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_consecutive_punctuation() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize("」」」！？。、");

        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].kind, TokenKind::LineStartProhibited);
        assert_eq!(tokens[1].kind, TokenKind::LineStartProhibited);
        assert_eq!(tokens[2].kind, TokenKind::LineStartProhibited);
        assert_eq!(tokens[3].kind, TokenKind::Strong);
        assert_eq!(tokens[4].kind, TokenKind::Strong);
        assert_eq!(tokens[5].kind, TokenKind::Period);
        assert_eq!(tokens[6].kind, TokenKind::Comma);
    }

    // ====================================================================
    // 5文字記号タグ（-+*?&）のトークナイズテスト
    // Requirement: 3.1, 3.2, 3.4, 4.2
    // ====================================================================

    #[test]
    fn test_tokenize_symbol_tag_hyphen() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\-");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\-");
    }

    #[test]
    fn test_tokenize_symbol_tag_plus() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\+");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\+");
    }

    #[test]
    fn test_tokenize_symbol_tag_asterisk() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\*");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\*");
    }

    #[test]
    fn test_tokenize_symbol_tag_underscore_question() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\_?");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\_?");
    }

    #[test]
    fn test_tokenize_symbol_tag_ampersand() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"\&[ID]");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::SakuraScript);
        assert_eq!(tokens[0].text, r"\&[ID]");
    }

    #[test]
    fn test_tokenize_symbol_tag_mixed_text() {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        let tokens = tokenizer.tokenize(r"こんにちは\-。");

        // こ, ん, に, ち, は, \-, 。 = 7 tokens
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].kind, TokenKind::General); // こ
        assert_eq!(tokens[1].kind, TokenKind::General); // ん
        assert_eq!(tokens[2].kind, TokenKind::General); // に
        assert_eq!(tokens[3].kind, TokenKind::General); // ち
        assert_eq!(tokens[4].kind, TokenKind::General); // は
        assert_eq!(tokens[5].kind, TokenKind::SakuraScript); // \-
        assert_eq!(tokens[5].text, r"\-");
        assert_eq!(tokens[6].kind, TokenKind::Period); // 。
    }
}
