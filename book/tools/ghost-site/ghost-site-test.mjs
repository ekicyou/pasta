// ghost-site-test.mjs — ghost-site の自己テスト。
//
// 観測すること:
//   - readNar がテスト内で組んだ zip（保存・deflate）を読み、ディレクトリのエントリを捨てる。
//   - `../x`・`a\b`・先頭 `/`・`C:`・NUL・空の区切り・重複・暗号化・zip64・非対応の方式を NarError で拒否する。
//   - parseUpdates が CRLF・0x01 区切り・charset 行・空行を扱い、date= を無視し、解釈できない行で失敗する。
//   - checkIntegrity が updates.txt の各行をエントリと照合し、問題を全部 1 件 1 行で集める。
//   - extractNar が nar 全体（bal/ も）を書き、空でない出力先を拒否する。照合の失敗時は何も書かない。
//   - verifyLive がローカルの HTTP サーバーの updates.txt を一意のクエリ付きで取り直し、nar の側の行（パス・md5・大きさ）だけを見比べ、--full で実 URL の全行のファイルを照合する。

import assert from 'node:assert/strict';
import zlib from 'node:zlib';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { spawnSync, execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  readNar,
  parseUpdates,
  checkIntegrity,
  extractNar,
  buildSite,
  verifyLive,
  listTree,
  narOnlyFiles,
  NarError,
} from './ghost-site.mjs';

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

// --- buildSite と CLI `build` ---
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

