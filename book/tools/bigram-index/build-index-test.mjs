// build-index-test.mjs — bigram 索引ビルダ本実装の自動検証（タスク 3.1 / 要件 2.4）。
//
// 目的:
//   本番フルコンテンツ（book/src 配下に執筆済みの introduction / getting-started /
//   grammar / lua / reference 全章）の mdBook 出力に対し、ビルダが searchindex を
//   2-gram 索引へ再生成し、語中 2 文字検索がヒットすることを実機で証明する。
//
//   - RED  : mdBook 標準索引（空白/語頭区切り）では語中 2 文字（例「ファレ」=
//            リファレンスの語中）がヒットしない。
//   - GREEN: ビルダ実行後の bigram 索引 ＋ 共有 tokenize でクエリすると、
//            語中 2 文字が当該語を含むページにヒットする。
//   - 索引サイズが 10MB 閾値未満であることを実測出力する。
//   - 再生成が決定論的（同一入力 → 同一バイト列）であることを確認する。
//
// 一連の `mdbook build book` → ビルダ → 本テスト が exit 0 となることを目指す。
// ブラウザ searcher.js への注入はタスク 3.2 の担当ゆえ、本テストは Node 上で
// mdBook 検索 UI と同一のクエリオプションを再現して 2.4 の成立を立証する。

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { tokenize } from './tokenize.mjs';
import {
  resolveHashed,
  readSearchIndex,
  rebuildBigramIndex,
  serializeSearchIndex,
  buildBigramIndex,
  SIZE_WARN_THRESHOLD_BYTES,
} from './build-index.mjs';

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '../../..');
const bookDir = path.resolve(repoRoot, 'book'); // book.toml のあるディレクトリ
const bookOut = path.resolve(bookDir, 'book'); // mdbook build 出力

// 本番 mdBook 出力が無ければここでビルドする（出力は gitignore、コミット不要）。
function ensureBookBuilt() {
  const built = fs.existsSync(bookOut) &&
    fs.readdirSync(bookOut).some((n) => /^searchindex-.*\.js$/.test(n));
  if (built) return;
  console.log('book out not found; running `mdbook build book`...');
  execFileSync('mdbook', ['build', bookDir], { stdio: 'inherit' });
}
ensureBookBuilt();

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

// mdBook 標準検索 UI と同一のクエリオプション（searcher.js 由来）。
const SEARCH_OPTIONS = {
  bool: 'AND',
  expand: true,
  fields: {
    title: { boost: 2 },
    body: { boost: 1 },
    breadcrumbs: { boost: 1 },
  },
};

function searchUrls(elasticlunr, indexObj, query) {
  const idx = elasticlunr.Index.load(indexObj.index);
  const results = idx.search(query, SEARCH_OPTIONS);
  return results.map((r) => indexObj.doc_urls[Number(r.ref)]);
}

// 索引内の全ドキュメント本文＋タイトルを連結したコーパス。
function corpusOf(indexObj) {
  const docs = indexObj.index.documentStore.docs;
  return Object.values(docs)
    .map((d) => `${d.title || ''} ${d.body || ''} ${d.breadcrumbs || ''}`)
    .join(' ');
}

// =========================================================================
// フィクスチャ取得（本番フルコンテンツ）
// =========================================================================
const elasticlunrFile = resolveHashed(bookOut, /^elasticlunr-.*\.min\.js$/);
const searchIndexFile = resolveHashed(bookOut, /^searchindex-.*\.js$/);
const elasticlunr = require(elasticlunrFile);
const original = readSearchIndex(searchIndexFile);

log('book out:', bookOut);
log('  elasticlunr:', path.basename(elasticlunrFile));
log('  searchindex:', path.basename(searchIndexFile));
log('  doc count   :', original.doc_urls.length);
log('');

// 全 5 章が索引に含まれていること（本番フルコンテンツの担保）。
{
  const sections = new Set(
    original.doc_urls.map((u) => (u || '').split('/')[0].split('.')[0]),
  );
  for (const sec of ['introduction', 'getting-started', 'grammar', 'lua', 'reference']) {
    check(`fixture: 章「${sec}」が索引に含まれる`, sections.has(sec),
      `sections=${JSON.stringify([...sections])}`);
  }
}
log('');

// 語中 2 文字テストケース: [語中bigram, 親語, ヒット判定の章接頭辞リスト]。
// いずれも本番コンテンツに実在し、かつ語頭ではなく語中に現れる 2 文字。
const CASES = [
  { mid: 'ファレ', word: 'リファレンス' },
  { mid: 'クショ', word: 'アクション' },
  { mid: 'ベント', word: 'イベント' },
];

// 母語が実際にコーパスに存在することを前提確認（テストの妥当性担保）。
{
  const corpus = corpusOf(original);
  for (const c of CASES) {
    check(`fixture: 親語「${c.word}」がコーパスに存在`, corpus.includes(c.word));
    check(`fixture: 語中片「${c.mid}」がコーパスに存在`, corpus.includes(c.mid));
  }
}
log('');

