//! Span type for source location tracking.

// ============================================================================
// Span - Source Location
// ============================================================================

/// Error type for Span operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpanError {
    /// Byte offset is out of bounds for the source text.
    #[error("byte offset out of bounds: {start}..{end} (source length: {source_len})")]
    OutOfBounds {
        start: usize,
        end: usize,
        source_len: usize,
    },
    /// Byte offset does not fall on a valid UTF-8 character boundary.
    #[error("invalid UTF-8 boundary at byte {byte}")]
    InvalidUtf8Boundary { byte: usize },
    /// Span is invalid (default/uninitialized).
    #[error("invalid span (uninitialized or default)")]
    InvalidSpan,
}

/// Source location in the original file.
///
/// All AST nodes carry span information for error reporting and debugging.
/// Includes both line/column positions (1-based) and byte offsets (0-based)
/// for precise source code reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// 1-based start line number
    pub start_line: usize,
    /// 1-based start column number
    pub start_col: usize,
    /// 1-based end line number
    pub end_line: usize,
    /// 1-based end column number
    pub end_col: usize,
    /// 0-based start byte offset from file beginning
    pub start_byte: usize,
    /// 0-based end byte offset from file beginning (exclusive)
    pub end_byte: usize,
}

impl Span {
    /// Create a new span with explicit coordinates including byte offsets.
    pub fn new(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            start_byte,
            end_byte,
        }
    }

    /// Extract the corresponding source code substring from the original source.
    ///
    /// # Arguments
    /// - `source`: The original source text
    ///
    /// # Returns
    /// - `Ok(&str)`: The substring corresponding to this span
    /// - `Err(SpanError)`: If byte offsets are out of bounds or invalid
    pub fn extract_source<'a>(&self, source: &'a str) -> Result<&'a str, SpanError> {
        // Check if span is valid (not default/uninitialized)
        if !self.is_valid() {
            return Err(SpanError::InvalidSpan);
        }

        // Check bounds
        if self.end_byte > source.len() {
            return Err(SpanError::OutOfBounds {
                start: self.start_byte,
                end: self.end_byte,
                source_len: source.len(),
            });
        }

        // Check UTF-8 boundaries
        if !source.is_char_boundary(self.start_byte) {
            return Err(SpanError::InvalidUtf8Boundary {
                byte: self.start_byte,
            });
        }
        if !source.is_char_boundary(self.end_byte) {
            return Err(SpanError::InvalidUtf8Boundary {
                byte: self.end_byte,
            });
        }

        Ok(&source[self.start_byte..self.end_byte])
    }

    /// Check if this span contains valid position information.
    ///
    /// A span is considered invalid if end_byte is 0,
    /// which indicates an uninitialized or default span.
    /// A valid span must have end_byte > 0.
    pub fn is_valid(&self) -> bool {
        // A valid span must have non-zero end_byte
        self.end_byte > 0
    }

    /// Get the byte length of this span.
    pub fn byte_len(&self) -> usize {
        self.end_byte.saturating_sub(self.start_byte)
    }
}

impl<'i> From<&pest::Span<'i>> for Span {
    /// Convert a pest::Span to our Span type.
    ///
    /// This extracts line/column positions (1-based) and byte offsets (0-based)
    /// from the pest Span.
    fn from(pest_span: &pest::Span<'i>) -> Self {
        let (start_line, start_col) = pest_span.start_pos().line_col();
        let (end_line, end_col) = pest_span.end_pos().line_col();
        Self::new(
            start_line,
            start_col,
            end_line,
            end_col,
            pest_span.start(),
            pest_span.end(),
        )
    }
}
