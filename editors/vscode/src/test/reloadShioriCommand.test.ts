// Real-module unit tests for the SHIORI-reload VSCode wiring (Task 5.2).
//
// Imports the REAL extension.ts against the vscode mock in ./mock/vscode.ts
// (wired via esbuild --alias:vscode=./src/test/mock/vscode.ts), mirroring
// runSceneAtCursorCommand.test.ts. Exercises ONLY the `pasta.reloadShiori`
// command registered by activate():
//   - connected               -> customRequest('pasta/reloadShiori') sent.
//   - not connected           -> no customRequest; warning shown.
//   - dirty document          -> save warning/prompt shown before sending.
//   - after send + detach     -> startDebugging called with the resolved config
//                                after the initial delay; on first failure ->
//                                retried; on eventual success -> stops; NO scene
//                                kick re-issued.
//   - timeout                 -> final failure surfaced.
//
// Timers are DRIVEN via an injectable delay (setReattachDelayForTests) so the
// test is fast/deterministic — NOT real setTimeout.
//
// The pure request/policy logic lives in reloadShiori.ts (Task 5.2) and has its
// own tests; here we only verify the vscode glue.

import * as assert from 'assert';
import * as path from 'path';
import * as mock from './mock/vscode';
import { activate, setReattachDelayForTests } from '../extension';
import { requestCommand } from '../reloadShiori';

const ROOT = path.resolve(__dirname, '..', '..');

// =============================================================================
// Async-aware sequential test runner (mirrors runSceneAtCursorCommand.test.ts)
// =============================================================================

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void | Promise<void> }> = [];

function test(name: string, fn: () => void | Promise<void>): void {
  queue.push({ name, fn });
}

const RELOAD_COMMAND = 'pasta.reloadShiori';

function makePastaSession(opts?: { failNext?: boolean }) {
  const session = {
    type: 'pasta',
    requests: [] as Array<{ command: string; args: unknown }>,
    failNext: opts?.failNext ?? false,
    customRequest: async (command: string, args: unknown) => {
      if (session.failNext) {
        throw new Error('リロード要求に失敗しました');
      }
      session.requests.push({ command, args });
      return { success: true };
    },
  };
  return session;
}

/** Build a mock active editor on a .pasta document. */
function makePastaEditor(opts?: { dirty?: boolean; uri?: string }) {
  const dirty = opts?.dirty ?? false;
  const uriStr = opts?.uri ?? 'file:///c/work/intro.pasta';
  const document = {
    languageId: 'pasta',
    isDirty: dirty,
    saveCalls: 0,
    uri: { toString: () => uriStr },
    save: async function () {
      this.saveCalls++;
      this.isDirty = false;
      return true;
    },
  };
  return { document, selection: { active: new mock.Position(0, 0) } };
}

// Replace the re-attach delay with an immediate no-op so the wait/retry loop
// runs synchronously (no real timers). Installed once before the suite.
setReattachDelayForTests(() => Promise.resolve());

// =============================================================================
// activate() registers the reloadShiori command
// =============================================================================

test('activate registers the pasta.reloadShiori command', async () => {
  mock.workspace.textDocuments = [];
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(ROOT) };
  await activate(context as never);
  assert.ok(mock.registeredCommands.has(RELOAD_COMMAND), 'reloadShiori command must be registered');
});

test('connected -> sends customRequest(pasta/reloadShiori) (R9.2)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor();
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setNextStartDebuggingResults([true]);
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.deepStrictEqual(
    session.requests.map((r) => r.command),
    [requestCommand],
    'exactly one reloadShiori customRequest must be sent',
  );
  assert.strictEqual(requestCommand, 'pasta/reloadShiori', 'command name is the cross-language contract');
});

test('NOT connected -> no customRequest; warning shown (R9.1/9.5 guard)', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.window.activeTextEditor = makePastaEditor();
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls.length, 0, 'no re-attach when not connected');
  assert.ok(
    mock.shownMessages.some((m) => m.kind === 'warning'),
    'a warning must be shown when not connected',
  );
});

