// verify-search.mjs — 日本語全文検索の受入検証（tasks.md 7.2 / 要件 2.1〜2.5）
//
// 目的:
//   本番フルコンテンツ（book/src 配下の全章）の mdBook 出力に対し、Requirement 2 の
//   受入基準を「実索引・実コンテンツ」で機械的に立証する（モックではない）。
//
//   - 2.1 検索一致ページが結果一覧として提示される（doc_url の集合が返る）。
//   - 2.2 検索がサーバーサイド処理なし（クライアント完結）で動作する。
//          ＝ 索引/ランタイムが静的 JS 同梱で、検索 UI が fetch/XHR/WebSocket 等の
//             サーバー通信を行わない（searcher.js を実静的解析）。
//   - 2.3 結果項目から該当ページへ遷移可能（結果 ref が実在 doc_url へ写像できる）。
//   - 2.4 任意の連続2文字以上の日本語語句（語中2文字・語中3文字）でも、その語句を
//          本文に含むページが結果に出る（語中部分一致の合格基準）。
//   - 2.5 HTTP 配信時に 2.4 の検索を提供（索引・ランタイム・クエリ tokenizer が
//          配信成果物として同梱され、整合する）。
//
//   さらに回帰防止として、クエリ側 tokenizer（theme/head.hbs にインライン同梱され
//   全ページ <head> へ展開されるもの）が索引側の正準 tokenize.mjs と「同一規則」で
//   あることを、実ビルド HTML から抽出した tokenizer を実行して逐語照合で確認する。
//   不一致＝索引とクエリの分割規則がずれ検索が破綻する回帰を検出する。
//
// 設計参照: design.md「Bigram Search」「Testing Strategy / 日本語検索」。
//
// 依存ゼロ（Node 標準ライブラリ ＋ 同梱 elasticlunr のみ）。成功で exit 0、失敗で exit 1。
//
// 使い方:
//   node book/tools/verify-search.mjs            # 必要ならビルド＋bigram再生成して検証
//   node book/tools/verify-search.mjs --no-build # 既存出力をそのまま検証（再生成のみ実施）

import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { tokenize } from './bigram-index/tokenize.mjs';
import {
  resolveHashed,
  readSearchIndex,
  SIZE_WARN_THRESHOLD_BYTES,
} from './bigram-index/build-index.mjs';

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url)); // .../book/tools
const bookDir = path.resolve(here, '..'); // .../book（book.toml のあるディレクトリ）
const repoRoot = path.resolve(bookDir, '..'); // リポジトリルート
const bookOut = path.resolve(bookDir, 'book'); // .../book/book = mdBook HTML 出力
const buildIndexScript = path.resolve(here, 'bigram-index/build-index.mjs');

const args = new Set(process.argv.slice(2));
const NO_BUILD = args.has('--no-build');

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

