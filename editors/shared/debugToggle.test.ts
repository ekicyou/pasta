// トグルの流れ（書く → リロード → 状態）と結果の文を偽の入出力で確かめる。

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createFakeGhostIO, type FakeGhostIO, type PostStep } from './fakeGhostIO.ts';
import type { PostOutcome } from './ghostIO.ts';
import { DEBUG_DISABLED_LINE, DEBUG_ENABLED_LINE, type GhostTarget } from './ghostFiles.ts';
import type { FlowContext, ReloadOutcome, Timing } from './reloadShiori.ts';
import {
  DEBUG_SAVE_NAME, DISABLE_DEBUG, ENABLE_DEBUG, debugStateOf, formatToggle, toggleDebug,
} from './debugToggle.ts';

const ghost: GhostTarget = { root: 'C:\\g', master: 'C:\\g\\ghost\\master', displayName: 'g', key: 'c:\\g' };
const LOG = 'C:\\g\\ghost\\master\\profile\\pasta\\logs\\pasta.log';
const timing: Timing = { loadTimeoutMs: 3, pollIntervalMs: 1, scriptLogWaitMs: 2, dumpWaitMs: 3, callTimeoutMs: 100 };

const L = {
  done: (loaded: boolean) => `2026-11-21T00:00:00Z  INFO pasta::windows: SHIORI load entry completed entry="loadu" loaded=${loaded}`,
  debug: (on: boolean) => `2026-11-21T00:00:00Z  INFO pasta::debug_state: ${on ? DEBUG_ENABLED_LINE : DEBUG_DISABLED_LINE}`,
  warn: (t: string) => `2026-11-21T00:00:00Z  WARN pasta_lua::loader: ${t}`,
};
const lines = (...ls: string[]) => ls.map((l) => l + '\n').join('');
const ok = (text: string, isError = false): PostOutcome => ({
  kind: 'response', status: 200, body: JSON.stringify({ jsonrpc: '2.0', id: 1, result: { content: [{ type: 'text', text }], isError } }),
});
const okAfter = (text: string, act: () => void): PostStep => () => {
  act();
  return ok(text);
};
const NO_LOG = '(no log entries)';
const NO_SCRIPT = 'OK:the ghost returned no script.';

const ENABLED = 'デバッグは有効です';
const DISABLED = 'デバッグは無効です';
const UNCONFIRMED = 'デバッグの有効・無効を確かめられませんでした';
const NO_LINE = 'デバッグの有効・無効を確かめられませんでした（pasta.log に有効・無効の行がありませんでした）';
const CANNOT_ENABLE = 'デバッグを有効にできませんでした。環境変数 PASTA_DEBUG が設定されていると、保存変数より先に効きます。ほかの理由はマニュアルの「うまく動かないとき」を見てください';

function setup() {
  const io = createFakeGhostIO();
  const ctx: FlowContext = { io, endpoint: { port: 9801, basewareName: 'SSP' }, ghost, timing, sleep: async () => {} };
  const call = (n: number) => {
    const p = JSON.parse(io.posts[n].body).params;
    return { name: p.name as string, args: p.arguments };
  };
  return { io, ctx, call };
}

/** 書き込みの 204 と、読み込み直し（get_log・reload で pasta.log に足す・get_log） */
function queueSetAndReload(io: FakeGhostIO, logLines: string[]) {
  io.queuePost(ok(NO_SCRIPT), ok(NO_LOG), okAfter('OK', () => io.appendFile(LOG, lines(...logLines))), ok(NO_LOG));
}