// --- listTree・narOnlyFiles と CLI `nar-only` ---
// 一時の git リポジトリで、hello/ の下をタグにし、その後で作業ツリーから消す（main で消したものに当たる）。
// コミットを作らず、ツリーのオブジェクトに軽いタグを付ける（利用者の git 設定の署名などに左右されない）。
const gitEnv = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith('GIT_')));
const git = (cwd, ...args) => {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8', env: gitEnv });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')}: ${r.stderr}`);
  return r.stdout.trim();
};
function withRepo(fn) {
  withTmp((tmp) => {
    const repo = path.join(tmp, 'repo');
    fs.mkdirSync(repo);
    git(repo, 'init', '-q');
    for (const [p, d] of [['hello/install.txt', 'type,ghost\r\n'], ['hello/ghost/master/old.pasta', 'old'], ['other/x.txt', 'x']]) {
      fs.mkdirSync(path.dirname(path.join(repo, p)), { recursive: true });
      fs.writeFileSync(path.join(repo, p), d);
    }
    git(repo, 'add', '-A');
    git(repo, 'tag', 'v1', git(repo, 'write-tree'));
    git(repo, 'rm', '-q', '-f', 'hello/ghost/master/old.pasta');
    fn(tmp, repo);
  });
}
const narOnlyNar = () => {
  const entries = [
    ['install.txt', 'type,ghost\r\n'],
    ['ghost/master/old.pasta', 'old'],
    ['ghost/master/pasta.dll', 'DLL'],
    ['ghost/master/scripts/README.md', '# readme'],
  ];
  return [
    file('updates.txt', updatesOf(entries)),
    file('updates2.dau', 'dau'),
    ...entries.map(([p, d]) => file(p, d)),
    file('ghost/master/updates.txt', 'nested'),
    file('ghost/master/updates2.dau', 'nested dau'),
    file('ghost/master/developer_options.txt', 'dev'),
    file('bal/updates.txt', 'bal'),
  ];
};
const NAR_ONLY = ['ghost/master/pasta.dll', 'ghost/master/scripts/README.md'];

test('listTree: タグのツリーを hello/ からの相対の名前の集合で読む', () => {
  withRepo((tmp, repo) => {
    assert.deepEqual([...listTree('v1:hello', repo)].sort(), ['ghost/master/old.pasta', 'install.txt']);
  });
});

test('listTree: 読めないタグは git の文を添えて失敗する', () => {
  withRepo((tmp, repo) => {
    assert.throws(() => listTree('nope:hello', repo), /nope/);
  });
});

test('narOnlyFiles: ツリーに在るもの・どの階層の除外の 3 つの名前も外し、順は nar のまま', () => {
  const files = narOnlyNar();
  const out = narOnlyFiles(files, new Set(['install.txt', 'ghost/master/old.pasta']));
  assert.deepEqual(out.map((f) => f.name), NAR_ONLY);
  assert.ok(out.every((f) => files.includes(f)), 'nar のファイルそのもの（バイトを変えない）');
});

const cliIn = (cwd, args) =>
  spawnSync(process.execPath, [SCRIPT, ...args], { cwd, encoding: 'utf8', env: { ...gitEnv, GITHUB_ACTIONS: '' } });

test('CLI nar-only: nar の側だけを nar のバイトのまま書き、名前を 1 行ずつ出す（main で消したものも残らない）', () => {
  withRepo((tmp, repo) => {
    const files = narOnlyNar();
    const out = path.join(tmp, 'out');
    const r = cliIn(repo, ['nar-only', '--nar', writeNar(tmp, files), '--tree', 'v1:hello', '--out', out]);
    assert.equal(r.status, 0, r.stdout + r.stderr);
    assert.deepEqual(r.stdout.trim().split(/\r?\n/), NAR_ONLY);
    assert.deepEqual(listAll(out), NAR_ONLY);
    for (const n of NAR_ONLY) assert.deepEqual(fs.readFileSync(path.join(out, n)), files.find((f) => f.name === n).data);
  });
});

test('CLI nar-only: nar に updates.txt が無ければ何も書かずに終了コード 1', () => {
  withRepo((tmp, repo) => {
    const out = path.join(tmp, 'out');
    const nar = writeNar(tmp, narOnlyNar().filter((f) => f.name !== 'updates.txt'));
    const r = cliIn(repo, ['nar-only', '--nar', nar, '--tree', 'v1:hello', '--out', out]);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /NG updates\.txt/);
    assert.ok(!fs.existsSync(out));
  });
});

test('CLI nar-only: 無いタグは git の文を出して終了コード 1、何も書かない', () => {
  withRepo((tmp, repo) => {
    const out = path.join(tmp, 'out');
    const r = cliIn(repo, ['nar-only', '--nar', writeNar(tmp, narOnlyNar()), '--tree', 'nope:hello', '--out', out]);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /nope/);
    assert.ok(!fs.existsSync(out));
  });
});

test('CLI nar-only: nar の側が 0 本・出力先が空でない・引数の不足は終了コード 1', () => {
  withRepo((tmp, repo) => {
    const entries = [['install.txt', 'type,ghost\r\n']];
    const only = writeNar(tmp, [file('updates.txt', updatesOf(entries)), file('install.txt', 'type,ghost\r\n')]);
    let r = cliIn(repo, ['nar-only', '--nar', only, '--tree', 'v1:hello', '--out', path.join(tmp, 'o1')]);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /0 本/);
    assert.ok(!fs.existsSync(path.join(tmp, 'o1')));

    const full = path.join(tmp, 'o2');
    fs.mkdirSync(full);
    fs.writeFileSync(path.join(full, 'keep.txt'), 'old');
    r = cliIn(repo, ['nar-only', '--nar', writeNar(tmp, narOnlyNar()), '--tree', 'v1:hello', '--out', full]);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /空でない/);
    assert.deepEqual(listAll(full), ['keep.txt']);

    r = cliIn(repo, ['nar-only', '--nar', 'x.nar', '--out', full]);
    assert.equal(r.status, 1);
    assert.match(r.stderr, /使い方/);
  });
});

// --- verifyLive と CLI `verify-live` ---
async function atest(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`ok   ${name}`);
  } catch (e) {
    failed++;
    console.log(`FAIL ${name}\n     ${e.message}`);
  }
}

// 127.0.0.1 の空きポートで立てる。handler(path, n) は { status, body } を返す（n は updates.txt を取られた回数）。
async function serve(handler) {
  const urls = [];
  let n = 0;
  const server = http.createServer((req, res) => {
    urls.push(req.url);
    const u = new URL(req.url, 'http://x');
    const p = decodeURIComponent(u.pathname.replace(/^\/site\//, ''));
    if (p === 'updates.txt') n++;
    const { status, body } = handler(p, n);
    res.writeHead(status);
    res.end(body);
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}/site/`;
  return { base, urls, close: () => new Promise((r) => server.close(r)) };
}

