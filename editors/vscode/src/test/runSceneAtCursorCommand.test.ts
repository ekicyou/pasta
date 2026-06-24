// Real-module unit tests for the runSceneAtCursor VSCode wiring (Task 5.1).
//
// Imports the REAL extension.ts against the vscode mock in ./mock/vscode.ts
// (wired via esbuild --alias:vscode=./src/test/mock/vscode.ts), mirroring
// playSceneCommand.test.ts (REPLACED by this file). Exercises ONLY the
// `pasta.runSceneAtCursor` command registered by activate():
//   - connected + valid cursor -> customRequest('pasta/playSceneAt', { uri, line })
//                                 with 1-based line; NO scene-name input box.
//   - no active pasta session   -> no customRequest; warning + "デバッグ開始"
//                                 action; choosing it -> startDebugging(resolved).
//   - dirty document            -> warning shown / save prompted.
//   - backend error response    -> error surfaced to author.
//
// The pure request/config logic lives in runSceneAtCursor.ts (Task 5.1) and has
// its own tests; here we only verify the vscode glue.

import * as assert from 'assert';
import * as path from 'path';
import * as mock from './mock/vscode';
import { activate } from '../extension';
import { requestCommand } from '../runSceneAtCursor';

const ROOT = path.resolve(__dirname, '..', '..');

// =============================================================================
// Async-aware sequential test runner (mirrors playSceneCommand.test.ts)
// =============================================================================

let passed = 0;
let failed = 0;
const queue: Array<{ name: string; fn: () => void | Promise<void> }> = [];

function test(name: string, fn: () => void | Promise<void>): void {
  queue.push({ name, fn });
}

const RUN_COMMAND = 'pasta.runSceneAtCursor';

function makePastaSession() {
  const session = {
    type: 'pasta',
    requests: [] as Array<{ command: string; args: unknown }>,
    failNext: false,
    customRequest: async (command: string, args: unknown) => {
      if (session.failNext) {
        throw new Error('カーソル下にシーンがありません');
      }
      session.requests.push({ command, args });
      return { success: true };
    },
  };
  return session;
}

/** Build a mock active editor on a .pasta document at the given 0-based line. */
function makePastaEditor(opts?: { line?: number; dirty?: boolean; uri?: string }) {
  const line = opts?.line ?? 0;
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
  return {
    document,
    selection: { active: new mock.Position(line, 0) },
  };
}

// =============================================================================
// activate() registers the runSceneAtCursor command
// =============================================================================

test('activate registers the pasta.runSceneAtCursor command', async () => {
  mock.workspace.textDocuments = [];
  const context = { subscriptions: [] as Array<{ dispose(): void }>, extensionUri: mock.Uri.file(ROOT) };
  await activate(context as never);
  assert.ok(mock.registeredCommands.has(RUN_COMMAND), 'runSceneAtCursor command must be registered');
});

test('connected + valid cursor sends customRequest(pasta/playSceneAt, {uri, line}) with 1-based line and never prompts (R1.2/R1.3/R1.4)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  const editor = makePastaEditor({ line: 4 }); // VSCode 0-based line 4 -> engine line 5
  mock.window.activeTextEditor = editor;
  mock.shownMessages.length = 0;
  mock.inputBoxCalls.length = 0;
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.deepStrictEqual(session.requests, [
    { command: requestCommand, args: { uri: 'file:///c/work/intro.pasta', line: 5 } },
  ]);
  assert.strictEqual(requestCommand, 'pasta/playSceneAt', 'command name is the cross-language contract');
  assert.strictEqual(mock.inputBoxCalls.length, 0, 'must NEVER prompt for a scene name');
});

test('no active editor or non-pasta document warns and sends nothing', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = undefined;
  mock.shownMessages.length = 0;
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.deepStrictEqual(session.requests, [], 'no editor -> no customRequest');
  assert.strictEqual(mock.shownMessages.length, 1);
  assert.strictEqual(mock.shownMessages[0].kind, 'warning');
});

