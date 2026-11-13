// 共有の開発の操作（DevSession）を偽の入出力と偽の DevHost で確かめる（要件 1.6・3.3〜3.5・4.1・4.3・5.1・13.1）。
// ベースウェアはポートごとの名前の表で答える。本物の時計は使わない。

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createFakeGhostIO, type PostRequest } from './fakeGhostIO.ts';
import type { PostOutcome } from './ghostIO.ts';
import type { GhostTarget } from './ghostFiles.ts';
import type { Endpoint } from './mcpClient.ts';
import { DEFAULT_TIMING } from './reloadShiori.ts';
import {
  BUSY, NOT_FOUND, SAVE_FAILED, DevSession, pickPlaceHolder,
  type DevCommand, type DevHost, type PickItem, type Report,
} from './devSession.ts';

const ghost: GhostTarget = { root: 'C:\\g', master: 'C:\\g\\ghost\\master', displayName: 'g', key: 'c:\\g' };
const other: GhostTarget = { root: 'C:\\h', master: 'C:\\h\\ghost\\master', displayName: 'h', key: 'c:\\h' };
const PORTS = [9801, 9821, 9802];

const reply = (result: unknown): PostOutcome => ({
  kind: 'response', status: 200, body: JSON.stringify({ jsonrpc: '2.0', id: 1, result }),
});

function setup() {
  const io = createFakeGhostIO();
  /** ポート → そこで動いているベースウェアの名前（無ければ届かない） */
  const names: Record<number, string | undefined> = {};
  const serve = (req: PostRequest): PostOutcome => {
    const name = names[req.port];
    if (name === undefined) return { kind: 'unreachable', detail: 'ECONNREFUSED' };
    const { method } = JSON.parse(req.body);
    return method === 'initialize' ? reply({}) : reply({ content: [{ type: 'text', text: name }], isError: false });
  };
  io.queuePost(...Array.from({ length: 200 }, () => serve));

  const written: string[][] = [];
  const notices: { level: string; text: string }[] = [];
  const picks: { items: readonly PickItem[]; placeHolder: string }[] = [];
  const saves: GhostTarget[] = [];
  const state = {
    pickIndex: 0 as number | undefined, // undefined＝選ばずに閉じる
    unsaved: [] as string[],
  };
  const host: DevHost = {
    io, ports: PORTS, timing: DEFAULT_TIMING, sleep: async () => {},
    write: (lines) => { written.push([...lines]); },
    notify: (level, text) => { notices.push({ level, text }); },
    pick: async (_g, items, placeHolder) => {
      picks.push({ items, placeHolder });
      return state.pickIndex === undefined ? undefined : items[state.pickIndex].endpoint;
    },
    progress: (_t, work) => work(),
    save: async (g) => { saves.push(g); return state.unsaved; },
  };
  const session = new DevSession(host);
  /** post の数を数えながら */
  const postsOf = (method: string) => io.posts.filter((p) => JSON.parse(p.body).method === method);
  return { io, names, written, notices, picks, saves, state, session, postsOf };
}

/** run を呼んだ相手を数える操作 */
function command(report: Report = { lines: ['終わり'] }, saves = false) {
  const runs: Endpoint[] = [];
  const cmd: DevCommand = {
    title: '▶ シーンを実行', saves,
    run: async (ctx) => { runs.push(ctx.endpoint); return report; },
  };
  return { cmd, runs };
}

const HEAD = /^\[\d\d:\d\d:\d\d\] ▶ シーンを実行 — g（C:\\g）→ SSP 127\.0\.0\.1:9821$/;

test('相手が 1 つなら選ばせずに送り、見出しの行は時刻・操作・ゴースト・相手の形', async () => {
  const s = setup();
  s.names[9821] = 'SSP';
  const { cmd, runs } = command();
  await s.session.execute(cmd, ghost);
  assert.deepEqual(runs, [{ port: 9821, basewareName: 'SSP' }]);
  assert.equal(s.picks.length, 0);
  assert.match(s.written[0][0], HEAD);
  assert.deepEqual(s.written[1], ['終わり']);
  assert.deepEqual(s.notices, []); // 成功は知らせない
  assert.equal(s.session.running, undefined);
});

test('覚えた相手は get_property 1 回で確かめ、探し直さない', async () => {
  const s = setup();
  s.names[9821] = 'SSP';
  const { cmd, runs } = command();
  await s.session.execute(cmd, ghost);
  const inits = s.postsOf('initialize').length;
  const before = s.io.posts.length;
  await s.session.execute(cmd, ghost);
  assert.equal(s.io.posts.length - before, 1);
  assert.equal(s.postsOf('initialize').length, inits);
  assert.equal(s.io.posts.at(-1)!.port, 9821);
  assert.deepEqual(runs.map((e) => e.port), [9821, 9821]);
});

test('覚えた相手の名前が変わったら探し直す', async () => {
  const s = setup();
  s.names[9821] = 'SSP';
  const { cmd, runs } = command();
  await s.session.execute(cmd, ghost);
  s.names[9821] = 'areka'; // 同じポートで別のベースウェア
  const before = s.io.posts.length;
  await s.session.execute(cmd, ghost);
  // 確かめ 1 回のあと、全部のポートへ initialize
  assert.equal(s.io.posts.slice(before).filter((p) => JSON.parse(p.body).method === 'initialize').length, PORTS.length);
  assert.deepEqual(runs.at(-1), { port: 9821, basewareName: 'areka' });
});