const liveFiles = () => {
  const entries = [['install.txt', 'type,ghost\r\n'], ['ghost/master/pasta.dll', 'DLL'], ['ghost/master/辞書.pasta', '＊会話\r\n']];
  return [file('updates.txt', updatesOf(entries)), ...entries.map(([p, d]) => file(p, d)), file('bal/descript.txt', 'balloon')];
};
const siteOf = (files) => (p) => {
  const f = files.find((x) => x.name === p);
  return f ? { status: 200, body: f.data } : { status: 404, body: 'not found' };
};
const STALE = Buffer.from('charset,UTF-8\r\nold\r\n');
// タグのツリー（リポジトリの側）。liveFiles のうち nar の側＝比べる行は ghost/master/pasta.dll の 1 行。
const TREE = new Set(['install.txt', 'ghost/master/辞書.pasta']);
const DLL_ROW = 'ghost/master/pasta.dll';
// liveFiles の updates.txt の pasta.dll の行だけを row に差し替えた（null なら消した）実 URL の updates.txt。
const liveWithDllRow = (row) =>
  Buffer.from(
    liveFiles()[0]
      .data.toString('utf8')
      .split('\r\n')
      .map((l) => (l.startsWith(`file,${DLL_ROW}\x01`) ? row : l))
      .filter((l) => l !== null)
      .join('\r\n'),
  );

await atest('verifyLive: 日付の欄とリポジトリの側の行が違っても、nar の側の行が同じなら一致する', async () => {
  const files = liveFiles();
  const live = updatesOf([['install.txt', 'changed on main'], [DLL_ROW, 'DLL'], ['ghost/master/new.pasta', 'new']]).replace(
    /date=[^\x01]*/g,
    'date=2030-01-01T00:00:00',
  );
  const s = await serve((p) => (p === 'updates.txt' ? { status: 200, body: live } : { status: 404, body: '' }));
  try {
    const r = await verifyLive({ files, tree: TREE, base: s.base, attempts: 3, intervalMs: 10, full: false });
    assert.deepEqual(r.problems, []);
    assert.equal(r.ok, true);
    assert.equal(r.attempts, 1);
    assert.equal(r.compared, 1);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: nar の側の行の md5・大きさの違いや行の欠けは、上限まで取り直して失敗し、行を 1 件 1 行で出す', async () => {
  for (const [row, re] of [
    [`file,${DLL_ROW}\x01${md5('DLY')}\x01size=3\x01date=2026-11-01T10:00:00\x01`, /^NG ghost\/master\/pasta\.dll: md5 が違う/],
    [`file,${DLL_ROW}\x01${md5('DLL')}\x01size=4\x01date=2026-11-01T10:00:00\x01`, /^NG ghost\/master\/pasta\.dll: 大きさが違う/],
    [null, /^NG ghost\/master\/pasta\.dll: 実 URL の updates\.txt に無い/],
  ]) {
    const body = liveWithDllRow(row);
    const s = await serve(() => ({ status: 200, body }));
    try {
      const r = await verifyLive({ files: liveFiles(), tree: TREE, base: s.base, attempts: 2, intervalMs: 10, full: false });
      assert.equal(r.ok, false, String(row));
      assert.equal(r.attempts, 2);
      assert.equal(s.urls.length, 2);
      assert.match(r.problems[0], /^NG updates\.txt: 2 回取っても nar の側の 1 行が一致しない/);
      assert.equal(r.problems.length, 2, r.problems.join('\n'));
      assert.match(r.problems[1], re);
    } finally {
      await s.close();
    }
  }
});

await atest('verifyLive: 比べる行が 0 本なら取らずに失敗する', async () => {
  let calls = 0;
  const r = await verifyLive({
    files: liveFiles(),
    tree: new Set([...TREE, DLL_ROW]),
    base: 'http://127.0.0.1:9/',
    attempts: 3,
    intervalMs: 10,
    full: false,
    fetchFn: async () => (calls++, new Response('')),
  });
  assert.equal(r.ok, false);
  assert.equal(calls, 0);
  assert.equal(r.attempts, 0);
  assert.equal(r.compared, 0);
  assert.match(r.problems[0], /^NG updates\.txt: nar の側の行が 0 本/);
});

await atest('verifyLive: 初回で一致すれば 1 回で通り、取得に一意のクエリ cb= を付ける', async () => {
  const files = liveFiles();
  const s = await serve(siteOf(files));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 3, intervalMs: 10, tree: TREE, full: false });
    assert.deepEqual(r.problems, []);
    assert.equal(r.ok, true);
    assert.equal(r.attempts, 1);
    assert.equal(s.urls.length, 1, '--full でなければ updates.txt だけを取る');
    assert.match(s.urls[0], /^\/site\/updates\.txt\?cb=[^&]+-1$/);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: Actions ではラベルを run の ID と試行の番号にする（再実行でも一意）', async () => {
  const files = liveFiles();
  const s = await serve(siteOf(files));
  const saved = { id: process.env.GITHUB_RUN_ID, at: process.env.GITHUB_RUN_ATTEMPT };
  process.env.GITHUB_RUN_ID = '987';
  process.env.GITHUB_RUN_ATTEMPT = '2';
  try {
    await verifyLive({ files, base: s.base, attempts: 1, intervalMs: 10, tree: TREE, full: false });
    assert.match(s.urls[0], /\?cb=987\.2-1$/);
  } finally {
    for (const [k, v] of [['GITHUB_RUN_ID', saved.id], ['GITHUB_RUN_ATTEMPT', saved.at]]) {
      if (v === undefined) delete process.env[k];
      else process.env[k] = v;
    }
    await s.close();
  }
});

