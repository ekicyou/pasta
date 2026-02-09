//! Error types for the Pasta Language Server.

/// pasta_lsp固有のエラー型
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
