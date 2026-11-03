// E2E tests for DevCommands (task 6.1): the common entrance of the 3 commands, against the mock vscode
// (bundled with --alias:vscode=./src/test/mock/vscode.ts), fakeMcpServer and temp ghosts.
//
// Requirements: 2.4 / 2.5 (document and ghost), 3.2–3.7 (choosing and remembering the endpoint),
// 7.1–7.5 (output, notifications, progress, one command per ghost).

import * as assert from 'assert';
import * as fs from 'fs';
import * as http from 'http';
import * as os from 'os';
import * as path from 'path';
import * as vscode from './mock/vscode';
import { startFakeMcpServer, FakeMcpServer } from './fakeMcpServer';
import { OUTPUT_CHANNEL, registerDevCommands } from '../devCommands';
import { DEFAULT_TIMING } from '../reloadShiori';

let passed = 0;
let failed = 0;

async function test(name: string, fn: () => Promise<void>) {
  try {
    await fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e: any) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

// mock Uri keeps the path as given and findGhostMaster splits on '/', so ghosts live under a '/'-separated path
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'devCommands-')).replace(/\\/g, '/');
let ghostNo = 0;

function makeGhost(name = 'テストゴースト'): { root: string; dic: string } {
  const root = `${tmp}/g${ghostNo++}`;
  const master = `${root}/ghost/master`;
  fs.mkdirSync(`${master}/dic`, { recursive: true });
  fs.writeFileSync(`${master}/descript.txt`, `charset,UTF-8\nname,${name}\n`);
  const dic = `${master}/dic/main.pasta`;
  fs.writeFileSync(dic, '＊会話\n  さくら：こんにちは\n');
  return { root, dic };
}

function openEditor(file: string, languageId = 'pasta', line = 0) {
  const document = vscode.createTextDocument(file, { isDirty: false, languageId });
  vscode.window.activeTextEditor = { document, selection: { active: new vscode.Position(line, 0) } };
}

const run = (id: string) => vscode.commands.executeCommand(id);
const callsOf = (s: FakeMcpServer, name: string) => s.state.calls.filter((c) => c.name === name);
const messagesSince = (n: number) => vscode.shownMessages.slice(n);
const outputSince = (n: number) => vscode.outputLines.slice(n).map((l) => l.message);

