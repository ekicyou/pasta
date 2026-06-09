// Pure source-presentation toggle logic for the Pasta debug adapter (Task 4.1).
//
// This module is deliberately FREE of the `vscode` runtime so it can be
// imported and exercised by plain node unit tests (the test bundle does not
// provide a `vscode` module). The `vscode`-aware wiring (command registration,
// status bar, customRequest dispatch, event subscription) lives in
// `extension.ts` (task 4.3) and depends on the helpers defined here. This
// mirrors the separation already used by `debugAttachTarget.ts`.
//
// Requirements covered (client-side): 1.4, 2.3, 2.5, 2.6.

/** The two recognised presentation modes (mirrors the package.json enum). */
export type SourcePresentation = 'pasta' | 'lua';

/**
 * The DAP customRequest command name — also the name of the backend's
 * custom EVENT that pushes the current mode.
 *
 * MUST match the case-sensitive command/event string the Rust backend uses
 * (`crates/pasta_lua/src/debug/`: `pasta/sourcePresentation`).
 */
export const requestCommand = 'pasta/sourcePresentation' as const;

/**
 * Return the OTHER presentation mode (`pasta`⇔`lua`).
 *
 * Pure involution: `nextMode(nextMode(m)) === m`. Used by the toggle command
 * to flip the tracked mode before issuing the customRequest (requirement 2.3).
 */
export function nextMode(current: SourcePresentation): SourcePresentation {
  return current === 'pasta' ? 'lua' : 'pasta';
}

/**
 * Build the customRequest argument payload `{ mode }` sent to the backend
 * (requirement 2.3). The backend reads `args.mode` and applies it server-side.
 */
export function setPayload(mode: SourcePresentation): { mode: SourcePresentation } {
  return { mode };
}

/**
 * Extract the current presentation mode from a customRequest RESPONSE body or
 * a custom-EVENT body, both shaped `{ mode: "pasta" | "lua" }`.
 *
 * Returns `undefined` for any malformed/invalid body — a missing key, a
 * wrong-typed value, an unrecognised token (e.g. `"xml"`), or a non-object
 * (`null`/`undefined`/array/primitive). NEVER throws. This mirrors the
 * backend's strict handling on the client side (requirement 1.4): the
 * extension must not accept garbage and must not update its tracked mode /
 * status bar from an unparseable body. Tokens are matched case-sensitively
 * because the backend always emits the canonical lowercase form.
 */
export function parseMode(body: unknown): SourcePresentation | undefined {
  if (typeof body !== 'object' || body === null || Array.isArray(body)) {
    return undefined;
  }
  const mode = (body as { mode?: unknown }).mode;
  return mode === 'pasta' || mode === 'lua' ? mode : undefined;
}

/**
 * Status-bar text for the always-visible current-mode indicator
 * (requirement 2.5 / 2.6). Uses the `$(eye)` codicon prefix per design and
 * names the concrete source extension being presented.
 */
export function statusLabel(mode: SourcePresentation): string {
  return mode === 'pasta' ? '$(eye) 提示: .pasta' : '$(eye) 提示: .lua';
}
