// Real-module unit tests for the playScene VSCode wiring (Task 5.2).
//
// Imports the REAL extension.ts against the vscode mock in ./mock/vscode.ts
// (wired via esbuild --alias:vscode=./src/test/mock/vscode.ts), mirroring
// vscodeModules.test.ts. Exercises ONLY the `pasta.debug.playScene` command
// registered by activate():
//   - no active pasta session  -> warn, no customRequest (R1.3)
//   - showInputBox cancelled    -> no customRequest (R1.4)
//   - empty/whitespace name     -> no customRequest (R1.4/2.5 client guard)
//   - valid name                -> customRequest('pasta/playScene', { scene }) (R1.2)
//   - customRequest rejects     -> showErrorMessage (R2.5)
//
// The pure request logic lives in playSceneRequest.ts (Task 5.1) and has its
// own tests; here we only verify the vscode glue.

import * as assert from 'assert';
import * as path from 'path';
import * as mock from './mock/vscode';
import { activate } from '../extension';
import { requestCommand } from '../playSceneRequest';

const ROOT = path.resolve(__dirname, '..', '..');

// =============================================================================
// Async-aware sequential test runner (mirrors vscodeModules.test.ts)
// =============================================================================

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void | Promise<void> }> = [];

function test(name: string, fn: () => void | Promise<void>): void {
  queue.push({ name, fn });
}

const PLAY_COMMAND = 'pasta.debug.playScene';

function makePastaSession() {
  const session = {
    type: 'pasta',
    requests: [] as Array<{ command: string; args: unknown }>,
    failNext: false,
    customRequest: async (command: string, args: unknown) => {
      if (session.failNext) {
        throw new Error('backend unavailable');
      }
      session.requests.push({ command, args });
      return {};
    },
  };
  return session;
}

// =============================================================================
// activate() registers the playScene command
// =============================================================================
// activate registers persistent listeners on the shared mock emitters; run it
// once up front, then drive the command via mock.commands.executeCommand.

test('activate registers the pasta.debug.playScene command', async () => {
  mock.workspace.textDocuments = [];
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(ROOT) };
  await activate(context as never);
  assert.ok(mock.registeredCommands.has(PLAY_COMMAND), 'playScene command must be registered');
});

test('playScene without an active pasta session warns and sends nothing (R1.3)', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  await mock.commands.executeCommand(PLAY_COMMAND);
  assert.strictEqual(mock.shownMessages.length, 1, 'exactly one notification');
  assert.strictEqual(mock.shownMessages[0].kind, 'warning');
  assert.ok(mock.shownMessages[0].message.includes('Pasta デバッグセッション'));
  assert.strictEqual(mock.inputBoxCalls.length, 0, 'must not prompt for a scene name when not connected');
});

test('playScene cancelled (showInputBox undefined) sends nothing (R1.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  mock.setNextInputBoxResult(undefined); // user pressed Esc / cancelled
  await mock.commands.executeCommand(PLAY_COMMAND);
  assert.strictEqual(mock.inputBoxCalls.length, 1, 'a scene name must be requested');
  assert.deepStrictEqual(session.requests, [], 'cancel must not send a customRequest');
  assert.strictEqual(mock.shownMessages.length, 0, 'cancel is silent (no error)');
});

test('playScene with an empty/whitespace name sends nothing (R1.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  mock.setNextInputBoxResult('   ');
  await mock.commands.executeCommand(PLAY_COMMAND);
  assert.deepStrictEqual(session.requests, [], 'whitespace-only name must not send a customRequest');
});

test('playScene with a valid name sends customRequest(pasta/playScene, { scene }) (R1.2)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  mock.setNextInputBoxResult('挨拶');
  await mock.commands.executeCommand(PLAY_COMMAND);
  assert.deepStrictEqual(session.requests, [{ command: requestCommand, args: { scene: '挨拶' } }]);
  assert.strictEqual(requestCommand, 'pasta/playScene', 'command name is the cross-language contract');
  assert.strictEqual(mock.shownMessages.length, 0, 'success is silent');
});

test('playScene surfaces an error when customRequest rejects (R2.5)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  session.failNext = true;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  mock.setNextInputBoxResult('挨拶');
  await mock.commands.executeCommand(PLAY_COMMAND);
  assert.strictEqual(mock.shownMessages.length, 1);
  assert.strictEqual(mock.shownMessages[0].kind, 'error');
  assert.ok(mock.shownMessages[0].message.includes('backend unavailable'));
});

// =============================================================================
// Runner
// =============================================================================

(async () => {
  console.log('\n[playScene command Unit Tests (vscode mock alias)]');
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
  console.log(`\nplaySceneCommand: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
