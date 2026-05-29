//! Transport layer for the Pasta Language Server.
//!
//! Provides platform abstraction for WASM and native environments.
//! WASM entry point exposes `wasm_analyze()` which wraps `AnalysisEngine::analyze()`
//! and returns results as JS objects via `serde-wasm-bindgen`.

use serde::Serialize;

/// WASM-friendly semantic token data (mirrors tower-lsp SemanticToken fields)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmSemanticToken {
    pub delta_line: u32,
    pub delta_start_character: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers: u32,
}

/// WASM-friendly diagnostic data (mirrors tower-lsp Diagnostic fields)
#[derive(Debug, Clone, Serialize)]
pub struct WasmDiagnostic {
    pub range: WasmRange,
    pub message: String,
    pub severity: u32,
}

/// WASM-friendly range
#[derive(Debug, Clone, Serialize)]
pub struct WasmRange {
    pub start: WasmPosition,
    pub end: WasmPosition,
}

/// WASM-friendly position
#[derive(Debug, Clone, Serialize)]
pub struct WasmPosition {
    pub line: u32,
    pub character: u32,
}

/// WASM analysis result containing tokens and diagnostics
#[derive(Debug, Clone, Serialize)]
pub struct WasmAnalysisResult {
    pub tokens: Vec<WasmSemanticToken>,
    pub diagnostics: Vec<WasmDiagnostic>,
}

impl WasmAnalysisResult {
    /// Convert from AnalysisEngine result (tower-lsp types) to WASM-friendly types
    pub fn from_analysis(result: &crate::analysis::AnalysisResult) -> Self {
        let tokens = result
            .tokens
            .iter()
            .map(|t| WasmSemanticToken {
                delta_line: t.delta_line,
                delta_start_character: t.delta_start,
                length: t.length,
                token_type: t.token_type,
                token_modifiers: t.token_modifiers_bitset,
            })
            .collect();

        let diagnostics = result
            .diagnostics
            .iter()
            .map(|d| {
                use tower_lsp::lsp_types::DiagnosticSeverity;
                let severity = match d.severity {
                    Some(DiagnosticSeverity::ERROR) => 1,
                    Some(DiagnosticSeverity::WARNING) => 2,
                    Some(DiagnosticSeverity::INFORMATION) => 3,
                    Some(DiagnosticSeverity::HINT) => 4,
                    _ => 1, // Default to Error
                };
                WasmDiagnostic {
                    range: WasmRange {
                        start: WasmPosition {
                            line: d.range.start.line,
                            character: d.range.start.character,
                        },
                        end: WasmPosition {
                            line: d.range.end.line,
                            character: d.range.end.character,
                        },
                    },
                    message: d.message.clone(),
                    severity,
                }
            })
            .collect();

        WasmAnalysisResult {
            tokens,
            diagnostics,
        }
    }
}

/// WASM entry point (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use crate::analysis::AnalysisEngine;
    use wasm_bindgen::prelude::*;

    /// Analyze a pasta source string and return semantic tokens + diagnostics.
    ///
    /// This is the primary WASM entry point called from the VSCode extension.
    /// Results are returned as a JS object via serde-wasm-bindgen.
    ///
    /// `wasm-bindgen` converts JavaScript strings into Rust `&str`.
    /// JavaScript strings are UTF-16; unpaired surrogates are replaced with
    /// U+FFFD during conversion, so the Rust side always receives valid UTF-8.
    /// Invalid byte sequences cannot reach `AnalysisEngine::analyze()`.
    ///
    /// # Panics
    /// Protected by `catch_unwind` — returns an empty result on parser panic.
    #[wasm_bindgen]
    pub fn wasm_analyze(source: &str) -> JsValue {
        // `AnalysisEngine::analyze("")` is already expected to return an empty,
        // non-panicking result, so an empty JS string safely maps to the same path.
        // `catch_unwind` keeps any upstream parser panic from unwinding across the
        // WASM ABI boundary and crashing the embedding JavaScript host.
        let result = std::panic::catch_unwind(|| AnalysisEngine::analyze(source));

        let analysis_result = match result {
            Ok(r) => WasmAnalysisResult::from_analysis(&r),
            Err(_) => WasmAnalysisResult {
                tokens: vec![],
                diagnostics: vec![],
            },
        };

        // `WasmAnalysisResult` and all nested types derive `Serialize`, so
        // `serde-wasm-bindgen` converts a statically defined Rust shape into a
        // `JsValue`. If serialization still fails unexpectedly, return `null`
        // instead of panicking across the WASM boundary.
        serde_wasm_bindgen::to_value(&analysis_result).unwrap_or(JsValue::NULL)
    }
}

