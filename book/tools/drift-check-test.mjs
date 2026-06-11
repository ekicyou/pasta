// drift-check-test.mjs — drift-check 本実装の自動検証（タスク 4.1 / 要件 10.2, 10.4）。
//
// 検証方針:
//   doc/spec・book/src を恒久変更しないため、検証は
//   (A) 実リポジトリ現状でのクリーン判定（exit 0 相当）と、
//   (B) 一時サンドボックス（tmp ディレクトリへ最小フィクスチャを構築）への
//       ドリフト・未マップ・リンク切れ注入で行う。
//   サンドボックスは runDriftCheck(repoRoot) の repoRoot を差し替えて使う
//   （本物の doc/spec・book/src には一切書き込まない）。
//
// 観測する完了条件（design「Drift Detection & Gate」Testing）:
//   - クリーン（記録ハッシュ=現値・全章マップ・リンク健全）→ failed=false。
//   - doc/spec 改変でハッシュ不一致 → ドリフト検出＆ failed=true。
//   - 未マップ章注入 → 未マップ警告（既定では failed に寄与せず、strict で failed）。
//   - book 内 .md リンク切れ / 存在しない GitHub blob URL → リンク切れ検出＆ failed=true。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  REPO_ROOT,
  REPO_SLUG,
  sha256File,
  parseManualSources,
  detectDrift,
  detectUnmapped,
  detectBrokenLinks,
  extractLinks,
  githubUrlToRepoPath,
  runDriftCheck,
  reportDriftCheck,
} from './drift-check.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));

// --- 最小 assert ハーネス（依存ゼロ） ---
let passed = 0;
let failed = 0;
const log = (...a) => console.log(...a);
function check(name, cond, detail) {
  if (cond) {
    passed++;
    log(`  PASS  ${name}`);
  } else {
    failed++;
    log(`  FAIL  ${name}${detail ? '  -- ' + detail : ''}`);
  }
}

function sha256(str) {
  return crypto.createHash('sha256').update(Buffer.from(str, 'utf8')).digest('hex');
}

// 一時サンドボックスを作る。doc/spec と book/src と book/manual-sources.toml を
// 最小構成で配置する。戻り値はルートパス。
function makeSandbox() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drift-check-'));
  fs.mkdirSync(path.join(root, 'doc', 'spec'), { recursive: true });
  fs.mkdirSync(path.join(root, 'book', 'src', 'grammar'), { recursive: true });
  return root;
}

function writeFile(root, rel, content) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, content, 'utf8');
  return abs;
}

