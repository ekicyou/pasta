// poc-test.mjs — 日本語 bigram 検索 PoC の自動検証（要件 2.4）。
//
// 目的（tasks.md 1.2）:
//   フィクスチャ「構造体の章」に対し、語中 2 文字クエリ「造体」で当該ページがヒットすることを
//   自動テストで証明する。
//     - RED : bigram 再生成前の mdBook 標準索引では「造体」がヒットしない（語中部分一致不可）。
//     - GREEN: 共有 tokenize.mjs で bigram 再生成した索引＋同 tokenize でクエリすると、
//             「造体」「構造体」いずれも当該ページにヒットする。
//
// 検証は mdBook 標準検索 UI と同じ呼び出し（elasticlunr.Index.search + AND/expand オプション）を
// Node 上で再現する。ブラウザ searcher.js への注入は本実装タスク 3.2 の担当ゆえ、
// PoC では Node 検証で 2.4 の成立を立証する。

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
} from './build-index.mjs';

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url));
const pocDir = path.resolve(here, 'poc');
const pocBookOut = path.resolve(pocDir, 'book');

// PoC フィクスチャの mdBook 出力が無ければここでビルドする（出力はコミット不要）。
function ensurePocBuilt() {
  const built = fs.existsSync(pocBookOut) &&
    fs.readdirSync(pocBookOut).some((n) => /^searchindex-.*\.js$/.test(n));
  if (built) return;
  console.log('PoC fixture not built; running `mdbook build`...');
  execFileSync('mdbook', ['build', pocDir], { stdio: 'inherit' });
}
ensurePocBuilt();

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

// 索引を search し、ヒットした doc_urls の集合（先頭=パス部分）を返す。
function searchUrls(elasticlunr, indexObj, query) {
  const idx = elasticlunr.Index.load(indexObj.index);
  const results = idx.search(query, SEARCH_OPTIONS);
  return results.map((r) => indexObj.doc_urls[Number(r.ref)]);
}

function hitsStructure(urls) {
  return urls.some((u) => u && u.startsWith('structure.html'));
}

// --- フィクスチャ存在チェック ---
const elasticlunrFile = resolveHashed(pocBookOut, /^elasticlunr-.*\.min\.js$/);
const searchIndexFile = resolveHashed(pocBookOut, /^searchindex-.*\.js$/);
const elasticlunr = require(elasticlunrFile);
const original = readSearchIndex(searchIndexFile);

log('PoC fixture book out:', pocBookOut);
log('  elasticlunr:', path.basename(elasticlunrFile));
log('  searchindex:', path.basename(searchIndexFile));
log('  doc_urls:', JSON.stringify(original.doc_urls));
log('');

// =========================================================================
// RED PHASE: bigram 再生成前の標準索引で「造体」がヒットしないことを捕捉
// =========================================================================
log('--- RED PHASE (mdBook 標準索引 / 語中2文字クエリ) ---');
{
  // 標準索引はビルド時に空白区切りでトークン化されている。
  // クエリ側もデフォルト tokenizer（空白/ハイフン区切り）で検索する＝ブラウザ既定相当。
  const savedTok = elasticlunr.tokenizer;
  elasticlunr.tokenizer = savedTok; // 明示: デフォルトのまま
  const urlsCenter = searchUrls(elasticlunr, original, '造体'); // 語中2文字
  const urlsFull = searchUrls(elasticlunr, original, '構造体'); // 完全語
  log('  query "造体"  -> hits:', JSON.stringify(urlsCenter));
  log('  query "構造体" -> hits:', JSON.stringify(urlsFull));
  // RED の核心: 語中2文字「造体」では当該ページがヒットしない（要件 2.4 が標準では未達）。
  check('RED: 標準索引では「造体」が構造体ページにヒットしない', !hitsStructure(urlsCenter),
    `urls=${JSON.stringify(urlsCenter)}`);
}
log('');

