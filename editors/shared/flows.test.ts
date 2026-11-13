// 共有の流れ（読み込み直し・再生・書き出しの待ち・書き換えの頼み）を偽の入出力で確かめる（要件 10.2・13.1）。
// 本物の時計は使わない: sleep は回数を数え、決めた回でファイルを足すだけ。

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createFakeGhostIO, type FakeGhostIO, type PostStep } from './fakeGhostIO.ts';
import type { PostOutcome } from './ghostIO.ts';
import { DEBUG_HINT, type GhostTarget } from './ghostFiles.ts';
import { type FlowContext, type ReloadOutcome, type Timing, blocksPlayback, formatReload, reloadAndWait } from './reloadShiori.ts';
import { formatScene, runScene } from './runSceneAtCursor.ts';
import { dumpVariables, formatDump, formatSet, setSaveVariable } from './dumpVariables.ts';

const ghost: GhostTarget = { root: 'C:\\g', master: 'C:\\g\\ghost\\master', displayName: 'g', key: 'c:\\g' };
const LOG = 'C:\\g\\ghost\\master\\profile\\pasta\\logs\\pasta.log';
const VARS = 'C:\\g\\ghost\\master\\profile\\pasta\\debug\\variables.toml';
// 待ちの上限は sleep 3 回（読み込み・書き出し）・2 回（台本の記録の引き直し）
const timing: Timing = { loadTimeoutMs: 3, pollIntervalMs: 1, scriptLogWaitMs: 2, dumpWaitMs: 3, callTimeoutMs: 100 };

const L = {
  done: (loaded: boolean) => `2026-11-13T00:00:00Z  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=${loaded}`,
  error: (t: string) => `2026-11-13T00:00:00Z ERROR pasta_lua::loader: ${t}`,
  warn: (t: string) => `2026-11-13T00:00:00Z  WARN pasta_lua::loader: ${t}`,
};
const lines = (...ls: string[]) => ls.map((l) => l + '\n').join('');

/** tools/call の答え（content[0].text） */
const ok = (text: string, isError = false): PostOutcome => ({
  kind: 'response', status: 200, body: JSON.stringify({ jsonrpc: '2.0', id: 1, result: { content: [{ type: 'text', text }], isError } }),
});
/** 答えの前にファイルを触る */
const okAfter = (text: string, act: () => void): PostStep => () => {
  act();
  return ok(text);
};
const NO_LOG = '(no log entries)';
const NO_SCRIPT = 'OK:the ghost returned no script.';
const entry = (id: number, label: string, text: string) => `#${id} 2026/11/13 00:00:00 [${label}] g : ${text}`;

function setup(onSleep: Record<number, (io: FakeGhostIO) => void> = {}) {
  const io = createFakeGhostIO();
  const waits: number[] = [];
  const ctx: FlowContext = {
    io, endpoint: { port: 9801, basewareName: 'SSP' }, ghost, timing,
    sleep: async (ms) => {
      waits.push(ms);
      onSleep[waits.length]?.(io);
    },
  };
  /** n 番目の頼みの tools/call の名前と引数 */
  const call = (n: number) => {
    const p = JSON.parse(io.posts[n].body).params;
    return { name: p.name as string, args: p.arguments };
  };
  return { io, ctx, waits, call };
}

function done(o: ReloadOutcome) {
  assert.equal(o.kind, 'done');
  return o as Extract<ReloadOutcome, { kind: 'done' }>;
}

/** 読み込み直しが誤り無く終わる筋書き（get_log・reload・get_log の 3 回） */
function queueCleanReload(io: FakeGhostIO) {
  io.queuePost(ok(NO_LOG), okAfter('OK', () => io.appendFile(LOG, lines(L.done(true)))), ok(NO_LOG));
}

// ---- 読み込み直し ----

