// Pure attach-target resolution for the Pasta debug adapter (Task 6.1).
//
// This module is deliberately FREE of the `vscode` runtime so it can be
// imported and exercised by plain node unit tests (the test bundle does not
// provide a `vscode` module). The `vscode`-aware factory class lives in
// `debugAdapterFactory.ts` and depends on the helpers defined here.

/**
 * Default TCP port the Pasta debug backend attaches on.
 *
 * MUST match the Rust default (`pasta.toml [debug] port`, default `9276`).
 */
export const DEFAULT_DEBUG_PORT = 9276;

/** Default loopback host the backend binds to. */
export const DEFAULT_DEBUG_HOST = '127.0.0.1';

/** Minimal shape of a debug configuration we read host/port from. */
export interface AttachConfig {
  host?: unknown;
  port?: unknown;
  [key: string]: unknown;
}

/** Resolved attach target (host + numeric port). */
export interface AttachTarget {
  host: string;
  port: number;
}

/**
 * Resolve the attach `{ host, port }` from a debug configuration.
 *
 * Pure and `vscode`-free so it is node-testable:
 * - missing `host` falls back to `127.0.0.1`; any value is coerced to a string.
 * - missing `port` falls back to {@link DEFAULT_DEBUG_PORT}; a string port
 *   (e.g. from a JSON launch config) is coerced to a number.
 */
export function resolveAttachTarget(config: AttachConfig): AttachTarget {
  const host = String(config.host ?? DEFAULT_DEBUG_HOST);
  const port = Number(config.port ?? DEFAULT_DEBUG_PORT);
  return { host, port };
}
