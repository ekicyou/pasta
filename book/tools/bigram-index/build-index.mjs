// build-index.mjs — mdBook の検索インデックス（searchindex-<hash>.js）を日本語 bigram で再生成する build-time ツール。
//
// mdbook build の後段で、各ドキュメント本文を共有 tokenize.mjs（2-gram）で索引化し直し、
// 同じスキーマ（doc_urls + index）・ファイル名・ラッパ形式で上書きする（searcher.js が無改変で読める）。
// 冪等・決定論的。失敗時は非ゼロ終了でビルド失敗扱い。
//
// - 検索成果物はハッシュ付きファイル名なので、固定名にせずグロブで解決する。
// - 再構築時は pipeline.reset() で bigram トークンを保つ（stemmer/stopword が壊さない）。
// - 索引の肥大は 10MB 閾値で監視する。

import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { tokenize } from './tokenize.mjs';

const require = createRequire(import.meta.url);

// 索引サイズの警告閾値。
export const SIZE_WARN_THRESHOLD_BYTES = 10 * 1024 * 1024; // 10MB

// book 出力ディレクトリ内のハッシュ付きファイルをグロブで解決する。
export function resolveHashed(bookOutDir, re) {
  const f = fs.readdirSync(bookOutDir).find((n) => re.test(n));
  if (!f) throw new Error(`not found in ${bookOutDir}: ${re}`);
  return path.join(bookOutDir, f);
}

// searchindex-<hash>.js から JSON 文字列を取り出してパースする。
// `lastIndexOf('))')` には依存しない。JSON 本文にはコード例由来の `))` が含まれ得るので、
// `JSON.parse(` 直後の単一引用符文字列をエスケープ規則どおりに走査し、最初の未エスケープの単一引用符を終端とする。
export function readSearchIndex(searchIndexFile) {
  const s = fs.readFileSync(searchIndexFile, 'utf8');
  const { jsonStr } = extractJsonLiteral(s);
  return JSON.parse(jsonStr);
}

// `window.search = Object.assign(window.search, JSON.parse('...'));` 形式から
// 単一引用符リテラルを走査して JSON 文字列へデコードする（eval 不使用・依存ゼロ）。
// 戻り値: { jsonStr, open, close } — open/close はリテラル本文（引用符の内側）の
// ソース上の開始/終了インデックス。
export function extractJsonLiteral(s) {
  const marker = 'JSON.parse(';
  const start = s.indexOf(marker);
  if (start < 0) throw new Error('searchindex: JSON.parse( not found');

  let i = start + marker.length;
  while (i < s.length && /\s/.test(s[i])) i++;
  if (s[i] !== "'") {
    throw new Error("searchindex: expected single-quoted literal after JSON.parse(");
  }
  const open = i + 1; // 本文先頭（引用符の内側）

  let out = '';
  let j = open;
  for (; j < s.length; j++) {
    const c = s[j];
    if (c === '\\') {
      const next = s[j + 1];
      if (next === undefined) throw new Error('searchindex: dangling escape');
      if (next === "'") { out += "'"; j++; }
      else if (next === '\\') { out += '\\'; j++; }
      else if (next === 'n') { out += '\n'; j++; }
      else if (next === 'r') { out += '\r'; j++; }
      else if (next === 't') { out += '\t'; j++; }
      else { out += c; } // 未知のエスケープはそのまま（JSON.parse 側に委ねる）
    } else if (c === "'") {
      return { jsonStr: out, open, close: j };
    } else {
      out += c;
    }
  }
  throw new Error('searchindex: unterminated single-quoted literal');
}

