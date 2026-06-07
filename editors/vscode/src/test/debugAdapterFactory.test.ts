// Unit tests for the pure attach-target resolver (Task 6.1)
//
// These are NODE tests: they import ONLY the pure `resolveAttachTarget`
// function and the `DEFAULT_DEBUG_PORT` constant from the vscode-free
// `debugAttachTarget` module. They MUST NOT touch the real `vscode` runtime
// module — the resolver is deliberately separated from the factory class
// (which lives in `debugAdapterFactory.ts`) so it stays node-testable.

import * as assert from 'assert';
import { resolveAttachTarget, DEFAULT_DEBUG_PORT } from '../debugAttachTarget';

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

console.log(`\nDebugAdapterFactory: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
