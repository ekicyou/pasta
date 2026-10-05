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

/// タグの引数 1 つ分（ARG）。`SAKURA_TAG_PATTERN` の中に 2 回現れる（先頭の引数と `,` の後ろの引数）。
/// 先頭の任意の引用 `"…"`（中の `""` は 1 文字）と、「`\` ＋ 1 文字の組・`]` `\` `,` 以外の 1 文字」の並び。
/// 引用の直後と、引用の無い ARG の先頭は `"` で始まれない（引用は引数の先頭でだけ開く）。
macro_rules! sakura_tag_arg {
    () => {
        r#"(?:"(?:[^"]|"")*")?(?:(?:\\.|[^\]\\,"])(?:\\.|[^\]\\,])*)?"#
    };
}

impl Tokenizer {
    /// 単位（タグ・エスケープ・囲み）に一致する正規表現。タグの読み取りの正。
    /// 選択肢は左から優先する。
    ///   1. 囲み       `\_?` … 次の `\_?`（最短一致）／ `\_!` … 次の `\_!`
    ///   2. エスケープ `\\` `\%`
    ///   3. タグ       `\` ＋（`[spb]` ＋ 数字 | `w` ＋ 1〜9 | `_` 0〜2 個 ＋ 名前の 1 文字 ＋ 任意の引数）
    ///
    /// 引数は `[` ARG (`,` ARG)* `]`（ARG は `sakura_tag_arg!`）。閉じなければ名前までがタグになる。
    /// 名前の後ろの文字は一致に入らず、通常の文字として分類される（`\nHello` は `\n` ＋ `Hello`）。
    /// 一致はすべて `TokenKind::SakuraScript` になるので、ウェイトは付かず、
    /// 改行推定では幅 0 として前の文字に付いて運ばれる。
    /// `(?s)` は `.` を改行にも一致させる（テキストに改行文字があっても囲みの読みが変わらない）。
    ///
    /// # ReDoS Safety
    /// - 先読みも後方参照も使わない。Rust の `regex` クレートは線形時間で一致を求めるため、
    ///   入力がどうであれ ReDoS は起きない。
    /// - 後戻りの有無で結果が変わらない形にしてある。引用の無い ARG は `"` で始まれず、
    ///   引用の直後も `"` で始まれないので、引数の先頭の `"` は引用として読むしかない。
    ///   引用の中の `""` は「中の 1 文字」と読んでも「閉じて開き直す」と読んでも同じ状態になる。
    ///   そのため左から 1 回読むだけの読み取り（Lua・pest の写し）と結果が一致する。
    pub const SAKURA_TAG_PATTERN: &'static str = concat!(
        r"(?s)\\_\?.*?\\_\?|\\_!.*?\\_!",
        r"|\\[\\%]",
        r"|\\(?:[spb][0-9]|w[1-9]|_{0,2}[0-9A-Za-z!+*?&-](?:\[",
        sakura_tag_arg!(),
        r"(?:,",
        sakura_tag_arg!(),
        r")*\])?)",
    );
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

    /// Get a reference to the sakura tag regex.
    pub fn tag_regex(&self) -> &Regex {
        &self.sakura_tag_regex
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
            if bytes[pos] == b'\\'
                && let Some(mat) = self.sakura_tag_regex.find(&input[pos..])
                && mat.start() == 0
            {
                // Found sakura script tag at current position
                tokens.push(Token::new(TokenKind::SakuraScript, mat.as_str()));
                pos += mat.len();
                continue;
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

    // ====================================================================
    // `\\`（エスケープされた `\`）を 1 単位として読む
    // Requirement: 4.2, 4.4, 4.5, 4.7
    // ====================================================================

    fn kinds_texts(input: &str) -> Vec<(TokenKind, String)> {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        tokenizer
            .tokenize(input)
            .into_iter()
            .map(|t| (t.kind, t.text))
            .collect()
    }

    fn sakura(text: &str) -> (TokenKind, String) {
        (TokenKind::SakuraScript, text.to_string())
    }

    fn general(text: &str) -> (TokenKind, String) {
        (TokenKind::General, text.to_string())
    }

    #[test]
    fn test_tokenize_escaped_backslash_before_tag_chars() {
        // `C:\\new` は `\\` ＋ `new`（`\new` のタグにならない）
        assert_eq!(
            kinds_texts(r"C:\\new"),
            vec![
                general("C"),
                (TokenKind::LineStartProhibited, ":".to_string()),
                sakura(r"\\"),
                general("n"),
                general("e"),
                general("w"),
            ]
        );
    }

    #[test]
    fn test_tokenize_escaped_backslash_at_line_end() {
        assert_eq!(kinds_texts(r"あ\\"), vec![general("あ"), sakura(r"\\")]);
        // 終端の `\e` と並んでも壊れない
        assert_eq!(
            kinds_texts(r"あ\\\e"),
            vec![general("あ"), sakura(r"\\"), sakura(r"\e")]
        );
    }

    #[test]
    fn test_tokenize_double_escaped_backslash() {
        assert_eq!(kinds_texts(r"\\\\"), vec![sakura(r"\\"), sakura(r"\\")]);
    }

    #[test]
    fn test_tokenize_escaped_backslash_then_newline_tag() {
        assert_eq!(kinds_texts(r"\\\n"), vec![sakura(r"\\"), sakura(r"\n")]);
    }

    #[test]
    fn test_tokenize_escaped_backslash_then_surface_text() {
        // `\\s[0]` は `\\` ＋ 平文 `s[0]`（表情タグではない）
        assert_eq!(
            kinds_texts(r"\\s[0]"),
            vec![
                sakura(r"\\"),
                general("s"),
                (TokenKind::LineEndProhibited, "[".to_string()),
                general("0"),
                (TokenKind::LineStartProhibited, "]".to_string()),
            ]
        );
        // エスケープの後の本物のタグはタグのまま
        assert_eq!(
            kinds_texts(r"\\\s[0]"),
            vec![sakura(r"\\"), sakura(r"\s[0]")]
        );
    }

    #[test]
    fn test_tokenize_existing_tags_unchanged() {
        assert_eq!(
            kinds_texts(r"\h\s[0]\_w[500]\![open,inputbox]\-\+\*\_?\&[ID]\n\w8\e"),
            vec![
                sakura(r"\h"),
                sakura(r"\s[0]"),
                sakura(r"\_w[500]"),
                sakura(r"\![open,inputbox]"),
                sakura(r"\-"),
                sakura(r"\+"),
                sakura(r"\*"),
                sakura(r"\_?"),
                sakura(r"\&[ID]"),
                sakura(r"\n"),
                sakura(r"\w8"),
                sakura(r"\e"),
            ]
        );
    }

    // ====================================================================
    // SSP にそろえたタグの読み取り（設計の TagPattern の表）
    // Requirement: 7.1〜7.10, 8.1, 9.1, 9.4, 9.6
    // ====================================================================

    /// 一致（`SakuraScript`）を `〔〕` で囲み、それ以外の文字はそのまま並べる
    fn read(input: &str) -> String {
        let tokenizer = Tokenizer::new(&default_config()).unwrap();
        tokenizer
            .tokenize(input)
            .into_iter()
            .map(|t| match t.kind {
                TokenKind::SakuraScript => format!("〔{}〕", t.text),
                _ => t.text,
            })
            .collect()
    }

    #[test]
    fn test_tag_pattern_table() {
        #[rustfmt::skip]
        let cases: &[(&str, &str)] = &[
            // 名前は `_` 0〜2 個 ＋ 1 文字、数字付きの形は引数を取らない（7.1〜7.4）
            (r"\nHello", r"〔\n〕Hello"),
            (r"\n!?", r"〔\n〕!?"),
            (r"\w9OK", r"〔\w9〕OK"),
            (r"\w0", r"〔\w〕0"),
            (r"\s12", r"〔\s1〕2"),
            (r"\s3[x]", r"〔\s3〕[x]"),
            (r"\_w[100]", r"〔\_w[100]〕"),
            (r"\__w[1]", r"〔\__w[1]〕"),
            (r"\-\+\*", r"〔\-〕〔\+〕〔\*〕"),
            (r"\&[amp]", r"〔\&[amp]〕"),
            // 単位にならない `\`（7.10）
            (r"\_", r"\_"),
            (r"\___a", r"\___a"),
            (r"\あ", r"\あ"),
            (r"あ\", r"あ\"),
            // 引数のエスケープと先頭の引用（7.5〜7.7）
            (r#"\![raise,X,"a]b"]"#, r#"〔\![raise,X,"a]b"]〕"#),
            (r"\q[a\]b,X]", r"〔\q[a\]b,X]〕"),
            (r#"\![a,"b""c]d"]e"#, r#"〔\![a,"b""c]d"]〕e"#),
            (r#"\![call,ghost,"the ""Name"""]"#, r#"〔\![call,ghost,"the ""Name"""]〕"#),
            (r#"\q[5"x,OnX]"#, r#"〔\q[5"x,OnX]〕"#),
            (r#"\![raise,X,a"]b]"#, r#"〔\![raise,X,a"]〕b]"#),
            (r#"\q["a]b"c,X]z"#, r#"〔\q["a]b"c,X]〕z"#),
            (r#"\q[a\,"b]c"]"#, r#"〔\q[a\,"b]〕c"]"#),
            // 閉じない引数・引用は名前までがタグ（7.8）
            (r"\s[0", r"〔\s〕[0"),
            (r#"\q["abc,OnX]y"#, r#"〔\q〕["abc,OnX]y"#),
            (r#"\![a,"b""]"#, r#"〔\!〕[a,"b""]"#),
            // エスケープ（8.1）
            (r"\\", r"〔\\〕"),
            (r"100\%です", r"100〔\%〕です"),
            // 囲み（9.1・9.4）
            (r"\_?\s[1]\_?", r"〔\_?\s[1]\_?〕"),
            (r"\_?\_?", r"〔\_?\_?〕"),
            (r"\_?abc", r"〔\_?〕abc"),
            (r"\_!a\_?b\_!c", r"〔\_!a\_?b\_!〕c"),
        ];
        for (input, want) in cases {
            assert_eq!(read(input), *want, "input: {input:?}");
        }
    }
}