await atest('verifyLive: 最初の 2 回は古い内容でも、取り直して 3 回目の一致で通る（クエリは回ごとに違う）', async () => {
  const files = liveFiles();
  const site = siteOf(files);
  const s = await serve((p, n) => (p === 'updates.txt' && n <= 2 ? { status: 200, body: STALE } : site(p)));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 5, intervalMs: 10, tree: TREE, full: false });
    assert.equal(r.ok, true, r.problems.join('\n'));
    assert.equal(r.attempts, 3);
    assert.equal(s.urls.length, 3);
    assert.equal(new Set(s.urls).size, 3);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: 上限まで違い続ければ失敗し、最後の状態（HTTP・長さ・md5）を出す', async () => {
  const files = liveFiles();
  const s = await serve(() => ({ status: 200, body: STALE }));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 3, intervalMs: 10, tree: TREE, full: false });
    assert.equal(r.ok, false);
    assert.equal(r.attempts, 3);
    assert.equal(s.urls.length, 3);
    assert.equal(r.problems.length, 1);
    assert.match(r.problems[0], /^NG updates\.txt: /);
    assert.ok(r.problems[0].includes('HTTP 200'), r.problems[0]);
    assert.ok(r.problems[0].includes(`長さ ${STALE.length}`), r.problems[0]);
    assert.ok(r.problems[0].includes(md5(STALE)), r.problems[0]);
  } finally {
    await s.close();
  }
});

