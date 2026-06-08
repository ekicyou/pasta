// Pasta debug adapter factory (Task 6.1)
//
// The DAP backend itself lives on the Rust side (pasta_lua's `debug` module),
// which listens for an ATTACH connection on a TCP port when debugging is
// enabled. This module is the thin VSCode-side glue that returns a
// `DebugAdapterServer(port, host)` descriptor pointing VSCode at that backend.
//
// The host/port resolution is intentionally kept in a separate, `vscode`-free
// module (`debugAttachTarget.ts`) so it can be exercised by plain node unit
// tests. This module re-exports those helpers for convenience; only the
// factory class below references the `vscode` runtime.

import * as vscode from 'vscode';
import {
  AttachConfig,
  resolveAttachTarget,
  resolveSourcePresentation,
  SOURCE_PRESENTATION_KEY,
} from './debugAttachTarget';

export {
  DEFAULT_DEBUG_PORT,
  DEFAULT_DEBUG_HOST,
  resolveAttachTarget,
  resolveSourcePresentation,
  SOURCE_PRESENTATION_KEY,
} from './debugAttachTarget';
export type { AttachConfig, AttachTarget, SourcePresentation } from './debugAttachTarget';

/**
 * Returns a {@link vscode.DebugAdapterServer} descriptor pointing at the
 * Rust DAP backend (attach-only — there is no bundled JS adapter).
 *
 * The descriptor only carries the transport `{ host, port }`; the resolved
 * `session.configuration` (including {@link SOURCE_PRESENTATION_KEY}) is what
 * VSCode serialises as the DAP `attach` request `arguments`, so the
 * `sourcePresentation` passthrough is structural — see
 * {@link PastaDebugConfigurationProvider} for the only-when-explicit
 * normalisation.
 */
export class PastaDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
  createDebugAdapterDescriptor(
    session: vscode.DebugSession,
    _executable: vscode.DebugAdapterExecutable | undefined,
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const { host, port } = resolveAttachTarget(session.configuration as AttachConfig);
    return new vscode.DebugAdapterServer(port, host);
  }
}

/**
 * Normalises the `attach` debug configuration before VSCode forwards it to the
 * Rust DAP backend as the `attach` request `arguments` (Task 6.2 /
 * requirement 6.3 / design 581/586).
 *
 * This is a CLIENT-side PURE passthrough — it performs NO `.pasta`<->`.lua`
 * conversion (that is entirely server-side, task 5.5):
 * - When the author set an explicit, recognised {@link SOURCE_PRESENTATION_KEY}
 *   value, it is written back (lower-cased to match the server's
 *   case-insensitive parse) so it reaches the server under the exact
 *   case-sensitive key the backend reads.
 * - When the key is ABSENT or invalid, it is REMOVED so the client never
 *   injects a default — the server then falls back to its own
 *   env > file > 既定 resolution (only-when-explicit, design 581).
 */
export class PastaDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
  resolveDebugConfiguration(
    _folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    _token?: vscode.CancellationToken,
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    const explicit = resolveSourcePresentation(config as AttachConfig);
    if (explicit === undefined) {
      // Drop any absent/invalid value so no client default overrides the
      // server-side env > file > 既定 resolution (design 581).
      delete (config as Record<string, unknown>)[SOURCE_PRESENTATION_KEY];
    } else {
      // Forward the author's explicit value verbatim under the server key.
      (config as Record<string, unknown>)[SOURCE_PRESENTATION_KEY] = explicit;
    }
    return config;
  }
}
