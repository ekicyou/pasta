// Unit tests for RunScene (task 5.2). Plain node against fakeMcpServer and a temp ghost; the real clock is
// never used for the waits (sleep is injected and counted).
//
// Requirements: 4.4 / 4.5 (not sent when the reload says not OK to play), 4.6 (Reference0 = absolute path,
// Reference1 = 1-based line), 4.7 (script shown as-is, failure notation included), 4.8 (no response → script
// log re-query and selection rule), 4.9 (could not confirm), 4.10 (204 hint + OnPastaSceneAt: reason lines).

import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { startFakeMcpServer, FakeMcpServer, FakeState } from './fakeMcpServer';
import { DEFAULT_TIMING, FlowContext, Timing } from '../reloadShiori';
import { SceneOutcome, findPlayedScript, formatScene, runScene, sceneReferences } from '../runSceneAtCursor';
import type { GhostTarget } from '../../../shared/ghostFiles.ts';
import { nodeGhostIO } from '../nodeGhostIO';

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

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'runScene-'));
let ghostNo = 0;

function makeGhost(): GhostTarget {
  const root = path.join(tmp, `g${ghostNo++}`);
  const master = path.join(root, 'ghost', 'master');
  fs.mkdirSync(path.join(master, 'dic'), { recursive: true });
  return { root, master, displayName: path.basename(root), key: root.toLowerCase() };
}

const ts = () => new Date().toISOString();

/** Counts the waits; yields for real only briefly so the fake's reload timer can write pasta.log. */
function countingSleep(onSleep?: (n: number) => void) {
  const waits: number[] = [];
  const sleep = async (ms: number) => {
    waits.push(ms);
    onSleep?.(waits.length);
    await new Promise((r) => setTimeout(r, 1));
  };
  return { waits, sleep };
}

async function play(
  server: FakeMcpServer, ghost: GhostTarget, setup: Partial<FakeState>,
  opts: { sleep?: (ms: number) => Promise<void>; timing?: Partial<Timing>; line?: number } = {},
) {
  Object.assign(server.state, { reloadLog: ['loadedTrue'], raiseEvent: { mode: 'script', script: 'x' }, raiseEventLog: [] }, setup);
  server.state.runningRoots = [ghost.root];
  const file = path.join(ghost.master, 'dic', 'pose.pasta');
  const ctx: FlowContext = {
    io: nodeGhostIO,
    endpoint: { port: server.port, basewareName: 'SSP' }, ghost,
    timing: { ...DEFAULT_TIMING, ...opts.timing }, sleep: opts.sleep ?? countingSleep().sleep,
  };
  const before = server.state.calls.length;
  const outcome = await runScene(ctx, file, opts.line ?? 42);
  return { outcome, file, calls: server.state.calls.slice(before) };
}

function is<K extends SceneOutcome['kind']>(o: SceneOutcome, kind: K): Extract<SceneOutcome, { kind: K }> {
  assert.strictEqual(o.kind, kind, JSON.stringify(o));
  return o as Extract<SceneOutcome, { kind: K }>;
}

const GUIDE = /`?pasta\.toml`? に `?\[debug\] enabled = true`? を書いて/;