test('NOT connected -> no customRequest; warns; choosing デバッグ開始 -> startDebugging(resolved config) (R6.2/R6.3)', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.window.activeTextEditor = makePastaEditor({ line: 0 });
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  // The author clicks the "デバッグ開始" action button.
  mock.setNextWarningSelection('デバッグ開始');
  // No workspace pasta config -> default attach 127.0.0.1:9276.
  mock.setLaunchConfigurations(undefined);
  await mock.commands.executeCommand(RUN_COMMAND);
  // A warning offering the start-debugging action was shown.
  assert.strictEqual(mock.shownMessages.length, 1);
  assert.strictEqual(mock.shownMessages[0].kind, 'warning');
  assert.ok(
    mock.shownMessages[0].items?.includes('デバッグ開始'),
    'the warning must offer a デバッグ開始 action',
  );
  // startDebugging was invoked with the resolved (default) pasta config.
  assert.strictEqual(mock.startDebuggingCalls.length, 1, 'startDebugging must be called once');
  const cfg = mock.startDebuggingCalls[0].config;
  assert.strictEqual(cfg.type, 'pasta');
  assert.strictEqual(cfg.request, 'attach');
  assert.strictEqual(cfg.host, '127.0.0.1');
  assert.strictEqual(cfg.port, 9276);
});

test('NOT connected, author dismisses the warning -> no startDebugging', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.window.activeTextEditor = makePastaEditor({ line: 0 });
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setNextWarningSelection(undefined); // dismissed
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls.length, 0, 'dismissed warning must not start debugging');
});

test('NOT connected prefers a workspace pasta config when present (R6.3)', async () => {
  mock.debug.activeDebugSession = undefined;
  mock.window.activeTextEditor = makePastaEditor({ line: 0 });
  mock.shownMessages.length = 0;
  mock.startDebuggingCalls.length = 0;
  mock.setNextWarningSelection('デバッグ開始');
  mock.setLaunchConfigurations([
    { type: 'node', request: 'launch', name: 'Node' },
    { type: 'pasta', request: 'attach', name: 'My Pasta', host: '127.0.0.1', port: 5555 },
  ]);
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.strictEqual(mock.startDebuggingCalls.length, 1);
  assert.strictEqual(mock.startDebuggingCalls[0].config.port, 5555, 'workspace pasta config port wins');
});

test('dirty document warns about unsaved changes / prompts save (R7.2/R7.5)', async () => {
  const session = makePastaSession();
  mock.debug.activeDebugSession = session;
  const editor = makePastaEditor({ line: 2, dirty: true });
  mock.window.activeTextEditor = editor;
  mock.shownMessages.length = 0;
  // Author chooses "保存" on the dirty warning.
  mock.setNextWarningSelection('保存');
  await mock.commands.executeCommand(RUN_COMMAND);
  // A dirty warning was surfaced.
  assert.ok(
    mock.shownMessages.some((m) => m.kind === 'warning' && m.message.includes('未保存')),
    'a dirty/unsaved warning must be shown',
  );
  assert.strictEqual(editor.document.saveCalls, 1, 'choosing 保存 must save the document');
});

test('backend error response is surfaced to the author (R6.4)', async () => {
  const session = makePastaSession();
  session.failNext = true;
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor({ line: 0 });
  mock.shownMessages.length = 0;
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.ok(
    mock.shownMessages.some(
      (m) => m.kind === 'error' && m.message.includes('カーソル下にシーンがありません'),
    ),
    'the backend error reason must be surfaced',
  );
});

test('backend success:false response (resolved, not thrown) is surfaced as an error (R6.4)', async () => {
  const session = {
    type: 'pasta',
    requests: [] as Array<{ command: string; args: unknown }>,
    customRequest: async (command: string, args: unknown) => {
      session.requests.push({ command, args });
      return { success: false, message: 'カーソル下にシーンがありません' };
    },
  };
  mock.debug.activeDebugSession = session;
  mock.window.activeTextEditor = makePastaEditor({ line: 0 });
  mock.shownMessages.length = 0;
  await mock.commands.executeCommand(RUN_COMMAND);
  assert.ok(
    mock.shownMessages.some(
      (m) => m.kind === 'error' && m.message.includes('カーソル下にシーンがありません'),
    ),
    'a resolved success:false response must be surfaced as an error',
  );
});

// =============================================================================
// Runner
// =============================================================================

(async () => {
  console.log('\n[runSceneAtCursor command Unit Tests (vscode mock alias)]');
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
  console.log(`\nrunSceneAtCursorCommand: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
})();
