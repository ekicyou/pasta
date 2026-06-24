// Unit tests for the pure SHIORI-reload + re-attach policy (Task 5.2).
//
// These are NODE tests: they import ONLY the pure helpers from the vscode-free
// `reloadShiori` module. They MUST NOT touch the real `vscode` runtime — the
// policy is deliberately separated from `extension.ts` so it stays node-testable
// (mirrors `runSceneAtCursor.ts`).
//
// Requirements covered (client-side):
//   9.2 — the reload customRequest command string (Rust decode contract).
//   9.3 — re-attach config resolution reused from task 5.1 (workspace `pasta`
//         preferred, else default attach 127.0.0.1:9276).
//   9.3 / 9.4 — bounded wait/retry policy (keeps retrying until timeout, stops
//         after; schedule is finite and terminating).

import * as assert from 'assert';
import {
  requestCommand,
  resolvePastaDebugConfig,
  defaultPastaAttachConfig,
  shouldKeepRetrying,
  reattachDelaySchedule,
  REATTACH_INITIAL_DELAY_MS,
  REATTACH_RETRY_INTERVAL_MS,
  REATTACH_TIMEOUT_MS,
} from '../reloadShiori';
import { DEFAULT_DEBUG_HOST, DEFAULT_DEBUG_PORT } from '../debugAttachTarget';

let passed = 0;
let failed = 0;

function test(name: string, fn: () => void) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e: any) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

console.log('\n[reloadShiori Unit Tests]');

// ---------------------------------------------------------------------------
// requestCommand — DAP customRequest command name (Rust decode contract)
// ---------------------------------------------------------------------------

test('requestCommand is the exact "pasta/reloadShiori" contract string', () => {
  assert.strictEqual(requestCommand, 'pasta/reloadShiori');
});

// ---------------------------------------------------------------------------
// config resolution is REUSED from task 5.1 (not re-implemented) (requirement 9.3)
// ---------------------------------------------------------------------------

test('resolvePastaDebugConfig is re-exported and prefers a workspace pasta config', () => {
  const workspacePasta = {
    type: 'pasta',
    request: 'attach',
    name: 'My Pasta',
    host: '127.0.0.1',
    port: 5555,
  };
  const configs = [{ type: 'node', request: 'launch', name: 'Node' }, workspacePasta];
  assert.deepStrictEqual(resolvePastaDebugConfig(configs), workspacePasta);
});

test('resolvePastaDebugConfig falls back to default attach 127.0.0.1:9276', () => {
  const resolved = resolvePastaDebugConfig(undefined);
  assert.strictEqual(resolved.type, 'pasta');
  assert.strictEqual(resolved.request, 'attach');
  assert.strictEqual(resolved.host, DEFAULT_DEBUG_HOST);
  assert.strictEqual(resolved.port, DEFAULT_DEBUG_PORT);
  assert.deepStrictEqual(resolved, defaultPastaAttachConfig());
});

// ---------------------------------------------------------------------------
// retry policy constants (requirement 9.3 / 9.4)
// ---------------------------------------------------------------------------

test('retry policy constants are sane (initial < timeout, retry > 0)', () => {
  assert.ok(REATTACH_INITIAL_DELAY_MS > 0, 'initial delay positive');
  assert.ok(REATTACH_RETRY_INTERVAL_MS > 0, 'retry interval positive');
  assert.ok(
    REATTACH_TIMEOUT_MS > REATTACH_INITIAL_DELAY_MS,
    'timeout must exceed the initial delay so at least one attempt happens',
  );
});

// ---------------------------------------------------------------------------
// shouldKeepRetrying — bounded retry predicate (requirement 9.4)
// ---------------------------------------------------------------------------

test('shouldKeepRetrying is true before timeout and false at/after timeout', () => {
  assert.strictEqual(shouldKeepRetrying(0, 15000), true);
  assert.strictEqual(shouldKeepRetrying(14999, 15000), true);
  assert.strictEqual(shouldKeepRetrying(15000, 15000), false, 'stops exactly at timeout');
  assert.strictEqual(shouldKeepRetrying(20000, 15000), false, 'stops after timeout');
});

test('shouldKeepRetrying treats non-finite/negative elapsed as 0 (keep retrying)', () => {
  assert.strictEqual(shouldKeepRetrying(Number.NaN, 15000), true);
  assert.strictEqual(shouldKeepRetrying(-5, 15000), true);
});

// ---------------------------------------------------------------------------
// reattachDelaySchedule — finite, terminating schedule (requirement 9.4)
// ---------------------------------------------------------------------------

test('reattachDelaySchedule starts with the initial delay then retry intervals', () => {
  const schedule = reattachDelaySchedule(3000, 1000, 15000);
  assert.strictEqual(schedule[0], 3000, 'first wait is the initial settle delay');
  assert.ok(schedule.slice(1).every((d) => d === 1000), 'subsequent waits are the retry interval');
});

test('reattachDelaySchedule is bounded and terminating (cumulative retry < timeout)', () => {
  const schedule = reattachDelaySchedule(3000, 1000, 15000);
  // initial 3000 + N*1000 retry waits, with cumulative retry wait < 15000.
  // So N = 14 retry waits (14000 < 15000, 15000 not < 15000) -> length 15.
  assert.strictEqual(schedule.length, 15);
  const retryTotal = schedule.slice(1).reduce((a, b) => a + b, 0);
  assert.ok(retryTotal < 15000, 'cumulative retry waits stay under the timeout budget');
});

test('reattachDelaySchedule guards a non-positive retry interval (no infinite loop)', () => {
  assert.deepStrictEqual(reattachDelaySchedule(3000, 0, 15000), [3000]);
  assert.deepStrictEqual(reattachDelaySchedule(3000, -1, 15000), [3000]);
});

test('reattachDelaySchedule with defaults is non-empty and finite', () => {
  const schedule = reattachDelaySchedule();
  assert.ok(schedule.length >= 1);
  assert.ok(schedule.every((d) => Number.isFinite(d)));
});

console.log(`\nreloadShiori: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
