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
import {
  type SourcePresentation,
  requestCommand,
  nextMode,
  setPayload,
  parseMode,
  statusLabel,
} from './sourcePresentationToggle';

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

  // Register the runtime source-presentation toggle wiring (command + status
  // bar + custom-event subscription). All branching/format decisions are
  // delegated to the pure, unit-tested `sourcePresentationToggle` module;
  // this is thin vscode glue (requirements 2.1–2.6, 6.4).
  registerSourcePresentationToggle(context);

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

/** The debug session type the toggle UI applies to (mirrors package.json). */
const PASTA_DEBUG_TYPE = 'pasta';

/** True when a debug session is a Pasta session the toggle applies to. */
function isPastaSession(session: vscode.DebugSession | undefined): session is vscode.DebugSession {
  return session !== undefined && session.type === PASTA_DEBUG_TYPE;
}

/**
 * Wire the runtime source-presentation toggle: the command (3 entry points —
 * palette, debug toolbar button, clickable status bar item), the always-visible
 * status bar indicator, and the backend push-event subscription that is the
 * single source of truth for the displayed mode.
 *
 * The display is driven ONLY by the backend's `pasta/sourcePresentation` custom
 * EVENT (both the attach-time initial-mode push and the post-toggle push), never
 * from the customRequest response (design "System Flows" / requirement 2.5/2.6).
 * All mode decisions go through the pure `sourcePresentationToggle` helpers.
 */
function registerSourcePresentationToggle(context: vscode.ExtensionContext): void {
  // Last mode pushed by the active Pasta session's backend. `undefined` until
  // the first `pasta/sourcePresentation` event arrives (attach-time push).
  let trackedMode: SourcePresentation | undefined;

  const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right);
  // Clicking the status bar item invokes the same toggle command — the 3rd
  // entry point (requirement 2.3).
  statusBarItem.command = 'pasta.debug.toggleSourcePresentation';
  context.subscriptions.push(statusBarItem);

  // Reflect the tracked mode (and visibility) onto the status bar. Visible only
  // while a Pasta debug session is the active session (requirement 2.5).
  const refreshStatusBar = (): void => {
    if (isPastaSession(vscode.debug.activeDebugSession) && trackedMode !== undefined) {
      statusBarItem.text = statusLabel(trackedMode);
      statusBarItem.tooltip = `Pasta 提示モード: ${trackedMode}（クリックで .pasta⇔.lua 切替）`;
      statusBarItem.show();
    } else {
      statusBarItem.hide();
    }
  };

  // The toggle command (palette / toolbar button / status bar click).
  context.subscriptions.push(
    vscode.commands.registerCommand('pasta.debug.toggleSourcePresentation', async () => {
      const session = vscode.debug.activeDebugSession;
      if (!isPastaSession(session)) {
        // No active Pasta session: do not send anything; notify the user that
        // the toggle requires an active Pasta debug session (requirement 2.4).
        await vscode.window.showWarningMessage(
          '提示モードの切替には、実行中の Pasta デバッグセッションが必要です。'
        );
        return;
      }
      // Compute the next mode from the tracked mode (defaulting to the backend
      // default `.pasta` when no push has been seen yet) and send the set
      // request. The DISPLAY is updated by the resulting push event, not here.
      const target = nextMode(trackedMode ?? 'pasta');
      try {
        await session.customRequest(requestCommand, setPayload(target));
      } catch (err) {
        // Surface the failure; the tracked mode / status bar are left unchanged
        // because the display is push-event-driven (no optimistic update).
        await vscode.window.showErrorMessage(
          `提示モードの切替要求に失敗しました: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    })
  );

  // Backend push event = single source of truth for the displayed mode. Handles
  // BOTH the attach-time initial-mode event and the post-toggle event
  // (requirements 2.5/2.6).
  context.subscriptions.push(
    vscode.debug.onDidReceiveDebugSessionCustomEvent((event) => {
      if (event.event !== requestCommand || !isPastaSession(event.session)) {
        return;
      }
      const mode = parseMode(event.body);
      if (mode === undefined) {
        return;
      }
      trackedMode = mode;
      refreshStatusBar();
    })
  );

  // Session lifecycle: show the status bar for the active Pasta session, hide it
  // otherwise. Reset the tracked mode when the relevant session ends so a fresh
  // session starts from the backend's next push.
  context.subscriptions.push(
    vscode.debug.onDidChangeActiveDebugSession((session) => {
      if (!isPastaSession(session)) {
        trackedMode = undefined;
      }
      refreshStatusBar();
    })
  );

  context.subscriptions.push(
    vscode.debug.onDidTerminateDebugSession((session) => {
      if (isPastaSession(session)) {
        trackedMode = undefined;
        refreshStatusBar();
      }
    })
  );

  // Initial state (e.g. activation during an already-running session): reflect
  // whatever the current active session is.
  refreshStatusBar();
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
