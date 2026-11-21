// ghost-site.mjs — hello-pasta の nar をネットワーク更新の置き場所にする組み立て・照合ツール。
// Node 22 の標準モジュールだけを使う。

import zlib from 'node:zlib';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { pathToFileURL } from 'node:url';

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

/**
 * 読み出し → 照合 → index.html の衝突の確認 → 展開 → index.html の複写 → 出力の一覧が nar のエントリ＋index.html と一致することの確認。
 * 照合・衝突の失敗は書き出しの前に Error（message は問題を 1 件 1 行）を投げ、outDir に何も書かない。
 * @param {{ narPath: string, outDir: string, indexPath: string }} opts @returns {void}
 */
export function buildSite({ narPath, outDir, indexPath }) {
  const files = readNar(fs.readFileSync(narPath));
  const r = checkIntegrity(files);
  if (!r.ok) throw new Error(r.problems.join('\n'));
  if (files.some((f) => f.name === 'index.html')) {
    throw new Error('NG index.html: nar にも index.html があり、入口と衝突する');
  }
  const index = fs.readFileSync(indexPath);
  extractNar(files, outDir);
  fs.writeFileSync(path.join(outDir, 'index.html'), index);

  const expected = new Set([...files.map((f) => f.name), 'index.html']);
  const actual = new Set(
    fs
      .readdirSync(outDir, { recursive: true, withFileTypes: true })
      .filter((d) => d.isFile())
      .map((d) => path.relative(outDir, path.join(d.parentPath, d.name)).split(path.sep).join('/')),
  );
  const diff = [
    ...[...expected].filter((n) => !actual.has(n)).map((n) => `NG ${n}: 出力に無い`),
    ...[...actual].filter((n) => !expected.has(n)).map((n) => `NG ${n}: nar に無いファイルが出力にある`),
  ];
  if (diff.length > 0) throw new Error(diff.join('\n'));
}

const md5Of = (buf) => crypto.createHash('md5').update(buf).digest('hex');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// 1 回取る。HTTP の状態と中身、またはネットワークの失敗を返す（投げない）。
async function fetchOnce(fetchFn, url) {
  try {
    const res = await fetchFn(url);
    return { status: res.status, body: Buffer.from(await res.arrayBuffer()) };
  } catch (e) {
    return { error: e.cause?.message ? `${e.message}: ${e.cause.message}` : e.message };
  }
}
const stateOf = (r) => (r.error ? `取得できない（${r.error}）` : `HTTP ${r.status}・長さ ${r.body.length}・md5 ${md5Of(r.body)}`);

/**
 * 実 URL の updates.txt を nar の updates.txt とバイト一致で見比べる。一致するまで attempts 回まで intervalMs おきに取り直す
 * （HTTP 200 以外・ネットワークの失敗も取り直す。最後の回の後は待たない）。取得には一意のクエリ cb=<ラベル>-<回数> を付ける。
 * full なら、一致の後に updates.txt の各行のファイルを 1 回ずつ取って md5・大きさを照合し、問題を 1 件 1 行で全部集める。
 * 戻り値の attempts は updates.txt を取った回数（一致した回、諦めたときは上限。nar に updates.txt が無ければ 0）。
 * @param {{ files: NarFile[], base: string, attempts: number, intervalMs: number, full: boolean, fetchFn?: typeof fetch }} opts
 * @returns {Promise<CheckResult & { attempts: number }>}
 */