test('有効にする: デバッグ を boolean の true で書き、リロードして有効の行で有効と言う', async () => {
  const { io, ctx, call } = setup();
  queueSetAndReload(io, [L.debug(true), L.done(true)]);
  const o = await toggleDebug(ctx, true);
  assert.deepEqual(call(0).args, { event: 'OnPastaSetSaveVariable', references: ['デバッグ', 'true', 'boolean'], ghost_name: 'C:\\g' });
  assert.equal(call(2).name, 'reload');
  assert.equal(io.posts.length, 4);
  assert.equal(o.kind, 'reloaded');
  assert.equal(o.kind === 'reloaded' && o.state, 'enabled');
  const r = formatToggle(true, o);
  assert.deepEqual(r, { lines: ['＄＊デバッグ を true（boolean）にしました', '読み込みは誤り無く終わりました', ENABLED] });
});

test('無効にする: デバッグ を false で書き、無効の行で無効と言う', async () => {
  const { io, ctx, call } = setup();
  queueSetAndReload(io, [L.debug(false), L.done(true)]);
  const o = await toggleDebug(ctx, false);
  assert.deepEqual(call(0).args.references, ['デバッグ', 'false', 'boolean']);
  const r = formatToggle(false, o);
  assert.deepEqual(r.lines, ['＄＊デバッグ を false（boolean）にしました', '読み込みは誤り無く終わりました', DISABLED]);
  assert.equal(r.notice, undefined);
});

test('書くのは デバッグ だけで、debug は書きも消しもしない', async () => {
  const { io, ctx } = setup();
  queueSetAndReload(io, [L.debug(true), L.done(true)]);
  await toggleDebug(ctx, true);
  const sets = io.posts.map((p) => JSON.parse(p.body).params.arguments).filter((a) => a.event === 'OnPastaSetSaveVariable');
  assert.deepEqual(sets.map((a) => a.references[0]), [DEBUG_SAVE_NAME]);
  assert.equal(DEBUG_SAVE_NAME, 'デバッグ');
  assert.equal(ENABLE_DEBUG, 'SHIORI デバッグを有効にする');
  assert.equal(DISABLE_DEBUG, 'SHIORI デバッグを無効にする');
});

test('書き込みを断られた・応答が無い・頼めない: リロードせず formatSet の文', async () => {
  const cases: { step: PostOutcome; line: string; notice: string; failure: boolean }[] = [
    { step: ok('Script: 【断り】'), line: 'ゴーストが断りました: 【断り】', notice: '保存変数を書き換えられませんでした', failure: false },
    { step: { kind: 'noResponse', detail: 'timeout' }, line: '応答が無かったため、書き換えを確かめられませんでした: timeout', notice: '応答が無かったため、書き換えを確かめられませんでした: timeout', failure: false },
    { step: { kind: 'unreachable', detail: 'ECONNREFUSED' }, line: '保存変数の書き換えを頼めませんでした: ECONNREFUSED', notice: '保存変数の書き換えを頼めませんでした: ECONNREFUSED', failure: true },
  ];
  for (const c of cases) {
    const { io, ctx } = setup();
    io.queuePost(c.step);
    const o = await toggleDebug(ctx, true);
    assert.equal(io.posts.length, 1, 'reload を送らない');
    assert.equal(o.kind, 'notWritten');
    const r = formatToggle(true, o);
    assert.deepEqual(r.lines, [c.line]);
    assert.equal(r.notice, c.notice);
    assert.equal(r.failure !== undefined, c.failure);
  }
});

test('リロードを頼めない: 要約と未確認の行・失敗は reload の結果', async () => {
  const { io, ctx } = setup();
  io.queuePost(ok(NO_SCRIPT), ok(NO_LOG), { kind: 'unreachable', detail: 'gone' });
  const o = await toggleDebug(ctx, true);
  assert.equal(o.kind === 'reloaded' && o.state, 'unconfirmed');
  const r = formatToggle(true, o);
  assert.deepEqual(r.lines, ['＄＊デバッグ を true（boolean）にしました', 'SHIORI の読み込み直しを頼めませんでした: gone', UNCONFIRMED]);
  assert.equal(r.notice, 'SHIORI の読み込み直しを頼めませんでした: gone');
  assert.deepEqual(r.failure, { kind: 'unreachable', detail: 'gone' });
});

