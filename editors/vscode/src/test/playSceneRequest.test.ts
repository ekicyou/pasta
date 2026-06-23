// Unit tests for the pure playScene request logic (Task 5.1)
//
// These are NODE tests: they import ONLY the pure helpers from the vscode-free
// `playSceneRequest` module. They MUST NOT touch the real `vscode` runtime
// module — the request logic is deliberately separated from `extension.ts`
// (task 5.2) so it stays node-testable. The module mirrors the
// `sourcePresentationToggle.ts` separation pattern.
//
// Requirements covered (client-side):
//   1.2 — the scene name is sent to the engine over the debug channel
//         (payload shape `{ scene }` for the customRequest).
//   2.5 — an empty/blank scene name is invalid (rejected before sending).

import * as assert from 'assert';
import {
  requestCommand,
  setPayload,
  validateSceneName,
} from '../playSceneRequest';

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

console.log('\n[playSceneRequest Unit Tests]');

// ---------------------------------------------------------------------------
// requestCommand — DAP customRequest command name (Rust decode contract)
// ---------------------------------------------------------------------------

test('requestCommand is the exact "pasta/playScene" contract string', () => {
  assert.strictEqual(requestCommand, 'pasta/playScene');
});

// ---------------------------------------------------------------------------
// setPayload — request argument payload (requirement 1.2)
// ---------------------------------------------------------------------------

test('setPayload("intro") produces { scene: "intro" }', () => {
  assert.deepStrictEqual(setPayload('intro'), { scene: 'intro' });
});

test('setPayload preserves the scene name verbatim (no trimming)', () => {
  assert.deepStrictEqual(setPayload(' x '), { scene: ' x ' });
});

// ---------------------------------------------------------------------------
// validateSceneName — pre-send validation (requirement 2.5)
// Empty / whitespace-only → false; any non-blank → true. NEVER throws.
// ---------------------------------------------------------------------------

test('validateSceneName("") is false (empty)', () => {
  assert.strictEqual(validateSceneName(''), false);
});

test('validateSceneName("   ") is false (whitespace only)', () => {
  assert.strictEqual(validateSceneName('   '), false);
});

test('validateSceneName(" x ") is true (non-blank after trim)', () => {
  assert.strictEqual(validateSceneName(' x '), true);
});

test('validateSceneName("intro") is true', () => {
  assert.strictEqual(validateSceneName('intro'), true);
});

console.log(`\nplaySceneRequest: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
