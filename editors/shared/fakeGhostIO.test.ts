import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createFakeGhostIO } from './fakeGhostIO.ts';

const dec = new TextDecoder();

test('ファイルを読む: 大きさ・更新時刻・途中から・足す・消す', async () => {
  const io = createFakeGhostIO();
  const log = io.join('C:\\g', 'ghost/master', 'pasta.log');
  assert.equal(log, 'C:\\g\\ghost\\master\\pasta.log');

  assert.equal(await io.stat(log), undefined);
  assert.equal(await io.read(log, 0), undefined);

  io.setFile(log, 'あ\n', 5_000_000);
  const s1 = await io.stat(log);
  assert.deepEqual(s1, { size: 4, mtimeMs: 5_000_000, isFile: true });

  io.appendFile('C:/g/ghost/master/pasta.log', 'b\n');
  const s2 = await io.stat(log);
  assert.equal(s2?.size, 6);
  assert.ok(s2!.mtimeMs > s1!.mtimeMs, '足すと更新時刻が進む');
  assert.equal(dec.decode(await io.read(log, s1!.size)), 'b\n');

  assert.deepEqual(await io.list('C:\\g\\ghost'), ['master']);
  assert.deepEqual(await io.stat('C:\\g\\ghost'), { size: 0, mtimeMs: 0, isFile: false });
  assert.equal(await io.read('C:\\g\\ghost', 0), undefined);

  io.deleteFile(log);
  assert.equal(await io.stat(log), undefined);
  assert.equal(await io.list('C:\\g'), undefined);
});

test('POST: 筋書きの順に答え、頼みを記録する', async () => {
  const io = createFakeGhostIO();
  io.queuePost(
    { kind: 'response', status: 200, body: '{"ok":1}' },
    (req) => {
      io.setFile('C:\\g\\x', req.body);
      return { kind: 'noResponse', detail: 'timeout' };
    },
    { kind: 'unreachable', detail: 'ECONNREFUSED' },
  );

  assert.deepEqual(await io.post(9801, 'a', 100), { kind: 'response', status: 200, body: '{"ok":1}' });
  assert.equal((await io.post(9802, 'b', 200)).kind, 'noResponse');
  assert.equal(dec.decode(await io.read('C:\\g\\x', 0)), 'b');
  assert.equal((await io.post(9803, 'c', 300)).kind, 'unreachable');
  await assert.rejects(io.post(9804, 'd', 400), /筋書きが尽きました/);

  assert.deepEqual(
    io.posts.map((p) => [p.port, p.body, p.timeoutMs]),
    [[9801, 'a', 100], [9802, 'b', 200], [9803, 'c', 300], [9804, 'd', 400]],
  );
});