/// Native entry point stub (for future stdio-based transport)
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    // Reserved for future native (stdio/TCP) transport implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::AnalysisEngine;

    #[test]
    fn test_wasm_analysis_result_from_simple_scene() {
        let source = "＊挨拶\n  Alice：こんにちは\n";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        assert!(!wasm_result.tokens.is_empty(), "should have tokens");
        assert!(
            wasm_result.diagnostics.is_empty(),
            "should have no diagnostics"
        );
    }

    #[test]
    fn test_wasm_analysis_result_from_error() {
        // Invalid syntax that triggers partial parse
        let source = "＊\n";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        // Should produce some result (possibly with diagnostics)
        // The exact behavior depends on pasta_dsl's handling
        // Simply verify conversion doesn't panic
        let _total = wasm_result.tokens.len() + wasm_result.diagnostics.len();
    }

    #[test]
    fn test_wasm_analysis_result_empty_input() {
        let source = "";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        assert!(
            wasm_result.tokens.is_empty(),
            "empty input should have no tokens"
        );
    }

    #[test]
    fn test_wasm_semantic_token_field_mapping() {
        // Use a global scene which is reliably tokenized by AnalysisEngine
        let source = "＊挨拶\n  Alice：こんにちは\n";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        assert!(
            !wasm_result.tokens.is_empty(),
            "scene should produce tokens"
        );
        let first = &wasm_result.tokens[0];
        // Global scene marker => NAMESPACE => token_type 1
        assert_eq!(
            first.token_type, 1,
            "global scene should be token type 1 (NAMESPACE)"
        );
        assert_eq!(first.delta_line, 0, "first token should be on line 0");
        assert!(first.length > 0, "token length should be > 0");
    }

    #[test]
    fn test_wasm_analysis_all_marker_types() {
        // Use the same minimal document that works in semantic_token_test
        // A single global scene with action line produces 5+ tokens:
        //   NAMESPACE(1), SCENE(2), ACTOR_NAME(8), OPERATOR(13), STRING(10)
        let source = "＊挨拶\n  Alice：こんにちは\n";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        assert!(
            wasm_result.tokens.len() >= 3,
            "should have multiple tokens (NAMESPACE, SCENE, ACTOR_NAME, etc.), got {}",
            wasm_result.tokens.len()
        );

        // Verify token types are correctly mapped through WasmAnalysisResult
        let token_types: Vec<u32> = wasm_result.tokens.iter().map(|t| t.token_type).collect();
        assert!(
            token_types.contains(&1), // NAMESPACE for global scene
            "should contain NAMESPACE (1), got {:?}",
            token_types
        );
    }

    #[test]
    fn test_wasm_diagnostic_severity_mapping() {
        // Use a Diagnostic with known severity
        use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

        let analysis_result = crate::analysis::AnalysisResult {
            tokens: vec![],
            diagnostics: vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "test error".to_string(),
                ..Default::default()
            }],
        };

        let wasm_result = WasmAnalysisResult::from_analysis(&analysis_result);
        assert_eq!(wasm_result.diagnostics.len(), 1);
        assert_eq!(wasm_result.diagnostics[0].severity, 1); // Error = 1
        assert_eq!(wasm_result.diagnostics[0].message, "test error");
        assert_eq!(wasm_result.diagnostics[0].range.start.line, 0);
        assert_eq!(wasm_result.diagnostics[0].range.end.character, 5);
    }

    #[test]
    fn test_wasm_analysis_result_serializable() {
        let source = "＊テスト\n  Alice：こんにちは\n";
        let result = AnalysisEngine::analyze(source);
        let wasm_result = WasmAnalysisResult::from_analysis(&result);

        // Verify it can be serialized to JSON (same as serde-wasm-bindgen path)
        let json = serde_json::to_string(&wasm_result).expect("should serialize to JSON");
        assert!(json.contains("tokens"), "JSON should contain tokens field");
        assert!(
            json.contains("diagnostics"),
            "JSON should contain diagnostics field"
        );
    }
}
