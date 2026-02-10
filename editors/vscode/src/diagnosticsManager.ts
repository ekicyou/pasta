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

  /** Clear all diagnostics */
  clearAll(): void {
    this.collection.clear();
  }

  dispose(): void {
    this.collection.dispose();
  }

  private convertDiagnostic(data: DiagnosticData): vscode.Diagnostic {
    const range = new vscode.Range(
      new vscode.Position(data.range.start.line, data.range.start.character),
      new vscode.Position(data.range.end.line, data.range.end.character)
    );

    const severity = this.convertSeverity(data.severity);

    return new vscode.Diagnostic(range, data.message, severity);
  }

  private convertSeverity(severity: number): vscode.DiagnosticSeverity {
    // LSP DiagnosticSeverity: 1=Error, 2=Warning, 3=Information, 4=Hint
    switch (severity) {
      case 1:
        return vscode.DiagnosticSeverity.Error;
      case 2:
        return vscode.DiagnosticSeverity.Warning;
      case 3:
        return vscode.DiagnosticSeverity.Information;
      case 4:
        return vscode.DiagnosticSeverity.Hint;
      default:
        return vscode.DiagnosticSeverity.Error;
    }
  }
}