// =========================================================================
// RED PHASE: 標準索引では語中 2 文字がヒットしない
// =========================================================================
log('--- RED PHASE (mdBook 標準索引 / 語中2文字クエリ) ---');
{
  const savedTok = elasticlunr.tokenizer; // デフォルトのまま（ブラウザ既定相当）
  elasticlunr.tokenizer = savedTok;
  for (const c of CASES) {
    const urls = searchUrls(elasticlunr, original, c.mid);
    log(`  query "${c.mid}" -> hits: ${urls.length}`);
    check(`RED: 標準索引では語中片「${c.mid}」がヒットしない`, urls.length === 0,
      `urls=${JSON.stringify(urls.slice(0, 5))}`);
  }
}
log('');

// =========================================================================
// GREEN PHASE: bigram 再生成索引 + 共有 tokenize で語中 2 文字がヒット
// =========================================================================
log('--- GREEN PHASE (bigram 再生成索引 + 共有 tokenize クエリ) ---');
let rebuilt;
{
  rebuilt = rebuildBigramIndex(elasticlunr, original);

  const savedTok = elasticlunr.tokenizer;
  elasticlunr.tokenizer = (s) => tokenize(s);
  try {
    for (const c of CASES) {
      const midUrls = searchUrls(elasticlunr, rebuilt, c.mid);
      const wordUrls = searchUrls(elasticlunr, rebuilt, c.word);
      log(`  query "${c.mid}" -> ${midUrls.length} hits; "${c.word}" -> ${wordUrls.length} hits`);
      check(`GREEN: 語中片「${c.mid}」がヒットする`, midUrls.length > 0,
        `urls=${JSON.stringify(midUrls.slice(0, 5))}`);
      check(`GREEN: 親語「${c.word}」がヒットする`, wordUrls.length > 0,
        `urls=${JSON.stringify(wordUrls.slice(0, 5))}`);
      // 語中片のヒット集合は親語のヒット集合を包含するはず（同一語起源）。
      const wordSet = new Set(wordUrls);
      check(`GREEN: 「${c.mid}」のヒットが親語「${c.word}」のヒットを包含`,
        wordUrls.length > 0 && [...wordSet].every((u) => midUrls.includes(u)),
        `mid=${JSON.stringify(midUrls.slice(0, 5))} word=${JSON.stringify(wordUrls.slice(0, 5))}`);
    }
  } finally {
    elasticlunr.tokenizer = savedTok;
  }
}
log('');

// =========================================================================
// SIZE / DETERMINISM: 索引サイズ閾値・決定論性
// =========================================================================
log('--- SIZE & DETERMINISM ---');
{
  const s1 = serializeSearchIndex(rebuilt);
  const bytes = Buffer.byteLength(s1, 'utf8');
  const mb = (bytes / 1024 / 1024).toFixed(3);
  log(`  bigram index size: ${bytes} bytes (${mb} MB), threshold ${SIZE_WARN_THRESHOLD_BYTES} bytes (10 MB)`);
  check('SIZE: 索引サイズが 10MB 閾値未満', bytes < SIZE_WARN_THRESHOLD_BYTES,
    `${bytes} >= ${SIZE_WARN_THRESHOLD_BYTES}`);

  // 決定論性: 同一 original から 2 回再構築 → 同一バイト列。
  const rebuilt2 = rebuildBigramIndex(elasticlunr, original);
  const s2 = serializeSearchIndex(rebuilt2);
  check('DETERMINISM: 再構築は決定論的（同一入力→同一バイト列）', s1 === s2,
    `len1=${s1.length} len2=${s2.length}`);
}
log('');

// =========================================================================
// ROBUST EXTRACT: 上書き済みファイルを再パースできる（冪等・ラウンドトリップ）
// =========================================================================
log('--- ROUND-TRIP (write -> re-read) ---');
{
  // 実ファイルへ書き出してから読み戻す（buildBigramIndex の write 経路）。
  const res = buildBigramIndex(bookOut, { write: true });
  log(`  written: ${path.basename(res.searchIndexFile)} size=${res.sizeBytes} overThreshold=${res.overThreshold}`);
  check('ROUND-TRIP: buildBigramIndex の報告サイズが閾値未満', !res.overThreshold,
    `size=${res.sizeBytes}`);

  // 書き戻したファイルを robust extract で再パースできること。
  const reread = readSearchIndex(res.searchIndexFile);
  check('ROUND-TRIP: 上書きファイルを再パースでき doc 数が一致',
    reread.doc_urls.length === original.doc_urls.length,
    `re=${reread.doc_urls.length} orig=${original.doc_urls.length}`);

  // 再パース索引でも語中片がヒットすること（実ファイル経由の最終確認）。
  const savedTok = elasticlunr.tokenizer;
  elasticlunr.tokenizer = (s) => tokenize(s);
  try {
    const urls = searchUrls(elasticlunr, reread, CASES[0].mid);
    check(`ROUND-TRIP: 実ファイル索引で語中片「${CASES[0].mid}」がヒット`, urls.length > 0,
      `urls=${JSON.stringify(urls.slice(0, 5))}`);
  } finally {
    elasticlunr.tokenizer = savedTok;
  }
}
log('');

log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
