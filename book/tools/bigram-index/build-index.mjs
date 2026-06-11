// build-index.mjs — mdBook の検索インデックス（searchindex-<hash>.js）を
// 日本語 bigram で再生成する build-time ツール（本実装 / タスク 3.1）。
//
// 役割（design.md「Bigram Search」Batch Contract）:
//   - mdbook build 後段で動作。標準出力の searchindex-<hash>.js を読み、
//     各ドキュメント本文を共有 tokenize.mjs（2-gram）で索引化し直す。
//   - mdBook 同梱 elasticlunr を再利用し、同一スキーマ（doc_urls + index）で
//     上書き出力する（searcher.js が無改変で読める形式を維持）。
//   - 出力ファイル名・ラッパ形式（window.search = Object.assign(... JSON.parse('...')))
//     を維持する。
//   - 冪等・決定論的（同一入力 → 同一索引）。失敗時は非ゼロ終了でビルド失敗扱い。
//
// 注意（タスク 1.1/1.2 Implementation Notes）:
//   - 検索成果物はハッシュ付きファイル名。固定名前提にせずグロブで解決する。
//   - 索引リテラル抽出は `lastIndexOf('))')` に依存しない（JSON 本文に `))` が
//     含まれ得るため脆い）。`JSON.parse('` 直後の単一引用符文字列リテラルを
//     エスケープ規則に沿って正規に走査し、JSON 境界を確実に取る。
//   - 索引再構築時は pipeline.reset() で bigram トークンを保全する
//     （stemmer/stopword がトークンを壊さない）。
//   - 索引肥大は 10MB 閾値で監視する（本番フルコンテンツで実測）。

import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { tokenize } from './tokenize.mjs';

const require = createRequire(import.meta.url);

// 索引サイズの警告閾値（design「Bigram Search」/ research D-1）。
export const SIZE_WARN_THRESHOLD_BYTES = 10 * 1024 * 1024; // 10MB

// book 出力ディレクトリ内でハッシュ付きファイルをグロブ解決。
export function resolveHashed(bookOutDir, re) {
  const f = fs.readdirSync(bookOutDir).find((n) => re.test(n));
  if (!f) throw new Error(`not found in ${bookOutDir}: ${re}`);
  return path.join(bookOutDir, f);
}

// searchindex-<hash>.js から JSON 文字列を取り出してパースする。
//
// 堅牢化（1.2 申し送り①）: `lastIndexOf('))')` には依存しない。JSON 本文には
// コード例由来の `))` が複数含まれ得るため、終端目印として使うと過剰/過少に
// 切り出す恐れがある。代わりに `JSON.parse(` 直後に来る単一引用符文字列リテラルを
// バックスラッシュエスケープ規則どおりに一文字ずつ走査し、最初の「エスケープ
// されていない単一引用符」を終端として確実に JSON 境界を取る。
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

  // marker 直後の空白を読み飛ばし、開きの単一引用符を見つける。
  let i = start + marker.length;
  while (i < s.length && /\s/.test(s[i])) i++;
  if (s[i] !== "'") {
    throw new Error("searchindex: expected single-quoted literal after JSON.parse(");
  }
  const open = i + 1; // 本文先頭（引用符の内側）

  // エスケープを尊重しつつ、終端の単一引用符まで走査・デコードする。
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
      // 終端のエスケープされていない単一引用符。
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

  // 元のグローバル tokenizer を退避し、bigram に差し替える。
  const savedTokenizer = elasticlunr.tokenizer;
  elasticlunr.tokenizer = (str) => tokenize(str);

  let rebuilt;
  try {
    rebuilt = elasticlunr(function () {
      for (const f of fields) this.addField(f);
      this.setRef(ref);
      this.saveDocument(true);
      // bigram トークンは語幹処理・ストップワードで壊さない。
      // pipeline を空にし、tokenize の結果をそのまま索引する。
      this.pipeline.reset();
    });

    // 元の生ドキュメントを再投入。
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

// 高レベル API: book 出力ディレクトリの searchindex を bigram 化して上書きする。
// 索引サイズを 10MB 閾値で監視し、超過時は warnings に記録する（既定では非致命）。
export function buildBigramIndex(bookOutDir, { write = true } = {}) {
  const elasticlunrFile = resolveHashed(bookOutDir, /^elasticlunr-.*\.min\.js$/);
  const searchIndexFile = resolveHashed(bookOutDir, /^searchindex-.*\.js$/);

  // require は呼び出しモジュール基準で相対解決するため、絶対パス化しておく。
  const elasticlunr = require(path.resolve(elasticlunrFile));
  const original = readSearchIndex(searchIndexFile);
  const rebuilt = rebuildBigramIndex(elasticlunr, original);

  // 書き込み有無に関わらず確定サイズを算出（決定論的）。
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
  // 既定パスはモジュール位置基準で book/book を指す。new URL().pathname は
  // Windows でドライブ文字つき `/C:/...` を返し path.resolve が `C:\C:\...` に
  // 壊れるため、fileURLToPath で正規に OS パスへ変換する（POSIX では同一挙動）。
  const dir = process.argv[2] || path.resolve(
    path.dirname(fileURLToPath(import.meta.url)), '../../book',
  );
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
