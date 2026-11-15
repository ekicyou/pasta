// ghost-site.mjs — hello-pasta の nar をネットワーク更新の置き場所にする組み立て・照合ツール
// （hello-pasta-network-update / design「GhostSite」）。Node 22 の標準モジュールだけを使う。
//
// タスク 1.1: readNar（zip の読み出しと名前の安全性）・parseUpdates（updates.txt の解析）。
// タスク 1.2: checkIntegrity（updates.txt とエントリの照合）・extractNar（出力先への展開）。

import zlib from 'node:zlib';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

/** @typedef {{ name: string, data: Buffer }} NarFile */
/** @typedef {{ path: string, md5: string, size: number }} UpdateRecord */
/** @typedef {{ ok: boolean, problems: string[] }} CheckResult */

/** nar が zip でない・名前が危険・形式が非対応のときに投げる。 */
export class NarError extends Error {
  constructor(message) {
    super(message);
    this.name = 'NarError';
  }
}

// 危険な名前: `\`・先頭 `/`・`:`（ドライブ指定）・NUL・空の区切り・`.`/`..` の区切り。
function checkName(name) {
  const bad =
    name === '' ||
    /[\\:\0]/.test(name) ||
    name.startsWith('/') ||
    name.split('/').some((s) => s === '' || s === '.' || s === '..');
  if (bad) throw new NarError(`nar のエントリの名前が不正: ${JSON.stringify(name)}`);
}

/**
 * zip（nar）を読む。ディレクトリのエントリは捨てる。名前が不正・形式が非対応なら NarError を投げる。
 * shortcut: CRC32 は確かめない（md5 照合が上位にある）。zip の整合の検査が要る形式（zip64 など）が入るときに足す。
 * @param {Buffer} buf @returns {NarFile[]}
 */