test('時間切れ: 要約と未確認の行', async () => {
  const { io, ctx } = setup();
  io.setFile(LOG, '');
  io.queuePost(ok(NO_SCRIPT), ok(NO_LOG), ok('OK'), ok(NO_LOG));
  const r = formatToggle(true, await toggleDebug(ctx, true));
  assert.ok(r.lines[1].startsWith('読み込みの終わりを確かめられませんでした'));
  assert.equal(r.lines.at(-1), UNCONFIRMED);
  assert.equal(r.notice, '読み込みの終わりを確かめられませんでした');
});

test('loaded=false: 有効の行があっても未確認', async () => {
  const { io, ctx } = setup();
  queueSetAndReload(io, [L.debug(true), L.done(false)]);
  const r = formatToggle(true, await toggleDebug(ctx, true));
  assert.equal(r.lines.at(-1), UNCONFIRMED);
  assert.equal(r.notice, '読み込みの誤りが 1 件あります');
});

test('有効・無効の行が無い: 未確認の行で、無効とは言わない', async () => {
  const { io, ctx } = setup();
  queueSetAndReload(io, [L.done(true)]);
  const r = formatToggle(false, await toggleDebug(ctx, false));
  assert.equal(r.lines.at(-1), NO_LINE);
  assert.equal(r.notice, NO_LINE);
  assert.ok(!r.lines.includes(DISABLED));
});

test('頼んだ状態と違う: 違いの文が行と知らせにあり、PASTA_DEBUG を名指しする・誤りの知らせより先', async () => {
  const { io, ctx } = setup();
  queueSetAndReload(io, [`2026-11-21T00:00:00Z ERROR pasta_lua::loader: boom`, L.debug(false), L.done(true)]);
  const r = formatToggle(true, await toggleDebug(ctx, true));
  assert.equal(r.lines.at(-2), DISABLED);
  assert.equal(r.lines.at(-1), CANNOT_ENABLE);
  assert.equal(r.notice, CANNOT_ENABLE);
  assert.ok(r.notice.includes('PASTA_DEBUG'));
  assert.ok(r.lines.includes('誤り 1 件・警告 0 件'));

  const b = setup();
  queueSetAndReload(b.io, [L.debug(true), L.done(true)]);
  const r2 = formatToggle(false, await toggleDebug(b.ctx, false));
  assert.equal(r2.notice, 'デバッグを無効にできませんでした。環境変数 PASTA_DEBUG が設定されていると、保存変数より先に効きます。ほかの理由はマニュアルの「うまく動かないとき」を見てください');
});

test('読み込みの警告の行が要約に出る', async () => {
  const { io, ctx } = setup();
  queueSetAndReload(io, [L.warn('save variable is not a boolean'), L.debug(true), L.done(true)]);
  const r = formatToggle(true, await toggleDebug(ctx, true));
  assert.ok(r.lines.includes(`  [pasta.log] ${L.warn('save variable is not a boolean')}`));
  assert.equal(r.lines.at(-1), ENABLED);
  assert.equal(r.notice, undefined);
});

test('debugStateOf の表', () => {
  const pastaLog = { path: LOG };
  const d = (finished: boolean, loaded: boolean | undefined, debug: boolean | undefined): ReloadOutcome => ({
    kind: 'done', finished, loaded, debug, errors: [], warnings: [], pastaLogReadable: true, pastaLog,
  });
  const table: [ReloadOutcome, string][] = [
    [{ kind: 'refused', result: { kind: 'unreachable', detail: 'x' } }, 'unconfirmed'],
    [d(false, undefined, true), 'unconfirmed'],
    [d(true, false, true), 'unconfirmed'],
    [d(true, undefined, true), 'unconfirmed'],
    [d(true, true, undefined), 'unconfirmed'],
    [d(true, true, true), 'enabled'],
    [d(true, true, false), 'disabled'],
  ];
  for (const [o, want] of table) {
    assert.equal(debugStateOf(o), want, JSON.stringify(o));
  }
});