export async function verifyLive({ files, base, attempts, intervalMs, full, fetchFn = fetch }) {
  if (!(Number.isInteger(attempts) && attempts >= 1)) throw new Error(`attempts は 1 以上の整数: ${attempts}`);
  const local = files.find((f) => f.name === 'updates.txt');
  if (!local) return { ok: false, problems: ['NG updates.txt: nar の直下に無い'], attempts: 0 };
  const root = base.endsWith('/') ? base : `${base}/`;
  const { GITHUB_RUN_ID: id, GITHUB_RUN_ATTEMPT: at } = process.env;
  // 再実行（同じ run ID）でも別のクエリになるよう試行の番号も入れる。
  const label = id && at ? `${id}.${at}` : id || Date.now().toString(36);
  const urlOf = (p, cb) => {
    const u = new URL(p.split('/').map(encodeURIComponent).join('/'), root);
    u.searchParams.set('cb', `${label}-${cb}`);
    return u.href;
  };

  let last;
  let n = 1;
  for (; n <= attempts; n++) {
    last = await fetchOnce(fetchFn, urlOf('updates.txt', n));
    if (last.status === 200 && last.body.equals(local.data)) break;
    last.mismatch = true;
    if (n < attempts) await sleep(intervalMs);
  }
  if (last.mismatch) {
    return {
      ok: false,
      problems: [`NG updates.txt: ${attempts} 回取っても nar と一致しない（最後: ${stateOf(last)}、nar: 長さ ${local.data.length}・md5 ${md5Of(local.data)}）`],
      attempts,
    };
  }
  if (!full) return { ok: true, problems: [], attempts: n };

  let records;
  try {
    records = parseUpdates(local.data.toString('utf8'));
  } catch (e) {
    return { ok: false, problems: [`NG updates.txt: ${e.message}`], attempts: n };
  }
  const problems = [];
  for (const r of records) {
    const got = await fetchOnce(fetchFn, urlOf(r.path, 'full'));
    if (got.status !== 200) {
      problems.push(`NG ${r.path}: ${stateOf(got)}`);
      continue;
    }
    const actual = md5Of(got.body);
    if (actual !== r.md5) problems.push(`NG ${r.path}: md5 が違う（updates.txt: ${r.md5}、実 URL: ${actual}）`);
    if (got.body.length !== r.size) problems.push(`NG ${r.path}: 大きさが違う（updates.txt: ${r.size}、実 URL: ${got.body.length}）`);
  }
  return { ok: problems.length === 0, problems, attempts: n };
}

const USAGE = `使い方:
  node ghost-site.mjs build --nar <file> --out <dir> --index <html>
  node ghost-site.mjs verify-live --nar <file> --base <url> [--attempts 30] [--interval 30] [--full]
    （--interval は秒）`;

const usage = (msg) => {
  console.error(msg ? `${msg}\n${USAGE}` : USAGE);
  return 1;
};

// 終了コード 0＝成功、1＝失敗（引数の誤りも 1）。問題は 1 行 1 件で標準出力、GitHub Actions では ::error:: 注釈も出す。
async function main(argv) {
  const [cmd, ...rest] = argv;
  let values;
  try {
    ({ values } = parseArgs({
      args: rest,
      options: {
        nar: { type: 'string' },
        out: { type: 'string' },
        index: { type: 'string' },
        base: { type: 'string' },
        attempts: { type: 'string', default: '30' },
        interval: { type: 'string', default: '30' },
        full: { type: 'boolean', default: false },
      },
    }));
  } catch (e) {
    return usage(e.message);
  }
  let run;
  if (cmd === 'build' && values.nar && values.out && values.index) {
    run = async () => buildSite({ narPath: values.nar, outDir: values.out, indexPath: values.index });
  } else if (cmd === 'verify-live' && values.nar && values.base) {
    const attempts = Number(values.attempts);
    const interval = Number(values.interval);
    if (!Number.isInteger(attempts) || attempts < 1) return usage(`--attempts は 1 以上の整数: ${values.attempts}`);
    if (!Number.isInteger(interval) || interval < 0) return usage(`--interval は 0 以上の整数（秒）: ${values.interval}`);
    run = async () => {
      const files = readNar(fs.readFileSync(values.nar));
      const r = await verifyLive({ files, base: values.base, attempts, intervalMs: interval * 1000, full: values.full });
      if (!r.ok) throw new Error(r.problems.join('\n'));
      const listed = values.full ? parseUpdates(files.find((f) => f.name === 'updates.txt').data.toString('utf8')).length : 0;
      console.log(`OK updates.txt: ${r.attempts} 回目で一致${values.full ? `（--full: ${listed} ファイルも一致）` : ''}`);
    };
  } else {
    return usage();
  }
  try {
    await run();
    return 0;
  } catch (e) {
    for (const line of e.message.split('\n')) {
      console.log(line);
      if (process.env.GITHUB_ACTIONS === 'true') console.log(`::error::${line}`);
    }
    return 1;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = await main(process.argv.slice(2));
}