function rmrf(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

// ============================================================
log('\n== (A) 実リポジトリ現状: クリーン判定 ==');
{
  const result = runDriftCheck(REPO_ROOT);
  check('実リポジトリでドリフト 0 件', result.drift.length === 0,
    `drift=${JSON.stringify(result.drift)}`);
  check('実リポジトリで未マップ 0 件（ch08/ch12/README 除外）', result.unmapped.length === 0,
    `unmapped=${JSON.stringify(result.unmapped)}`);
  check('実リポジトリでリンク切れ 0 件', result.broken.length === 0,
    `broken=${JSON.stringify(result.broken)}`);
  check('実リポジトリで failed=false（exit 0 相当）', result.failed === false);
  // レポートが例外なく生成できる。
  check('reportDriftCheck が文字列を返す', typeof reportDriftCheck(result) === 'string');
}

// ============================================================
log('\n== (B-0) TOML パーサ単体 ==');
{
  const toml = [
    '# comment line',
    'algorithm = "sha256"',
    '',
    '[[mapping]]',
    'chapter = "book/src/grammar/markers.md"  # trailing comment',
    'source  = "doc/spec/02-markers.md"',
    'hash    = "deadbeef"',
  ].join('\n');
  const parsed = parseManualSources(toml);
  check('algorithm を読める', parsed.algorithm === 'sha256');
  check('mapping を 1 件読める', parsed.mappings.length === 1);
  check('chapter（行末コメント除去）', parsed.mappings[0].chapter === 'book/src/grammar/markers.md',
    parsed.mappings[0].chapter);
  check('source を読める', parsed.mappings[0].source === 'doc/spec/02-markers.md');
  check('hash を読める', parsed.mappings[0].hash === 'deadbeef');
}

// ============================================================
log('\n== (B-1) クリーン サンドボックス ==');
{
  const root = makeSandbox();
  try {
    const specBody = '# 02 markers\n\n本文。\n';
    writeFile(root, 'doc/spec/02-markers.md', specBody);
    writeFile(root, 'doc/spec/08-attributes.md', '# 08 未実装\n'); // 除外対象
    writeFile(root, 'doc/spec/12-future.md', '# 12 future\n'); // 除外対象
    writeFile(root, 'doc/spec/README.md', '# index\n'); // 除外対象
    writeFile(root, 'book/src/grammar/markers.md',
      'マーカー解説。[block](block-structure.md) '
      + `[spec](https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md)\n`);
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(specBody)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    check('クリーン: ドリフト 0', result.drift.length === 0, JSON.stringify(result.drift));
    check('クリーン: 未マップ 0（除外が効く）', result.unmapped.length === 0,
      JSON.stringify(result.unmapped));
    check('クリーン: リンク切れ 0', result.broken.length === 0, JSON.stringify(result.broken));
    check('クリーン: failed=false', result.failed === false);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-2) ドリフト注入（doc/spec 改変でハッシュ不一致） ==');
{
  const root = makeSandbox();
  try {
    const recordedBody = '# 02 markers\n\nもとの本文。\n';
    // 記録ハッシュは recordedBody のものだが、実ファイルは改変版を置く。
    const changedBody = '# 02 markers\n\n改変された本文（追従が必要）。\n';
    writeFile(root, 'doc/spec/02-markers.md', changedBody);
    writeFile(root, 'book/src/grammar/markers.md', 'マーカー解説。\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(recordedBody)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    check('ドリフト 1 件検出', result.drift.length === 1, JSON.stringify(result.drift));
    check('理由が hash-mismatch', result.drift[0] && result.drift[0].reason === 'hash-mismatch');
    check('ドリフトで failed=true（exit 1 相当）', result.failed === true);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-3) 未マップ章注入 ==');
{
  const root = makeSandbox();
  try {
    const body02 = '# 02\n';
    writeFile(root, 'doc/spec/02-markers.md', body02);
    // マッピングに無い章（網羅対象＝除外対象でない）を追加。
    writeFile(root, 'doc/spec/05-literals.md', '# 05 literals\n');
    writeFile(root, 'book/src/grammar/markers.md', 'マーカー。\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(body02)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    check('未マップ 1 件警告（05-literals）',
      result.unmapped.length === 1 && result.unmapped[0] === 'doc/spec/05-literals.md',
      JSON.stringify(result.unmapped));
    check('未マップのみでは failed=false（既定）', result.failed === false);

    const strictResult = runDriftCheck(root, { strict: true });
    check('strict 指定で未マップが failed=true', strictResult.failed === true);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-4) リンク切れ注入（book 内 .md / GitHub blob URL） ==');
{
  const root = makeSandbox();
  try {
    const body02 = '# 02\n';
    writeFile(root, 'doc/spec/02-markers.md', body02);
    // (a) 存在しない book 内 .md リンク + (b) リポジトリ内に存在しない blob URL。
    writeFile(root, 'book/src/grammar/markers.md',
      '壊れた相対リンク [x](does-not-exist.md) '
      + `壊れた blob [y](https://github.com/${REPO_SLUG}/blob/main/doc/spec/99-nope.md) `
      + '外部リンク [ok](https://example.com/page) ' // 外部はスキップ（健全扱い）
      + '健全相対 [ok2](block-structure.md)\n');
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(body02)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    const kinds = result.broken.map((b) => b.kind).sort();
    check('リンク切れ 2 件検出', result.broken.length === 2, JSON.stringify(result.broken));
    check('内訳: internal-md と github-repo-path',
      JSON.stringify(kinds) === JSON.stringify(['github-repo-path', 'internal-md']),
      JSON.stringify(kinds));
    check('外部リンク・健全相対は誤検出しない',
      !result.broken.some((b) => /example\.com|block-structure/.test(b.target)));
    check('リンク切れで failed=true（exit 1 相当）', result.failed === true);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-5) ユニット: extractLinks / githubUrlToRepoPath ==');
{
  const links = extractLinks('[a](foo.md) ![img](pic.png) [b](<spaced url.md>) [c](http://x/y "t")');
  check('extractLinks: 相対/画像/山括弧/タイトル付きを抽出',
    links.includes('foo.md') && links.includes('pic.png')
    && links.includes('spaced url.md') && links.includes('http://x/y'),
    JSON.stringify(links));

  check('githubUrlToRepoPath: 自リポ blob → 相対パス',
    githubUrlToRepoPath(`https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md`)
    === 'doc/spec/02-markers.md');
  check('githubUrlToRepoPath: 自リポ tree → 相対パス',
    githubUrlToRepoPath(`https://github.com/${REPO_SLUG}/tree/main/doc/spec`)
    === 'doc/spec');
  check('githubUrlToRepoPath: 別リポは null',
    githubUrlToRepoPath('https://github.com/other/repo/blob/main/x.md') === null);
  check('githubUrlToRepoPath: 外部 URL は null',
    githubUrlToRepoPath('https://example.com/doc/spec/02.md') === null);
}

// ============================================================
log('\n== (B-6) 決定論性（同一入力 → 同一結果） ==');
{
  const r1 = runDriftCheck(REPO_ROOT);
  const r2 = runDriftCheck(REPO_ROOT);
  check('2 回実行で同一レポート',
    reportDriftCheck(r1) === reportDriftCheck(r2));
}

// ============================================================
log('\n== (B-7) detectDrift 異常系（incomplete-mapping / missing-source） ==');
{
  const root = makeSandbox();
  try {
    writeFile(root, 'doc/spec/02-markers.md', '# 02\n');

    // hash 欠落 → incomplete-mapping。
    const noHash = {
      algorithm: 'sha256',
      mappings: [{ chapter: 'book/src/grammar/markers.md', source: 'doc/spec/02-markers.md' }],
    };
    const d1 = detectDrift(noHash, root);
    check('hash 欠落で incomplete-mapping',
      d1.length === 1 && d1[0].reason === 'incomplete-mapping', JSON.stringify(d1));

    // source 欠落 → incomplete-mapping。
    const noSrc = {
      algorithm: 'sha256',
      mappings: [{ chapter: 'book/src/grammar/markers.md', hash: 'deadbeef' }],
    };
    const d2 = detectDrift(noSrc, root);
    check('source 欠落で incomplete-mapping',
      d2.length === 1 && d2[0].reason === 'incomplete-mapping', JSON.stringify(d2));

    // source ファイル不在 → missing-source。
    const gone = {
      algorithm: 'sha256',
      mappings: [{
        chapter: 'book/src/grammar/markers.md',
        source: 'doc/spec/00-gone.md',
        hash: 'deadbeef',
      }],
    };
    const d3 = detectDrift(gone, root);
    check('source 不在で missing-source',
      d3.length === 1 && d3[0].reason === 'missing-source', JSON.stringify(d3));

    // 端到端: source 不在の mapping を含む toml → runDriftCheck で failed=true。
    writeFile(root, 'book/src/grammar/markers.md', 'マーカー。\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/00-gone.md"',
      'hash    = "deadbeef"',
    ].join('\n'));
    const r = runDriftCheck(root);
    check('missing-source で failed=true（exit 1 相当）',
      r.failed === true && r.drift.some((d) => d.reason === 'missing-source'),
      JSON.stringify(r.drift));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-8) sha256File 改行正規化（CRLF/LF/CR → 同一ハッシュ） ==');
{
  const root = makeSandbox();
  try {
    const lf = writeFile(root, 'lf.md', '# t\nline\n');
    const crlf = writeFile(root, 'crlf.md', '# t\r\nline\r\n');
    const cr = writeFile(root, 'cr.md', '# t\rline\r');
    const other = writeFile(root, 'other.md', '# t\nline2\n');
    check('CRLF と LF が同一ハッシュ（改行差はドリフトでない）',
      sha256File(crlf) === sha256File(lf));
    check('単独 CR も LF と同一ハッシュ（\\r\\n? 正規化）',
      sha256File(cr) === sha256File(lf));
    check('内容差は別ハッシュ', sha256File(other) !== sha256File(lf));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-9) TOML パーサ端ケース（文字列内 # / エスケープ / 複数 mapping） ==');
{
  const toml = [
    'algorithm = "sha256"',
    '',
    '[[mapping]]',
    'chapter = "doc#frag.md"', // 文字列内の # はコメントでない
    'source  = "with \\"quote\\".md"', // エスケープ済み引用符
    'hash    = "a\\\\b"', // エスケープ済みバックスラッシュ
    '',
    '[[mapping]]',
    'chapter = "c2"',
    'source  = "s2"',
    'hash    = "h2"',
  ].join('\n');
  const parsed = parseManualSources(toml);
  check('mapping を 2 件読める', parsed.mappings.length === 2,
    JSON.stringify(parsed.mappings));
  check('文字列内の # を保持（コメント扱いしない）',
    parsed.mappings[0].chapter === 'doc#frag.md', parsed.mappings[0].chapter);
  check('エスケープ済み引用符をデコード',
    parsed.mappings[0].source === 'with "quote".md', parsed.mappings[0].source);
  check('エスケープ済みバックスラッシュをデコード',
    parsed.mappings[0].hash === 'a\\b', parsed.mappings[0].hash);
  check('2 件目の mapping も読める',
    parsed.mappings[1].chapter === 'c2' && parsed.mappings[1].source === 's2'
    && parsed.mappings[1].hash === 'h2');
}

// ============================================================
log('\n== (B-10) detectUnmapped: doc/spec ディレクトリ不在 → 空配列 ==');
{
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drift-check-'));
  try {
    const unmapped = detectUnmapped({ algorithm: 'sha256', mappings: [] }, root);
    check('doc/spec 不在で空配列（例外なし）',
      Array.isArray(unmapped) && unmapped.length === 0, JSON.stringify(unmapped));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-11) リンクのフラグメント/クエリ許容（誤検出なし） ==');
{
  const root = makeSandbox();
  try {
    const body02 = '# 02\n';
    writeFile(root, 'doc/spec/02-markers.md', body02);
    writeFile(root, 'book/src/grammar/markers.md',
      'アンカー付き [a](block-structure.md#sec) '
      + 'クエリ付き [b](block-structure.md?x=1) '
      + `blob フラグメント [c](https://github.com/${REPO_SLUG}/blob/main/doc/spec/02-markers.md#L10)\n`);
    writeFile(root, 'book/src/grammar/block-structure.md', '# block\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(body02)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    check('フラグメント/クエリ付き実在リンクは壊れ扱いしない',
      result.broken.length === 0, JSON.stringify(result.broken));
    check('failed=false', result.failed === false);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-12) reportDriftCheck の分岐（FAIL / 未マップ警告付き OK） ==');
{
  // hash-mismatch を含む失敗レポート。
  const failRep = reportDriftCheck({
    manualSources: { algorithm: 'sha256', mappings: [{}] },
    drift: [{
      chapter: 'book/src/grammar/markers.md',
      source: 'doc/spec/02-markers.md',
      reason: 'hash-mismatch',
      recorded: 'aaaa',
      current: 'bbbb',
      detail: '由来 doc/spec が変更されたのに章が未追従（ドリフト）',
    }],
    unmapped: [],
    broken: [],
    failed: true,
  });
  check('失敗レポートに RESULT: FAIL', failRep.includes('RESULT: FAIL'));
  check('hash-mismatch で recorded/current を表示',
    failRep.includes('recorded=aaaa') && failRep.includes('current =bbbb'));
  check('DRIFT 行に章と source を表示',
    /DRIFT\s+book\/src\/grammar\/markers\.md <- doc\/spec\/02-markers\.md/.test(failRep));

  // 未マップ警告のみの OK レポート。
  const warnRep = reportDriftCheck({
    manualSources: { algorithm: 'sha256', mappings: [] },
    drift: [],
    unmapped: ['doc/spec/05-literals.md'],
    broken: [],
    failed: false,
  });
  check('未マップのみは警告付き OK', /RESULT: OK（ただし未マップ章の警告あり/.test(warnRep));
  check('UNMAPPED 行を表示', warnRep.includes('UNMAPPED  doc/spec/05-literals.md'));
}

// ============================================================
log('\n== (B-13) CLI 結線（exit code / RESULT 出力 / DRIFT_STRICT） ==');
{
  const cliPath = path.join(here, 'drift-check.mjs');
  const r = spawnSync(process.execPath, [cliPath], { encoding: 'utf8', timeout: 60000 });
  check('CLI: クリーンな実リポジトリで exit 0', r.status === 0, `status=${r.status}`);
  check('CLI: RESULT: OK を出力', /RESULT: OK/.test(r.stdout || ''),
    (r.stdout || '').slice(-200));

  // DRIFT_STRICT=1 でも実リポジトリ（未マップ 0）では exit 0（strict 結線の煙テスト）。
  const rs = spawnSync(process.execPath, [cliPath], {
    encoding: 'utf8',
    timeout: 60000,
    env: { ...process.env, DRIFT_STRICT: '1' },
  });
  check('CLI: DRIFT_STRICT=1 でも未マップ 0 なら exit 0', rs.status === 0,
    `status=${rs.status}`);
}

// ============================================================
log('\n== (B-14) パストラバーサル拒否（repoRoot 外を指すリンクは実在しても broken） ==');
// ハードニング境界（cell 3.59 / R3.2, R3.3）:
//   `..` 等で repoRoot の外へ脱出するリンク（blob URL・相対 .md とも）は、
//   解決先が実在しても「リンク切れ」として報告する（リポジトリ外の存在プローブ防止）。
//   一方、repoRoot 「内」に留まる `..` 相対参照は従来どおり実在チェックのみ（正常系不変）。
{
  // サンドボックスを入れ子にし、repoRoot（= outer/repo）の外に実在ファイルを置く。
  const outer = fs.mkdtempSync(path.join(os.tmpdir(), 'drift-check-'));
  const root = path.join(outer, 'repo');
  try {
    fs.mkdirSync(path.join(root, 'doc', 'spec'), { recursive: true });
    fs.mkdirSync(path.join(root, 'book', 'src', 'grammar'), { recursive: true });
    writeFile(outer, 'outside.md', '# repoRoot の外に実在するファイル\n');

    const body02 = '# 02\n';
    writeFile(root, 'doc/spec/02-markers.md', body02);
    writeFile(root, 'book/src/grammar/markers.md',
      // (a) blob URL の `..` 脱出（解決先は実在する）。
      `[esc-blob](https://github.com/${REPO_SLUG}/blob/main/../outside.md) `
      // (b) 相対 .md リンクの `..` 脱出（grammar→src→book→repo→outer。解決先は実在する）。
      + '[esc-rel](../../../../outside.md) '
      // (c) repoRoot 内に留まる `..` 相対参照（実在）→ 従来どおり健全。
      + '[ok-up](../../../doc/spec/02-markers.md)\n');
    writeFile(root, 'book/manual-sources.toml', [
      'algorithm = "sha256"',
      '',
      '[[mapping]]',
      'chapter = "book/src/grammar/markers.md"',
      'source  = "doc/spec/02-markers.md"',
      `hash    = "${sha256(body02)}"`,
    ].join('\n'));

    const result = runDriftCheck(root);
    const escBlob = result.broken.find(
      (b) => b.kind === 'github-repo-path' && /トラバーサル/.test(b.detail));
    const escRel = result.broken.find(
      (b) => b.kind === 'internal-md' && /トラバーサル/.test(b.detail));
    check('blob URL の repoRoot 脱出を実在でも broken 扱い', !!escBlob,
      JSON.stringify(result.broken));
    check('相対 .md の repoRoot 脱出を実在でも broken 扱い', !!escRel,
      JSON.stringify(result.broken));
    check('repoRoot 内の .. 相対参照（実在）は誤検出しない（境界の内側は不変）',
      !result.broken.some((b) => /ok-up|02-markers/.test(b.target)),
      JSON.stringify(result.broken));
    check('トラバーサル検出 2 件のみ', result.broken.length === 2,
      JSON.stringify(result.broken));
    check('トラバーサルで failed=true（exit 1 相当）', result.failed === true);
  } finally {
    rmrf(outer);
  }
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
