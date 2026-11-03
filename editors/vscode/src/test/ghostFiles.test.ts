// Unit tests for GhostFiles (task 4.1). Plain node in a temp folder:
//   npx esbuild src/test/ghostFiles.test.ts --bundle --platform=node --format=cjs --outfile=out/test/ghostFiles.test.js && node out/test/ghostFiles.test.js
//
// Requirements: 2.1 (root = two above ghost/master within 3 levels up, with descript.txt),
// 2.3 (display name = descript name, else the root folder name).

import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { resolveGhost, decodeDescriptName, MAX_ASCENT } from '../ghostFiles';

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
      const g = await resolveGhost(fileAt(master, d), master);
      assert.ok(g, `depth ${d} should be found`);
      assert.strictEqual(g.root, root);
      assert.strictEqual(g.master, master);
      assert.strictEqual(g.key, root.toLowerCase());
      assert.strictEqual(g.displayName, 'Depth Ghost');
    }
  });

  await test('depth 4 is not found', async () => {
    const master = makeGhost('Deep', utf8('charset,UTF-8\r\nname,x\r\n'));
    assert.strictEqual(await resolveGhost(fileAt(master, MAX_ASCENT + 1), master), undefined);
  });

  await test('no descript.txt is not found; no master is not found', async () => {
    const master = makeGhost('NoDescript');
    assert.strictEqual(await resolveGhost(fileAt(master, 0), master), undefined);
    assert.strictEqual(await resolveGhost(fileAt(master, 0), undefined), undefined);
  });

  await test('a file outside master is not found', async () => {
    const master = makeGhost('Outside', utf8('name,x\r\n'));
    assert.strictEqual(await resolveGhost(path.join(tmp, 'Outside', 'a.pasta'), master), undefined);
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
    const g = await resolveGhost(fileAt(master, 1), master);
    assert.strictEqual(g?.displayName, 'Fallback');
  });

  fs.rmSync(tmp, { recursive: true, force: true });
  console.log(`\n${passed} passed, ${failed} failed`);
  process.exit(failed > 0 ? 1 : 0);
}

main();
