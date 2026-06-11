//! Error types for the Pasta Language Server.

/// pasta_lsp固有のエラー型
// `LangServerError` は `lib.rs` から再公開される公開 API であり、
// 現状クレート内で直接使用されていなくても downstream から各バリアントを参照・構築できる。
#[derive(Debug, thiserror::Error)]
pub enum LangServerError {
    /// pasta_dslパースエラー（Diagnosticsに変換して続行）
    #[error("Parse error: {0}")]
    Parse(String),

    /// ドキュメントが見つからない（didOpen前のリクエスト）
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// 内部エラー（パニックキャッチ含む）
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_messages_for_all_variants() {
        // Display 文字列は downstream（ログ・診断メッセージ）の可観測契約
        assert_eq!(
            LangServerError::Parse("unexpected token".to_string()).to_string(),
            "Parse error: unexpected token"
        );
        assert_eq!(
            LangServerError::DocumentNotFound("file:///a.pasta".to_string()).to_string(),
            "Document not found: file:///a.pasta"
        );
        assert_eq!(
            LangServerError::Internal("panic caught".to_string()).to_string(),
            "Internal error: panic caught"
        );
    }

    #[test]
    fn test_implements_std_error_without_source() {
        // thiserror 派生で std::error::Error を実装し、source は None（包装エラーなし）
        let err: Box<dyn std::error::Error> = Box::new(LangServerError::Parse("e".to_string()));
        assert!(err.source().is_none());
    }
}
