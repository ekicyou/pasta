// DiagnosticsManager - Manages VSCode diagnostic collection
//
// Converts WASM analysis diagnostics into VSCode Diagnostic objects
// and displays them in the Problems panel.

import * as vscode from 'vscode';
import { DiagnosticData } from './wasmBridge';

/**
 * Manages diagnostics for *.pasta documents.
 *
 * Maintains a DiagnosticCollection and updates it when
 * WASM analysis produces new diagnostic data.
 */
export class DiagnosticsManager implements vscode.Disposable {
  private readonly collection: vscode.DiagnosticCollection;

  constructor() {
    this.collection = vscode.languages.createDiagnosticCollection('pasta');
  }

  /**
   * Set diagnostics for a document URI.
   * Replaces any existing diagnostics for the URI.
   */
  setDiagnostics(uri: vscode.Uri, diagnosticData: ReadonlyArray<DiagnosticData>): void {
    const diagnostics = diagnosticData.map((d) => this.convertDiagnostic(d));
    this.collection.set(uri, diagnostics);
  }

  /** Clear diagnostics for a specific document */
  clear(uri: vscode.Uri): void {
    this.collection.delete(uri);
  }

  dispose(): void {
    this.collection.dispose();
  }

  private convertDiagnostic(data: DiagnosticData): vscode.Diagnostic {
    const range = new vscode.Range(
      new vscode.Position(data.range.start.line, data.range.start.character),
      new vscode.Position(data.range.end.line, data.range.end.character)
    );

    // 入力のたびの診断は構文の誤りだけなので、重さは常に Error
    return new vscode.Diagnostic(range, data.message, vscode.DiagnosticSeverity.Error);
  }
}
