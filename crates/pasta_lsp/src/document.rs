//! Document manager for the Pasta Language Server.
//!
//! Manages opened documents, text synchronization, and analysis result caching.

use std::collections::HashMap;

/// ドキュメント状態
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// ドキュメント全文
    pub text: String,
    /// バージョン番号
    pub version: i32,
    /// 最新の解析結果（キャッシュ）
    pub analysis: Option<crate::analysis::AnalysisResult>,
}

impl DocumentState {
    /// Create a new document state.
    pub fn new(text: String, version: i32) -> Self {
        Self {
            text,
            version,
            analysis: None,
        }
    }
}

/// ドキュメントマネージャ
#[derive(Debug, Default)]
pub struct DocumentManager {
    documents: HashMap<String, DocumentState>,
}

impl DocumentManager {
    /// Create a new, empty document manager.
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Register a newly opened document.
    pub fn open(&mut self, uri: String, text: String, version: i32) {
        self.documents
            .insert(uri, DocumentState::new(text, version));
    }

    /// Apply changes to an opened document.
    ///
    /// Supports both full content replacement and incremental updates.
    pub fn change(&mut self, uri: &str, content_changes: &[ContentChange], version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            for change in content_changes {
                if let Some(range) = &change.range {
                    // Incremental update
                    let start_offset =
                        line_col_to_offset(&doc.text, range.start_line, range.start_col);
                    let end_offset = line_col_to_offset(&doc.text, range.end_line, range.end_col);
                    // `start <= end` guard: LSP contracts require start <= end, but a
                    // buggy/malicious client can send an inverted range, which would
                    // panic in `replace_range`. Treat it as a no-op, consistent with
                    // the existing out-of-range behavior.
                    if let (Some(start), Some(end)) = (start_offset, end_offset)
                        && start <= end
                    {
                        doc.text.replace_range(start..end, &change.text);
                    }
                } else {
                    // Full content replacement
                    doc.text = change.text.clone();
                }
            }
            doc.version = version;
            doc.analysis = None; // Invalidate cache
        }
    }

    /// Remove a closed document.
    pub fn close(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    /// Get a reference to a document state.
    pub fn get(&self, uri: &str) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    /// Get a mutable reference to a document state.
    pub fn get_mut(&mut self, uri: &str) -> Option<&mut DocumentState> {
        self.documents.get_mut(uri)
    }
}

/// A content change event (simplified from LSP's TextDocumentContentChangeEvent).
#[derive(Debug, Clone)]
pub struct ContentChange {
    /// The range of the document that changed (None = full replacement).
    pub range: Option<TextRange>,
    /// The new text for the range.
    pub text: String,
}