test('2 つ以上なら選ばせ、選んだ相手を覚える', async () => {
  const s = setup();
  s.names[9801] = 'SSP';
  s.names[9802] = 'areka';
  s.state.pickIndex = 1;
  const { cmd, runs } = command();
  await s.session.execute(cmd, ghost);
  assert.equal(s.picks.length, 1);
  assert.deepEqual(s.picks[0].items.map((i) => i.label), ['SSP（127.0.0.1:9801）', 'areka（127.0.0.1:9802）']);
  assert.equal(s.picks[0].placeHolder, pickPlaceHolder(ghost));
  assert.deepEqual(runs, [{ port: 9802, basewareName: 'areka' }]);
  // 次は選ばせない
  await s.session.execute(cmd, ghost);
  assert.equal(s.picks.length, 1);
  assert.equal(runs.at(-1)!.port, 9802);
});

test('選ぶ窓を閉じたら流れを走らせず、覚えもしない', async () => {
  const s = setup();
  s.names[9801] = 'SSP';
  s.names[9802] = 'areka';
  s.state.pickIndex = undefined;
  const { cmd, runs } = command({ lines: [] }, true);
  await s.session.execute(cmd, ghost);
  assert.equal(runs.length, 0);
  assert.equal(s.saves.length, 0);
  assert.deepEqual(s.written, []);
  assert.deepEqual(s.notices, []);
  await s.session.execute(cmd, ghost);
  assert.equal(s.picks.length, 2);
});

test('相手が見つからなければ走らせず、警告で知らせる', async () => {
  const s = setup();
  const { cmd, runs } = command();
  await s.session.execute(cmd, ghost);
  assert.equal(runs.length, 0);
  assert.ok(s.written[0].some((l) => l.startsWith(NOT_FOUND)));
  assert.equal(s.notices.length, 1);
  assert.equal(s.notices[0].level, 'warning');
  assert.ok(s.notices[0].text.startsWith(NOT_FOUND));
});

test('同じゴーストの 2 本目は止め、別のゴーストは止めない', async () => {
  const s = setup();
  s.names[9801] = 'SSP';
  let release!: () => void;
  const gate = new Promise<void>((r) => { release = r; });
  let started!: () => void;
  const running = new Promise<void>((r) => { started = r; });
  const runs: string[] = [];
  const slow: DevCommand = {
    title: '▶ シーンを実行', saves: false,
    run: async (ctx) => { runs.push(ctx.ghost.key); started(); await gate; return { lines: [] }; },
  };
  const first = s.session.execute(slow, ghost);
  await running;
  assert.equal(s.session.running, ghost);
  // 止めは await の前に出る。止まらなくても試験が待ち続けないよう、2 本目は待たずに見る
  const second = s.session.execute(slow, ghost);
  assert.match(s.written.at(-1)?.[0] ?? '', /^\[\d\d:\d\d:\d\d\] ▶ シーンを実行 — このゴーストへのコマンドが進んでいます（g）$/);
  assert.deepEqual(s.notices, [{ level: 'warning', text: BUSY }]);
  const quick = command();
  await s.session.execute(quick.cmd, other);
  assert.equal(quick.runs.length, 1);
  release();
  await Promise.all([first, second]);
  assert.deepEqual(runs, ['c:\\g']);
  assert.equal(s.session.running, undefined);
  await s.session.execute(quick.cmd, ghost); // 終われば止めない
  assert.equal(quick.runs.length, 2);
});

test('保存できないファイルがあれば流れを走らせない', async () => {
  const s = setup();
  s.names[9801] = 'SSP';
  s.state.unsaved = ['C:\\g\\ghost\\master\\dic\\a.pasta'];
  const { cmd, runs } = command({ lines: [] }, true);
  await s.session.execute(cmd, ghost);
  assert.deepEqual(s.saves, [ghost]);
  assert.equal(runs.length, 0);
  assert.deepEqual(s.written.at(-1), [SAVE_FAILED, '  C:\\g\\ghost\\master\\dic\\a.pasta']);
  assert.deepEqual(s.notices, [{ level: 'warning', text: `${SAVE_FAILED}: C:\\g\\ghost\\master\\dic\\a.pasta` }]);
  assert.equal(s.session.running, undefined);
});

test('保存しない操作は save を呼ばない・保存できれば走らせる', async () => {
  const s = setup();
  s.names[9801] = 'SSP';
  const plain = command();
  await s.session.execute(plain.cmd, ghost);
  assert.equal(s.saves.length, 0);
  const saving = command({ lines: [] }, true);
  await s.session.execute(saving.cmd, ghost);
  assert.equal(s.saves.length, 1);
  assert.equal(saving.runs.length, 1);
});

test('知らせは ERROR_KINDS の失敗なら誤り、それ以外は警告', async () => {
  const cases: [Report['failure'], 'error' | 'warning'][] = [
    [{ kind: 'unreachable', detail: 'x' }, 'error'],
    [{ kind: 'badShape', detail: 'x' }, 'error'],
    [{ kind: 'rpcError', code: -1, message: 'x' }, 'error'],
    [{ kind: 'ng', text: 'NG' }, 'error'],
    [{ kind: 'noResponse', detail: 'x' }, 'warning'],
    [undefined, 'warning'],
  ];
  for (const [failure, level] of cases) {
    const s = setup();
    s.names[9801] = 'SSP';
    const { cmd } = command({ lines: ['行'], notice: '止まりました', failure });
    await s.session.execute(cmd, ghost);
    assert.deepEqual(s.written.at(-1), ['行']);
    assert.deepEqual(s.notices, [{ level, text: '止まりました' }], `failure=${failure?.kind}`);
  }
});