// 混んだ CI でも揺れないよう、待つ・待たないの差を秒の桁で見る。
await atest('verifyLive: 回の間は intervalMs 待ち、最後の回の後は待たない', async () => {
  const files = liveFiles();
  const s = await serve(() => ({ status: 200, body: STALE }));
  try {
    let t = Date.now();
    await verifyLive({ files, base: s.base, attempts: 1, intervalMs: 10000, tree: TREE, full: false });
    let took = Date.now() - t;
    assert.ok(took < 3000, `1 回だけなら待たない（最後の後に待てば 10 秒以上）: ${took}ms`);
    t = Date.now();
    await verifyLive({ files, base: s.base, attempts: 2, intervalMs: 1000, tree: TREE, full: false });
    took = Date.now() - t;
    assert.ok(took >= 900 && took < 1800, `2 回なら間の 1 回だけ待つ: ${took}ms`);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: 404 は取り直し、上限なら HTTP 404 で失敗する。後から 200 になれば通る', async () => {
  const files = liveFiles();
  let s = await serve(() => ({ status: 404, body: 'nope' }));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 2, intervalMs: 10, tree: TREE, full: false });
    assert.equal(r.ok, false);
    assert.equal(s.urls.length, 2);
    assert.match(r.problems[0], /HTTP 404/);
  } finally {
    await s.close();
  }
  const site = siteOf(files);
  s = await serve((p, n) => (n === 1 ? { status: 404, body: 'nope' } : site(p)));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 2, intervalMs: 10, tree: TREE, full: false });
    assert.equal(r.ok, true, r.problems.join('\n'));
    assert.equal(r.attempts, 2);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: ネットワークの失敗も取り直し、上限なら取得できないとして失敗する', async () => {
  const files = liveFiles();
  let calls = 0;
  const fetchFn = async () => {
    calls++;
    throw new TypeError('fetch failed');
  };
  const r = await verifyLive({ files, base: 'http://127.0.0.1:9/site/', attempts: 3, intervalMs: 10, tree: TREE, full: false, fetchFn });
  assert.equal(r.ok, false);
  assert.equal(calls, 3);
  assert.equal(r.attempts, 3);
  assert.match(r.problems[0], /^NG updates\.txt: .*取得できない.*fetch failed/);
});

await atest('verifyLive --full: 全ファイルを取って照合し（日本語のパスは区切りごとに符号化）、全部合えば通る', async () => {
  const files = liveFiles();
  const s = await serve(siteOf(files));
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 1, intervalMs: 10, tree: TREE, full: true });
    assert.deepEqual(r.problems, []);
    assert.equal(r.ok, true);
    assert.ok(s.urls.some((u) => u.startsWith(`/site/ghost/master/${encodeURIComponent('辞書.pasta')}?`)), s.urls.join(' '));
    assert.equal(s.urls.length, 4, 'updates.txt＋行の 3 ファイル（行に無い bal/ は取らない）');
  } finally {
    await s.close();
  }
});

await atest('verifyLive --full: 1 ファイルだけ改変されていれば検出し、欠けたファイルも 1 件 1 行で出す', async () => {
  const files = liveFiles();
  const site = siteOf(files);
  const s = await serve((p) =>
    p === 'ghost/master/pasta.dll' ? { status: 200, body: 'DLX' } : p === 'install.txt' ? { status: 404, body: '' } : site(p),
  );
  try {
    const r = await verifyLive({ files, base: s.base, attempts: 1, intervalMs: 10, tree: TREE, full: true });
    assert.equal(r.ok, false);
    assert.deepEqual(r.problems.map((x) => x.replace(/:.*/, '')), ['NG install.txt', 'NG ghost/master/pasta.dll']);
    assert.match(r.problems[0], /HTTP 404/);
    assert.match(r.problems[1], /md5 が違う/);
  } finally {
    await s.close();
  }
});

await atest('verifyLive: attempts が 1 未満なら取らずに投げる', async () => {
  let calls = 0;
  await assert.rejects(
    verifyLive({
      files: liveFiles(),
      base: 'http://127.0.0.1:9/',
      attempts: 0,
      intervalMs: 10,
      tree: TREE,
      full: false,
      fetchFn: async () => (calls++, new Response('')),
    }),
    /attempts/,
  );
  assert.equal(calls, 0);
});

await atest('verifyLive: nar に updates.txt が無ければ取らずに失敗する', async () => {
  let calls = 0;
  const r = await verifyLive({
    files: liveFiles().filter((f) => f.name !== 'updates.txt'),
    base: 'http://127.0.0.1:9/',
    attempts: 3,
    intervalMs: 10,
    tree: TREE,
    full: false,
    fetchFn: async () => (calls++, new Response('')),
  });
  assert.equal(r.ok, false);
  assert.equal(calls, 0);
  assert.equal(r.attempts, 0);
  assert.deepEqual(r.problems, ['NG updates.txt: nar の直下に無い']);
});

// サーバーと同じプロセスなので spawnSync でなく非同期で起動する。
const acli = (args, env = {}, cwd = undefined) =>
  new Promise((resolve) => {
    execFile(process.execPath, [SCRIPT, ...args], { cwd, encoding: 'utf8', env: { ...gitEnv, GITHUB_ACTIONS: '', ...env } }, (err, stdout, stderr) =>
      resolve({ status: err ? err.code : 0, stdout, stderr }),
    );
  });

