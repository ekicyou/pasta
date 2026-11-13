// Unit tests for GhostFiles (task 4.1). Plain node in a temp folder:
//   npx esbuild src/test/ghostFiles.test.ts --bundle --platform=node --format=cjs --outfile=out/test/ghostFiles.test.js && node out/test/ghostFiles.test.js
//
// Requirements: 2.1 (root = two above ghost/master within 3 levels up, with descript.txt),
// 2.3 (display name = descript name, else the root folder name),
// 5.6 (pasta.log location from pasta.toml [logging] file_path, default + note when unusable),
// 4.3 / 5.7 (pasta.log tail: only complete appended lines, levels, loaded=true|false, unreadable),
// 6.2 / 6.3 (variables.toml location, mtime, contents; a missing file has no mtime).

import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import {
  decodeDescriptName, pastaLogPath, DEFAULT_PASTA_LOG,
  openLogCursor, readNewLines, parsePastaLogLine, loadCompletion, variablesPath, mtimeOf, readText,
} from '../../../shared/ghostFiles.ts';
import { MAX_ASCENT, nodeGhostIO as io, resolveGhost } from '../nodeGhostIO';

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

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'ghostFiles-'));

/** tmp/<ghost>/Ghost/MASTER with an optional descript (spelled oddly to check case-insensitivity). */
function makeGhost(ghost: string, descript?: Uint8Array): string {
  const master = path.join(tmp, ghost, 'Ghost', 'MASTER');
  fs.mkdirSync(master, { recursive: true });
  if (descript) {
    fs.writeFileSync(path.join(master, 'Descript.TXT'), descript);
  }
  return master;
}

/** A file `depth` folders below master. */
function fileAt(master: string, depth: number): string {
  const dir = path.join(master, ...Array.from({ length: depth }, (_, i) => `d${i}`));
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, 'a.pasta');
}

const utf8 = (s: string) => new Uint8Array(Buffer.from(s, 'utf8'));
// "name,あ漢" in Shift_JIS, no charset line
const sjisName = new Uint8Array([...Buffer.from('name,'), 0x82, 0xa0, 0x8a, 0xbf, 0x0d, 0x0a]);