test('読み込み直し: 印の行で終わり、前からある行と印より後の行は数えない', async () => {
  const { io, ctx, waits, call } = setup();
  io.setFile(LOG, lines(L.error('前の回'), L.done(false)));
  io.queuePost(
    ok(entry(4, 'error', '前')),
    okAfter('OK', () => io.appendFile(LOG, lines(L.warn('w1'), L.done(true), L.error('後の行')))),
    ok(NO_LOG),
  );
  const o = done(await reloadAndWait(ctx));
  assert.equal(o.finished, true);
  assert.equal(o.loaded, true);
  assert.deepEqual(o.errors, []);
  assert.deepEqual(o.warnings, [{ source: 'pasta.log', text: L.warn('w1') }]);
  assert.equal(o.pastaLogReadable, true);
  assert.deepEqual(waits, []);
  assert.equal(blocksPlayback(o), false);
  assert.deepEqual(call(1), { name: 'reload', args: { target: 'shiori', ghost_name: 'C:\\g' } });
  assert.equal(call(2).args.since_id, 4, '前の番号より後の誤りだけを引く');
});

test('読み込み直し: loaded=false の印は誤りに数えて止める', async () => {
  const { io, ctx } = setup({ 1: (io) => io.appendFile(LOG, lines(L.done(false))) });
  io.queuePost(ok(NO_LOG), ok('OK'), ok(NO_LOG));
  const o = done(await reloadAndWait(ctx));
  assert.equal(o.finished, true);
  assert.equal(o.loaded, false);
  assert.deepEqual(o.errors.map((e) => e.text), [L.done(false)]);
  assert.equal(blocksPlayback(o), true);
  assert.equal(formatReload(o).notice, '読み込みの誤りが 1 件あります');
});

test('読み込み直し: 印が出なければ sleep の上限で時間切れ', async () => {
  const { io, ctx, waits } = setup();
  io.setFile(LOG, '');
  io.queuePost(ok(NO_LOG), ok('OK'), ok(NO_LOG));
  const o = done(await reloadAndWait(ctx));
  assert.equal(o.finished, false);
  assert.deepEqual(waits, [1, 1, 1]);
  assert.equal(blocksPlayback(o), true);
  assert.equal(formatReload(o).notice, '読み込みの終わりを確かめられませんでした');
});

test('読み込み直し: ベースウェアの誤りの記録を誤りと警告に分けて足す・読めなければ誤りに数える', async () => {
  const { io, ctx } = setup();
  io.queuePost(
    ok(NO_LOG),
    okAfter('OK', () => io.appendFile(LOG, lines(L.done(true)))),
    ok([entry(1, 'SHIORI error', 'boom'), entry(2, 'warning', 'careful')].join('\n')),
  );
  const o = done(await reloadAndWait(ctx));
  assert.deepEqual(o.errors, [{ source: 'baseware', text: '[SHIORI error] boom' }]);
  assert.deepEqual(o.warnings, [{ source: 'baseware', text: '[warning] careful' }]);
  assert.ok(formatReload(o).lines.includes('  [ベースウェア] [SHIORI error] boom'));

  const b = setup();
  b.io.queuePost(ok(NO_LOG), okAfter('OK', () => b.io.appendFile(LOG, lines(L.done(true)))), { kind: 'unreachable', detail: 'ECONNREFUSED' });
  const o2 = done(await reloadAndWait(b.ctx));
  assert.deepEqual(o2.errors, [{ source: 'baseware', text: '誤りの記録を読めませんでした: ECONNREFUSED' }]);
});

test('読み込み直し: pasta.log が読めなければ、ベースウェアの記録だけで要約したと言う', async () => {
  const { io, ctx } = setup();
  io.queuePost(ok(NO_LOG), ok('OK'), ok(entry(1, 'error', 'boom')));
  const o = done(await reloadAndWait(ctx));
  assert.equal(o.pastaLogReadable, false);
  assert.equal(o.finished, false);
  assert.ok(formatReload(o).lines.includes(`pasta.log を読めなかったため、ベースウェアの誤りの記録だけで要約しました（${LOG}）`));
});

