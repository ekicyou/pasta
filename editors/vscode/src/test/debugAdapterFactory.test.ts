// Unit tests for the pure attach-target resolver (Task 6.1)
//
// These are NODE tests: they import ONLY the pure `resolveAttachTarget`
// function and the `DEFAULT_DEBUG_PORT` constant from the vscode-free
// `debugAttachTarget` module. They MUST NOT touch the real `vscode` runtime
// module — the resolver is deliberately separated from the factory class
// (which lives in `debugAdapterFactory.ts`) so it stays node-testable.

import * as assert from 'assert';
import { resolveAttachTarget, DEFAULT_DEBUG_PORT, resolveSourcePresentation } from '../debugAttachTarget';

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

console.log('\n[DebugAdapterFactory Unit Tests]');

test('DEFAULT_DEBUG_PORT matches the Rust backend default (9276)', () => {
  assert.strictEqual(DEFAULT_DEBUG_PORT, 9276);
});

test('resolveAttachTarget({}) falls back to 127.0.0.1:9276', () => {
  const target = resolveAttachTarget({});
  assert.deepStrictEqual(target, { host: '127.0.0.1', port: 9276 });
});

test('resolveAttachTarget(undefined config field) falls back to defaults', () => {
  // Simulate a DebugSession.configuration with no host/port keys.
  const target = resolveAttachTarget({ type: 'pasta', request: 'attach', name: 'Attach' });
  assert.strictEqual(target.host, '127.0.0.1');
  assert.strictEqual(target.port, 9276);
});

test('resolveAttachTarget honours explicit host and numeric port', () => {
  const target = resolveAttachTarget({ host: '1.2.3.4', port: 5678 });
  assert.deepStrictEqual(target, { host: '1.2.3.4', port: 5678 });
});

test('resolveAttachTarget coerces a string port to a number', () => {
  const target = resolveAttachTarget({ host: '1.2.3.4', port: '5678' });
  assert.strictEqual(target.port, 5678);
  assert.strictEqual(typeof target.port, 'number');
});

test('resolveAttachTarget coerces a non-string host to a string', () => {
  // host could conceivably arrive as a non-string from a malformed config.
  const target = resolveAttachTarget({ host: 127 as unknown as string, port: 9276 });
  assert.strictEqual(target.host, '127');
  assert.strictEqual(typeof target.host, 'string');
});

// ---------------------------------------------------------------------------
// sourcePresentation passthrough (Task 6.2 / requirement 6.3 / design 581/586)
//
// CLIENT-side PURE passthrough: the resolver only forwards an EXPLICIT,
// valid `sourcePresentation` value; it never injects a client default and
// performs NO `.pasta`<->`.lua` conversion (that is 100% server-side).
// The forwarded key must match the server's case-sensitive `sourcePresentation`
// attach-arg key read in `crates/pasta_lua/src/debug/dap.rs`.
// ---------------------------------------------------------------------------

test('resolveSourcePresentation forwards an explicit "lua" value', () => {
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 'lua' }), 'lua');
});

test('resolveSourcePresentation forwards an explicit "pasta" value', () => {
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 'pasta' }), 'pasta');
});

test('resolveSourcePresentation returns undefined when the key is ABSENT (no client default)', () => {
  // Only-when-explicit (design 581): an unset value must NOT be forced to a
  // client default so the server falls back to env > file > 既定.
  assert.strictEqual(resolveSourcePresentation({ host: '1.2.3.4', port: 9276 }), undefined);
  assert.strictEqual(resolveSourcePresentation({}), undefined);
});

test('resolveSourcePresentation is case-insensitive on the value (mirrors server SourceMode::parse)', () => {
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 'LUA' }), 'lua');
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 'Pasta' }), 'pasta');
});

test('resolveSourcePresentation ignores an invalid value (server falls back to default)', () => {
  // No client conversion: an unrecognised value is dropped so the server's
  // own fallback (default `pasta` + warning, design 615) decides.
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 'banana' }), undefined);
  assert.strictEqual(resolveSourcePresentation({ sourcePresentation: 42 as unknown as string }), undefined);
});

// ---------------------------------------------------------------------------
// Port hardening (cell 3.56 / G3): an invalid `port` value must fall back to
// DEFAULT_DEBUG_PORT instead of producing a broken DebugAdapterServer
// descriptor (NaN / out-of-range / non-integer port). Valid numeric and
// numeric-string ports are unaffected (behaviour-preserving for valid input).
// ---------------------------------------------------------------------------

test('resolveAttachTarget falls back to the default port for a non-numeric string', () => {
  const target = resolveAttachTarget({ port: 'abc' });
  assert.strictEqual(target.port, DEFAULT_DEBUG_PORT);
});

test('resolveAttachTarget falls back to the default port for out-of-range values', () => {
  assert.strictEqual(resolveAttachTarget({ port: 0 }).port, DEFAULT_DEBUG_PORT);
  assert.strictEqual(resolveAttachTarget({ port: -1 }).port, DEFAULT_DEBUG_PORT);
  assert.strictEqual(resolveAttachTarget({ port: 65536 }).port, DEFAULT_DEBUG_PORT);
});

test('resolveAttachTarget falls back to the default port for a non-integer port', () => {
  assert.strictEqual(resolveAttachTarget({ port: 12.5 }).port, DEFAULT_DEBUG_PORT);
});

test('resolveAttachTarget falls back to the default port for non-number/non-string types', () => {
  // Number(true) === 1 would otherwise silently "work"; reject wrong types.
  assert.strictEqual(resolveAttachTarget({ port: true }).port, DEFAULT_DEBUG_PORT);
  assert.strictEqual(resolveAttachTarget({ port: {} }).port, DEFAULT_DEBUG_PORT);
  assert.strictEqual(resolveAttachTarget({ port: [] }).port, DEFAULT_DEBUG_PORT);
});

test('resolveAttachTarget keeps accepting the valid port boundary values', () => {
  assert.strictEqual(resolveAttachTarget({ port: 1 }).port, 1);
  assert.strictEqual(resolveAttachTarget({ port: 65535 }).port, 65535);
});

console.log(`\nDebugAdapterFactory: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