export function readNar(buf) {
  // 中央ディレクトリの終端（EOCD）を末尾から探す（コメントは最大 65535 バイト）。
  let eocd = -1;
  for (let i = buf.length - 22; i >= Math.max(0, buf.length - 22 - 0xffff); i--) {
    if (buf.readUInt32LE(i) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new NarError('nar が zip でない（中央ディレクトリの終端が無い）');
  const count = buf.readUInt16LE(eocd + 10);
  let p = buf.readUInt32LE(eocd + 16);
  if (count === 0xffff || p === 0xffffffff || (eocd >= 20 && buf.readUInt32LE(eocd - 20) === 0x07064b50)) {
    throw new NarError('非対応の zip: zip64');
  }
  const files = [];
  const seen = new Set();
  for (let n = 0; n < count; n++) {
    if (p + 46 > buf.length || buf.readUInt32LE(p) !== 0x02014b50) {
      throw new NarError('nar が zip でない（中央ディレクトリが壊れている）');
    }
    const flags = buf.readUInt16LE(p + 8);
    const method = buf.readUInt16LE(p + 10);
    const csize = buf.readUInt32LE(p + 20);
    const usize = buf.readUInt32LE(p + 24);
    const nameLen = buf.readUInt16LE(p + 28);
    const extraLen = buf.readUInt16LE(p + 30);
    const commentLen = buf.readUInt16LE(p + 32);
    const offset = buf.readUInt32LE(p + 42);
    const rawName = buf.subarray(p + 46, p + 46 + nameLen).toString('utf8');
    p += 46 + nameLen + extraLen + commentLen;

    const isDir = rawName.endsWith('/');
    const name = isDir ? rawName.slice(0, -1) : rawName;
    checkName(name);
    if (flags & 0x0001) throw new NarError(`非対応の zip: 暗号化（${rawName}）`);
    if (csize === 0xffffffff || usize === 0xffffffff || offset === 0xffffffff) {
      throw new NarError(`非対応の zip: zip64（${rawName}）`);
    }
    if (method !== 0 && method !== 8) throw new NarError(`非対応の zip: 圧縮の方式 ${method}（${rawName}）`);
    if (isDir) continue;
    if (seen.has(name)) throw new NarError(`nar のエントリの名前が重複: ${name}`);
    seen.add(name);

    if (offset + 30 > buf.length || buf.readUInt32LE(offset) !== 0x04034b50) {
      throw new NarError(`nar が zip でない（ローカルヘッダが壊れている: ${name}）`);
    }
    const start = offset + 30 + buf.readUInt16LE(offset + 26) + buf.readUInt16LE(offset + 28);
    const body = buf.subarray(start, start + csize);
    let data;
    try {
      data = method === 8 ? zlib.inflateRawSync(body) : Buffer.from(body);
    } catch (e) {
      throw new NarError(`nar のエントリを展開できない: ${name}（${e.message}）`);
    }
    if (data.length !== usize) throw new NarError(`nar のエントリの大きさが合わない: ${name}`);
    files.push({ name, data });
  }
  return files;
}

const UPDATE_LINE = /^file,([^\x01]+)\x01([0-9a-f]{32})\x01size=(\d+)\x01(?:[^\x01]*\x01)*$/;

/**
 * updates.txt を読む。charset 行と空行は無視、`file,<パス>` 0x01 `<md5>` 0x01 `size=<n>` 0x01 ... の行だけを返す。
 * date= は読むが使わない。解釈できない行は Error。
 * @param {string} text @returns {UpdateRecord[]}
 */
export function parseUpdates(text) {
  const records = [];
  text.split(/\r?\n/).forEach((line, i) => {
    if (line === '' || line.startsWith('charset,')) return;
    const m = UPDATE_LINE.exec(line);
    if (!m) throw new Error(`updates.txt の ${i + 1} 行目を解釈できない: ${JSON.stringify(line)}`);
    records.push({ path: m[1], md5: m[2], size: Number(m[3]) });
  });
  return records;
}

/**
 * ルートの updates.txt の各行をエントリと照合する。updates.txt が無い・解釈できない・行が指すファイルが無い・
 * md5 か大きさが違う、を problems に 1 件 1 行で全部集める。行に載らないエントリ（bal/ 以下・入れ子の updates.txt など）は許す。
 * @param {NarFile[]} files @returns {CheckResult}
 */
export function checkIntegrity(files) {
  const byName = new Map(files.map((f) => [f.name, f]));
  const problems = [];
  const updates = byName.get('updates.txt');
  if (!updates) {
    problems.push('NG updates.txt: nar の直下に無い');
    return { ok: false, problems };
  }
  let records;
  try {
    records = parseUpdates(updates.data.toString('utf8'));
  } catch (e) {
    problems.push(`NG updates.txt: ${e.message}`);
    return { ok: false, problems };
  }
  for (const r of records) {
    try {
      checkName(r.path);
    } catch {
      problems.push(`NG ${r.path}: updates.txt の行の名前が不正`);
      continue;
    }
    const f = byName.get(r.path);
    if (!f) {
      problems.push(`NG ${r.path}: nar に無い`);
      continue;
    }
    const actual = crypto.createHash('md5').update(f.data).digest('hex');
    if (actual !== r.md5) problems.push(`NG ${r.path}: md5 が違う（updates.txt: ${r.md5}、nar: ${actual}）`);
    if (f.data.length !== r.size) problems.push(`NG ${r.path}: 大きさが違う（updates.txt: ${r.size}、nar: ${f.data.length}）`);
  }
  return { ok: problems.length === 0, problems };
}

/**
 * files を除外なしで outDir へ書く。outDir は無いか空でなければならない。
 * 書き始める前に全部の名前を確かめ、outDir の外を指す名前があれば何も書かずに NarError を投げる。
 * @param {NarFile[]} files @param {string} outDir @returns {void}
 */
export function extractNar(files, outDir) {
  const root = path.resolve(outDir);
  if (fs.existsSync(root) && fs.readdirSync(root).length > 0) {
    throw new Error(`出力先が空でない: ${outDir}`);
  }
  const targets = files.map((f) => {
    checkName(f.name);
    const dest = path.resolve(root, f.name);
    if (!dest.startsWith(root + path.sep)) throw new NarError(`出力先の外を指す名前: ${f.name}`);
    return [dest, f.data];
  });
  fs.mkdirSync(root, { recursive: true });
  for (const [dest, data] of targets) {
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.writeFileSync(dest, data);
  }
}
