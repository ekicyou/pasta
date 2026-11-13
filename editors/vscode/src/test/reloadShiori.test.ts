// Unit tests for ReloadShiori (task 5.1). Plain node against fakeMcpServer and a temp ghost; the real clock
// is never used (sleep is injected and appends pasta.log lines on chosen polls).
//
// Requirements: 4.3 / 5.3 (reload then wait for the marker), 4.4 / 5.8 (timeout), 4.5 / 5.4 / 5.5 (errors vs
// warnings, loaded=false), 5.7 (pasta.log unreadable), 5.9 (never plays). Contract: pasta.dll writes the marker.

import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { startFakeMcpServer, FakeMcpServer } from './fakeMcpServer';
import { DEFAULT_TIMING, FlowContext, ReloadOutcome, blocksPlayback, formatReload, reloadAndWait } from '../../../shared/reloadShiori.ts';
import { DEFAULT_PASTA_LOG, type GhostTarget } from '../../../shared/ghostFiles.ts';
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

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'reloadShiori-'));
let ghostNo = 0;

function makeGhost(pastaToml?: string): GhostTarget {
  const root = path.join(tmp, `g${ghostNo++}`);
  const master = path.join(root, 'ghost', 'master');
  fs.mkdirSync(master, { recursive: true });
  if (pastaToml !== undefined) {
    fs.writeFileSync(path.join(master, 'pasta.toml'), pastaToml);
  }
  return { root, master, displayName: path.basename(root), key: root.toLowerCase() };
}

const ts = () => new Date().toISOString();
const L = {
  done: (loaded: boolean) => `${ts()}  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=${loaded}`,
  error: (t = 'load error') => `${ts()} ERROR pasta_lua::loader: ${t}`,
  warn: (t = 'load warning') => `${ts()}  WARN pasta_lua::loader: ${t}`,
  info: () => `${ts()}  INFO pasta_lua::loader: loading`,
};

/** sleep that records the waits and appends `script[n]` to pasta.log on the n-th poll (never waits for real). */
function scriptedSleep(logFile: string, script: Record<number, string[]>) {
  const waits: number[] = [];
  const sleep = async (ms: number) => {
    waits.push(ms);
    const lines = script[waits.length];
    if (lines) {
      fs.mkdirSync(path.dirname(logFile), { recursive: true });
      fs.appendFileSync(logFile, lines.map((l) => l + '\n').join(''));
    }
  };
  return { waits, sleep };
}

async function run(server: FakeMcpServer, ghost: GhostTarget, sleep: (ms: number) => Promise<void>): Promise<ReloadOutcome> {
  server.state.runningRoots = [ghost.root];
  const ctx: FlowContext = { io: nodeGhostIO, endpoint: { port: server.port, basewareName: 'SSP' }, ghost, timing: DEFAULT_TIMING, sleep };
  return reloadAndWait(ctx);
}

const defaultLog = (g: GhostTarget) => path.join(g.master, ...DEFAULT_PASTA_LOG.split('/'));

function done(o: ReloadOutcome) {
  assert.strictEqual(o.kind, 'done');
  return o as Extract<ReloadOutcome, { kind: 'done' }>;
}

