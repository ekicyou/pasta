// Pasta DSL VSCode Extension - Entry Point
//
// Provides semantic highlighting and diagnostics for *.pasta files
// using pasta_lsp compiled to WebAssembly.

import * as vscode from 'vscode';
import { WasmBridge } from './wasmBridge';
import { DocumentSync } from './documentSync';
import { SemanticTokensProvider, PASTA_TOKENS_LEGEND } from './semanticTokensProvider';
import { DiagnosticsManager } from './diagnosticsManager';
import { activateWordRefDecorator } from './wordRefDecorator';
import { PastaDebugAdapterFactory, PastaDebugConfigurationProvider } from './debugAdapterFactory';

/** Activation state of the extension */
export interface ActivationState {
  wasmReady: boolean;
  fallbackMode: boolean;
}

const activationState: ActivationState = {
  wasmReady: false,
  fallbackMode: false,
};

let wasmBridge: WasmBridge | undefined;
let documentSync: DocumentSync | undefined;
let diagnosticsManager: DiagnosticsManager | undefined;
let outputChannel: vscode.OutputChannel | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  outputChannel = vscode.window.createOutputChannel('Pasta Language');
  context.subscriptions.push(outputChannel);
  outputChannel.appendLine('Pasta DSL extension activating...');

  // Initialize word-ref box decorations
  activateWordRefDecorator(context);

  // Register the Pasta debug adapter factory (attach to the Rust DAP backend).
  // Independent of WASM readiness — debugging targets a running pasta_lua VM.
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory('pasta', new PastaDebugAdapterFactory())
  );

  // Normalise the `sourcePresentation` attach arg before VSCode forwards the
  // configuration to the Rust DAP backend (Task 6.2 / requirement 6.3):
  // explicit value -> forwarded verbatim; absent/invalid -> removed so the
  // server falls back to its own env > file > 既定 resolution.
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider('pasta', new PastaDebugConfigurationProvider())
  );

  // Initialize diagnostics manager
  diagnosticsManager = new DiagnosticsManager();
  context.subscriptions.push(diagnosticsManager);

  // Initialize WASM bridge
  wasmBridge = new WasmBridge(outputChannel);

  try {
    const wasmUri = vscode.Uri.joinPath(context.extensionUri, 'wasm', 'pasta_lsp_wasm_bg.wasm');
    await wasmBridge.initialize(wasmUri);
    activationState.wasmReady = true;
    outputChannel.appendLine('WASM bridge initialized successfully.');
  } catch (error) {
    activationState.fallbackMode = true;
    outputChannel.appendLine(`WASM initialization failed: ${error}`);
    vscode.window.showErrorMessage(
      `Pasta WASM initialization failed: ${error}. Using TextMate grammar fallback.`
    );
  }

  // Register semantic tokens provider (only if WASM is ready)
  if (activationState.wasmReady && wasmBridge) {
    const semanticTokensProvider = new SemanticTokensProvider(wasmBridge);
    context.subscriptions.push(
      vscode.languages.registerDocumentSemanticTokensProvider(
        { language: 'pasta' },
        semanticTokensProvider,
        PASTA_TOKENS_LEGEND
      )
    );

    // Initialize document sync
    documentSync = new DocumentSync(wasmBridge, semanticTokensProvider, diagnosticsManager, outputChannel);
    context.subscriptions.push(documentSync);
    documentSync.activate(context);
  }

  outputChannel.appendLine(
    `Pasta DSL extension activated. Mode: ${activationState.wasmReady ? 'Full (WASM + TextMate)' : 'Fallback (TextMate only)'}`
  );
}

export function deactivate(): void {
  wasmBridge?.dispose();
  documentSync?.dispose();
  diagnosticsManager?.dispose();
  outputChannel?.appendLine('Pasta DSL extension deactivated.');
}

/** Get current activation state (for testing) */
export function getActivationState(): Readonly<ActivationState> {
  return { ...activationState };
}