async function main() {
  console.log('devCommands');
  const a = await startFakeMcpServer({ basewareName: 'SSP' });
  const b = await startFakeMcpServer({ basewareName: 'areka' });
  // a port that answers initialize and get_property (for brokenRoot only) but returns a non-JSON body for everything else (badShape)
  let brokenRoot = '';
  const broken = http.createServer((req, res) => {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      const msg = JSON.parse(body);
      if (msg.id === undefined) { res.writeHead(202).end(); return; }
      const result = msg.method === 'initialize' ? { protocolVersion: '2025-06-18', capabilities: {} }
        : msg.params?.name === 'get_property'
          ? msg.params.arguments?.ghost_name === brokenRoot
            ? { content: [{ type: 'text', text: 'Broken' }], isError: false }
            : { content: [{ type: 'text', text: 'NG:Cannot find active ghost from specified name' }], isError: true }
          : undefined;
      if (result) res.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result }));
      else res.writeHead(200, { 'Content-Type': 'text/plain' }).end('not json');
    });
  });
  await new Promise<void>((r) => broken.listen(0, '127.0.0.1', r));
  const brokenPort = (broken.address() as { port: number }).port;

  const context = { subscriptions: [] as { dispose(): unknown }[] };
  const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, Math.min(ms, 5)));
  registerDevCommands(context as any, { ports: [a.port, b.port, brokenPort], timing: DEFAULT_TIMING, sleep });

  await test('registers the 3 commands and one output channel', async () => {
    for (const id of ['pasta.runSceneAtCursor', 'pasta.reloadShiori', 'pasta.showSaveVariables']) {
      assert.ok(vscode.registeredCommands.has(id), id);
    }
    assert.strictEqual(context.subscriptions.length, 4);
  });

  await test('2.5: ▶ シーンを実行 on a .lua stops with a warning and sends nothing', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    openEditor(`${g.root}/ghost/master/scripts/x.lua`, 'lua');
    const m = vscode.shownMessages.length;
    const calls = a.state.calls.length;
    await run('pasta.runSceneAtCursor');
    assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['warning']);
    assert.match(messagesSince(m)[0].message, /ゴーストを特定できません/);
    assert.strictEqual(a.state.calls.length, calls);
  });

  await test('2.5: an untitled document stops', async () => {
    const g = makeGhost();
    openEditor(g.dic);
    vscode.window.activeTextEditor.document.isUntitled = true;
    const m = vscode.shownMessages.length;
    await run('pasta.reloadShiori');
    assert.match(messagesSince(m)[0].message, /ゴーストを特定できません/);
  });

  await test('2.4: a file outside any ghost stops with its path', async () => {
    const file = `${tmp}/loose/a.pasta`;
    openEditor(file);
    const m = vscode.shownMessages.length;
    await run('pasta.reloadShiori');
    assert.strictEqual(messagesSince(m)[0].kind, 'warning');
    assert.ok(messagesSince(m)[0].message.includes('ゴーストのフォルダの中のファイルではありません'));
    assert.ok(messagesSince(m)[0].message.includes(file));
  });

  await test('3.7: no endpoint runs the ghost → warning with name, root and what to check; nothing sent', async () => {
    const g = makeGhost('見つからない子');
    a.state.runningRoots = [];
    b.state.runningRoots = [];
    openEditor(g.dic);
    const m = vscode.shownMessages.length;
    const o = vscode.outputLines.length;
    await run('pasta.reloadShiori');
    const msg = messagesSince(m);
    assert.strictEqual(msg.length, 1);
    assert.strictEqual(msg[0].kind, 'warning');
    for (const part of ['SSP・areka が見つかりません', '見つからない子', g.root, 'MCP の窓口']) {
      assert.ok(msg[0].message.includes(part), part);
    }
    assert.ok(outputSince(o).some((l) => l.includes(g.root)));
    assert.strictEqual(callsOf(a, 'reload').length + callsOf(b, 'reload').length, 0);
  });

  await test('3.3 / 3.4 / 3.5: two endpoints → pick; closing sends nothing; the pick is remembered', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    b.state.runningRoots = [g.root];
    openEditor(g.dic);
    const reloads = () => [callsOf(a, 'reload').length, callsOf(b, 'reload').length];
    const before = reloads();

    const q = vscode.quickPickCalls.length;
    vscode.setNextQuickPickSelection(undefined); // close the pick
    const m = vscode.shownMessages.length;
    await run('pasta.reloadShiori');
    assert.deepStrictEqual(vscode.quickPickCalls.slice(q).map((c) => c.items),
      [[`SSP（127.0.0.1:${a.port}）`, `areka（127.0.0.1:${b.port}）`]]);
    assert.deepStrictEqual(reloads(), before);
    assert.strictEqual(messagesSince(m).length, 0, 'closing the pick shows nothing');

    vscode.setNextQuickPickSelection(`areka（127.0.0.1:${b.port}）`);
    const o = vscode.outputLines.length;
    await run('pasta.reloadShiori');
    assert.deepStrictEqual(reloads(), [before[0], before[1] + 1]);
    assert.ok(outputSince(o).some((l) => /^\[\d\d:\d\d:\d\d\] SHIORI をリロード — テストゴースト（.*）→ areka 127\.0\.0\.1:\d+$/.test(l)
      && l.includes(g.root) && l.includes(String(b.port))), outputSince(o).join('\n'));

    const q2 = vscode.quickPickCalls.length;
    await run('pasta.showSaveVariables');
    assert.strictEqual(vscode.quickPickCalls.length, q2, 'not asked again');
    assert.strictEqual(callsOf(b, 'raise_event').filter((c) => c.arguments.ghost_name === g.root).length, 1);
    assert.strictEqual(callsOf(a, 'raise_event').filter((c) => c.arguments.ghost_name === g.root).length, 0);
  });

  await test('7.1 / 7.2 / 7.4: success writes to the one output channel, shows it, no notification, window progress', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    b.state.runningRoots = [];
    openEditor(g.dic, 'pasta', 1);
    const m = vscode.shownMessages.length;
    const o = vscode.outputLines.length;
    const s = vscode.shownOutputChannels.length;
    const p = vscode.progressCalls.length;
    await run('pasta.runSceneAtCursor');
    assert.strictEqual(messagesSince(m).length, 0, JSON.stringify(messagesSince(m)));
    assert.ok(vscode.outputLines.slice(o).every((l) => l.channel === OUTPUT_CHANNEL));
    assert.ok(outputSince(o).some((l) => l.includes('fake script')), outputSince(o).join('\n'));
    assert.ok(vscode.shownOutputChannels.slice(s).every((c) => c === OUTPUT_CHANNEL) && vscode.shownOutputChannels.length > s);
    const progress = vscode.progressCalls.slice(p) as { location: number; title: string }[];
    assert.ok(progress.length > 0 && progress.every((x) => x.location === vscode.ProgressLocation.Window));
    assert.ok(progress.some((x) => x.title === 'Pasta: ▶ シーンを実行'));
    assert.strictEqual(vscode.createdStatusBarItems.length, 0);
    const scene = callsOf(a, 'raise_event').pop()!;
    assert.deepStrictEqual(scene.arguments.references, [g.dic, '2']);
  });

  await test('3.6: a different baseware answering the remembered port → forget and rediscover', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    b.state.runningRoots = [];
    openEditor(g.dic);
    await run('pasta.reloadShiori'); // remembers SSP on a.port
    a.state.basewareName = 'areka-on-same-port';
    const probes = callsOf(a, 'get_property').length;
    const o = vscode.outputLines.length;
    await run('pasta.reloadShiori');
    assert.ok(callsOf(a, 'get_property').length >= probes + 2, 'identify, then a fresh scan');
    assert.ok(outputSince(o).some((l) => l.includes(`→ areka-on-same-port 127.0.0.1:${a.port}`)), outputSince(o).join('\n'));
    a.state.basewareName = 'SSP';
  });

  await test('3.6: the remembered endpoint no longer runs the ghost → rediscover and use the other', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    b.state.runningRoots = [];
    openEditor(g.dic);
    await run('pasta.reloadShiori'); // remembers a
    a.state.runningRoots = [];
    b.state.runningRoots = [g.root];
    const before = callsOf(b, 'reload').length;
    await run('pasta.reloadShiori');
    assert.strictEqual(callsOf(b, 'reload').length, before + 1);
  });

  await test('7.5: a second command on the same ghost while one runs is refused', async () => {
    const g = makeGhost();
    a.state.runningRoots = [g.root];
    b.state.runningRoots = [];
    a.state.reloadDelayMs = 100;
    openEditor(g.dic);
    const m = vscode.shownMessages.length;
    const before = callsOf(a, 'reload').length;
    await Promise.all([run('pasta.reloadShiori'), run('pasta.reloadShiori')]);
    a.state.reloadDelayMs = 0;
    assert.strictEqual(callsOf(a, 'reload').length, before + 1);
    assert.deepStrictEqual(messagesSince(m).map((x) => [x.kind, x.message]), [['warning', 'このゴーストへのコマンドが進んでいます']]);
    await run('pasta.reloadShiori'); // released afterwards
    assert.strictEqual(callsOf(a, 'reload').length, before + 2);
  });

  await test('7.3: a malformed answer is an error notification with the text in the output', async () => {
    const g = makeGhost();
    a.state.runningRoots = [];
    b.state.runningRoots = [];
    brokenRoot = g.root;
    openEditor(g.dic);
    const m = vscode.shownMessages.length;
    const o = vscode.outputLines.length;
    await run('pasta.reloadShiori'); // only the broken port "runs" it
    assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['error']);
    assert.ok(outputSince(o).some((l) => l.includes('Broken 127.0.0.1')));
    assert.ok(outputSince(o).some((l) => l.includes('読み込み直しを頼めませんでした')), outputSince(o).join('\n'));
  });

  await a.close();
  await b.close();
  await new Promise<void>((r) => broken.close(() => r()));
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(`\n${passed} passed, ${failed} failed`);
  if (failed > 0) {
    process.exit(1);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
