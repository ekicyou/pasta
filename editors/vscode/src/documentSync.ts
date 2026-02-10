// DocumentSync - Watches document changes and triggers WASM analysis
//
// Monitors open/change/close events for *.pasta files,
// debounces changes, and dispatches analysis results.

import * as vscode from 'vscode';
import { WasmBridge } from './wasmBridge';
import { SemanticTokensProvider } from './semanticTokensProvider';
import { DiagnosticsManager } from './diagnosticsManager';

/** Debounce delay in milliseconds */
const DEBOUNCE_MS = 200;

/**
 * Synchronizes document changes with WASM analysis.
 *
 * Watches for document open/change/close events on *.pasta files,
 * debounces rapid edits (200ms), and triggers analysis through WasmBridge.
 */
export class DocumentSync implements vscode.Disposable {
  private readonly wasmBridge: WasmBridge;
  private readonly semanticTokensProvider: SemanticTokensProvider;
  private readonly diagnosticsManager: DiagnosticsManager;
  private readonly outputChannel: vscode.OutputChannel;
  private readonly disposables: vscode.Disposable[] = [];
  private readonly debounceTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();

  constructor(
    wasmBridge: WasmBridge,
    semanticTokensProvider: SemanticTokensProvider,
    diagnosticsManager: DiagnosticsManager,
    outputChannel: vscode.OutputChannel
  ) {
    this.wasmBridge = wasmBridge;
    this.semanticTokensProvider = semanticTokensProvider;
    this.diagnosticsManager = diagnosticsManager;
    this.outputChannel = outputChannel;
  }

  /** Register document event listeners */
  activate(context: vscode.ExtensionContext): void {
    this.disposables.push(
      vscode.workspace.onDidOpenTextDocument((doc) => this.onDocumentOpen(doc)),
      vscode.workspace.onDidChangeTextDocument((e) => this.onDocumentChange(e)),
      vscode.workspace.onDidCloseTextDocument((doc) => this.onDocumentClose(doc))
    );

    // Analyze any already-open pasta documents
    for (const doc of vscode.workspace.textDocuments) {
      if (this.isPastaDocument(doc)) {
        this.analyzeDocument(doc);
      }
    }
  }

  dispose(): void {
    // Clear all pending timers
    for (const timer of this.debounceTimers.values()) {
      clearTimeout(timer);
    }
    this.debounceTimers.clear();

    for (const d of this.disposables) {
      d.dispose();
    }
    this.disposables.length = 0;
  }

  private onDocumentOpen(document: vscode.TextDocument): void {
    if (!this.isPastaDocument(document)) {
      return;
    }
    this.analyzeDocument(document);
  }

  private onDocumentChange(event: vscode.TextDocumentChangeEvent): void {
    if (!this.isPastaDocument(event.document)) {
      return;
    }
    this.debouncedAnalyze(event.document);
  }

  private onDocumentClose(document: vscode.TextDocument): void {
    if (!this.isPastaDocument(document)) {
      return;
    }

    const uri = document.uri.toString();

    // Clear pending timer
    const timer = this.debounceTimers.get(uri);
    if (timer) {
      clearTimeout(timer);
      this.debounceTimers.delete(uri);
    }

    // Clear cached data
    this.semanticTokensProvider.clearTokens(uri);
    this.diagnosticsManager.clear(document.uri);
  }

  private debouncedAnalyze(document: vscode.TextDocument): void {
    const uri = document.uri.toString();

    // Clear existing timer
    const existing = this.debounceTimers.get(uri);
    if (existing) {
      clearTimeout(existing);
    }

    // Set new debounce timer
    const timer = setTimeout(() => {
      this.debounceTimers.delete(uri);
      this.analyzeDocument(document);
    }, DEBOUNCE_MS);

    this.debounceTimers.set(uri, timer);
  }

  private analyzeDocument(document: vscode.TextDocument): void {
    if (!this.wasmBridge.isReady()) {
      return;
    }

    try {
      const text = document.getText();
      const { tokens, diagnostics } = this.wasmBridge.analyzeDocument(text);

      this.semanticTokensProvider.updateTokens(document.uri.toString(), tokens);
      this.diagnosticsManager.setDiagnostics(document.uri, diagnostics);
    } catch (error) {
      this.outputChannel.appendLine(`Document analysis failed for ${document.uri}: ${error}`);
      this.diagnosticsManager.clear(document.uri);
      // TextMate grammar highlights remain active as fallback
    }
  }

  private isPastaDocument(document: vscode.TextDocument): boolean {
    return document.languageId === 'pasta';
  }
}