test('読み込み直し: BOM 付きの pasta.toml の 1 行目の [logging] を読む', async () => {
  const { io, ctx } = setup();
  const custom = 'C:\\g\\ghost\\master\\profile\\mine\\my.log';
  io.setFile('C:\\g\\ghost\\master\\pasta.toml', '\uFEFF[logging]\nfile_path = "profile/mine/my.log"\n');
  io.queuePost(ok(NO_LOG), okAfter('OK', () => io.appendFile(custom, lines(L.done(true)))), ok(NO_LOG));
  const o = done(await reloadAndWait(ctx));
  assert.equal(o.pastaLog.path, custom);
  assert.equal(o.pastaLog.note, undefined);
  assert.equal(o.finished, true);
});

// ---- 再生 ----

test('再生: 台本を返す・Reference2 が無ければ references は 2 つ', async () => {
  const { io, ctx, call } = setup();
  queueCleanReload(io);
  io.queuePost(ok(entry(7, 'Ghost:OnTalk', 'x')), ok('OK:\nScript: \\0hello\\e'));
  const o = await runScene(ctx, 'C:\\g\\ghost\\master\\dic\\a.pasta', 12);
  assert.equal(o.kind, 'played');
  assert.equal(o.kind === 'played' && o.script, '\\0hello\\e');
  assert.equal(o.kind === 'played' && o.viaLog, false);
  assert.deepEqual(call(3).args, { log_type: 'script', ghost_name: 'C:\\g', max_count: 1 });
  assert.deepEqual(call(4), {
    name: 'raise_event',
    args: { event: 'OnPastaSceneAt', references: ['C:\\g\\ghost\\master\\dic\\a.pasta', '12'], ghost_name: 'C:\\g' },
  });
  assert.ok(formatScene(o).lines.includes('台本:'));
});

test('再生: ローカル変数を付けると references は 3 つ', async () => {
  const { io, ctx, call } = setup();
  queueCleanReload(io);
  io.queuePost(ok(NO_LOG), ok('Script: \\0x\\e'));
  await runScene(ctx, 'C:\\a.pasta', 3, '{ n = 1 }');
  assert.deepEqual(call(4).args.references, ['C:\\a.pasta', '3', '{ n = 1 }']);
});

test('再生: 204 は案内と、この再生で増えた OnPastaSceneAt: の警告', async () => {
  const { io, ctx } = setup();
  queueCleanReload(io);
  const reason = L.warn('OnPastaSceneAt: シーンが見つかりません');
  io.queuePost(ok(NO_LOG), okAfter(NO_SCRIPT, () => io.appendFile(LOG, lines(reason, L.warn('ほかの警告')))));
  const o = await runScene(ctx, 'C:\\a.pasta', 3);
  assert.equal(o.kind, 'noScript');
  assert.deepEqual(o.kind === 'noScript' && o.reasons, [reason]);
  const f = formatScene(o);
  assert.equal(f.notice, DEBUG_HINT);
  assert.ok(f.lines.some((l) => l.includes('台本を返しませんでした（204）')));
  assert.ok(f.lines.includes(`  ${reason}`));
});

test('再生: 応答が無ければ、送る前の番号より後の台本の記録を引き直して見つける', async () => {
  const { io, ctx, call, waits } = setup();
  queueCleanReload(io);
  io.queuePost(
    ok(entry(7, 'Ghost:OnTalk', 'x')),
    { kind: 'noResponse', detail: 'timeout' },
    ok(NO_LOG),
    ok([entry(8, 'Ghost:OnPastaSceneAt', '\\0old'), entry(9, 'Ghost:OnPastaSceneAt', '\\0new'), entry(10, 'Ghost:OnTalk', 'y')].join('\n')),
  );
  const o = await runScene(ctx, 'C:\\a.pasta', 3);
  assert.equal(o.kind, 'played');
  assert.equal(o.kind === 'played' && o.script, '\\0new');
  assert.equal(o.kind === 'played' && o.viaLog, true);
  assert.equal(call(5).args.since_id, 7);
  assert.equal(io.posts.length, 7, 'raise_event は送り直さない');
  assert.deepEqual(waits, [1]);
  assert.ok(formatScene(o).lines.includes('台本（応答が無かったため、再生の記録から）:'));
});

