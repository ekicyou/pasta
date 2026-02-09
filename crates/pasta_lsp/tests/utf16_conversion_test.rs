//! UTF-16位置変換テスト (Task 7.4)
//!
//! BMP内文字、BMP外文字、サロゲートペア、結合文字の変換正確性テスト

use pasta_lsp::analysis::{utf8_len_to_utf16, utf8_offset_to_utf16};

// ============================================================================
// BMP内文字（ASCII）
// ============================================================================

#[test]
fn test_utf16_ascii_offset() {
    let text = "hello world";
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 5), 5);
    assert_eq!(utf8_offset_to_utf16(text, 11), 11);
}

#[test]
fn test_utf16_ascii_length() {
    assert_eq!(utf8_len_to_utf16("hello"), 5);
    assert_eq!(utf8_len_to_utf16(""), 0);
    assert_eq!(utf8_len_to_utf16("a"), 1);
}

// ============================================================================
// BMP内文字（日本語）
// ============================================================================

#[test]
fn test_utf16_japanese_offset() {
    let text = "＊挨拶";
    // ＊ = 3 bytes UTF-8, 1 UTF-16 code unit
    // 挨 = 3 bytes UTF-8, 1 UTF-16 code unit
    // 拶 = 3 bytes UTF-8, 1 UTF-16 code unit
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 3), 1);
    assert_eq!(utf8_offset_to_utf16(text, 6), 2);
    assert_eq!(utf8_offset_to_utf16(text, 9), 3);
}

#[test]
fn test_utf16_japanese_length() {
    assert_eq!(utf8_len_to_utf16("こんにちは"), 5);
    assert_eq!(utf8_len_to_utf16("＊"), 1);
    assert_eq!(utf8_len_to_utf16("Alice"), 5);
}

// ============================================================================
// BMP外文字（絵文字）— サロゲートペア対応
// ============================================================================

#[test]
fn test_utf16_emoji_offset() {
    let text = "a😀b";
    // 'a' = 1 byte UTF-8, 1 UTF-16 code unit
    // '😀' = 4 bytes UTF-8, 2 UTF-16 code units (surrogate pair)
    // 'b' = 1 byte UTF-8, 1 UTF-16 code unit
    assert_eq!(utf8_offset_to_utf16(text, 0), 0); // before 'a'
    assert_eq!(utf8_offset_to_utf16(text, 1), 1); // before emoji
    assert_eq!(utf8_offset_to_utf16(text, 5), 3); // before 'b' (emoji = 2 units)
    assert_eq!(utf8_offset_to_utf16(text, 6), 4); // end
}

#[test]
fn test_utf16_emoji_length() {
    assert_eq!(utf8_len_to_utf16("😀"), 2); // surrogate pair
    assert_eq!(utf8_len_to_utf16("a😀b"), 4); // 1 + 2 + 1
}

#[test]
fn test_utf16_multiple_emoji() {
    let text = "😀😁";
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 4), 2); // after first emoji
    assert_eq!(utf8_offset_to_utf16(text, 8), 4); // end
    assert_eq!(utf8_len_to_utf16(text), 4);
}

// ============================================================================
// CJK拡張B（BMP外、サロゲートペア必要）
// ============================================================================

#[test]
fn test_utf16_cjk_extension_b() {
    // U+20000 (𠀀) is in CJK Unified Ideographs Extension B
    // 4 bytes in UTF-8, 2 code units in UTF-16
    let text = "𠀀";
    assert_eq!(utf8_len_to_utf16(text), 2);
    assert_eq!(utf8_offset_to_utf16(text, 0), 0);
    assert_eq!(utf8_offset_to_utf16(text, 4), 2);
}

// ============================================================================
// 混合テキスト
// ============================================================================

#[test]
fn test_utf16_mixed_ascii_japanese_emoji() {
    let text = "A＊😀";
    // 'A' = 1 byte, 1 UTF-16 unit
    // '＊' = 3 bytes, 1 UTF-16 unit
    // '😀' = 4 bytes, 2 UTF-16 units
    assert_eq!(utf8_offset_to_utf16(text, 0), 0); // before A
    assert_eq!(utf8_offset_to_utf16(text, 1), 1); // before ＊
    assert_eq!(utf8_offset_to_utf16(text, 4), 2); // before emoji
    assert_eq!(utf8_offset_to_utf16(text, 8), 4); // end
}

// ============================================================================
// 境界条件
// ============================================================================

#[test]
fn test_utf16_offset_zero_always_zero() {
    assert_eq!(utf8_offset_to_utf16("anything", 0), 0);
    assert_eq!(utf8_offset_to_utf16("", 0), 0);
    assert_eq!(utf8_offset_to_utf16("日本語", 0), 0);
}

#[test]
fn test_utf16_offset_beyond_text_length() {
    let text = "hello";
    // Should clamp to text length
    assert_eq!(utf8_offset_to_utf16(text, 100), 5);
}

#[test]
fn test_utf16_empty_text() {
    assert_eq!(utf8_offset_to_utf16("", 0), 0);
    assert_eq!(utf8_len_to_utf16(""), 0);
}
