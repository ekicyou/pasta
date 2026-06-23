// Pure SHIORI-reload + re-attach policy for the Pasta debug adapter (Task 5.2).
//
// This module is deliberately FREE of the `vscode` runtime so it can be
// imported and exercised by plain node unit tests (the test bundle does not
// provide a `vscode` module). The `vscode`-aware wiring (command registration,
// connection guard, dirty check, customRequest dispatch, detach detection,
// wait/retry re-attach loop, error reporting) lives in `extension.ts` and
// depends on the helpers defined here. This mirrors the separation already used
// by `runSceneAtCursor.ts` (task 5.1).
//
// Requirements covered (client-side): 7.4, 9.1, 9.3, 9.4, 9.5.

// Re-export the start-debugging config resolution from task 5.1 so the
// re-attach path uses the SAME workspace-`pasta`-preferred / default-`9276`
// resolution as the "デバッグ開始" guidance (Decision 5 / OQ-4: DRY — do NOT
// duplicate the resolver). The wiring imports `resolvePastaDebugConfig` from
// here (or directly from runSceneAtCursor); either way there is one source.
export {
  resolvePastaDebugConfig,
  defaultPastaAttachConfig,
  type ResolvedPastaDebugConfig,
} from './runSceneAtCursor';

/**
 * The DAP customRequest command name used to request a SHIORI reload.
 *
 * MUST match the case-sensitive command string the Rust backend decodes
 * (`crates/pasta_lua/src/debug/dap/decode.rs`: `pasta/reloadShiori`). This
 * equality is the cross-language contract (requirement 9.2). The engine reacts
 * by emitting `\![reload,shiori]` as sakura script output (NOT sent directly to
 * SSP); the reload detaches the debug session, triggering the re-attach below.
 */
export const requestCommand = 'pasta/reloadShiori' as const;

// ---------------------------------------------------------------------------
// Re-attach wait/retry policy (Decision 5 ② / requirement 9.3 / 9.4)
// ---------------------------------------------------------------------------
//
// After the reload request the engine emits `\![reload,shiori]`, SSP reloads
// SHIORI, and the debug session detaches. The reload is asynchronous, so the
// backend is not immediately ready to accept a new attach. The policy is:
//   1. wait REATTACH_INITIAL_DELAY_MS for the reload to settle,
//   2. attempt `startDebugging`,
//   3. on failure, retry every REATTACH_RETRY_INTERVAL_MS,
//   4. give up once total elapsed time reaches REATTACH_TIMEOUT_MS and surface
//      a final failure to the author.
// The concrete values are tuned conservatively per the design note ("リトライ
// 上限/タイムアウトの具体値は実装時に調整"); they are exported so the wiring and
// the unit tests share one source of truth.

/** Initial wait after the reload request before the first attach attempt. */
export const REATTACH_INITIAL_DELAY_MS = 3000;

/** Interval between attach retries once the first attempt has failed. */
export const REATTACH_RETRY_INTERVAL_MS = 1000;

/** Total budget (from the first attempt) after which re-attach gives up. */
export const REATTACH_TIMEOUT_MS = 15000;

/**
 * Pure retry-schedule decision: given the elapsed time since the first attach
 * attempt (in ms), should the loop keep retrying?
 *
 * `true` while `elapsedMs < timeoutMs`; `false` once the budget is exhausted.
 * Total and never throws; a non-finite / negative elapsed value is treated as
 * "0 elapsed" (keep retrying) so a clock glitch never aborts early. Pinned by
 * the unit tests so the wiring's loop can be reasoned about without real timers.
 */
export function shouldKeepRetrying(
  elapsedMs: number,
  timeoutMs: number = REATTACH_TIMEOUT_MS,
): boolean {
  const elapsed = Number.isFinite(elapsedMs) && elapsedMs > 0 ? elapsedMs : 0;
  return elapsed < timeoutMs;
}

/**
 * Pure re-attach schedule: the ordered list of delays (ms) to wait BEFORE each
 * successive attach attempt, assuming every attempt fails (the worst case).
 *
 * The first entry is the initial settle delay; subsequent entries are the retry
 * interval, emitted until the cumulative wait would reach/exceed the timeout.
 * The wiring drives attempts lazily (stopping on the first success) — this pure
 * form exists so the bounded, terminating nature of the loop is unit-testable
 * without real timers. Always finite and non-empty (at least the initial wait).
 */
export function reattachDelaySchedule(
  initialDelayMs: number = REATTACH_INITIAL_DELAY_MS,
  retryIntervalMs: number = REATTACH_RETRY_INTERVAL_MS,
  timeoutMs: number = REATTACH_TIMEOUT_MS,
): number[] {
  const schedule = [initialDelayMs];
  // `elapsed` tracks the wall time consumed by waits performed so far. We keep
  // appending retry waits while the budget (measured from the first attempt,
  // i.e. after the initial delay) has not been exhausted. A non-positive retry
  // interval would loop forever, so guard it.
  if (retryIntervalMs <= 0) {
    return schedule;
  }
  let elapsed = 0;
  while (shouldKeepRetrying(elapsed + retryIntervalMs, timeoutMs)) {
    schedule.push(retryIntervalMs);
    elapsed += retryIntervalMs;
  }
  return schedule;
}