// =========================================================================
// ステップ 1: mdbook build → bigram 索引再生成（または既存出力前提）。
//   実索引・実コンテンツに対する検証であることを担保する。
// =========================================================================
function ensureBuiltAndIndexed() {
  const hasOut =
    fs.existsSync(bookOut) &&
    fs.readdirSync(bookOut).some((n) => /^searchindex-.*\.js$/.test(n));

  if (!NO_BUILD || !hasOut) {
    log('Running `mdbook build book` for fresh static output...');
    execFileSync('mdbook', ['build', bookDir], { stdio: 'inherit', cwd: repoRoot });
  } else {
    log('Using existing output (--no-build):', bookOut);
  }

  // bigram 索引を再生成（mdbook 標準索引 → 2-gram 索引へ上書き）。
  log('Regenerating bigram search index via build-index.mjs...');
  execFileSync('node', [buildIndexScript, bookOut], { stdio: 'inherit', cwd: repoRoot });
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

// elasticlunr 索引へクエリし、結果を {urls, refs} として返す。
//   urls = 結果一覧（2.1）。refs→doc_urls 写像が遷移先（2.3）の実在性を表す。
function searchResults(elasticlunr, indexObj, query) {
  const idx = elasticlunr.Index.load(indexObj.index);
  const results = idx.search(query, SEARCH_OPTIONS);
  const refs = results.map((r) => Number(r.ref));
  const urls = refs.map((ref) => indexObj.doc_urls[ref]);
  return { urls, refs, raw: results };
}

// 索引内の全ドキュメント本文＋タイトルを連結したコーパス（母語の実在確認用）。
function corpusOf(indexObj) {
  const docs = indexObj.index.documentStore.docs;
  return Object.values(docs)
    .map((d) => `${d.title || ''} ${d.body || ''} ${d.breadcrumbs || ''}`)
    .join(' ');
}

// =========================================================================
// 実行本体
// =========================================================================
ensureBuiltAndIndexed();
log('');
log('Search verification target:', bookOut);

const elasticlunrFile = resolveHashed(bookOut, /^elasticlunr-.*\.min\.js$/);
const searchIndexFile = resolveHashed(bookOut, /^searchindex-.*\.js$/);
const elasticlunr = require(elasticlunrFile);
const indexObj = readSearchIndex(searchIndexFile);

log('  elasticlunr:', path.basename(elasticlunrFile));
log('  searchindex:', path.basename(searchIndexFile));
log('  doc count   :', indexObj.doc_urls.length);
log('');

// -------------------------------------------------------------------------
// フィクスチャ前提: 本番フルコンテンツ（全章）が索引に含まれること。
// -------------------------------------------------------------------------
log('--- フィクスチャ（本番フルコンテンツ）---');
{
  const sections = new Set(
    indexObj.doc_urls.map((u) => (u || '').split('/')[0].split('.')[0]),
  );
  for (const sec of ['introduction', 'getting-started', 'grammar', 'lua', 'reference']) {
    check(`章「${sec}」が索引に含まれる`, sections.has(sec),
      `sections=${JSON.stringify([...sections])}`);
  }
  check('索引 doc 数が十分（>=15、本番規模）', indexObj.doc_urls.length >= 15,
    `docs=${indexObj.doc_urls.length}`);
}
log('');

// =========================================================================
// 2.4 / 2.1 / 2.3: 語中2文字・語中3文字の日本語クエリで該当ページがヒットする。
//   各ケースは「親語」が本番コンテンツに実在し、テストクエリは語中（語頭ではない）
//   の連続2文字 / 連続3文字。親語ヒット集合を語中片ヒット集合が包含することも確認。
// =========================================================================
log('--- 2.4 語中部分一致（語中2文字・語中3文字）+ 2.1 結果一覧 + 2.3 遷移先実在 ---');

// [親語, 語中2文字, 語中3文字]。いずれも本番コンテンツに実在し語中に現れる。
const CASES = [
  { word: 'スクリプト', mid2: 'クリ', mid3: 'クリプ' },
  { word: 'リファレンス', mid2: 'ァレ', mid3: 'ファレ' },
  { word: 'アクション', mid2: 'クシ', mid3: 'クショ' },
  { word: 'イベント', mid2: 'ベン', mid3: 'ベント' },
  { word: 'ブロック', mid2: 'ロッ', mid3: 'ロック' },
];

const corpus = corpusOf(indexObj);

elasticlunr.tokenizer = (s) => tokenize(s); // クエリ側も索引と同一規則で分割
for (const c of CASES) {
  // 母語・語中片が実コンテンツに実在することを前提確認（テストの妥当性）。
  check(`前提: 親語「${c.word}」がコーパスに実在`, corpus.includes(c.word));
  check(`前提: 語中2文字「${c.mid2}」がコーパスに実在`, corpus.includes(c.mid2));
  check(`前提: 語中3文字「${c.mid3}」がコーパスに実在`, corpus.includes(c.mid3));

  const word = searchResults(elasticlunr, indexObj, c.word);
  const r2 = searchResults(elasticlunr, indexObj, c.mid2);
  const r3 = searchResults(elasticlunr, indexObj, c.mid3);
  log(`  "${c.word}"→${word.urls.length} / 語中2 "${c.mid2}"→${r2.urls.length} / 語中3 "${c.mid3}"→${r3.urls.length}`);

  // 2.1: 一致ページが結果一覧として提示される。
  check(`2.1: 親語「${c.word}」で結果一覧が返る`, word.urls.length > 0);

  // 2.4: 語中2文字・語中3文字いずれでもヒットする（観測可能な完了条件）。
  check(`2.4: 語中2文字「${c.mid2}」がヒットする`, r2.urls.length > 0,
    `urls=${JSON.stringify(r2.urls.slice(0, 5))}`);
  check(`2.4: 語中3文字「${c.mid3}」がヒットする`, r3.urls.length > 0,
    `urls=${JSON.stringify(r3.urls.slice(0, 5))}`);

  // 2.4(包含): 語中片のヒットは親語のヒットを包含するはず（同一語起源）。
  const wordSet = new Set(word.urls);
  check(`2.4: 語中2文字「${c.mid2}」が親語ヒットを包含`,
    word.urls.length > 0 && [...wordSet].every((u) => r2.urls.includes(u)),
    `mid2=${JSON.stringify(r2.urls.slice(0, 5))} word=${JSON.stringify(word.urls.slice(0, 5))}`);
  check(`2.4: 語中3文字「${c.mid3}」が親語ヒットを包含`,
    word.urls.length > 0 && [...wordSet].every((u) => r3.urls.includes(u)),
    `mid3=${JSON.stringify(r3.urls.slice(0, 5))} word=${JSON.stringify(word.urls.slice(0, 5))}`);

  // 2.3: 結果 ref がすべて実在 doc_url（.html）へ写像でき、遷移可能であること。
  const okNav = r3.refs.length > 0 &&
    r3.refs.every((ref) => typeof indexObj.doc_urls[ref] === 'string' &&
      /\.html(\#|$)/.test(indexObj.doc_urls[ref]));
  check(`2.3: 「${c.mid3}」の結果項目が実在ページ(.html)へ遷移可能`, okNav,
    `urls=${JSON.stringify(r3.urls.slice(0, 5))}`);
}
log('');

// -------------------------------------------------------------------------
// 健全性: 無関係語（コンテンツに存在しない連続2文字）は 0 ヒット。
//   ＝何でもヒットしてしまう退化索引でないことの担保。
// -------------------------------------------------------------------------
log('--- 健全性（無関係語は 0 ヒット）---');
for (const w of ['ゑゐ', 'をゎ', 'ヿヶ']) {
  const present = corpus.includes(w);
  const r = searchResults(elasticlunr, indexObj, w);
  check(`無関係語「${w}」は 0 ヒット（コーパス不在=${!present}）`,
    !present && r.urls.length === 0, `urls=${JSON.stringify(r.urls.slice(0, 5))}`);
}
log('');

// =========================================================================
// 2.2 / 2.5: サーバー通信なし・クライアント完結で検索が動作する。
//   - 検索ランタイム searcher.js が fetch / XHR / WebSocket 等の動的サーバー
//     通信 API を一切含まない（実静的解析）。
//   - 索引（searchindex-*.js）・ランタイム（elasticlunr-*.min.js）・クエリ
//     tokenizer（head.hbs インライン）が静的 JS として配信成果物に同梱され、
//     全章 HTML から参照される（HTTP 配信で検索が成立 = 2.5）。
// =========================================================================
log('--- 2.2/2.5 サーバー通信なし・クライアント完結 ---');
{
  const searcherFile = resolveHashed(bookOut, /^searcher-.*\.js$/);
  const searcherSrc = fs.readFileSync(searcherFile, 'utf8');
  // 動的サーバー通信を示す API（コメント等の誤検出を避けつつ呼び出し形で検出）。
  const NET_PATTERNS = [
    /\bfetch\s*\(/,
    /\bXMLHttpRequest\b/,
    /\bnew\s+WebSocket\b/,
    /\bnavigator\.sendBeacon\b/,
    /\bimport\s*\(/, // 動的 import（実行時取得）
    /\bEventSource\b/,
  ];
  const offenders = NET_PATTERNS.filter((re) => re.test(searcherSrc)).map((re) => re.source);
  check('2.2: searcher.js がサーバー通信 API（fetch/XHR/WebSocket 等）を含まない',
    offenders.length === 0, `found=${JSON.stringify(offenders)}`);

  // 索引・ランタイムが静的 JS として実在（クライアント完結の前提）。
  check('2.2: 索引が静的 JS（searchindex-*.js）として同梱されている',
    fs.existsSync(searchIndexFile));
  check('2.2: 検索ランタイム（elasticlunr-*.min.js）が静的 JS として同梱されている',
    fs.existsSync(elasticlunrFile));

  // 代表ページが索引・ランタイム・searcher・クエリ tokenizer を実際に参照していること。
  const sampleHtmls = ['index.html', 'grammar/index.html', 'lua/index.html']
    .map((p) => path.join(bookOut, p))
    .filter((p) => fs.existsSync(p));
  let refIndex = 0;
  let refRuntime = 0;
  let refSearcher = 0;
  let refTokenizer = 0;
  for (const p of sampleHtmls) {
    const h = fs.readFileSync(p, 'utf8');
    if (/searchindex-[^"']*\.js/.test(h)) refIndex++;
    if (/elasticlunr-[^"']*\.min\.js/.test(h)) refRuntime++;
    if (/searcher-[^"']*\.js/.test(h)) refSearcher++;
    if (h.includes('BEGIN canonical bigram tokenize')) refTokenizer++;
  }
  check('2.5: 代表ページが索引(searchindex)を参照', refIndex === sampleHtmls.length,
    `${refIndex}/${sampleHtmls.length}`);
  check('2.5: 代表ページが検索ランタイム(elasticlunr)を参照', refRuntime === sampleHtmls.length,
    `${refRuntime}/${sampleHtmls.length}`);
  check('2.5: 代表ページが searcher を参照', refSearcher === sampleHtmls.length,
    `${refSearcher}/${sampleHtmls.length}`);
  check('2.5: 代表ページにクエリ bigram tokenizer がインライン同梱されている',
    refTokenizer === sampleHtmls.length, `${refTokenizer}/${sampleHtmls.length}`);
}
log('');

// =========================================================================
// 索引サイズ: 再生成 bigram 索引が 10MB 閾値未満（実測出力）。
// =========================================================================
log('--- 索引サイズ（10MB 閾値）---');
{
  const bytes = fs.statSync(searchIndexFile).size;
  const mb = (bytes / 1024 / 1024).toFixed(3);
  log(`  searchindex size: ${bytes} bytes (${mb} MB), threshold ${SIZE_WARN_THRESHOLD_BYTES} bytes (10 MB)`);
  check('索引サイズが 10MB 閾値未満', bytes < SIZE_WARN_THRESHOLD_BYTES,
    `${bytes} >= ${SIZE_WARN_THRESHOLD_BYTES}`);
}
log('');

// =========================================================================
// 回帰防止: クエリ側 tokenizer（head.hbs インライン）が索引側 tokenize.mjs と一致。
//   実ビルド HTML から <script> ブロックを抽出し、サンドボックスで関数を実行して
//   tokenize.mjs と逐語照合する。不一致＝索引・クエリ規則のずれ（検索破綻）の回帰。
// =========================================================================
log('--- 回帰防止: クエリ tokenizer が索引 tokenize.mjs と一致 ---');
{
  const htmlFile = path.join(bookOut, 'grammar/index.html');
  const html = fs.readFileSync(htmlFile, 'utf8');
  const beginIdx = html.indexOf('BEGIN canonical bigram tokenize');
  const endIdx = html.indexOf('END canonical bigram tokenize');
  check('tokenizer ブロックがビルド HTML に存在', beginIdx >= 0 && endIdx > beginIdx);

  if (beginIdx >= 0 && endIdx > beginIdx) {
    const scriptOpen = html.lastIndexOf('<script>', beginIdx) + '<script>'.length;
    const scriptClose = html.indexOf('</script>', endIdx);
    const inlineSrc = html.slice(scriptOpen, scriptClose);

    // インライン IIFE は tokenizer をグローバル代入監視に閉じ込めているため、
    // tokenize 関数本体だけをサンドボックスへ取り出して実行する。
    const fnBegin = inlineSrc.indexOf('function normalize');
    const fnEnd = inlineSrc.indexOf('END canonical bigram tokenize');
    const fnSrc = inlineSrc.slice(fnBegin, inlineSrc.lastIndexOf('}', fnEnd) + 1);

    const sandbox = { result: null };
    vm.createContext(sandbox);
    vm.runInContext(`${fnSrc}\nresult = tokenize;`, sandbox, { timeout: 1000 });
    const inlineTokenize = sandbox.result;
    check('インライン tokenizer 関数を抽出・実行できる', typeof inlineTokenize === 'function');

    if (typeof inlineTokenize === 'function') {
      // 索引・クエリ双方で出現する代表入力で逐語照合する（実コンテンツ語＋境界ケース）。
      const samples = [
        'スクリプト', 'リファレンス', 'アクション行', 'イベントハンドラ',
        'Lua API の使い方', 'クリプ', 'ファレ', 'ロック',
        'Pasta DSL 文法 v0.2.0', 'ＡＢＣ１２３ 全角', 'a-b-c hello world',
        'ー長音ーテスト', '単独漢字あ', '', 'ABC',
      ];
      let mismatches = 0;
      for (const s of samples) {
        const a = JSON.stringify(tokenize(s));
        const b = JSON.stringify(inlineTokenize(s));
        if (a !== b) {
          mismatches++;
          log(`    MISMATCH "${s}": mjs=${a} hbs=${b}`);
        }
      }
      check('クエリ tokenizer(head.hbs) が索引 tokenize.mjs と逐語一致（規則整合）',
        mismatches === 0, `mismatches=${mismatches}/${samples.length}`);

      // 実際に同一クエリで同一ヒット集合になることも確認（規則整合の最終証跡）。
      const savedTok = elasticlunr.tokenizer;
      elasticlunr.tokenizer = (s) => inlineTokenize(s);
      const viaHbs = searchResults(elasticlunr, indexObj, CASES[0].mid3).urls.sort();
      elasticlunr.tokenizer = (s) => tokenize(s);
      const viaMjs = searchResults(elasticlunr, indexObj, CASES[0].mid3).urls.sort();
      elasticlunr.tokenizer = savedTok;
      check(`クエリ tokenizer 一致でヒット集合が同一（"${CASES[0].mid3}"）`,
        JSON.stringify(viaHbs) === JSON.stringify(viaMjs),
        `hbs=${JSON.stringify(viaHbs.slice(0, 5))} mjs=${JSON.stringify(viaMjs.slice(0, 5))}`);
    }
  }
}
log('');

log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
