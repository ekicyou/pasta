//! LSP Protocol Handler for the Pasta Language Server.
//!
//! Implements the tower-lsp `LanguageServer` trait for handling
//! LSP protocol messages.

use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::analysis::{self, AnalysisEngine};
use crate::document::{ContentChange, DocumentManager, TextRange};

/// Pasta Language Server
pub struct PastaLangServer {
    /// LSP client for sending notifications
    client: Client,
    /// Document manager (thread-safe via RwLock)
    documents: RwLock<DocumentManager>,
}

impl PastaLangServer {
    /// Create a new PastaLangServer instance.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(DocumentManager::new()),
        }
    }

    /// Analyze a document and publish diagnostics.
    async fn analyze_and_publish(&self, uri: &Url, text: &str) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            AnalysisEngine::analyze(text)
        }));

        match result {
            Ok(analysis_result) => {
                // Cache the analysis result
                if let Ok(mut docs) = self.documents.write() {
                    if let Some(doc) = docs.get_mut(uri.as_str()) {
                        doc.analysis = Some(analysis_result.clone());
                    }
                }

                // Publish diagnostics
                self.client
                    .publish_diagnostics(uri.clone(), analysis_result.diagnostics, None)
                    .await;
            }
            Err(_) => {
                // Parser panicked — log and continue
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Parser panic for document: {}", uri),
                    )
                    .await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PastaLangServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: analysis::semantic_tokens_legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Pasta Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        {
            let Ok(mut docs) = self.documents.write() else {
                return;
            };
            docs.open(uri.to_string(), text.clone(), version);
        }

        self.analyze_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        let changes: Vec<ContentChange> = params
            .content_changes
            .iter()
            .map(|c| ContentChange {
                range: c.range.map(|r| TextRange {
                    start_line: r.start.line,
                    start_col: r.start.character,
                    end_line: r.end.line,
                    end_col: r.end.character,
                }),
                text: c.text.clone(),
            })
            .collect();

        let text = {
            let Ok(mut docs) = self.documents.write() else {
                return;
            };
            docs.change(uri.as_str(), &changes, version);
            docs.get(uri.as_str())
                .map(|d| d.text.clone())
                .unwrap_or_default()
        };

        self.analyze_and_publish(&uri, &text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let Ok(mut docs) = self.documents.write() else {
                return;
            };
            docs.close(uri.as_str());
        }
        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        let tokens = {
            let Ok(docs) = self.documents.read() else {
                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: vec![],
                })));
            };
            docs.get(uri.as_str())
                .and_then(|d| d.analysis.as_ref())
                .map(|a| a.tokens.clone())
                .unwrap_or_default()
        };

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}
