// Unit tests for the pure run-scene-at-cursor request logic (Task 5.1).
//
// These are NODE tests: they import ONLY the pure helpers from the vscode-free
// `runSceneAtCursor` module. They MUST NOT touch the real `vscode` runtime —
// the request logic is deliberately separated from `extension.ts` so it stays
// node-testable (mirrors `sourcePresentationToggle.ts` / `debugAttachTarget.ts`).
//
// Requirements covered (client-side):
//   4.1 / 1.2 — the position (uri, line) is sent over the debug channel
//               (command string + payload shape; line is 1-based).
//   6.3       — start-debugging config resolution prefers a workspace `pasta`
//               config, else the default attach (127.0.0.1:9276).

import * as assert from 'assert';
import {
  requestCommand,
  setPayload,
  resolvePastaDebugConfig,
  defaultPastaAttachConfig,
} from '../runSceneAtCursor';
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

console.log('\n[runSceneAtCursor Unit Tests]');

// ---------------------------------------------------------------------------
// requestCommand — DAP customRequest command name (Rust decode contract)
// ---------------------------------------------------------------------------

test('requestCommand is the exact "pasta/playSceneAt" contract string', () => {
  assert.strictEqual(requestCommand, 'pasta/playSceneAt');
});

// ---------------------------------------------------------------------------
// setPayload — request argument payload (requirements 1.2 / 4.1)
// ---------------------------------------------------------------------------

test('setPayload produces { uri, line }', () => {
  assert.deepStrictEqual(setPayload('file:///x.pasta', 12), {
    uri: 'file:///x.pasta',
    line: 12,
  });
});

test('setPayload forwards the line verbatim (1-based; offset is added at the call site)', () => {
  // The extension passes selection.active.line + 1 (VSCode line is 0-based).
  // So cursor on VSCode line 0 -> engine line 1.
  const vscodeZeroBasedLine = 0;
  const engineLine = vscodeZeroBasedLine + 1;
  assert.deepStrictEqual(setPayload('u', engineLine), { uri: 'u', line: 1 });
});

// ---------------------------------------------------------------------------
// resolvePastaDebugConfig — start-debugging config resolution (requirement 6.3)
// ---------------------------------------------------------------------------

test('resolvePastaDebugConfig prefers a workspace pasta config (verbatim)', () => {
  const workspacePasta = {
    type: 'pasta',
    request: 'attach',
    name: 'My Pasta',
    host: '127.0.0.1',
    port: 5555,
  };
  const configs = [
    { type: 'node', request: 'launch', name: 'Node' },
    workspacePasta,
  ];
  assert.deepStrictEqual(resolvePastaDebugConfig(configs), workspacePasta);
});

test('resolvePastaDebugConfig falls back to default attach 127.0.0.1:9276 when no pasta config', () => {
  const configs = [{ type: 'node', request: 'launch', name: 'Node' }];
  const resolved = resolvePastaDebugConfig(configs);
  assert.strictEqual(resolved.type, 'pasta');
  assert.strictEqual(resolved.request, 'attach');
  assert.strictEqual(resolved.host, DEFAULT_DEBUG_HOST);
  assert.strictEqual(resolved.port, DEFAULT_DEBUG_PORT);
});

test('resolvePastaDebugConfig falls back to default for undefined / non-array input', () => {
  assert.deepStrictEqual(resolvePastaDebugConfig(undefined), defaultPastaAttachConfig());
  assert.deepStrictEqual(resolvePastaDebugConfig(null), defaultPastaAttachConfig());
  assert.deepStrictEqual(resolvePastaDebugConfig('nope'), defaultPastaAttachConfig());
});

test('defaultPastaAttachConfig is 127.0.0.1:9276 attach', () => {
  assert.deepStrictEqual(defaultPastaAttachConfig(), {
    type: 'pasta',
    request: 'attach',
    name: 'Attach to Pasta Debug Backend',
    host: '127.0.0.1',
    port: 9276,
  });
});

console.log(`\nrunSceneAtCursor: ${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  process.exit(1);
}
