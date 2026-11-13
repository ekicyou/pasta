// Unit tests for DumpVariables (task 5.3). Plain node against fakeMcpServer and a temp ghost; the waits are
// counted through the injected sleep, never the real clock.
//
// Requirements: 6.2 (OnPastaDumpVariables sent, the freshly written variables.toml shown),
// 6.3 (missing or not rewritten → old contents not shown, the 4.10 hint plus the OnPastaDumpVariables: lines).

import * as assert from 'assert';
import * as fs from 'fs';
import * as http from 'http';
import * as os from 'os';
import * as path from 'path';
import { startFakeMcpServer, FakeMcpServer, FakeState } from './fakeMcpServer';
import { DEFAULT_TIMING, FlowContext } from '../reloadShiori';
import { DumpOutcome, dumpVariables, formatDump } from '../dumpVariables';
import { type GhostTarget, variablesPath } from '../../../shared/ghostFiles.ts';
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

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'dumpVariables-'));
let ghostNo = 0;

function makeGhost(): GhostTarget {
  const root = path.join(tmp, `g${ghostNo++}`);
  const master = path.join(root, 'ghost', 'master');
  fs.mkdirSync(master, { recursive: true });
  return { root, master, displayName: path.basename(root), key: root.toLowerCase() };
}

/** An old variables.toml, dated well in the past so a rewrite is clearly newer. */
function writeOld(g: GhostTarget, text = '[old]\nvalue = 0\n') {
  const p = variablesPath(nodeGhostIO, g.master);
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, text);
  const past = new Date(Date.now() - 60_000);
  fs.utimesSync(p, past, past);
}

async function dump(server: FakeMcpServer, g: GhostTarget, setup: Partial<FakeState>) {
  Object.assign(server.state, { raiseEvent: { mode: 'noScript' }, raiseEventLog: [], writeVariables: true }, setup);
  server.state.runningRoots = [g.root];
  const waits: number[] = [];
  const ctx: FlowContext = {
    io: nodeGhostIO,
    endpoint: { port: server.port, basewareName: 'SSP' }, ghost: g, timing: DEFAULT_TIMING,
    sleep: async (ms) => { waits.push(ms); await new Promise((r) => setTimeout(r, 1)); },
  };
  const before = server.state.calls.length;
  const outcome = await dumpVariables(ctx);
  return { outcome, waits, calls: server.state.calls.slice(before) };
}

function is<K extends DumpOutcome['kind']>(o: DumpOutcome, kind: K): Extract<DumpOutcome, { kind: K }> {
  assert.strictEqual(o.kind, kind, JSON.stringify(o));
  return o as Extract<DumpOutcome, { kind: K }>;
}

const HINT = /pasta\.toml に \[debug\] enabled = true を書いて/;

async function main() {
  console.log('dumpVariables');
  const server = await startFakeMcpServer();

  await test('newer: sends OnPastaDumpVariables [] to the root, no reload, shows the rewritten contents (6.2・6.4)', async () => {
    const g = makeGhost();
    writeOld(g);
    const { outcome, calls } = await dump(server, g, { variablesText: '[new]\nvalue = 2\n' });
    const o = is(outcome, 'shown');
    assert.strictEqual(o.text, '[new]\nvalue = 2\n');
    assert.strictEqual(o.path, variablesPath(nodeGhostIO, g.master));
    assert.deepStrictEqual(calls.map((c) => c.name), ['raise_event']);
    assert.deepStrictEqual(calls[0].arguments, { event: 'OnPastaDumpVariables', references: [], ghost_name: g.root });
    const f = formatDump(o);
    assert.ok(f.lines.some((l) => l.includes('value = 2')), f.lines.join('\n'));
    assert.ok(!f.lines.some((l) => l.includes('value = 0')));
    assert.strictEqual(f.notice, undefined);
  });

  await test('newer: a file that did not exist before counts as new; a script or no response answer still shows it (6.2)', async () => {
    for (const mode of ['noScript', 'script', 'drop'] as const) {
      const { outcome } = await dump(server, makeGhost(), { raiseEvent: { mode, script: 'x' }, variablesText: '[fake]\nvalue = 1\n' });
      assert.strictEqual(is(outcome, 'shown').text, '[fake]\nvalue = 1\n', mode);
    }
  });

  await test('stays old: old contents not shown; hint plus only the OnPastaDumpVariables: lines appended to pasta.log (6.3)', async () => {
    const g = makeGhost();
    writeOld(g, '[old]\nsecret_old = 0\n');
    const logPath = path.join(g.master, 'profile', 'pasta', 'logs', 'pasta.log');
    fs.mkdirSync(path.dirname(logPath), { recursive: true });
    fs.writeFileSync(logPath, '2026-11-03T00:00:00Z  WARN pasta: OnPastaDumpVariables: earlier, not ours\n');
    const { outcome, waits } = await dump(server, g, {
      writeVariables: false,
      raiseEventLog: [
        '2026-11-03T00:00:01Z  WARN pasta_lua::debug: OnPastaDumpVariables: could not write variables.toml',
        '2026-11-03T00:00:01Z  INFO pasta: unrelated line',
      ],
    });
    const o = is(outcome, 'stale');
    assert.deepStrictEqual(o.reasons, ['2026-11-03T00:00:01Z  WARN pasta_lua::debug: OnPastaDumpVariables: could not write variables.toml']);
    assert.strictEqual(waits.length, Math.ceil(DEFAULT_TIMING.dumpWaitMs / DEFAULT_TIMING.pollIntervalMs), 'waits up to the 5 s limit');
    const f = formatDump(o);
    assert.ok(!f.lines.some((l) => l.includes('secret_old')), f.lines.join('\n'));
    assert.ok(f.lines.some((l) => HINT.test(l)), f.lines.join('\n'));
    assert.ok(f.lines.some((l) => l.includes('could not write variables.toml')));
    assert.match(f.notice ?? '', HINT);
  });

  await test('missing: never written (debug disabled, 204) → stale with the hint (6.3)', async () => {
    const g = makeGhost();
    const o = is((await dump(server, g, { writeVariables: false })).outcome, 'stale');
    assert.deepStrictEqual(o.reasons, []);
    assert.strictEqual(o.path, variablesPath(nodeGhostIO, g.master));
    assert.match(formatDump(o).notice ?? '', HINT);
  });

  await test('failed: NG (ghost not running), unreachable and bad shape are failures, not stale', async () => {
    const g = makeGhost();
    server.state.runningRoots = [];
    const ctx: FlowContext = { io: nodeGhostIO, endpoint: { port: server.port, basewareName: 'SSP' }, ghost: g, timing: DEFAULT_TIMING, sleep: async () => {} };
    const ng = is(await dumpVariables(ctx), 'failed');
    assert.strictEqual(ng.result.kind, 'ng');
    assert.match(formatDump(ng).notice ?? '', /NG:/);

    const closed = await startFakeMcpServer();
    const port = closed.port;
    await closed.close();
    const un = is(await dumpVariables({ ...ctx, endpoint: { port, basewareName: 'SSP' } }), 'failed');
    assert.strictEqual(un.result.kind, 'unreachable');

    const http500 = http.createServer((_req, res) => res.writeHead(500).end('boom'));
    await new Promise<void>((r) => http500.listen(0, '127.0.0.1', r));
    const bad = is(await dumpVariables({ ...ctx, endpoint: { port: (http500.address() as any).port, basewareName: 'SSP' } }), 'failed');
    assert.strictEqual(bad.result.kind, 'badShape');
    await new Promise((r) => http500.close(r));
  });

  await server.close();
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
