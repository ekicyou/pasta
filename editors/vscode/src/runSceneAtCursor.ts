// Pure run-scene-at-cursor request logic for the Pasta debug adapter (Task 5.1).
//
// This module is deliberately FREE of the `vscode` runtime so it can be
// imported and exercised by plain node unit tests (the test bundle does not
// provide a `vscode` module). The `vscode`-aware wiring (command registration,
// active-editor/cursor read, connection guard, dirty check, customRequest
// dispatch, start-debugging guidance, error reporting) lives in `extension.ts`
// and depends on the helpers defined here. This mirrors the separation already
// used by `sourcePresentationToggle.ts` / `debugAttachTarget.ts`.
//
// Requirements covered (client-side): 1.2, 1.3, 4.1, 6.3.

import { DEFAULT_DEBUG_HOST, DEFAULT_DEBUG_PORT } from './debugAttachTarget';

/**
 * The DAP customRequest command name used to send a position-based scene kick
 * to the engine.
 *
 * MUST match the case-sensitive command string the Rust backend decodes
 * (`crates/pasta_lua/src/debug/dap/decode.rs`: `pasta/playSceneAt`). This
 * equality is the cross-language contract (requirements 4.1 / 4.4).
 */
export const requestCommand = 'pasta/playSceneAt' as const;

/** The position payload `{ uri, line }` sent to the engine. */
export interface PlaySceneAtPayload {
  uri: string;
  line: number;
}

/**
 * Build the customRequest argument payload `{ uri, line }` sent to the backend
 * (requirements 1.2 / 1.3 / 4.1).
 *
 * The `line` is the engine-facing **1-based** line number (the Rust scene
 * index is 1-based). VSCode `Position.line` is 0-based, so the call site MUST
 * convert (`selection.active.line + 1`) before calling this helper — this
 * helper does NOT add the offset; it forwards the line verbatim. The 1-based
 * convention is pinned by the unit tests.
 */
export function setPayload(uri: string, line: number): PlaySceneAtPayload {
  return { uri, line };
}

/** Minimal shape of a `launch.json` debug configuration entry we inspect. */
export interface LaunchConfig {
  type?: unknown;
  request?: unknown;
  name?: unknown;
  [key: string]: unknown;
}

/**
 * A resolved Pasta debug configuration suitable for `vscode.debug.startDebugging`.
 * Either a verbatim workspace `pasta` config, or the built-in default attach.
 */
export interface ResolvedPastaDebugConfig {
  type: 'pasta';
  request: string;
  name: string;
  host?: string;
  port?: number;
  [key: string]: unknown;
}

/** The built-in default attach configuration (`127.0.0.1:9276`). */
export function defaultPastaAttachConfig(): ResolvedPastaDebugConfig {
  return {
    type: 'pasta',
    request: 'attach',
    name: 'Attach to Pasta Debug Backend',
    host: DEFAULT_DEBUG_HOST,
    port: DEFAULT_DEBUG_PORT,
  };
}

/**
 * Resolve the debug configuration used to start/re-attach a Pasta session
 * (requirement 6.3; shared with task 5.2's re-attach).
 *
 * Pure and `vscode`-free so it is node-testable. The call site reads the
 * workspace `launch.json` configurations (`workspace.getConfiguration('launch')
 * .get('configurations')`) and passes them here:
 * - Prefer the FIRST workspace config whose `type` is `'pasta'` (forwarded
 *   verbatim), so an author's custom host/port/name is honoured.
 * - Otherwise fall back to the built-in default attach to
 *   `127.0.0.1:DEFAULT_DEBUG_PORT`.
 *
 * Total and never throws; a non-array / undefined input falls back to default.
 */
export function resolvePastaDebugConfig(
  launchConfigs: unknown,
): ResolvedPastaDebugConfig {
  if (Array.isArray(launchConfigs)) {
    const pasta = launchConfigs.find(
      (c): c is LaunchConfig =>
        typeof c === 'object' &&
        c !== null &&
        (c as LaunchConfig).type === 'pasta',
    );
    if (pasta) {
      return pasta as unknown as ResolvedPastaDebugConfig;
    }
  }
  return defaultPastaAttachConfig();
}