async function main() {
  console.log('reloadShiori');
  const server = await startFakeMcpServer({ reloadLog: [] });

  await test('ends on the marker; order of calls; no playback; old lines and lines after the marker are ignored', async () => {
    const g = makeGhost();
    const log = defaultLog(g);
    fs.mkdirSync(path.dirname(log), { recursive: true });
    fs.writeFileSync(log, L.error('old error before reload') + '\n');
    server.addLog('error', 'SHIORI', 'old baseware error', path.basename(g.root));
    const before = server.state.calls.length;
    const { waits, sleep } = scriptedSleep(log, { 2: [L.info()], 3: [L.done(true), L.error('after marker')] });
    const o = done(await run(server, g, sleep));
    assert.strictEqual(o.finished, true);
    assert.strictEqual(o.loaded, true);
    assert.deepStrictEqual(o.errors, []);
    assert.deepStrictEqual(o.warnings, []);
    assert.strictEqual(o.pastaLogReadable, true);
    assert.deepStrictEqual(waits, [250, 250, 250]);
    const calls = server.state.calls.slice(before).map((c) => `${c.name}:${JSON.stringify(c.arguments)}`);
    assert.deepStrictEqual(calls, [
      `get_log:${JSON.stringify({ log_type: 'error', ghost_name: g.root, max_count: 1 })}`,
      `reload:${JSON.stringify({ target: 'shiori', ghost_name: g.root })}`,
      `get_log:${JSON.stringify({ log_type: 'error', ghost_name: g.root, since_id: server.state.logs.at(-1)!.id })}`,
    ]);
    assert.strictEqual(blocksPlayback(o), false);
    assert.deepStrictEqual(formatReload(o), { lines: ['読み込みは誤り無く終わりました'] });
  });

  await test('timeout after 20 s of 250 ms polls: blocks, hint on [logging] level/filter, summary so far', async () => {
    const g = makeGhost();
    const { waits, sleep } = scriptedSleep(defaultLog(g), { 1: [L.warn('half way')] });
    const o = done(await run(server, g, sleep));
    assert.strictEqual(o.finished, false);
    assert.strictEqual(o.loaded, undefined);
    assert.strictEqual(waits.length, 80);
    assert.strictEqual(waits.reduce((a, b) => a + b, 0), 20000);
    assert.strictEqual(o.warnings.length, 1);
    assert.strictEqual(blocksPlayback(o), true);
    const f = formatReload(o);
    assert.match(f.lines[0], /読み込みの終わりを確かめられませんでした.*\[logging\] の level・filter/);
    assert.ok(f.lines.includes('誤り 0 件・警告 1 件'), f.lines.join('\n'));
    assert.ok(f.lines.some((l) => l.includes('[pasta.log]') && l.includes('half way')));
    assert.strictEqual(f.notice, '読み込みの終わりを確かめられませんでした');
  });

  await test('loaded=false: the marker line is an error and blocks playback', async () => {
    const g = makeGhost();
    const { sleep } = scriptedSleep(defaultLog(g), { 1: [L.done(false)] });
    const o = done(await run(server, g, sleep));
    assert.strictEqual(o.finished, true);
    assert.strictEqual(o.loaded, false);
    assert.strictEqual(o.errors.length, 1);
    assert.match(o.errors[0].text, /loaded=false/);
    assert.strictEqual(blocksPlayback(o), true);
    assert.strictEqual(formatReload(o).notice, '読み込みの誤りが 1 件あります');
  });

  await test('errors vs warnings from pasta.log and the baseware log (label containing warn is a warning)', async () => {
    const g = makeGhost();
    const name = path.basename(g.root);
    const { sleep } = scriptedSleep(defaultLog(g), {
      1: [L.error('e1'), L.warn('w1'), L.info()],
      2: [L.done(true)],
    });
    const orig = sleep;
    let added = false;
    const o = done(await run(server, g, async (ms) => {
      if (!added) { // baseware records appear after reload was asked
        server.addLog('error', 'SHIORI', 'dic_error', name);
        server.addLog('error', 'Warning', 'dic_warning', name);
        added = true;
      }
      await orig(ms);
    }));
    assert.deepStrictEqual(o.errors.map((i) => `${i.source}|${i.text.split(' ').at(-1)}`), ['pasta.log|e1', 'baseware|dic_error']);
    assert.deepStrictEqual(o.warnings.map((i) => `${i.source}|${i.text.split(' ').at(-1)}`), ['pasta.log|w1', 'baseware|dic_warning']);
    assert.strictEqual(blocksPlayback(o), true);
    const f = formatReload(o);
    assert.ok(f.lines.includes('誤り 2 件・警告 2 件'));
    assert.ok(f.lines.includes('  [ベースウェア] [SHIORI] dic_error'), f.lines.join('\n'));
    assert.ok(f.lines.includes('  [ベースウェア] [Warning] dic_warning'), f.lines.join('\n'));
  });

  await test('warnings only do not block playback', async () => {
    const g = makeGhost();
    const { sleep } = scriptedSleep(defaultLog(g), { 1: [L.warn(), L.done(true)] });
    const o = done(await run(server, g, sleep));
    assert.strictEqual(blocksPlayback(o), false);
    assert.strictEqual(formatReload(o).notice, undefined);
  });

  await test('pasta.log unreadable (never created): baseware-only summary with the annotation and path', async () => {
    const g = makeGhost();
    const { sleep } = scriptedSleep(defaultLog(g), {});
    const o = done(await run(server, g, sleep));
    assert.strictEqual(o.pastaLogReadable, false);
    assert.strictEqual(o.finished, false);
    const f = formatReload(o);
    assert.ok(f.lines.some((l) => l.includes('pasta.log を読めなかった') && l.includes(defaultLog(g))), f.lines.join('\n'));
  });

  await test('pasta.log created after reload is read from its head; [logging] file_path is followed', async () => {
    const g = makeGhost('[logging]\nfile_path = "profile/custom/my.log"\n');
    const log = path.join(g.master, 'profile', 'custom', 'my.log');
    const { sleep } = scriptedSleep(log, { 1: [L.done(true)] });
    const o = done(await run(server, g, sleep));
    assert.strictEqual(o.pastaLog.path, log);
    assert.strictEqual(o.finished, true);
    assert.strictEqual(o.pastaLogReadable, true);
  });

  await test('unusable file_path: the location note is in the summary', async () => {
    const g = makeGhost('[logging]\nfile_path = "../x.log"\n');
    const { sleep } = scriptedSleep(defaultLog(g), { 1: [L.done(true)] });
    const o = done(await run(server, g, sleep));
    assert.ok(o.pastaLog.note);
    assert.ok(formatReload(o).lines.includes(o.pastaLog.note!));
  });

  await test('reload refused (ghost not running): refused, blocks, notice', async () => {
    const g = makeGhost();
    server.state.runningRoots = [];
    const ctx: FlowContext = {
      io: nodeGhostIO,
      endpoint: { port: server.port, basewareName: 'SSP' }, ghost: g, timing: DEFAULT_TIMING, sleep: async () => {},
    };
    const o = await reloadAndWait(ctx);
    assert.strictEqual(o.kind, 'refused');
    assert.strictEqual(blocksPlayback(o), true);
    assert.match(formatReload(o).notice ?? '', /頼めませんでした: NG:/);
  });

  await test('contract: pasta.dll writes "SHIORI load entry completed" with loaded in windows.rs and v3.rs', async () => {
    const repo = path.resolve(__dirname, '..', '..', '..', '..');
    for (const f of ['windows.rs', 'v3.rs']) {
      const src = fs.readFileSync(path.join(repo, 'crates', 'pasta_shiori', 'src', f), 'utf8');
      assert.match(src, /info!\([^)]*\bloaded\b[^)]*"SHIORI load entry completed"\)/, f);
    }
  });

  await server.close();
  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(`\nreloadShiori: ${passed} passed, ${failed} failed\n`);
  if (failed > 0) {
    process.exit(1);
  }
}

main();
