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
  /**
   * Optional present-mode override (Task 6.2 / requirement 6.3). When the
   * author sets it in the launch config it is forwarded VERBATIM as the DAP
   * `attach` argument key {@link SOURCE_PRESENTATION_KEY}. The client performs
   * NO `.pasta`<->`.lua` conversion — that is entirely server-side.
   */
  sourcePresentation?: unknown;
  [key: string]: unknown;
}

/**
 * The DAP `attach`-argument key carrying the present mode.
 *
 * MUST match the case-sensitive key the Rust backend reads
 * (`crates/pasta_lua/src/debug/dap.rs`: `args.get("sourcePresentation")`).
 */
export const SOURCE_PRESENTATION_KEY = 'sourcePresentation';

/** The two recognised present modes (mirrors the package.json enum). */
export type SourcePresentation = 'pasta' | 'lua';

/**
 * Resolve the EXPLICIT `sourcePresentation` value to forward on the `attach`
 * request arguments (Task 6.2 / requirement 6.3 / design 581/586).
 *
 * Pure and `vscode`-free so it is node-testable. This is a CLIENT-side PURE
 * passthrough — it performs NO `.pasta`<->`.lua` conversion (that is entirely
 * server-side, task 5.5):
 * - Returns the normalised value (`"pasta"`/`"lua"`) ONLY when the author set a
 *   recognised string. The value is lower-cased to mirror the server's
 *   case-insensitive `SourceMode::parse`.
 * - Returns `undefined` when the key is ABSENT or not a recognised value, so
 *   the client never injects a default and the server falls back to its own
 *   env > file > 既定 resolution (only-when-explicit, design 581). An invalid
 *   value is dropped here so the server's default-`pasta`-plus-warning path
 *   (design 615) decides.
 */
export function resolveSourcePresentation(config: AttachConfig): SourcePresentation | undefined {
  const raw = config.sourcePresentation;
  if (typeof raw !== 'string') {
    return undefined;
  }
  const normalized = raw.toLowerCase();
  return normalized === 'pasta' || normalized === 'lua' ? normalized : undefined;
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
 *   (e.g. from a JSON launch config) is coerced to a number. Any INVALID port
 *   (non-number/non-string type, NaN, non-integer, outside 1–65535) also falls
 *   back to {@link DEFAULT_DEBUG_PORT} instead of producing a broken
 *   `DebugAdapterServer` descriptor (cell 3.56 / G3 hardening — valid input
 *   behaviour is unchanged).
 */
export function resolveAttachTarget(config: AttachConfig): AttachTarget {
  const host = String(config.host ?? DEFAULT_DEBUG_HOST);
  return { host, port: resolvePort(config.port) };
}

/** Coerce + validate a raw `port` config value; fall back to the default. */
function resolvePort(raw: unknown): number {
  if (raw === undefined || raw === null) {
    return DEFAULT_DEBUG_PORT;
  }
  const port =
    typeof raw === 'number' ? raw : typeof raw === 'string' ? Number(raw) : Number.NaN;
  return Number.isInteger(port) && port >= 1 && port <= 65535 ? port : DEFAULT_DEBUG_PORT;
}