async function main() {
  console.log('runSceneAtCursor');
  const server = await startFakeMcpServer();

  await test('sceneReferences: Reference0 is the path as given, Reference1 the 1-based line as a string (4.6)', async () => {
    assert.deepStrictEqual(sceneReferences('C:\\g\\ghost\\master\\dic\\pose.pasta', 1), ['C:\\g\\ghost\\master\\dic\\pose.pasta', '1']);
    assert.deepStrictEqual(sceneReferences('/g/dic/a.pasta', 42), ['/g/dic/a.pasta', '42']);
  });

  await test('plays: reload, latest script id, then OnPastaSceneAt [abs path, "42"] without strict; script shown (4.6・4.7)', async () => {
    const g = makeGhost();
    const { outcome, file, calls } = await play(server, g, { raiseEvent: { mode: 'script', script: '\\0\\s[0]こんにちは\\e' } });
    const o = is(outcome, 'played');
    assert.strictEqual(o.script, '\\0\\s[0]こんにちは\\e');
    assert.strictEqual(o.viaLog, false);
    assert.ok(path.isAbsolute(file));
    const names = calls.map((c) => c.name);
    assert.deepStrictEqual(names.slice(names.indexOf('reload')), ['reload', 'get_log', 'get_log', 'raise_event']);
    assert.deepStrictEqual(calls[calls.length - 2].arguments, { log_type: 'script', ghost_name: g.root, max_count: 1 });
    assert.deepStrictEqual(calls[calls.length - 1].arguments, { event: 'OnPastaSceneAt', references: [file, '42'], ghost_name: g.root });
    const f = formatScene(o);
    assert.ok(f.lines.includes('読み込みは誤り無く終わりました'), f.lines.join('\n'));
    assert.ok(f.lines.some((l) => l.includes('\\0\\s[0]こんにちは\\e')), f.lines.join('\n'));
    assert.strictEqual(f.notice, undefined);
  });

  await test('a failure-notation script is shown as-is, not as an error (4.7)', async () => {
    const notation = '【シーンが見つからない：OnPastaSceneAt dic/無い.pasta:1】';
    const { outcome } = await play(server, makeGhost(), { raiseEvent: { mode: 'script', script: notation } });
    const o = is(outcome, 'played');
    assert.strictEqual(o.script, notation);
    const f = formatScene(o);
    assert.ok(f.lines.some((l) => l.includes(notation)));
    assert.strictEqual(f.notice, undefined);
  });

  await test('not sent when the reload adds an error, ends with loaded=false, or times out (4.4・4.5)', async () => {
    for (const reloadLog of [['error', 'loadedTrue'], ['loadedFalse'], []] as FakeState['reloadLog'][]) {
      const { outcome, calls } = await play(server, makeGhost(), { reloadLog });
      const o = is(outcome, 'notPlayed');
      assert.ok(!calls.some((c) => c.name === 'raise_event'), JSON.stringify(reloadLog));
      assert.ok(formatScene(o).notice, 'a notice explains why it did not play');
    }
  });

  await test('not sent when the reload is refused (ghost not running there)', async () => {
    const g = makeGhost();
    server.state.runningRoots = [];
    const ctx: FlowContext = { io: nodeGhostIO, endpoint: { port: server.port, basewareName: 'SSP' }, ghost: g, timing: DEFAULT_TIMING, sleep: countingSleep().sleep };
    const before = server.state.calls.length;
    const o = is(await runScene(ctx, path.join(g.master, 'dic', 'a.pasta'), 1), 'notPlayed');
    assert.ok(!server.state.calls.slice(before).some((c) => c.name === 'raise_event'));
    assert.match(formatScene(o).notice ?? '', /頼めませんでした/);
  });

  await test('findPlayedScript: the LAST record labelled exactly Ghost:OnPastaSceneAt; none → undefined (4.8)', async () => {
    const e = (id: number, label: string, text: string) => ({ id, label, name: 'g', text });
    assert.strictEqual(findPlayedScript([
      e(1, 'Ghost:OnPastaSceneAt', 'first'), e(2, 'Ghost:OnPastaScene', 'other event'),
      e(3, 'Ghost:OnPastaSceneAt', 'last'), e(4, 'Ghost:OnSecondChange', 'later, other'), e(5, 'Ghost:OnPastaSceneAtX', 'no'),
    ]), 'last');
    assert.strictEqual(findPlayedScript([e(1, 'Ghost:OnBoot', 'x')]), undefined);
    assert.strictEqual(findPlayedScript([]), undefined);
  });

  await test('no response (connection dropped): script from get_log(script, since_id) of the latest id before sending (4.8)', async () => {
    const g = makeGhost();
    const oldId = server.addLog('script', 'Ghost:OnPastaSceneAt', 'old script from before', path.basename(g.root));
    const { outcome, calls } = await play(server, g, { raiseEvent: { mode: 'drop', script: '\\0played via log\\e' } });
    const o = is(outcome, 'played');
    assert.strictEqual(o.script, '\\0played via log\\e');
    assert.strictEqual(o.viaLog, true);
    const requery = calls.slice(calls.findIndex((c) => c.name === 'raise_event') + 1);
    assert.deepStrictEqual(requery[0]?.arguments, { log_type: 'script', ghost_name: g.root, since_id: oldId });
    assert.ok(formatScene(o).lines.some((l) => l.includes('再生の記録')), 'says the script came from the log');
  });

  await test('no response, record appears after 2 waits: re-queries poll by poll, then plays (4.8)', async () => {
    const g = makeGhost();
    const start = server.state.calls.length;
    let afterRaise = 0;
    const { waits, sleep } = countingSleep(() => {
      if (server.state.calls.slice(start).some((c) => c.name === 'raise_event') && ++afterRaise === 2) {
        server.addLog('script', 'Ghost:OnPastaSceneAt', 'late script', path.basename(g.root));
      }
    });
    const { outcome, calls } = await play(server, g, { raiseEvent: { mode: 'silent' } }, { sleep, timing: { callTimeoutMs: 200 } });
    const o = is(outcome, 'played');
    assert.strictEqual(o.script, 'late script');
    assert.strictEqual(o.viaLog, true);
    const after = calls.slice(calls.findIndex((c) => c.name === 'raise_event') + 1);
    assert.strictEqual(after.filter((c) => c.name === 'get_log').length, 3);
    assert.deepStrictEqual(waits.slice(-2), [250, 250]);
  });

  await test('no response and no record within 5 s: could not confirm, 20 waits of 250 ms (4.9)', async () => {
    const g = makeGhost();
    const { waits, sleep } = countingSleep();
    const { outcome, calls } = await play(server, g, { raiseEvent: { mode: 'silent' } }, { sleep, timing: { callTimeoutMs: 200 } });
    const o = is(outcome, 'unconfirmed');
    const after = calls.slice(calls.findIndex((c) => c.name === 'raise_event') + 1);
    assert.strictEqual(after.length, 21);
    assert.ok(after.every((c) => c.name === 'get_log' && c.arguments.log_type === 'script'));
    const sceneWaits = waits.slice(waits.length - 20);
    assert.strictEqual(sceneWaits.reduce((a, b) => a + b, 0), 5000);
    const f = formatScene(o);
    assert.match(f.notice ?? '', /台本を確かめられませんでした/);
  });

  await test('204: [debug] enabled hint plus only the OnPastaSceneAt: warn lines appended to pasta.log (4.10)', async () => {
    const reason = `${ts()}  WARN pasta_lua::runtime: OnPastaSceneAt: debug is disabled; scene not played`;
    const { outcome } = await play(server, makeGhost(), {
      raiseEvent: { mode: 'noScript' },
      raiseEventLog: [
        reason,
        `${ts()}  INFO pasta_lua::runtime: OnPastaSceneAt: not a warning`,
        `${ts()}  WARN pasta_lua::runtime: OnPastaScene: scene not found: global='x'`,
      ],
    });
    const o = is(outcome, 'noScript');
    assert.deepStrictEqual(o.reasons, [reason]);
    const f = formatScene(o);
    assert.ok(f.lines.some((l) => GUIDE.test(l)), f.lines.join('\n'));
    assert.ok(f.lines.some((l) => l.includes('OnPastaSceneAt: debug is disabled')));
    assert.match(f.notice ?? '', GUIDE);
  });

  await test('204 with no reason lines still gives the hint', async () => {
    const o = is((await play(server, makeGhost(), { raiseEvent: { mode: 'noScript' } })).outcome, 'noScript');
    assert.deepStrictEqual(o.reasons, []);
    assert.ok(formatScene(o).lines.some((l) => GUIDE.test(l)));
  });

  await server.close();
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(`\nrunSceneAtCursor: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
}

main();
