// E2E tests for DevCommands (tasks 6.1 / 6.2): the 3 commands, against the mock vscode
// (bundled with --alias:vscode=./src/test/mock/vscode.ts), fakeMcpServer and temp ghosts.
//
// Requirements: 2.4 / 2.5 (document and ghost), 3.2–3.7 (choosing and remembering the endpoint),
// 7.1–7.5 (output, notifications, progress, one command per ghost), 4.2–4.11 / 5.2–5.8 / 6.2–6.4
// (saving, then the reload / play / dump flows).

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

const run = (id: string, ...args: unknown[]) => vscode.commands.executeCommand(id, ...args);
const callsOf = (s: FakeMcpServer, name: string) => s.state.calls.filter((c) => c.name === name);
const messagesSince = (n: number) => vscode.shownMessages.slice(n);
const outputSince = (n: number) => vscode.outputLines.slice(n).map((l) => l.message);

/** A ghost running only on `s`, with the editor on its dic; returns the ghost and counters to diff against. */
function onlyOn(s: FakeMcpServer, other: FakeMcpServer, line = 0) {
  const g = makeGhost();
  s.state.runningRoots = [g.root];
  other.state.runningRoots = [];
  openEditor(g.dic, 'pasta', line);
  return { g, m: vscode.shownMessages.length, o: vscode.outputLines.length, scenes: callsOf(s, 'raise_event').length };
}

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

  // --- 6.2: the 3 flows ---------------------------------------------------------

  const resetServer = (s: FakeMcpServer) => {
    s.state.reloadLog = ['loadedTrue'];
    s.state.raiseEvent = { mode: 'script', script: '\\0\\s[0]fake script\\e' };
    s.state.raiseEventLog = [];
    s.state.writeVariables = true;
  };

  await test('4.5: load errors increased → no raise_event, the summary and a warning', async () => {
    a.state.reloadLog = ['error', 'loadedTrue'];
    const { m, o, scenes } = onlyOn(a, b);
    await run('pasta.runSceneAtCursor');
    resetServer(a);
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes);
    assert.ok(outputSince(o).some((l) => l.includes('誤り 1 件')), outputSince(o).join('\n'));
    assert.ok(outputSince(o).includes('シーンを再生しませんでした'));
    assert.deepStrictEqual(messagesSince(m).map((x) => [x.kind, x.message]), [['warning', '読み込みの誤りが 1 件あります']]);
  });

  await test('4.4 / 5.8: the load never finishes → no raise_event, the time-out notice', async () => {
    a.state.reloadLog = [];
    const { m, o, scenes } = onlyOn(a, b);
    await run('pasta.runSceneAtCursor');
    resetServer(a);
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes);
    assert.ok(outputSince(o).some((l) => l.startsWith('読み込みの終わりを確かめられませんでした')), outputSince(o).join('\n'));
    assert.deepStrictEqual(messagesSince(m).map((x) => [x.kind, x.message]), [['warning', '読み込みの終わりを確かめられませんでした']]);
  });

  await test('5.8: SHIORI をリロード summarises what increased before a time-out and never plays', async () => {
    a.state.reloadLog = ['warn'];
    const { m, o, scenes } = onlyOn(a, b);
    await run('pasta.reloadShiori');
    resetServer(a);
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes);
    assert.ok(outputSince(o).some((l) => l.includes('警告 1 件')), outputSince(o).join('\n'));
    assert.deepStrictEqual(messagesSince(m).map((x) => x.message), ['読み込みの終わりを確かめられませんでした']);
  });

  await test('4.8: no response (connection dropped) → the script from get_log(script)', async () => {
    a.state.raiseEvent = { mode: 'drop', script: '\\0played via log\\e' };
    const { m, o } = onlyOn(a, b);
    await run('pasta.runSceneAtCursor');
    resetServer(a);
    assert.ok(outputSince(o).includes('台本（応答が無かったため、再生の記録から）:'), outputSince(o).join('\n'));
    assert.ok(outputSince(o).includes('\\0played via log\\e'));
    assert.strictEqual(messagesSince(m).length, 0);
  });

  await test('4.9: no response and nothing in the script log → "could not confirm" warning', async () => {
    a.state.raiseEvent = { mode: 'empty' };
    const { m, o } = onlyOn(a, b);
    await run('pasta.runSceneAtCursor');
    resetServer(a);
    assert.ok(outputSince(o).some((l) => l.startsWith('台本を確かめられませんでした')), outputSince(o).join('\n'));
    assert.deepStrictEqual(messagesSince(m).map((x) => [x.kind, x.message]), [['warning', '台本を確かめられませんでした']]);
  });

  await test('4.10: 204 → the [debug] enabled guide with the OnPastaSceneAt: reason line', async () => {
    a.state.raiseEvent = { mode: 'noScript' };
    a.state.raiseEventLog = [`${new Date().toISOString()}  WARN pasta_lua::debug: OnPastaSceneAt: debug is disabled`];
    const { m, o } = onlyOn(a, b);
    await run('pasta.runSceneAtCursor');
    resetServer(a);
    assert.ok(outputSince(o).some((l) => l.includes('[debug] enabled = true')), outputSince(o).join('\n'));
    assert.ok(outputSince(o).some((l) => l.includes('OnPastaSceneAt: debug is disabled')));
    assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['warning']);
    assert.match(messagesSince(m)[0].message, /\[debug\] enabled = true/);
  });

  await test('6.2 / 6.4: 保存変数を表示 shows the fresh variables.toml; no save, no reload', async () => {
    const { g, m, o } = onlyOn(a, b);
    vscode.workspace.textDocuments.push(vscode.createTextDocument(g.dic, { isDirty: true }));
    const saves = vscode.savedDocuments.length;
    const reloads = callsOf(a, 'reload').length;
    await run('pasta.showSaveVariables');
    vscode.workspace.textDocuments.length = 0;
    assert.ok(outputSince(o).includes('[fake]\nvalue = 1\n'), outputSince(o).join('\n'));
    assert.strictEqual(messagesSince(m).length, 0);
    assert.strictEqual(vscode.savedDocuments.length, saves, 'nothing saved');
    assert.strictEqual(callsOf(a, 'reload').length, reloads, 'no reload');
    assert.strictEqual(callsOf(a, 'raise_event').pop()!.arguments.event, 'OnPastaDumpVariables');
  });

  await test('6.3 / 6.4: variables.toml not written → the guide; no save, no reload', async () => {
    a.state.writeVariables = false;
    const { g, m, o } = onlyOn(a, b);
    vscode.workspace.textDocuments.push(vscode.createTextDocument(g.dic, { isDirty: true }));
    const saves = vscode.savedDocuments.length;
    const reloads = callsOf(a, 'reload').length;
    await run('pasta.showSaveVariables');
    vscode.workspace.textDocuments.length = 0;
    resetServer(a);
    assert.ok(outputSince(o).some((l) => l.includes('新しく書き出されませんでした')), outputSince(o).join('\n'));
    assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['warning']);
    assert.strictEqual(vscode.savedDocuments.length, saves);
    assert.strictEqual(callsOf(a, 'reload').length, reloads);
  });

  for (const id of ['pasta.runSceneAtCursor', 'pasta.reloadShiori']) {
    await test(`4.2 / 5.2: ${id} saves only the dirty .pasta / .lua / pasta.toml under the root, before reload`, async () => {
      const { g } = onlyOn(a, b);
      const other = makeGhost();
      const master = `${g.root}/ghost/master`;
      const under = [g.dic, `${master}/scripts/x.lua`, `${master}/PASTA.TOML`];
      const docs = [
        ...under.map((f) => vscode.createTextDocument(f, { isDirty: true })),
        vscode.createTextDocument(`${master}/dic/clean.pasta`, { isDirty: false }),
        vscode.createTextDocument(`${master}/readme.txt`, { isDirty: true }),
        vscode.createTextDocument(other.dic, { isDirty: true }),
        vscode.createTextDocument(`${g.root}x/ghost/master/dic/near.pasta`, { isDirty: true }), // same prefix, not under
      ];
      vscode.workspace.textDocuments.push(...docs);
      const saves = vscode.savedDocuments.length;
      const reloads = callsOf(a, 'reload').length;
      let reloadsAtSave = -1;
      const save = docs[0].save;
      docs[0].save = () => { reloadsAtSave = callsOf(a, 'reload').length - reloads; return save(); };
      await run(id);
      vscode.workspace.textDocuments.length = 0;
      assert.deepStrictEqual(vscode.savedDocuments.slice(saves).map((d) => d.path).sort(), [...under].sort());
      assert.strictEqual(reloadsAtSave, 0, 'saved before reload');
      assert.strictEqual(callsOf(a, 'reload').length, reloads + 1);
    });
  }

  await test('4.2: a document that cannot be saved stops before reload, with its path', async () => {
    const { g, m, o, scenes } = onlyOn(a, b);
    const bad = `${g.root}/ghost/master/dic/bad.pasta`;
    vscode.workspace.textDocuments.push(
      vscode.createTextDocument(g.dic, { isDirty: true }),
      vscode.createTextDocument(bad, { isDirty: true, saveResult: false }));
    const reloads = callsOf(a, 'reload').length;
    await run('pasta.runSceneAtCursor');
    vscode.workspace.textDocuments.length = 0;
    assert.strictEqual(callsOf(a, 'reload').length, reloads, 'no reload');
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes);
    assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['warning']);
    assert.ok(messagesSince(m)[0].message.includes('保存できなかったファイルがあるので止めました'));
    assert.ok(messagesSince(m)[0].message.includes(bad));
    assert.ok(outputSince(o).some((l) => l.includes(bad)));
  });

  // --- vscode-scene-outline-play 2: the command takes (uri, line0) -------------------

  /** A ghost on `a` with a second (open) document `target`, while the active editor sits on dic at line 0. */
  function argTarget(opts: { isDirty?: boolean; languageId?: string; rel?: string } = {}) {
    const ctx = onlyOn(a, b, 0);
    const file = `${ctx.g.root}/ghost/master/${opts.rel ?? 'dic/other.pasta'}`;
    const doc = vscode.createTextDocument(file, { isDirty: opts.isDirty ?? false, languageId: opts.languageId });
    vscode.workspace.textDocuments.push(doc);
    return { ...ctx, doc, uri: vscode.Uri.file(file), calls: a.state.calls.length };
  }

  await test('4.2 / 4.3 / 3.4: (uri, 2) plays that document at line 3, after save and reload', async () => {
    const { doc, uri, calls, m } = argTarget({ isDirty: true });
    const saves = vscode.savedDocuments.length;
    await run('pasta.runSceneAtCursor', uri, 2);
    vscode.workspace.textDocuments.length = 0;
    assert.deepStrictEqual(messagesSince(m), []);
    assert.deepStrictEqual(vscode.savedDocuments.slice(saves).map((d) => d.path), [doc.uri.path]);
    const names = a.state.calls.slice(calls).map((c) => c.name);
    const scene = a.state.calls.slice(calls).find((c) => c.name === 'raise_event')!;
    assert.strictEqual(scene.arguments.event, 'OnPastaSceneAt');
    assert.deepStrictEqual(scene.arguments.references, [doc.uri.fsPath, '3']);
    assert.ok(names.indexOf('reload') >= 0 && names.indexOf('reload') < names.indexOf('raise_event'), names.join(','));
  });

  await test('4.1: a right-click style call (uri only) plays the active editor at the cursor line', async () => {
    const { g, uri, scenes } = argTarget();
    vscode.window.activeTextEditor.selection = { active: new vscode.Position(1, 0) };
    await run('pasta.runSceneAtCursor', uri);
    vscode.workspace.textDocuments.length = 0;
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes + 1);
    assert.deepStrictEqual(callsOf(a, 'raise_event').pop()!.arguments.references, [g.dic, '2']);
  });

  for (const [name, setup] of [
    ['an untitled document', () => { const t = argTarget(); (t.doc as any).isUntitled = true; return t; }],
    ['a non-.pasta document', () => argTarget({ rel: 'scripts/x.lua', languageId: 'lua' })],
    ['a Uri that cannot be opened', () => ({ ...argTarget(), uri: vscode.Uri.file(`${tmp}/nowhere/missing.pasta`) })],
  ] as const) {
    await test(`4.4: (uri, 2) on ${name} stops with the ghost message and sends nothing`, async () => {
      const { uri, calls, m } = setup();
      await run('pasta.runSceneAtCursor', uri, 2);
      vscode.workspace.textDocuments.length = 0;
      assert.deepStrictEqual(messagesSince(m).map((x) => x.kind), ['warning']);
      assert.match(messagesSince(m)[0].message, /ゴーストを特定できません/);
      assert.strictEqual(a.state.calls.length, calls, 'nothing sent');
    });
  }

  await test('3.9: a second (uri, line) call while one runs is not sent', async () => {
    const { uri, m } = argTarget();
    a.state.reloadDelayMs = 100;
    const scenes = callsOf(a, 'raise_event').length;
    await Promise.all([run('pasta.runSceneAtCursor', uri, 0), run('pasta.runSceneAtCursor', uri, 0)]);
    a.state.reloadDelayMs = 0;
    vscode.workspace.textDocuments.length = 0;
    assert.strictEqual(callsOf(a, 'raise_event').length, scenes + 1);
    assert.deepStrictEqual(messagesSince(m).map((x) => [x.kind, x.message]), [['warning', 'このゴーストへのコマンドが進んでいます']]);
  });

  await test('4.6: the other 2 commands still work when given a Uri', async () => {
    const { uri } = argTarget();
    const reloads = callsOf(a, 'reload').length;
    const dumps = callsOf(a, 'raise_event').filter((c) => c.arguments.event === 'OnPastaDumpVariables').length;
    await run('pasta.reloadShiori', uri);
    await run('pasta.showSaveVariables', uri);
    vscode.workspace.textDocuments.length = 0;
    assert.strictEqual(callsOf(a, 'reload').length, reloads + 1);
    assert.strictEqual(callsOf(a, 'raise_event').filter((c) => c.arguments.event === 'OnPastaDumpVariables').length, dumps + 1);
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
