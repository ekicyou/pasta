// Unit tests for the pure source-presentation toggle logic (Task 4.1)
//
// These are NODE tests: they import ONLY the pure helpers from the vscode-free
// `sourcePresentationToggle` module. They MUST NOT touch the real `vscode`
// runtime module — the toggle logic is deliberately separated from
// `extension.ts` (task 4.3) so it stays node-testable. The module mirrors the
// `debugAttachTarget.ts` separation pattern.
//
// Requirements covered (client-side):
//   1.4 — malformed/invalid bodies are rejected (no garbage accepted).
//   2.3 — setPayload shape sent to the backend customRequest.
//   2.5 — statusLabel for the always-visible status bar.
//   2.6 — parseMode extracts the mode from response/event bodies for updates.

import * as assert from 'assert';
import {
  nextMode,
  requestCommand,
  setPayload,
  parseMode,
  statusLabel,
  type SourcePresentation,
} from '../sourcePresentationToggle';

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

console.log('\n[sourcePresentationToggle Unit Tests]');

// ---------------------------------------------------------------------------
// nextMode — flips both directions (requirement 2.3 toggle semantics)
// ---------------------------------------------------------------------------

test('nextMode("pasta") returns "lua"', () => {
  assert.strictEqual(nextMode('pasta'), 'lua');
});

test('nextMode("lua") returns "pasta"', () => {
  assert.strictEqual(nextMode('lua'), 'pasta');
});

test('nextMode is an involution (applying twice returns the original)', () => {
  const modes: SourcePresentation[] = ['pasta', 'lua'];
  for (const m of modes) {
    assert.strictEqual(nextMode(nextMode(m)), m);
  }
});

// ---------------------------------------------------------------------------
// requestCommand — DAP customRequest command name (and custom-event name)
// ---------------------------------------------------------------------------

test('requestCommand is the exact "pasta/sourcePresentation" contract string', () => {
  assert.strictEqual(requestCommand, 'pasta/sourcePresentation');
});

// ---------------------------------------------------------------------------
// setPayload — request argument payload (requirement 2.3)
// ---------------------------------------------------------------------------

test('setPayload("lua") produces { mode: "lua" }', () => {
  assert.deepStrictEqual(setPayload('lua'), { mode: 'lua' });
});

test('setPayload("pasta") produces { mode: "pasta" }', () => {
  assert.deepStrictEqual(setPayload('pasta'), { mode: 'pasta' });
});

// ---------------------------------------------------------------------------
// parseMode — extract current mode from a response OR custom-event body.
// Valid → the mode; malformed/invalid → undefined (requirement 1.4 client-side,
// 2.6 update source). NEVER throws.
// ---------------------------------------------------------------------------

test('parseMode reads a valid response/event body { mode: "lua" }', () => {
  assert.strictEqual(parseMode({ mode: 'lua' }), 'lua');
});

test('parseMode reads a valid response/event body { mode: "pasta" }', () => {
  assert.strictEqual(parseMode({ mode: 'pasta' }), 'pasta');
});

test('parseMode ignores extra keys and still reads mode', () => {
  assert.strictEqual(parseMode({ mode: 'lua', allThreadsStopped: true }), 'lua');
});

test('parseMode returns undefined when the mode key is missing', () => {
  assert.strictEqual(parseMode({}), undefined);
  assert.strictEqual(parseMode({ notMode: 'lua' }), undefined);
});

test('parseMode returns undefined for an invalid token like "xml"', () => {
  assert.strictEqual(parseMode({ mode: 'xml' }), undefined);
});

test('parseMode returns undefined when mode is the wrong type', () => {
  assert.strictEqual(parseMode({ mode: 42 }), undefined);
  assert.strictEqual(parseMode({ mode: true }), undefined);
  assert.strictEqual(parseMode({ mode: null }), undefined);
  assert.strictEqual(parseMode({ mode: { nested: 'lua' } }), undefined);
});

test('parseMode returns undefined for null/undefined/non-object bodies', () => {
  assert.strictEqual(parseMode(null), undefined);
  assert.strictEqual(parseMode(undefined), undefined);
  assert.strictEqual(parseMode('lua'), undefined);
  assert.strictEqual(parseMode(42), undefined);
  assert.strictEqual(parseMode(['lua']), undefined);
});

test('parseMode is strict on case (server emits canonical lowercase tokens)', () => {
  // The backend always emits canonical "pasta"/"lua"; anything else is garbage
  // and must not be accepted (requirement 1.4 client-side strictness).
  assert.strictEqual(parseMode({ mode: 'LUA' }), undefined);
  assert.strictEqual(parseMode({ mode: 'Pasta' }), undefined);
});

test('parseMode never throws on hostile input', () => {
  const hostile: unknown[] = [null, undefined, 0, '', NaN, [], {}, Symbol('x'), () => 'lua'];
  for (const body of hostile) {
    assert.strictEqual(parseMode(body), undefined);
  }
});

// ---------------------------------------------------------------------------
// statusLabel — always-visible status bar text (requirement 2.5)
// ---------------------------------------------------------------------------

test('statusLabel("pasta") shows the codicon-prefixed .pasta label', () => {
  assert.strictEqual(statusLabel('pasta'), '$(eye) 提示: .pasta');
});

test('statusLabel("lua") shows the codicon-prefixed .lua label', () => {
  assert.strictEqual(statusLabel('lua'), '$(eye) 提示: .lua');
});

console.log(`\nsourcePresentationToggle: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
