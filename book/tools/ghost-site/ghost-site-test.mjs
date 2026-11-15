// ghost-site-test.mjs — ghost-site の自己テスト（hello-pasta-network-update タスク 1.1–1.3 / 要件 1.1, 1.2, 1.5, 3.1–3.3, 4.4, 7.1–7.3, 11.4）。
//
// 観測する完了条件（design「GhostSite」Invariants・Testing Strategy）:
//   - readNar がテスト内で組んだ zip（保存・deflate）を読み、ディレクトリのエントリを捨てる。
//   - `../x`・`a\b`・先頭 `/`・`C:`・NUL・空の区切り・重複・暗号化・zip64・非対応の方式を NarError で拒否する。
//   - parseUpdates が CRLF・0x01 区切り・charset 行・空行を扱い、date= を無視し、解釈できない行で失敗する。
//   - checkIntegrity が updates.txt の各行をエントリと照合し、問題を全部 1 件 1 行で集める。
//   - extractNar が nar 全体（bal/ も）を書き、空でない出力先を拒否する。照合の失敗時は何も書かない。

import assert from 'node:assert/strict';
import zlib from 'node:zlib';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { readNar, parseUpdates, checkIntegrity, extractNar, buildSite, NarError } from './ghost-site.mjs';

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
// e: { name, data?, method? (0|8), flags?, zip64?, body?（圧縮後の中身を直に与える） }
function makeZip(entries) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const e of entries) {
    const name = Buffer.from(e.name, 'utf8');
    const raw = Buffer.from(e.data ?? '');
    const method = e.method ?? 8;
    const body = e.body ?? (method === 8 ? zlib.deflateRawSync(raw) : raw);
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

test('readNar: 壊れた deflate を NarError で拒否する', () => {
  rejects(makeZip([{ name: 'a.txt', data: 'xyz', method: 8, body: Buffer.from([0xff, 0xff, 0xff, 0xff]) }]), /a\.txt/);
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

// --- checkIntegrity ---
const md5 = (s) => crypto.createHash('md5').update(s).digest('hex');
const file = (name, data) => ({ name, data: Buffer.from(data) });
function updatesOf(entries) {
  return (
    'charset,UTF-8\r\n' +
    entries
      .map(([p, d]) => `file,${p}\x01${md5(d)}\x01size=${Buffer.byteLength(d)}\x01date=2026-11-01T10:00:00\x01\r\n`)
      .join('')
  );
}
const goodFiles = () => [
  file('updates.txt', updatesOf([['install.txt', 'type,ghost\r\n'], ['ghost/master/pasta.dll', 'DLL']])),
  file('install.txt', 'type,ghost\r\n'),
  file('ghost/master/pasta.dll', 'DLL'),
  file('ghost/master/updates.txt', 'nested'),
  file('bal/descript.txt', 'balloon'),
  file('bal/updates.txt', 'garbage that is not checked'),
];

test('checkIntegrity: 全行が合えば通る（bal/ 以下・入れ子の updates.txt は行に無くてよい）', () => {
  assert.deepEqual(checkIntegrity(goodFiles()), { ok: true, problems: [] });
});

test('checkIntegrity: ルートの updates.txt が無い', () => {
  const r = checkIntegrity(goodFiles().filter((f) => f.name !== 'updates.txt'));
  assert.equal(r.ok, false);
  assert.deepEqual(r.problems, ['NG updates.txt: nar の直下に無い']);
});

test('checkIntegrity: updates.txt を解釈できない', () => {
  const files = goodFiles();
  files[0] = file('updates.txt', 'charset,UTF-8\r\ngarbage\r\n');
  const r = checkIntegrity(files);
  assert.equal(r.ok, false);
  assert.equal(r.problems.length, 1);
  assert.match(r.problems[0], /^NG updates\.txt: .*2 行目/);
});

test('checkIntegrity: 行が指すファイルが無い', () => {
  const r = checkIntegrity(goodFiles().filter((f) => f.name !== 'install.txt'));
  assert.deepEqual(r.problems, ['NG install.txt: nar に無い']);
});

test('checkIntegrity: md5 が違う', () => {
  const files = goodFiles().map((f) => (f.name === 'ghost/master/pasta.dll' ? file(f.name, 'dll') : f));
  assert.deepEqual(checkIntegrity(files).problems, [
    `NG ghost/master/pasta.dll: md5 が違う（updates.txt: ${md5('DLL')}、nar: ${md5('dll')}）`,
  ]);
});

test('checkIntegrity: 大きさが違う', () => {
  const files = goodFiles();
  files[0] = file('updates.txt', updatesOf([['install.txt', 'type,ghost\r\n']]).replace('size=12', 'size=13'));
  assert.deepEqual(checkIntegrity(files).problems, ['NG install.txt: 大きさが違う（updates.txt: 13、nar: 12）']);
});

test('checkIntegrity: 行のパスが危険なら問題にする', () => {
  const files = goodFiles();
  files[0] = file('updates.txt', updatesOf([['../evil', 'x']]));
  const r = checkIntegrity(files);
  assert.equal(r.problems.length, 1);
  assert.match(r.problems[0], /^NG \.\.\/evil: .*名前/);
});

test('checkIntegrity: 問題が複数なら全部を 1 件 1 行で出す', () => {
  const files = goodFiles()
    .filter((f) => f.name !== 'install.txt')
    .map((f) => (f.name === 'ghost/master/pasta.dll' ? file(f.name, 'dll!') : f));
  const r = checkIntegrity(files);
  assert.equal(r.ok, false);
  assert.equal(r.problems.length, 3);
  assert.equal(r.problems[0], 'NG install.txt: nar に無い');
  assert.match(r.problems[1], /^NG ghost\/master\/pasta\.dll: md5 が違う/);
  assert.match(r.problems[2], /^NG ghost\/master\/pasta\.dll: 大きさが違う/);
  assert.ok(r.problems.every((p) => !p.includes('\n')));
});

// --- extractNar ---
function withTmp(fn) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghost-site-test-'));
  try {
    fn(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}
const listAll = (dir) =>
  fs
    .readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((d) => d.isFile())
    .map((d) => path.relative(dir, path.join(d.parentPath, d.name)).split(path.sep).join('/'))
    .sort();

test('extractNar: 無い出力先を作り、bal/ を含む全体を除外なしで書く', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const files = goodFiles();
    extractNar(files, out);
    assert.deepEqual(listAll(out), files.map((f) => f.name).sort());
    for (const f of files) assert.deepEqual(fs.readFileSync(path.join(out, f.name)), f.data);
  });
});

test('extractNar: 空の出力先には書く', () => {
  withTmp((tmp) => {
    extractNar([file('a.txt', 'A')], tmp);
    assert.deepEqual(listAll(tmp), ['a.txt']);
  });
});

test('extractNar: 空でない出力先を拒否し、何も書かない', () => {
  withTmp((tmp) => {
    fs.writeFileSync(path.join(tmp, 'keep.txt'), 'old');
    assert.throws(() => extractNar([file('a.txt', 'A')], tmp), /空でない/);
    assert.deepEqual(listAll(tmp), ['keep.txt']);
  });
});

test('extractNar: 出力先の外を指す名前を拒否し、何も書かない', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    assert.throws(() => extractNar([file('ok.txt', 'A'), file('../escape.txt', 'B')], out), NarError);
    assert.ok(!fs.existsSync(path.join(tmp, 'escape.txt')));
    assert.ok(!fs.existsSync(out));
  });
});