test('再生: 応答が無く、記録にも見つからなければ確かめられなかったと言う', async () => {
  const { io, ctx, waits } = setup();
  queueCleanReload(io);
  io.queuePost(ok(NO_LOG), { kind: 'noResponse', detail: 'timeout' }, ok(NO_LOG), { kind: 'unreachable', detail: 'gone' }, ok(NO_LOG));
  const o = await runScene(ctx, 'C:\\a.pasta', 3);
  assert.equal(o.kind, 'unconfirmed');
  assert.equal(o.kind === 'unconfirmed' && o.detail, 'gone');
  assert.deepEqual(waits, [1, 1]);
  assert.equal(formatScene(o).notice, '台本を確かめられませんでした');
});

// ---- 書き出しの待ち ----

test('書き出しの待ち: 更新時刻が進むまで待ってから読む', async () => {
  const { io, ctx, waits, call } = setup({ 2: (io) => io.setFile(VARS, 'new = 1\n') });
  io.setFile(VARS, 'old = 1\n');
  io.queuePost(ok(NO_SCRIPT));
  const o = await dumpVariables(ctx);
  assert.equal(o.kind, 'shown');
  assert.equal(o.kind === 'shown' && o.text, 'new = 1\n');
  assert.deepEqual(waits, [1, 1]);
  assert.deepEqual(call(0).args, { event: 'OnPastaDumpVariables', references: [], ghost_name: 'C:\\g' });
});

test('書き出しの待ち: 更新時刻が進まなければ古いとして、OnPastaDumpVariables: の行を添える', async () => {
  const { io, ctx, waits } = setup();
  io.setFile(VARS, 'old = 1\n');
  const reason = L.warn('OnPastaDumpVariables: デバッグが無効です');
  io.queuePost(okAfter(NO_SCRIPT, () => io.appendFile(LOG, lines(reason))));
  const o = await dumpVariables(ctx);
  assert.equal(o.kind, 'stale');
  assert.deepEqual(o.kind === 'stale' && o.reasons, [reason]);
  assert.deepEqual(waits, [1, 1, 1]);
  const f = formatDump(o);
  assert.ok(!f.lines.some((l) => l.includes('old = 1')), '古い中身は出さない');
  assert.ok(f.lines.includes(`  ${reason}`));
  assert.equal(f.notice, DEBUG_HINT);
});

// ---- 書き換えの頼み ----

test('書き換えの頼み: references は名前・値・型、204 は成功', async () => {
  const { io, ctx, call } = setup();
  io.queuePost(ok(NO_SCRIPT));
  const change = { type: 'number', value: '3' } as const;
  const o = await setSaveVariable(ctx, '回数', change);
  assert.deepEqual(o, { kind: 'set' });
  assert.deepEqual(call(0).args, { event: 'OnPastaSetSaveVariable', references: ['回数', '3', 'number'], ghost_name: 'C:\\g' });
  assert.deepEqual(formatSet('回数', change, o), { lines: ['＄＊回数 を 3（number）にしました'] });
});

test('書き換えの頼み: 削除は空の値と delete', async () => {
  const { io, ctx, call } = setup();
  io.queuePost(ok(NO_SCRIPT));
  const o = await setSaveVariable(ctx, '回数', { type: 'delete' });
  assert.deepEqual(o, { kind: 'set' });
  assert.deepEqual(call(0).args.references, ['回数', '', 'delete']);
});

test('書き換えの頼み: 表記が返ればゴーストの断り', async () => {
  const { io, ctx } = setup();
  io.queuePost(ok('Script: 【数にできない値：abc】'));
  const change = { type: 'number', value: 'abc' } as const;
  const o = await setSaveVariable(ctx, '回数', change);
  assert.deepEqual(o, { kind: 'refusedByGhost', script: '【数にできない値：abc】' });
  assert.equal(formatSet('回数', change, o).notice, '保存変数を書き換えられませんでした');
});
