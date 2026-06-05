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
import { fileURLToPath } from 'node:url';
import {
  REPO_ROOT,
  REPO_SLUG,
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
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
