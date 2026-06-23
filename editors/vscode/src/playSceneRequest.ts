// Pure playScene request logic for the Pasta debug adapter (Task 5.1).
//
// This module is deliberately FREE of the `vscode` runtime so it can be
// imported and exercised by plain node unit tests (the test bundle does not
// provide a `vscode` module). The `vscode`-aware wiring (command registration,
// input box, customRequest dispatch, error reporting) lives in `extension.ts`
// (task 5.2) and depends on the helpers defined here. This mirrors the
// separation already used by `sourcePresentationToggle.ts`.
//
// Requirements covered (client-side): 1.2, 2.5.

/**
 * The DAP customRequest command name used to send a scene kick to the engine.
 *
 * MUST match the case-sensitive command string the Rust backend decodes
 * (`crates/pasta_lua/src/debug/dap/decode.rs`: `pasta/playScene`). This
 * equality is the cross-language contract (requirement 1.2).
 */
export const requestCommand = 'pasta/playScene' as const;

/**
 * Build the customRequest argument payload `{ scene }` sent to the backend
 * (requirement 1.2). The backend reads `args.scene` and validates it
 * server-side. The scene name is forwarded verbatim — trimming is a UI-side
 * validation concern (see `validateSceneName`), not a payload transform.
 */
export function setPayload(scene: string): { scene: string } {
  return { scene };
}

/**
 * Validate a UI-entered scene name before sending (requirement 2.5).
 *
 * A name that is empty or whitespace-only is invalid (returns `false`); any
 * name with at least one non-whitespace character is valid (returns `true`).
 * This is the client-side pre-send guard that mirrors the backend's own
 * empty/invalid rejection. Pure and total — never throws.
 */
export function validateSceneName(name: string): boolean {
  return name.trim().length > 0;
}