// =========================================================================
// GREEN PHASE: bigram 再生成索引 + 共有 tokenize で「造体」がヒット
// =========================================================================
log('--- GREEN PHASE (bigram 再生成索引 + 共有 tokenize クエリ) ---');
{
  const rebuilt = rebuildBigramIndex(elasticlunr, original);

  // クエリ側も索引と同一の bigram tokenize を用いる（searcher.js が担う処理を再現）。
  const savedTok = elasticlunr.tokenizer;
  elasticlunr.tokenizer = (s) => tokenize(s);
  let urlsCenter, urlsFull, urlsOther;
  try {
    urlsCenter = searchUrls(elasticlunr, rebuilt, '造体'); // 語中2文字
    urlsFull = searchUrls(elasticlunr, rebuilt, '構造体'); // 完全語
    urlsOther = searchUrls(elasticlunr, rebuilt, '挨拶'); // 別章のみの語
  } finally {
    elasticlunr.tokenizer = savedTok;
  }
  log('  query "造体"  -> hits:', JSON.stringify(urlsCenter));
  log('  query "構造体" -> hits:', JSON.stringify(urlsFull));
  log('  query "挨拶"  -> hits:', JSON.stringify(urlsOther));

  check('GREEN: bigram 索引で「造体」が構造体ページにヒットする', hitsStructure(urlsCenter),
    `urls=${JSON.stringify(urlsCenter)}`);
  check('GREEN: bigram 索引で「構造体」が構造体ページにヒットする', hitsStructure(urlsFull),
    `urls=${JSON.stringify(urlsFull)}`);
  // ネガティブ確認: 「挨拶」（構造体章に無い語）は構造体ページにヒットしない（誤ヒット抑止）。
  check('GREEN: 「挨拶」は構造体ページにヒットしない（精度確認）', !hitsStructure(urlsOther),
    `urls=${JSON.stringify(urlsOther)}`);
  // 「挨拶」は別章にはヒットすること（索引が機能している保証）。
  check('GREEN: 「挨拶」は別章(other)にヒットする（索引健全性）',
    urlsOther.some((u) => u && u.startsWith('other.html')),
    `urls=${JSON.stringify(urlsOther)}`);
}
log('');

// --- tokenize ユニット確認（索引・クエリ規則の正準性） ---
log('--- tokenize unit checks ---');
check('tokenize("構造体") == ["構造","造体"]',
  JSON.stringify(tokenize('構造体')) === JSON.stringify(['構造', '造体']),
  JSON.stringify(tokenize('構造体')));
check('tokenize("造体") == ["造体"]',
  JSON.stringify(tokenize('造体')) === JSON.stringify(['造体']),
  JSON.stringify(tokenize('造体')));
check('tokenize("Lua API") は ASCII 語を保持',
  JSON.stringify(tokenize('Lua API')) === JSON.stringify(['lua', 'api']),
  JSON.stringify(tokenize('Lua API')));

// --- tokenize 境界・正規化の単体検証（3.63 G1 追加） ---
// 索引・クエリ双方の正準規則（head.hbs ミラーと同一）を網羅的に固定する。
function checkTokens(input, expected, label) {
  const got = tokenize(input);
  check(`tokenize ${label}`,
    JSON.stringify(got) === JSON.stringify(expected), JSON.stringify(got));
}
checkTokens(null, [], 'null -> []（null ガード）');
checkTokens(undefined, [], 'undefined -> []（undefined ガード）');
checkTokens('', [], '空文字列 -> []');
checkTokens(123, ['123'], '数値 123 は String 化して語トークン');
checkTokens('猫', ['猫'], '単独 CJK 1 文字はそのまま 1 トークン');
checkTokens('a猫b', ['a', '猫', 'b'], 'ASCII に挟まれた単独 CJK の分割');
checkTokens('ＰＡＳＴＡ ０１', ['pasta', '01'], '全角英数の半角化＋小文字化');
checkTokens('ｱｲｳ', ['ｱｲ', 'ｲｳ'], '半角カタカナも bigram 化');
checkTokens('サーバー', ['サー', 'ーバ', 'バー'], '長音記号 ー は CJK ランに連結');
checkTokens('spec-driven', ['spec', 'driven'], 'ハイフンは語区切り');
checkTokens('foo\tbar\nbaz', ['foo', 'bar', 'baz'], 'タブ・改行も空白区切り');
checkTokens('構造体abc', ['構造', '造体', 'abc'], 'CJK/ASCII 境界でラン分離');
checkTokens('㐀㐁', ['㐀㐁'], 'CJK 拡張A（BMP 内）は bigram 化');
// 特性固定: CJK 拡張B（U+20000 以降・サロゲートペア）は text[i] のコード単位走査
// により isCjk の 0x20000 分岐へ到達せず、非 CJK ランとして 1 語トークンになる。
// head.hbs ミラーも同一規則のため索引・クエリは整合する（現行挙動の文書化）。
checkTokens('𠀋𠀋', ['𠀋𠀋'], 'CJK 拡張B はサロゲート単位走査により 1 語トークン（特性固定）');
log('');

log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
