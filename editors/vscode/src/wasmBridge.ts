// WasmBridge - Bridge between TypeScript and pasta_lsp.wasm
//
// Loads the WASM module and exposes analyzeDocument() for
// semantic token analysis and diagnostics.

import * as vscode from 'vscode';

/** Semantic token data from WASM analysis */
export interface SemanticTokenData {
  deltaLine: number;
  deltaStartCharacter: number;
  length: number;
  tokenType: number;
  tokenModifiers: number;
}

/** Diagnostic data from WASM analysis */
export interface DiagnosticData {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  message: string;
  severity: number;
}

/** Result of WASM analysis */
export interface WasmAnalysisResult {
  tokens: ReadonlyArray<SemanticTokenData>;
  diagnostics: ReadonlyArray<DiagnosticData>;
}

/**
 * Bridge to pasta_lsp WASM module.
 *
 * Loads and initializes the WASM binary, then provides
 * synchronous document analysis via wasm_analyze().
 */
export class WasmBridge {
  private wasmModule: any = null;
  private ready = false;
  private readonly outputChannel: vscode.OutputChannel;

  constructor(outputChannel: vscode.OutputChannel) {
    this.outputChannel = outputChannel;
  }

  /**
   * Initialize the WASM module from the given URI.
   * @throws Error if WASM loading or initialization fails
   */
  async initialize(wasmUri: vscode.Uri): Promise<void> {
    const startTime = Date.now();

    try {
      // The wasm-pack --target nodejs output uses __dirname to find
      // the .wasm file, and auto-initializes via require('fs').readFileSync.
      // We need to resolve the JS bindings path relative to the extension.
      const wasmDir = vscode.Uri.joinPath(wasmUri, '..');
      const jsBindingsPath = vscode.Uri.joinPath(wasmDir, 'pasta_lsp_wasm.js').fsPath;

      // Use require() to load the CJS wasm-bindgen bindings.
      // The module self-initializes by reading the .wasm file from __dirname.
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const wasmBindings = require(jsBindingsPath);

      this.wasmModule = wasmBindings;
      this.ready = true;

      const elapsed = Date.now() - startTime;
      this.outputChannel.appendLine(`WASM module loaded in ${elapsed}ms`);
    } catch (error) {
      this.ready = false;
      throw new Error(`Failed to load WASM module: ${error}`);
    }
  }

  /**
   * Analyze a document and return semantic tokens + diagnostics.
   * @throws Error if WASM is not initialized or analysis fails
   */
  analyzeDocument(text: string): WasmAnalysisResult {
    if (!this.ready || !this.wasmModule) {
      throw new Error('WASM module is not initialized');
    }

    try {
      const result = this.wasmModule.wasm_analyze(text);
      return result as WasmAnalysisResult;
    } catch (error) {
      throw new Error(`WASM analysis failed: ${error}`);
    }
  }

  /** Check if the WASM module is ready for use */
  isReady(): boolean {
    return this.ready;
  }

  /** Dispose WASM resources */
  dispose(): void {
    this.wasmModule = null;
    this.ready = false;
  }
}