// 文字列を mdBook 形式の単一引用符リテラルへエンコードする。
function encodeJsStringLiteral(str) {
  return "'" + str.replace(/\\/g, '\\\\').replace(/'/g, "\\'") + "'";
}

// elasticlunr に bigram tokenize を適用して索引を再構築する。
// 元 index の documentStore.docs（生本文）を母体に、同一 fields・ref で作り直す。
export function rebuildBigramIndex(elasticlunr, original) {
  const origIndex = original.index;
  const fields = origIndex.fields;
  const ref = origIndex.ref;

  const savedTokenizer = elasticlunr.tokenizer;
  elasticlunr.tokenizer = (str) => tokenize(str);

  let rebuilt;
  try {
    rebuilt = elasticlunr(function () {
      for (const f of fields) this.addField(f);
      this.setRef(ref);
      this.saveDocument(true);
      // bigram トークンは語幹処理・ストップワードで壊さない。
      this.pipeline.reset();
    });

    const docs = origIndex.documentStore.docs;
    for (const id of Object.keys(docs)) {
      rebuilt.addDoc(docs[id]);
    }
  } finally {
    elasticlunr.tokenizer = savedTokenizer;
  }

  return {
    doc_urls: original.doc_urls,
    index: rebuilt.toJSON(),
    results_options: original.results_options,
    search_options: original.search_options,
  };
}

// bigram 索引を mdBook 形式のラッパ文字列へシリアライズする（決定論的）。
// 同一入力（indexObj）に対し常に同一バイト列を返す。
export function serializeSearchIndex(indexObj) {
  const jsonStr = JSON.stringify(indexObj);
  const literal = encodeJsStringLiteral(jsonStr);
  return `window.search = Object.assign(window.search, JSON.parse(${literal}));`;
}

// 索引の search_options で検索する（ブラウザの searcher と同じ設定。設定は書き写さない）。
// トークナイザは呼ぶ側が elasticlunr.tokenizer に入れる。
export function searchIndex(elasticlunr, indexObj, query) {
  const idx = elasticlunr.Index.load(indexObj.index);
  const raw = idx.search(query, indexObj.search_options);
  const refs = raw.map((r) => Number(r.ref));
  const urls = refs.map((ref) => indexObj.doc_urls[ref]);
  return { urls, refs, raw };
}

// 全ドキュメントの題・本文・パンくずを空白でつないだもの。
export function corpusOf(indexObj) {
  return Object.values(indexObj.index.documentStore.docs)
    .map((d) => `${d.title || ''} ${d.body || ''} ${d.breadcrumbs || ''}`)
    .join(' ');
}

// 題・本文・パンくずのどれかに word を含むドキュメントの URL。
export function urlsContaining(indexObj, word) {
  const docs = indexObj.index.documentStore.docs;
  return Object.keys(docs)
    .filter((id) => [docs[id].title, docs[id].body, docs[id].breadcrumbs]
      .some((s) => (s || '').includes(word)))
    .map((id) => indexObj.doc_urls[Number(id)]);
}

// 高レベル API: book 出力ディレクトリの searchindex を bigram 化して上書きする。
// 索引サイズが 10MB 閾値を超えたら warnings に記録する（既定では非致命）。
export function buildBigramIndex(bookOutDir, { write = true } = {}) {
  const elasticlunrFile = resolveHashed(bookOutDir, /^elasticlunr-.*\.min\.js$/);
  const searchIndexFile = resolveHashed(bookOutDir, /^searchindex-.*\.js$/);

  // require は呼び出しモジュール基準で相対解決するため、絶対パス化しておく。
  const elasticlunr = require(path.resolve(elasticlunrFile));
  const original = readSearchIndex(searchIndexFile);
  const rebuilt = rebuildBigramIndex(elasticlunr, original);

  const content = serializeSearchIndex(rebuilt);
  const sizeBytes = Buffer.byteLength(content, 'utf8');

  if (write) fs.writeFileSync(searchIndexFile, content, 'utf8');

  const warnings = [];
  const overThreshold = sizeBytes >= SIZE_WARN_THRESHOLD_BYTES;
  if (overThreshold) {
    warnings.push(
      `bigram index size ${sizeBytes} bytes (${(sizeBytes / 1024 / 1024).toFixed(2)} MB) ` +
      `>= threshold ${SIZE_WARN_THRESHOLD_BYTES} bytes (10 MB)`,
    );
  }

  return {
    elasticlunrFile,
    searchIndexFile,
    original,
    rebuilt,
    sizeBytes,
    overThreshold,
    warnings,
  };
}

// CLI: node build-index.mjs <book-out-dir>
// 成功時 exit 0、失敗時は例外を投げて非ゼロ終了（mdbook build 後段でビルド失敗扱い）。
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  // 引数は必須。既定のパスを持たない（テストが公開の出力へ書く道を残さない）。
  const dir = process.argv[2];
  if (!dir) {
    console.error('usage: node build-index.mjs <book-out-dir>');
    process.exit(1);
  }
  try {
    const { searchIndexFile, sizeBytes, warnings } = buildBigramIndex(dir);
    const mb = (sizeBytes / 1024 / 1024).toFixed(2);
    console.log(`bigram index written: ${searchIndexFile}`);
    console.log(`index size: ${sizeBytes} bytes (${mb} MB), threshold 10 MB`);
    for (const w of warnings) console.warn(`WARN: ${w}`);
  } catch (e) {
    console.error(`build-index failed: ${e && e.stack ? e.stack : e}`);
    process.exit(1);
  }
}