await atest('CLI verify-live: 一致は終了コード 0、違いは終了コード 1 と NG の行（Actions では ::error:: も）', async () => {
  const files = liveFiles();
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'ghost-site-test-'));
  const s = await serve(siteOf(files));
  // タグ v1 の hello/ がリポジトリの側（TREE と同じ）。
  const repo = path.join(tmp, 'repo');
  for (const n of TREE) {
    fs.mkdirSync(path.dirname(path.join(repo, 'hello', n)), { recursive: true });
    fs.writeFileSync(path.join(repo, 'hello', n), files.find((f) => f.name === n).data);
  }
  git(repo, 'init', '-q');
  git(repo, 'add', '-A');
  git(repo, 'tag', 'v1', git(repo, 'write-tree'));
  const vl = (nar, more = [], env = {}) =>
    acli(['verify-live', '--nar', nar, '--tree', 'v1:hello', '--base', s.base, '--attempts', '2', '--interval', '0', ...more], env, repo);
  try {
    const nar = writeNar(tmp, files);
    let r = await vl(nar, ['--full']);
    assert.equal(r.status, 0, r.stdout + r.stderr);
    assert.equal(r.stdout, 'OK updates.txt: 1 回目で nar の側の 1 行が一致（--full: 3 ファイルも一致）\n');
    r = await vl(nar);
    assert.equal(r.status, 0, r.stdout + r.stderr);
    assert.equal(r.stdout, 'OK updates.txt: 1 回目で nar の側の 1 行が一致\n');

    // nar の側の pasta.dll だけが実 URL と違う nar（nar の中は整合している）。
    const other = path.join(tmp, 'other.nar');
    fs.writeFileSync(
      other,
      narOf(
        files.map((f) =>
          f.name === DLL_ROW
            ? file(DLL_ROW, 'DLY')
            : f.name === 'updates.txt'
              ? file('updates.txt', f.data.toString('utf8').replace(md5('DLL'), md5('DLY')))
              : f,
        ),
      ),
    );
    r = await vl(other, [], { GITHUB_ACTIONS: 'true' });
    assert.equal(r.status, 1);
    assert.match(r.stdout, /^NG updates\.txt: /m);
    assert.match(r.stdout, /^::error::NG updates\.txt: /m);
    assert.match(r.stdout, /^NG ghost\/master\/pasta\.dll: md5 が違う/m);

    r = await acli(['verify-live', '--nar', nar, '--tree', 'nope:hello', '--base', s.base, '--attempts', '1', '--interval', '0'], {}, repo);
    assert.equal(r.status, 1);
    assert.match(r.stdout, /nope/);

    const junk = path.join(tmp, 'junk.nar');
    fs.writeFileSync(junk, 'not a zip at all, definitely not');
    r = await vl(junk, ['--attempts', '1']);
    assert.equal(r.status, 1);
    assert.equal(r.stdout.trim().split(/\r?\n/).length, 1);
    assert.match(r.stdout, /zip/);
  } finally {
    await s.close();
    fs.rmSync(tmp, { recursive: true, force: true });
  }
});

await atest('CLI verify-live: 引数の不足（--tree 無しも）・不正な回数や間隔は使い方を出して終了コード 1', async () => {
  for (const args of [
    ['verify-live'],
    ['verify-live', '--nar', 'x.nar'],
    ['verify-live', '--nar', 'x.nar', '--base', 'http://127.0.0.1:9/'],
    ['verify-live', '--nar', 'x.nar', '--tree', 'v1:hello', '--base', 'http://127.0.0.1:9/', '--attempts', '0'],
    ['verify-live', '--nar', 'x.nar', '--tree', 'v1:hello', '--base', 'http://127.0.0.1:9/', '--interval', 'x'],
  ]) {
    const r = await acli(args);
    assert.equal(r.status, 1, JSON.stringify(args));
    assert.match(r.stderr, /使い方/);
  }
});

console.log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
