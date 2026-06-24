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
import {
  requestCommand as playSceneAtRequestCommand,
  setPayload as setPlaySceneAtPayload,
  resolvePastaDebugConfig,
} from './runSceneAtCursor';
import {
  requestCommand as reloadShioriRequestCommand,
  reattachDelaySchedule,
} from './reloadShiori';

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

  // Register the position-based scene-run command (.pasta editor context menu).
  // Thin vscode glue over the pure, unit-tested `runSceneAtCursor` module:
  // active-editor/cursor read, session guard + start-debugging guidance, dirty
  // check, position-based customRequest dispatch, error surface (requirements
  // 1.1–1.4, 5.1–5.3, 6.1–6.4, 7.2/7.5).
  registerRunSceneAtCursorCommand(context);

  // Register the SHIORI-reload command (.pasta editor context menu + debug
  // toolbar, connected-only). Thin vscode glue over the pure, unit-tested
  // `reloadShiori` module: connection guard, dirty check, reload customRequest
  // dispatch, then detach-and-wait/retry auto re-attach (NO auto re-kick).
  // (requirements 7.4, 9.1, 9.3, 9.4, 9.5).
  registerReloadShioriCommand(context);

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

/** The language id for `.pasta` documents (mirrors package.json `languages`). */
const PASTA_LANGUAGE_ID = 'pasta';

/**
 * Resolve the Pasta debug configuration to start/re-attach (requirement 6.3).
 * Reads the workspace `launch.json` configurations and delegates the choice to
 * the pure `resolvePastaDebugConfig` (workspace `type: 'pasta'` preferred, else
 * default attach `127.0.0.1:9276`). Shared with task 5.2's re-attach.
 */
function resolveStartDebuggingConfig() {
  const launchConfigs = vscode.workspace
    .getConfiguration('launch')
    .get<unknown>('configurations');
  return resolvePastaDebugConfig(launchConfigs);
}

/**
 * Wire the position-based scene-run command (`pasta.runSceneAtCursor`). The
 * `editor/context` menu always shows it for `.pasta` (NOT debug-gated); the
 * connection check happens at invoke time. All pure decisions (command string,
 * payload, config resolution) are delegated to the `runSceneAtCursor` helpers.
 *
 * Flow (requirements 1.1–1.4, 5.1–5.3, 6.1–6.4, 7.2/7.5):
 *  - No active `.pasta` editor      -> warn and return (defensive).
 *  - Not connected                  -> send nothing; warn + offer a
 *    「デバッグ開始」action that starts a session via `startDebugging` with the
 *    resolved config (R6.2/R6.3). NEVER prompts for a scene name (R1.4).
 *  - Dirty buffer                   -> warn the running ghost may differ; offer
 *    「保存」 (best-effort; proceed after) (R7.2/R7.5).
 *  - Connected + valid cursor       -> `customRequest('pasta/playSceneAt',
 *    { uri, line })` with a 1-based line (R1.2/R1.3/R4.1). A thrown request OR a
 *    resolved `success:false` body surfaces the reason via showErrorMessage
 *    (R6.4).
 */
function registerRunSceneAtCursorCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('pasta.runSceneAtCursor', async () => {
      // Defensive: require an active `.pasta` editor (R1.1/R1.2).
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== PASTA_LANGUAGE_ID) {
        await vscode.window.showWarningMessage(
          '「▶ シーンを実行」には、アクティブな .pasta エディタが必要です。'
        );
        return;
      }

      // Connection check (R6.2/R6.3): if not connected, do NOT send. Warn and
      // offer a 「デバッグ開始」action that starts a session.
      const session = vscode.debug.activeDebugSession;
      if (!isPastaSession(session)) {
        const choice = await vscode.window.showWarningMessage(
          'Pasta デバッグセッションが接続されていません。シーンを実行するにはデバッグを開始してください。',
          'デバッグ開始'
        );
        if (choice === 'デバッグ開始') {
          const folder = vscode.workspace.workspaceFolders?.[0];
          await vscode.debug.startDebugging(folder, resolveStartDebuggingConfig());
        }
        return;
      }

      // Dirty check (R7.2/R7.5): warn that the running ghost may differ from the
      // unsaved buffer and offer to save (best-effort; proceed after — design
      // Decision 4). Reload guidance is via the separate SHIORIリロード command.
      if (editor.document.isDirty) {
        const choice = await vscode.window.showWarningMessage(
          '未保存の変更があります。実行中のゴーストは保存・リロード済みの内容で動作するため、表示とずれる場合があります。保存してリロードすることを推奨します。',
          '保存'
        );
        if (choice === '保存') {
          await editor.document.save();
        }
      }

      // Position-based dispatch (R1.2/R1.3/R4.1). VSCode `Position.line` is
      // 0-based; the engine expects a 1-based line -> add 1 here.
      const uri = editor.document.uri.toString();
      const line = editor.selection.active.line + 1;
      try {
        const result = await session.customRequest(
          playSceneAtRequestCommand,
          setPlaySceneAtPayload(uri, line)
        );
        // customRequest may RESOLVE with a `success:false` body (rather than
        // throwing); surface that reason too (R6.4).
        if (result && typeof result === 'object' && (result as { success?: unknown }).success === false) {
          const reason = (result as { message?: unknown }).message;
          await vscode.window.showErrorMessage(
            `シーンの実行に失敗しました: ${typeof reason === 'string' && reason.length > 0 ? reason : '不明なエラー'}`
          );
        }
      } catch (err) {
        // Surface backend/transport failures to the author (R6.4).
        await vscode.window.showErrorMessage(
          `シーンの実行に失敗しました: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    })
  );
}

/**
 * Injectable async delay used by the reload re-attach loop. Defaults to a real
 * `setTimeout`-backed sleep; the unit tests replace it with an immediate no-op
 * (via {@link setReattachDelayForTests}) so the wait/retry loop runs
 * synchronously WITHOUT real timers. The loop's TERMINATION does not depend on
 * this delay — it is driven by the pure, finite `reattachDelaySchedule` — so a
 * no-op delay keeps the test deterministic and bounded.
 */
let reattachDelay: (ms: number) => Promise<void> = (ms) =>
  new Promise((resolve) => setTimeout(resolve, ms));

/** Test seam: override the re-attach delay (e.g. with an immediate no-op). */
export function setReattachDelayForTests(fn: (ms: number) => Promise<void>): void {
  reattachDelay = fn;
}

/**
 * Wire the SHIORI-reload command (`pasta.reloadShiori`). Both menu contributions
 * are gated `when: debugType == 'pasta'` (connected-only, R9.1/9.5); the command
 * body ALSO guards defensively (no-op + warn if not connected).
 *
 * Flow (requirements 7.4, 9.1, 9.3, 9.4, 9.5):
 *  - Not connected            -> send nothing; warn that reload needs an active
 *    Pasta session and return (defensive; the menu `when` already hides it).
 *  - Dirty `.pasta` buffer     -> warn that the reload reads DISK so unsaved
 *    changes won't be reflected; offer 「保存」 (best-effort; proceed after) (R7.4).
 *  - Send `customRequest('pasta/reloadShiori')` -> the engine emits
 *    `\![reload,shiori]`; SSP reloads SHIORI and the debug session detaches.
 *  - Auto re-attach (R9.3/9.4): wait the initial settle delay, then call
 *    `startDebugging(folder, resolved)`; on failure retry every interval until
 *    success or the timeout budget is exhausted (driven by the pure, finite
 *    `reattachDelaySchedule`). A final timeout surfaces an error (R9.3).
 *  - Do NOT auto re-run any scene kick after re-attach (R9.5): the author
 *    re-kicks manually. (No playSceneAt is issued here by design.)
 */
function registerReloadShioriCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('pasta.reloadShiori', async () => {
      // Connection guard (R9.1/9.5): the menu `when` gates visibility, but the
      // body also no-ops + warns if somehow invoked without a Pasta session.
      const session = vscode.debug.activeDebugSession;
      if (!isPastaSession(session)) {
        await vscode.window.showWarningMessage(
          'SHIORIリロードには、実行中の Pasta デバッグセッションが必要です。'
        );
        return;
      }

      // Dirty check (R7.4): the reload reads DISK, so unsaved buffer edits are
      // NOT reflected. Warn + offer to save (best-effort; proceed after).
      const editor = vscode.window.activeTextEditor;
      if (
        editor &&
        editor.document.languageId === PASTA_LANGUAGE_ID &&
        editor.document.isDirty
      ) {
        const choice = await vscode.window.showWarningMessage(
          '未保存の変更があります。リロードはディスク上の内容を読み込むため、未保存の変更は反映されません。保存することを推奨します。',
          '保存'
        );
        if (choice === '保存') {
          await editor.document.save();
        }
      }

      // Send the reload request (R9.2). The engine emits `\![reload,shiori]`;
      // SSP reloads SHIORI and the debug session detaches.
      try {
        await session.customRequest(reloadShioriRequestCommand);
      } catch (err) {
        await vscode.window.showErrorMessage(
          `SHIORIリロード要求に失敗しました: ${err instanceof Error ? err.message : String(err)}`
        );
        return;
      }

      // Auto re-attach (R9.3/9.4): the reload detaches the session. Wait for it
      // to settle, then attempt to re-attach with the SAME resolved config used
      // by the 5.1 「デバッグ開始」 guidance (shared resolution). Retry on
      // failure until the pure, finite schedule is exhausted (timeout budget).
      //
      // NOTE (R9.5): we deliberately do NOT auto re-run any scene kick after a
      // successful re-attach — the author re-kicks manually. No playSceneAt is
      // issued here.
      const folder = vscode.workspace.workspaceFolders?.[0];
      const config = resolveStartDebuggingConfig();
      const schedule = reattachDelaySchedule();
      let reattached = false;
      for (const waitMs of schedule) {
        await reattachDelay(waitMs);
        const ok = await vscode.debug.startDebugging(folder, config);
        if (ok) {
          reattached = true;
          break;
        }
      }
      if (!reattached) {
        await vscode.window.showErrorMessage(
          'SHIORIリロード後の自動再アタッチに失敗しました。手動でデバッグを再開してください。'
        );
      }
    })
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