async function main() {
  console.log('ghostFiles');

  await test('depth 0 to 3 is found; root, master, key', async () => {
    const master = makeGhost('Depth', utf8('charset,UTF-8\r\nname,Depth Ghost\r\n'));
    const root = path.join(tmp, 'Depth');
    for (let d = 0; d <= MAX_ASCENT; d++) {
      const g = await resolveGhost(io, fileAt(master, d), master);
      assert.ok(g, `depth ${d} should be found`);
      assert.strictEqual(g.root, root);
      assert.strictEqual(g.master, master);
      assert.strictEqual(g.key, root.toLowerCase());
      assert.strictEqual(g.displayName, 'Depth Ghost');
    }
  });

  await test('depth 4 is not found', async () => {
    const master = makeGhost('Deep', utf8('charset,UTF-8\r\nname,x\r\n'));
    assert.strictEqual(await resolveGhost(io, fileAt(master, MAX_ASCENT + 1), master), undefined);
  });

  await test('no descript.txt is not found; no master is not found', async () => {
    const master = makeGhost('NoDescript');
    assert.strictEqual(await resolveGhost(io, fileAt(master, 0), master), undefined);
    assert.strictEqual(await resolveGhost(io, fileAt(master, 0), undefined), undefined);
  });

  await test('a file outside master is not found', async () => {
    const master = makeGhost('Outside', utf8('name,x\r\n'));
    assert.strictEqual(await resolveGhost(io, path.join(tmp, 'Outside', 'a.pasta'), master), undefined);
  });

  await test('UTF-8 name with BOM', async () => {
    const bytes = new Uint8Array([0xef, 0xbb, 0xbf, ...utf8('charset,utf-8\r\nname,ぱすた\r\n')]);
    assert.strictEqual(decodeDescriptName(bytes), 'ぱすた');
  });

  await test('Shift_JIS name (charset missing or not UTF-8)', async () => {
    assert.strictEqual(decodeDescriptName(sjisName), 'あ漢');
    const withCharset = new Uint8Array([...Buffer.from('charset,Shift_JIS\r\n'), ...sjisName]);
    assert.strictEqual(decodeDescriptName(withCharset), 'あ漢');
  });

  await test('no name, empty name, unreadable bytes give undefined', async () => {
    assert.strictEqual(decodeDescriptName(utf8('charset,UTF-8\r\nsakura.name,x\r\n')), undefined);
    assert.strictEqual(decodeDescriptName(utf8('charset,UTF-8\r\nname,\r\n')), undefined);
    assert.strictEqual(decodeDescriptName(new Uint8Array([...Buffer.from('charset,UTF-8\nname,'), 0xff, 0xfe])), undefined);
  });

  await test('display name falls back to the root folder name', async () => {
    const master = makeGhost('Fallback', utf8('charset,UTF-8\r\ncraftman,x\r\n'));
    const g = await resolveGhost(io, fileAt(master, 1), master);
    assert.strictEqual(g?.displayName, 'Fallback');
  });

  const m = path.join(tmp, 'g', 'ghost', 'master');
  const def = path.join(m, DEFAULT_PASTA_LOG);
  const at = (toml: string | undefined) => pastaLogPath(io, m, toml);

  await test('file_path: three forms, basic and literal strings', async () => {
    const want = { path: path.join(m, 'profile/custom/app.log') };
    assert.deepStrictEqual(at('[debug]\nenabled = true\n\n[ logging ]  # log\nlevel = "info"\nfile_path = "profile/custom/app.log" # c\n'), want);
    assert.deepStrictEqual(at("[logging]\nfile_path='profile/custom/app.log'\n"), want);
    assert.deepStrictEqual(at('logging.file_path = "profile/custom/app.log"\n[debug]\n'), want);
    assert.deepStrictEqual(at("logging = { level = 'debug', file_path = 'profile/custom/app.log' }\n"), want);
    // basic string: only \\ and \" are restored
    assert.deepStrictEqual(at('[logging]\nfile_path = "profile/a\\\\b\\"c.log"\n'), { path: path.join(m, 'profile/a/b"c.log') });
  });

  await test('file_path judged by path components like logger.rs (\\ separates on Windows)', async () => {
    const ok = (v: string, rel: string) => assert.deepStrictEqual(at(`[logging]\nfile_path = '${v}'\n`), { path: path.join(m, rel) }, v);
    ok('profile\\logs\\x.log', 'profile/logs/x.log');
    ok('profile//x.log', 'profile/x.log');
    ok('./profile/x.log', 'profile/x.log');
    ok('profile/./x.log', 'profile/x.log');
    ok('profile/dir/', 'profile/dir'); // pasta writes a file named dir
  });

  await test('file_path unset or no pasta.toml: default, no note', async () => {
    assert.deepStrictEqual(at(undefined), { path: def });
    assert.deepStrictEqual(at(''), { path: def });
    assert.deepStrictEqual(at('[logging]\nlevel = "debug"\n[other]\nfile_path = "profile/x.log"\n'), { path: def });
    assert.deepStrictEqual(at('# [logging]\n# file_path = "profile/x.log"\n'), { path: def });
  });

  await test('rejected values fall back to default with a note', async () => {
    for (const v of ['../x', 'C:\\x', 'Profile/x', 'profile/', 'profile/../x', '/profile/x', '', 'profile', 'profile\\', 'profile\\..\\x', '\\profile\\x',
      '\\\\srv\\profile\\x', 'C:/profile/x', 'profiles/x.log', 'profile.log', 'profile/a..b.log']) {
      const r = at(`[logging]\nfile_path = '${v}'\n`); // literal string: the value is exactly v
      assert.strictEqual(r.path, def, v);
      assert.ok(r.note && r.note.includes('file_path'), `note for ${v}`);
    }
  });

  await test('unreadable forms fall back to default with a note', async () => {
    for (const toml of [
      'logging = {\n  file_path = "profile/x.log"\n}\n',
      '[logging]\nfile_path = 42\n',
      '[logging]\nfile_path = """profile/x.log"""\n',
      '[logging]\nfile_path = "profile/x.log\n',
    ]) {
      const r = at(toml);
      assert.strictEqual(r.path, def, toml);
      assert.ok(r.note, `note for ${JSON.stringify(toml)}`);
    }
  });

  // Lines shaped like tracing's fmt layer in pasta (ansi off, target on): `<time>  <LEVEL> <target>: <msg>`
  const L = (level: string, msg: string) => `2026-11-03T01:02:03.456789Z ${level.padStart(5)} pasta_shiori::windows: ${msg}`;

  await test('readNewLines reads only complete appended lines and keeps the partial line', async () => {
    const log = path.join(tmp, 'tail', 'pasta.log');
    fs.mkdirSync(path.dirname(log), { recursive: true });
    fs.writeFileSync(log, 'old line\n');
    const c0 = await openLogCursor(io, log);
    assert.strictEqual(c0.offset, Buffer.byteLength('old line\n'));
    fs.appendFileSync(log, 'あ1\r\nb2\npart');
    const r1 = await readNewLines(io, c0);
    assert.ok(r1);
    assert.deepStrictEqual(r1.lines, ['あ1', 'b2']);
    fs.appendFileSync(log, 'ial\n');
    const r2 = await readNewLines(io, r1.next);
    assert.ok(r2);
    assert.deepStrictEqual(r2.lines, ['partial']);
    const r3 = await readNewLines(io, r2.next);
    assert.deepStrictEqual(r3?.lines, []);
  });

  await test('a log that does not exist yet starts at 0; unreadable gives undefined', async () => {
    const log = path.join(tmp, 'tail', 'later.log');
    const c = await openLogCursor(io, log);
    assert.strictEqual(c.offset, 0);
    assert.strictEqual(await readNewLines(io, c), undefined); // not there: unreadable
    fs.writeFileSync(log, 'x\n');
    assert.deepStrictEqual((await readNewLines(io, c))?.lines, ['x']);
    assert.strictEqual(await readNewLines(io, { path: path.join(tmp, 'tail'), offset: 0 }), undefined); // a folder
  });

  await test('a log that shrank is read again from the start', async () => {
    const log = path.join(tmp, 'tail', 'shrink.log');
    fs.writeFileSync(log, 'long long line\n');
    const c = await openLogCursor(io, log);
    fs.writeFileSync(log, 'new\n');
    assert.deepStrictEqual((await readNewLines(io, c))?.lines, ['new']);
  });

  await test('parsePastaLogLine reads the 5 levels', async () => {
    for (const lv of ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] as const) {
      const line = L(lv, 'msg ERROR WARN');
      assert.deepStrictEqual(parsePastaLogLine(line), { level: lv, text: line });
    }
    for (const line of ['', 'continued text', 'WARN at the start', '2026-11-03T01:02:03Z  NOTICE x: y', '2026-11-03T01:02:03Z  warn x: y']) {
      assert.strictEqual(parsePastaLogLine(line).level, undefined, line);
    }
  });

  await test('loadCompletion reads loaded=true and loaded=false only', async () => {
    assert.deepStrictEqual(loadCompletion(L('INFO', 'SHIORI load entry completed entry="load" loaded=true')), { loaded: true });
    assert.deepStrictEqual(loadCompletion(L('INFO', 'SHIORI load entry completed entry="v3" loaded=false')), { loaded: false });
    for (const line of [
      L('INFO', 'SHIORI load entry completed entry="load" loaded=1'),
      L('INFO', 'SHIORI load entry completed entry="load" loaded=trueish'),
      L('INFO', 'SHIORI load entry completed entry="load"'),
      L('INFO', 'other line loaded=true'),
    ]) {
      assert.strictEqual(loadCompletion(line), undefined, line);
    }
  });

  await test('variables.toml: location, mtime, contents; missing has no mtime', async () => {
    const master = path.join(tmp, 'Vars', 'ghost', 'master');
    const v = variablesPath(io, master);
    assert.strictEqual(v, path.join(master, 'profile', 'pasta', 'debug', 'variables.toml'));
    assert.strictEqual(await mtimeOf(io, v), undefined);
    assert.strictEqual(await readText(io, v), undefined);
    fs.mkdirSync(path.dirname(v), { recursive: true });
    fs.writeFileSync(v, 'a = "あ"\n');
    const m = await mtimeOf(io, v);
    assert.strictEqual(typeof m, 'number');
    assert.strictEqual(await readText(io, v), 'a = "あ"\n');
    assert.strictEqual(fs.readFileSync(v, 'utf8'), 'a = "あ"\n'); // never written
  });

  await test('readText drops a leading BOM, so a BOM pasta.toml still finds [logging] on line 1', async () => {
    const master = path.join(tmp, 'Bom', 'ghost', 'master');
    fs.mkdirSync(master, { recursive: true });
    const toml = path.join(master, 'pasta.toml');
    fs.writeFileSync(toml, '﻿[logging]\nfile_path = "profile/x.log"\n');
    const text = await readText(io, toml);
    assert.strictEqual(text, '[logging]\nfile_path = "profile/x.log"\n');
    assert.strictEqual(pastaLogPath(io, master, text).path, path.join(master, 'profile', 'x.log'));
  });

  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(`\n${passed} passed, ${failed} failed`);
  process.exit(failed > 0 ? 1 : 0);
}

main();
