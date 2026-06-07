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
import { AttachConfig, resolveAttachTarget } from './debugAttachTarget';

export {
  DEFAULT_DEBUG_PORT,
  DEFAULT_DEBUG_HOST,
  resolveAttachTarget,
} from './debugAttachTarget';
export type { AttachConfig, AttachTarget } from './debugAttachTarget';

/**
 * Returns a {@link vscode.DebugAdapterServer} descriptor pointing at the
 * Rust DAP backend (attach-only — there is no bundled JS adapter).
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