test('dirty document -> unsaved-changes warning shown / save prompted (R7.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  const editor = makePastaEditor({ dirty: true });
  mock.window.activeTextEditor = editor;
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setNextStartDebuggingResults([true]);
  mock.setNextWarningSelection('保存'); // author chooses to save
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.ok(
    mock.shownMessages.some((m) => m.kind === 'warning' && m.message.includes('未保存')),
    'a dirty/unsaved warning must be shown',
  );
  assert.strictEqual(editor.document.saveCalls, 1, 'choosing 保存 must save the document');
});

test('after send -> re-attaches via startDebugging with the resolved config (R9.3)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor();
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setLaunchConfigurations(undefined); // -> default attach 127.0.0.1:9276
  mock.setNextStartDebuggingResults([true]);
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls.length, 1, 'startDebugging must be called once on success');
  const cfg = mock.startDebuggingCalls[0].config;
  assert.strictEqual(cfg.type, 'pasta');
  assert.strictEqual(cfg.request, 'attach');
  assert.strictEqual(cfg.host, '127.0.0.1');
  assert.strictEqual(cfg.port, 9276);
});

test('re-attach prefers a workspace pasta config (shared with 5.1 resolution) (R9.3)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor();
  mock.startDebuggingCalls.length = 0;
  mock.setLaunchConfigurations([
    { type: 'node', request: 'launch', name: 'Node' },
    { type: 'pasta', request: 'attach', name: 'My Pasta', host: '127.0.0.1', port: 5555 },
  ]);
  mock.setNextStartDebuggingResults([true]);
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls[0].config.port, 5555, 'workspace pasta config port wins');
});

test('first attach fails -> retried until eventual success, then stops (R9.3/9.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor();
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setLaunchConfigurations(undefined);
  // fail, fail, then succeed -> exactly 3 attempts, no more.
  mock.setNextStartDebuggingResults([false, false, true]);
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls.length, 3, 'retries until success then stops');
  // No scene kick re-issued after re-attach (R9.5 — manual re-kick only): the
  // only customRequest is the reload itself, never a playSceneAt.
  assert.deepStrictEqual(
    session.requests.map((r) => r.command),
    [requestCommand],
    'NO scene kick must be auto re-issued after re-attach (manual re-kick only)',
  );
  assert.ok(
    !mock.shownMessages.some((m) => m.kind === 'error'),
    'eventual success must not surface a failure',
  );
});

test('all attaches fail until timeout -> final failure surfaced; loop bounded (R9.3/9.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor();
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setLaunchConfigurations(undefined);
  // Always fail. The injected delay is a no-op, so the loop must terminate via
  // its own elapsed-time budget, not real time. We feed a long stream of false.
  mock.setNextStartDebuggingResults(new Array(1000).fill(false));
  await mock.commands.executeCommand(RELOAD_COMMAND);
  assert.ok(mock.startDebuggingCalls.length >= 1, 'at least one attach attempt was made');
  assert.ok(mock.startDebuggingCalls.length < 1000, 'the retry loop is bounded (did not exhaust)');
  assert.ok(
    mock.shownMessages.some((m) => m.kind === 'error'),
    'a final re-attach failure must be surfaced to the author',
  );
});

// =============================================================================
// Runner
// =============================================================================

(async () => {
  console.log('\n[reloadShiori command Unit Tests (vscode mock alias)]');
  for (const { name, fn } of queue) {
    try {
      await fn();
      console.log(`  ✓ ${name}`);
      passed++;
    } catch (e: unknown) {
      console.error(`  ✗ ${name}`);
      console.error(`    ${e instanceof Error ? e.stack ?? e.message : String(e)}`);
      failed++;
    }
  }
  console.log(`\nreloadShioriCommand: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