test('照合 → 展開: 照合に失敗したとき出力先に何も書かない', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const files = goodFiles().map((f) => (f.name === 'install.txt' ? file(f.name, 'tampered!!!!') : f));
    const r = checkIntegrity(files);
    if (r.ok) extractNar(files, out);
    assert.equal(r.ok, false);
    assert.ok(!fs.existsSync(out));
  });
});

// --- buildSite と CLI `build`（タスク 1.3 / 要件 1.1, 1.2, 7.1–7.3） ---
const SCRIPT = fileURLToPath(new URL('./ghost-site.mjs', import.meta.url));
const INDEX = fileURLToPath(new URL('./index.html', import.meta.url));
const narOf = (files) => makeZip(files.map((f) => ({ name: f.name, data: f.data })));
function writeNar(tmp, files) {
  const p = path.join(tmp, 'x.nar');
  fs.writeFileSync(p, narOf(files));
  return p;
}

test('buildSite: 出力の一覧が nar のエントリ＋index.html と一致し、入口はそのまま複写される', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const files = goodFiles();
    buildSite({ narPath: writeNar(tmp, files), outDir: out, indexPath: INDEX });
    assert.deepEqual(listAll(out), [...files.map((f) => f.name), 'index.html'].sort());
    assert.deepEqual(fs.readFileSync(path.join(out, 'index.html')), fs.readFileSync(INDEX));
  });
});

test('buildSite: nar に index.html があれば失敗し、何も書かない', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const narPath = writeNar(tmp, [...goodFiles(), file('index.html', '<p>evil</p>')]);
    assert.throws(() => buildSite({ narPath, outDir: out, indexPath: INDEX }), /NG index\.html: nar にも index\.html があり、入口と衝突する/);
    assert.ok(!fs.existsSync(out));
  });
});