/// A text range (0-based line and UTF-16 column offsets).
#[derive(Debug, Clone)]
pub struct TextRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Convert 0-based line and UTF-16 column to byte offset.
fn line_col_to_offset(text: &str, line: u32, col: u32) -> Option<usize> {
    let mut byte_offset = 0usize;

    for (current_line, text_line) in text.split('\n').enumerate() {
        if current_line as u32 == line {
            // Convert UTF-16 column to byte offset within line
            let mut utf16_col = 0u32;
            for (byte_idx, ch) in text_line.char_indices() {
                if utf16_col == col {
                    return Some(byte_offset + byte_idx);
                }
                utf16_col += ch.len_utf16() as u32;
            }
            // Column at end of line
            if utf16_col == col {
                return Some(byte_offset + text_line.len());
            }
            return None;
        }
        byte_offset += text_line.len() + 1; // +1 for '\n'
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_and_get() {
        let mut dm = DocumentManager::new();
        dm.open("file:///test.pasta".to_string(), "hello".to_string(), 1);
        let doc = dm.get("file:///test.pasta").unwrap();
        assert_eq!(doc.text, "hello");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn test_full_content_change() {
        let mut dm = DocumentManager::new();
        dm.open("file:///test.pasta".to_string(), "hello".to_string(), 1);
        dm.change(
            "file:///test.pasta",
            &[ContentChange {
                range: None,
                text: "world".to_string(),
            }],
            2,
        );
        let doc = dm.get("file:///test.pasta").unwrap();
        assert_eq!(doc.text, "world");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_change_unregistered_document_is_noop() {
        let mut dm = DocumentManager::new();
        dm.change(
            "file:///missing.pasta",
            &[ContentChange {
                range: None,
                text: "world".to_string(),
            }],
            2,
        );
        assert!(dm.get("file:///missing.pasta").is_none());
    }

    #[test]
    fn test_close() {
        let mut dm = DocumentManager::new();
        dm.open("file:///test.pasta".to_string(), "hello".to_string(), 1);
        dm.close("file:///test.pasta");
        assert!(dm.get("file:///test.pasta").is_none());
    }

    #[test]
    fn test_incremental_change_out_of_range_is_noop() {
        let mut dm = DocumentManager::new();
        dm.open(
            "file:///test.pasta".to_string(),
            "hello\nworld\n".to_string(),
            1,
        );
        dm.change(
            "file:///test.pasta",
            &[ContentChange {
                range: Some(TextRange {
                    start_line: 10,
                    start_col: 0,
                    end_line: 10,
                    end_col: 1,
                }),
                text: "ignored".to_string(),
            }],
            2,
        );
        let doc = dm.get("file:///test.pasta").unwrap();
        assert_eq!(doc.text, "hello\nworld\n");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_incremental_change_inverted_range_is_noop() {
        // 境界回帰テスト（R3.3/R3.6）: LSP 契約上 range は start <= end だが、
        // 不正クライアントが逆転 range を送ると replace_range がパニックしていた
        // （slice index starts at 9 but ends at 2）。ハードニング後は no-op。
        let mut dm = DocumentManager::new();
        dm.open(
            "file:///test.pasta".to_string(),
            "hello\nworld\n".to_string(),
            1,
        );
        dm.change(
            "file:///test.pasta",
            &[ContentChange {
                range: Some(TextRange {
                    start_line: 1,
                    start_col: 3,
                    end_line: 0,
                    end_col: 2,
                }),
                text: "x".to_string(),
            }],
            2,
        );
        let doc = dm.get("file:///test.pasta").unwrap();
        assert_eq!(doc.text, "hello\nworld\n", "テキストは変更されない");
        assert_eq!(doc.version, 2, "バージョンは更新される");
    }

    #[test]
    fn test_line_col_to_offset_ascii() {
        let text = "hello\nworld\n";
        assert_eq!(line_col_to_offset(text, 0, 0), Some(0));
        assert_eq!(line_col_to_offset(text, 0, 5), Some(5));
        assert_eq!(line_col_to_offset(text, 1, 0), Some(6));
        assert_eq!(line_col_to_offset(text, 1, 5), Some(11));
    }

    #[test]
    fn test_line_col_to_offset_japanese() {
        let text = "＊挨拶\nAlice：こんにちは\n";
        // Line 0: "＊挨拶" → 3 chars, each 3 bytes (9 bytes), each 1 UTF-16 unit
        assert_eq!(line_col_to_offset(text, 0, 0), Some(0));
        assert_eq!(line_col_to_offset(text, 0, 1), Some(3)); // After ＊
        assert_eq!(line_col_to_offset(text, 0, 3), Some(9)); // End of line
        // Line 1: "Alice：こんにちは"
        assert_eq!(line_col_to_offset(text, 1, 0), Some(10)); // Start of line 1
    }

    #[test]
    fn test_multiple_content_changes_applied_sequentially() {
        // LSP didChange は複数 contentChanges を順次適用する。
        // 2 番目の change の range は 1 番目適用後のテキストに対して解釈される。
        let mut dm = DocumentManager::new();
        dm.open(
            "file:///test.pasta".to_string(),
            "hello\nworld\n".to_string(),
            1,
        );
        dm.change(
            "file:///test.pasta",
            &[
                // 1st: "hello" → "hey" (5 → 3 chars)
                ContentChange {
                    range: Some(TextRange {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 5,
                    }),
                    text: "hey".to_string(),
                },
                // 2nd: 適用後テキスト "hey" 全体 (col0..3) → "yo"
                // 元テキスト基準なら col0..3 = "hel" となり結果が異なる
                ContentChange {
                    range: Some(TextRange {
                        start_line: 0,
                        start_col: 0,
                        end_line: 0,
                        end_col: 3,
                    }),
                    text: "yo".to_string(),
                },
            ],
            2,
        );
        let doc = dm.get("file:///test.pasta").unwrap();
        assert_eq!(doc.text, "yo\nworld\n");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_line_col_to_offset_inside_surrogate_pair_returns_none() {
        // UTF-16 col がサロゲートペアの内部（😀 は 2 ユニット）を指す場合は
        // 文字境界に一致しないため None（補間や丸めをしない）
        let text = "😀x\n";
        assert_eq!(line_col_to_offset(text, 0, 0), Some(0));
        assert_eq!(line_col_to_offset(text, 0, 1), None, "ペア内部は None");
        assert_eq!(line_col_to_offset(text, 0, 2), Some(4), "😀 = 4 bytes");
        assert_eq!(line_col_to_offset(text, 0, 3), Some(5));
    }

    #[test]
    fn test_line_col_to_offset_out_of_range_returns_none() {
        let text = "hello\nworld\n";
        assert_eq!(line_col_to_offset(text, 2, 0), Some(12));
        assert_eq!(line_col_to_offset(text, 3, 0), None);
        assert_eq!(line_col_to_offset(text, 0, 6), None);
        assert_eq!(line_col_to_offset(text, 1, 6), None);
    }
}
