//! Semantic token type/modifier definitions and encoding utilities.

use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
};

// ============================================================================
// Token Type Definitions
// ============================================================================

/// Pasta DSL固有のセマンティックトークンタイプ
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::COMMENT,             // 0: コメント (#/＃)
    SemanticTokenType::NAMESPACE,           // 1: グローバルシーンマーカー (*/＊)
    SemanticTokenType::new("scene"),        // 2: ローカルシーンマーカー (-/・)
    SemanticTokenType::DECORATOR,           // 3: 属性マーカー (&/＆)
    SemanticTokenType::new("word"),         // 4: 単語マーカー (@/＠)
    SemanticTokenType::VARIABLE,            // 5: 変数マーカー ($/＄)
    SemanticTokenType::new("call"),         // 6: Callマーカー (>/＞)
    SemanticTokenType::new("actor"),        // 7: アクター辞書マーカー (%/％)
    SemanticTokenType::new("actorName"),    // 8: アクター名 (：の前)
    SemanticTokenType::new("codeBlock"),    // 9: Luaコードブロック
    SemanticTokenType::new("talk"),         // 10: 文字列リテラル（Talk）
    SemanticTokenType::new("sakuraScript"), // 11: さくらスクリプト
    SemanticTokenType::new("escape"),       // 12: エスケープシーケンス
    SemanticTokenType::OPERATOR,            // 13: コロン区切り
    SemanticTokenType::NUMBER,              // 14: 数値リテラル
    SemanticTokenType::new("cueMarker"),    // 15: キューコマンドマーカー (!/！)
    SemanticTokenType::new("cueCommand"),   // 16: キューコマンド名
];

/// トークンモディファイア
pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,   // 0: 宣言
    SemanticTokenModifier::DEFINITION,    // 1: 定義
    SemanticTokenModifier::new("global"), // 2: グローバルスコープ
];

/// Token type indices for convenience
pub mod token_type {
    pub const COMMENT: u32 = 0;
    pub const NAMESPACE: u32 = 1;
    pub const SCENE: u32 = 2;
    pub const DECORATOR: u32 = 3;
    pub const WORD: u32 = 4;
    pub const VARIABLE: u32 = 5;
    pub const CALL: u32 = 6;
    pub const ACTOR: u32 = 7;
    pub const ACTOR_NAME: u32 = 8;
    pub const CODE_BLOCK: u32 = 9;
    pub const TALK: u32 = 10;
    pub const SAKURA_SCRIPT: u32 = 11;
    pub const ESCAPE: u32 = 12;
    pub const OPERATOR: u32 = 13;
    pub const NUMBER: u32 = 14;
    pub const CUE_MARKER: u32 = 15;
    pub const CUE_COMMAND: u32 = 16;
}

/// Generate the SemanticTokensLegend for registration with the LSP client.
pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

// ============================================================================
// Raw Token (intermediate representation)
// ============================================================================

/// 中間表現: 絶対位置トークン（AST走査で生成）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    /// 0-based行番号
    pub line: u32,
    /// 0-based UTF-16列オフセット
    pub start_char: u32,
    /// UTF-16コードユニット数
    pub length: u32,
    /// TOKEN_TYPESのインデックス
    pub token_type: u32,
    /// TOKEN_MODIFIERSのビットマスク
    pub modifiers: u32,
}

// ============================================================================
// UTF-8 → UTF-16 Conversion
// ============================================================================

/// UTF-8テキストの行内バイトオフセットをUTF-16コードユニットオフセットに変換
pub fn utf8_offset_to_utf16(line_text: &str, byte_offset: usize) -> u32 {
    if byte_offset == 0 {
        return 0;
    }
    let clamped = byte_offset.min(line_text.len());
    line_text[..clamped].encode_utf16().count() as u32
}

/// UTF-8テキストのバイト長をUTF-16コードユニット数に変換
pub fn utf8_len_to_utf16(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

// ============================================================================
// Delta Encoding
// ============================================================================

/// RawToken列をLSP deltaエンコーディングに変換
pub fn encode_tokens(raw: &mut [RawToken]) -> Vec<SemanticToken> {
    if raw.is_empty() {
        return vec![];
    }

    raw.sort_by(|a, b| a.line.cmp(&b.line).then(a.start_char.cmp(&b.start_char)));

    let mut result = Vec::with_capacity(raw.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in raw.iter() {
        let delta_line = token.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.start_char - prev_start
        } else {
            token.start_char
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.modifiers,
        });

        prev_line = token.line;
        prev_start = token.start_char;
    }

    result
}