test('buildSite: 照合に失敗したとき何も書かない', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const narPath = writeNar(tmp, goodFiles().filter((f) => f.name !== 'install.txt'));
    assert.throws(() => buildSite({ narPath, outDir: out, indexPath: INDEX }), /NG install\.txt: nar に無い/);
    assert.ok(!fs.existsSync(out));
  });
});

test('buildSite: 空でない出力先を拒否する', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    fs.mkdirSync(out);
    fs.writeFileSync(path.join(out, 'keep.txt'), 'old');
    assert.throws(() => buildSite({ narPath: writeNar(tmp, goodFiles()), outDir: out, indexPath: INDEX }), /空でない/);
    assert.deepEqual(listAll(out), ['keep.txt']);
  });
});

test('index.html: 版や nar に依らない静的な案内で、ダウンロードのページへリンクする', () => {
  const html = fs.readFileSync(INDEX, 'utf8');
  assert.match(html, /<html lang="ja">/);
  assert.match(html, /<meta charset="utf-8">/);
  assert.match(html, /<meta name="viewport"/);
  assert.match(html, /<title>[^<]+<\/title>/);
  assert.match(html, /href="https:\/\/ekicyou\.github\.io\/pasta\/download\.html[^"]*"/);
  assert.doesNotMatch(html, /<script|http-equiv="refresh"|\d+\.\d+\.\d+/i);
});

const cli = (args, env = {}) =>
  spawnSync(process.execPath, [SCRIPT, ...args], { encoding: 'utf8', env: { ...process.env, GITHUB_ACTIONS: '', ...env } });

test('CLI build: 成功は終了コード 0', () => {
  withTmp((tmp) => {
    const out = path.join(tmp, 'out');
    const r = cli(['build', '--nar', writeNar(tmp, goodFiles()), '--out', out, '--index', INDEX]);
    assert.equal(r.status, 0, r.stdout + r.stderr);
    assert.ok(fs.existsSync(path.join(out, 'index.html')));
  });
});

test('CLI build: 照合の失敗は終了コード 1、問題を 1 行 1 件で標準出力に出す（Actions では ::error:: も）', () => {
  withTmp((tmp) => {
    const narPath = writeNar(
      tmp,
      goodFiles()
        .filter((f) => f.name !== 'install.txt')
        .map((f) => (f.name === 'ghost/master/pasta.dll' ? file(f.name, 'dll!') : f)),
    );
    const args = ['build', '--nar', narPath, '--out', path.join(tmp, 'out'), '--index', INDEX];
    const r = cli(args);
    assert.equal(r.status, 1);
    const lines = r.stdout.trim().split(/\r?\n/);
    assert.equal(lines.length, 3);
    assert.equal(lines[0], 'NG install.txt: nar に無い');
    const a = cli(args, { GITHUB_ACTIONS: 'true' });
    assert.equal(a.status, 1);
    assert.ok(a.stdout.includes('::error::NG install.txt: nar に無い'));
  });
});

test('CLI build: nar の衝突・zip でない・空でない出力先は終了コード 1 と 1 行の理由', () => {
  withTmp((tmp) => {
    const collide = writeNar(tmp, [...goodFiles(), file('index.html', 'x')]);
    let r = cli(['build', '--nar', collide, '--out', path.join(tmp, 'o1'), '--index', INDEX]);
    assert.equal(r.status, 1);
    assert.equal(r.stdout.trim(), 'NG index.html: nar にも index.html があり、入口と衝突する');

    const junk = path.join(tmp, 'junk.nar');
    fs.writeFileSync(junk, 'not a zip at all, definitely not');
    r = cli(['build', '--nar', junk, '--out', path.join(tmp, 'o2'), '--index', INDEX]);
    assert.equal(r.status, 1);
    assert.equal(r.stdout.trim().split(/\r?\n/).length, 1);
    assert.match(r.stdout, /zip/);

    const full = path.join(tmp, 'o3');
    fs.mkdirSync(full);
    fs.writeFileSync(path.join(full, 'keep.txt'), 'old');
    r = cli(['build', '--nar', writeNar(tmp, goodFiles()), '--out', full, '--index', INDEX]);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /空でない/);
  });
});

test('CLI: 引数の不足・不明なサブコマンドは使い方を出して終了コード 1', () => {
  for (const args of [[], ['build'], ['build', '--nar', 'x.nar'], ['nope']]) {
    const r = cli(args);
    assert.equal(r.status, 1, JSON.stringify(args));
    assert.match(r.stderr, /使い方/);
  }
});

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
