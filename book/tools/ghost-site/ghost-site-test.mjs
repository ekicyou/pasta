// ghost-site-test.mjs — ghost-site の自己テスト（hello-pasta-network-update タスク 1.1 / 要件 3.3, 11.4）。
//
// 観測する完了条件（design「GhostSite」Invariants・Testing Strategy）:
//   - readNar がテスト内で組んだ zip（保存・deflate）を読み、ディレクトリのエントリを捨てる。
//   - `../x`・`a\b`・先頭 `/`・`C:`・NUL・空の区切り・重複・暗号化・zip64・非対応の方式を NarError で拒否する。
//   - parseUpdates が CRLF・0x01 区切り・charset 行・空行を扱い、date= を無視し、解釈できない行で失敗する。

import assert from 'node:assert/strict';
import zlib from 'node:zlib';
import { readNar, parseUpdates, NarError } from './ghost-site.mjs';

// --- 最小 assert ハーネス（依存ゼロ） ---
let passed = 0;
let failed = 0;
function test(name, fn) {
  try {
    fn();
    passed++;
    console.log(`ok   ${name}`);
  } catch (e) {
    failed++;
    console.log(`FAIL ${name}\n     ${e.message}`);
  }
}

// --- テスト用の zip を手で組む ---
// e: { name, data?, method? (0|8), flags?, zip64? }
function makeZip(entries) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const e of entries) {
    const name = Buffer.from(e.name, 'utf8');
    const raw = Buffer.from(e.data ?? '');
    const method = e.method ?? 8;
    const body = method === 8 ? zlib.deflateRawSync(raw) : raw;
    const csize = e.zip64 ? 0xffffffff : body.length;
    const lh = Buffer.alloc(30);
    lh.writeUInt32LE(0x04034b50, 0);
    lh.writeUInt16LE(20, 4);
    lh.writeUInt16LE(e.flags ?? 0x0800, 6);
    lh.writeUInt16LE(method, 8);
    lh.writeUInt32LE(csize, 18);
    lh.writeUInt32LE(raw.length, 22);
    lh.writeUInt16LE(name.length, 26);
    const ch = Buffer.alloc(46);
    ch.writeUInt32LE(0x02014b50, 0);
    ch.writeUInt16LE(20, 4);
    ch.writeUInt16LE(20, 6);
    ch.writeUInt16LE(e.flags ?? 0x0800, 8);
    ch.writeUInt16LE(method, 10);
    ch.writeUInt32LE(csize, 20);
    ch.writeUInt32LE(raw.length, 24);
    ch.writeUInt16LE(name.length, 28);
    ch.writeUInt32LE(offset, 42);
    locals.push(lh, name, body);
    centrals.push(ch, name);
    offset += 30 + name.length + body.length;
  }
  const cd = Buffer.concat(centrals);
  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0);
  eocd.writeUInt16LE(entries.length, 8);
  eocd.writeUInt16LE(entries.length, 10);
  eocd.writeUInt32LE(cd.length, 12);
  eocd.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, cd, eocd]);
}

function rejects(zip, re) {
  assert.throws(() => readNar(zip), (err) => err instanceof NarError && re.test(err.message));
}

// --- readNar ---
test('readNar: 保存と deflate を展開し、ディレクトリを捨てる', () => {
  const big = 'あいう'.repeat(500);
  const files = readNar(
    makeZip([
      { name: 'ghost/', method: 0 },
      { name: 'ghost/master/', method: 0 },
      { name: 'install.txt', data: 'type,ghost\r\n', method: 0 },
      { name: 'ghost/master/descript.txt', data: big, method: 8 },
      { name: 'bal/updates.txt', data: '', method: 8 },
    ]),
  );
  assert.deepEqual(
    files.map((f) => f.name),
    ['install.txt', 'ghost/master/descript.txt', 'bal/updates.txt'],
  );
  assert.equal(files[0].data.toString(), 'type,ghost\r\n');
  assert.equal(files[1].data.toString(), big);
  assert.equal(files[2].data.length, 0);
  assert.ok(Buffer.isBuffer(files[0].data));
});

for (const bad of ['../x', 'a/../b', 'a\\b', '/abs', 'C:/x', 'c:x', 'a\0b', 'a//b', '']) {
  test(`readNar: 危険な名前 ${JSON.stringify(bad)} を拒否する`, () => {
    rejects(makeZip([{ name: bad, data: 'x' }]), /名前/);
  });
}

test('readNar: 重複した名前を拒否する', () => {
  rejects(makeZip([{ name: 'a.txt', data: '1' }, { name: 'a.txt', data: '2' }]), /重複/);
});

test('readNar: 暗号化を拒否する', () => {
  rejects(makeZip([{ name: 'a.txt', data: 'x', flags: 0x0801 }]), /暗号化/);
});

test('readNar: zip64 を拒否する', () => {
  rejects(makeZip([{ name: 'a.txt', data: 'x', zip64: true }]), /zip64/);
});

test('readNar: 方式 0・8 以外を拒否する', () => {
  rejects(makeZip([{ name: 'a.txt', data: 'x', method: 12 }]), /方式/);
});

test('readNar: zip でないものを拒否する', () => {
  rejects(Buffer.from('not a zip at all, definitely not'), /zip/);
});

// --- parseUpdates ---
const MD5A = '0123456789abcdef0123456789abcdef';
const MD5B = 'fedcba9876543210fedcba9876543210';

test('parseUpdates: CRLF・0x01 区切り・charset 行・空行・date= の無視', () => {
  const text =
    'charset,UTF-8\r\n' +
    `file,install.txt\x01${MD5A}\x01size=12\x01date=2026-11-01T10:00:00\x01\r\n` +
    '\r\n' +
    `file,ghost/master/pasta.dll\x01${MD5B}\x01size=0\x01date=2026-11-01T10:00:00\x01\r\n`;
  assert.deepEqual(parseUpdates(text), [
    { path: 'install.txt', md5: MD5A, size: 12 },
    { path: 'ghost/master/pasta.dll', md5: MD5B, size: 0 },
  ]);
});

test('parseUpdates: LF だけでも読む', () => {
  const text = `charset,UTF-8\nfile,a.txt\x01${MD5A}\x01size=3\x01date=2026-11-01T10:00:00\x01\n`;
  assert.deepEqual(parseUpdates(text), [{ path: 'a.txt', md5: MD5A, size: 3 }]);
});

for (const [label, line] of [
  ['区切りが 0x01 でない', `file,a.txt,${MD5A},size=3,date=x,`],
  ['md5 の桁が違う', `file,a.txt\x01abc\x01size=3\x01date=x\x01`],
  ['size= が無い', `file,a.txt\x01${MD5A}\x013\x01date=x\x01`],
  ['file, で始まらない', `garbage`],
]) {
  test(`parseUpdates: 解釈できない行（${label}）で失敗する`, () => {
    assert.throws(() => parseUpdates(`charset,UTF-8\r\n${line}\r\n`), /updates\.txt/);
  });
}

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
