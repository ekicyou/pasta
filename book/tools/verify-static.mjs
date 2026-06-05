// verify-static.mjs — 静的出力・オフライン閲覧の検証（tasks.md 7.1）
//
// 目的:
//   mdBook の HTML 出力 (`book/book/`) が「サーバープロセス無し・file:// で完全に閲覧可能」
//   であることを機械的に検証する。具体的には次を立証する。
//     R1.1 / R1.2 / R1.5 静的出力のみ・オフライン閲覧成立
//       - 出力配下の全ファイルが静的アセット拡張子のみ（サーバー実行物が無い）
//       - すべてのページ間リンク／アセット参照が相対パスで、参照先が出力配下に実在する
//         （= file:// で解決可能）
//       - SUMMARY 由来の全章 HTML が生成され、相互リンク（next/prev・章間）が解決する
//       - 目次 (toc) が全章を含む
//     R3.2 / R3.3 / R3.4 表示要素
//       - コードブロック (<pre><code class="language-...">) が生成されている（R3.3）
//       - ハイライト用 CSS / JS が同梱されている（R3.3）
//
// 設計参照: design.md "Testing Strategy / Build & Static Output", "Site Foundation".
//
// 依存ゼロ（Node 標準ライブラリのみ）。検証成功で exit 0、失敗で exit 1。
//
// 使い方:
//   node book/tools/verify-static.mjs            # book/book を検証（必要ならビルド）
//   node book/tools/verify-static.mjs --no-build # 既存出力をそのまま検証
//   node book/tools/verify-static.mjs --self-test # 検証ロジック自身の健全性テストも実行

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const bookDir = path.resolve(here, '..'); // .../book
const repoRoot = path.resolve(bookDir, '..'); // リポジトリルート
const outDir = path.resolve(bookDir, 'book'); // .../book/book = mdBook HTML 出力
const summaryFile = path.resolve(bookDir, 'src', 'SUMMARY.md');

const args = new Set(process.argv.slice(2));
const NO_BUILD = args.has('--no-build');
const SELF_TEST = args.has('--self-test');

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
// 静的アセット拡張子の許可リスト。
// ここに無い拡張子が出力配下に存在した場合は「サーバー実行物の可能性あり」として失敗扱い。
// （file:// 配信に必要な純粋な静的アセットのみを許可する。）
// =========================================================================
const ALLOWED_EXTS = new Set([
  '.html',
  '.css',
  '.js',
  '.mjs',
  '.json',
  '.map',
  '.woff',
  '.woff2',
  '.ttf',
  '.otf',
  '.eot',
  '.png',
  '.svg',
  '.jpg',
  '.jpeg',
  '.gif',
  '.ico',
  '.webp',
  '.txt',
  '.xml',
]);
// 拡張子を持たないが許可する固定ファイル名（mdBook/GitHub Pages 由来）。
const ALLOWED_NOEXT_NAMES = new Set(['.nojekyll', 'CNAME']);
// 明らかにサーバー実行物・非静的を示す拡張子（検出したら即失敗）。診断強化用。
const SERVER_EXTS = new Set([
  '.php', '.asp', '.aspx', '.jsp', '.cgi', '.py', '.rb', '.pl',
  '.exe', '.dll', '.sh', '.bat', '.ps1', '.wasm', '.node',
]);

// 出力配下の全ファイルを列挙（rel は POSIX 区切りで返す）。
function listFiles(root) {
  const out = [];
  function walk(dir) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, e.name);
      if (e.isDirectory()) walk(full);
      else if (e.isFile()) out.push(full);
    }
  }
  walk(root);
  return out.map((f) => ({
    abs: f,
    rel: path.relative(root, f).split(path.sep).join('/'),
  }));
}

// HTML から href / src 参照を抽出する（単純だが実 mdBook 出力に十分な正規表現ベース）。
function extractRefs(html) {
  const refs = [];
  const re = /\b(?:href|src)\s*=\s*"([^"]*)"/gi;
  let m;
  while ((m = re.exec(html)) !== null) {
    refs.push(m[1]);
  }
  return refs;
}

// 参照が「ローカル相対参照（file:// で解決すべき対象）」か判定。
// 絶対 URL・アンカーのみ・特殊スキームは除外する。
function isLocalRef(ref) {
  if (!ref) return false;
  const r = ref.trim();
  if (r === '') return false;
  if (r.startsWith('#')) return false; // 同一ページ内アンカー
  if (/^[a-z][a-z0-9+.-]*:/i.test(r)) return false; // http:, https:, mailto:, data:, javascript: 等
  if (r.startsWith('//')) return false; // プロトコル相対 = 外部
  // ルート絶対 ("/...") は file:// では解決できないため、後段でローカル参照として扱い失敗を検出する。
  return true;
}

// 参照から query / fragment を除去し、パス本体のみ返す。
function refPath(ref) {
  let r = ref;
  const hash = r.indexOf('#');
  if (hash >= 0) r = r.slice(0, hash);
  const q = r.indexOf('?');
  if (q >= 0) r = r.slice(0, q);
  return decodeURIComponent(r);
}

// 参照先を、参照元 HTML の所在ディレクトリ基準で解決し、出力配下の実ファイルへ写像する。
// 解決後パスがディレクトリの場合は index.html を補う（mdBook は出力しないが堅牢性のため）。
function resolveRef(fromRelDir, ref, root) {
  const p = refPath(ref);
  if (p === '') return null; // 純フラグメント等（呼び出し側で除外済み想定）
  // ルート絶対参照は file:// では解決不能 → 存在しない扱いで検出させる。
  if (p.startsWith('/')) {
    return { abs: path.join(root, p), root: false };
  }
  const abs = path.resolve(root, fromRelDir, p);
  return { abs, root: true };
}

// SUMMARY.md から章ファイル (.md) を抽出し、対応する出力 HTML の相対パスへ写像する。
function summaryChapters() {
  const md = fs.readFileSync(summaryFile, 'utf8');
  const re = /\]\(([^)]+\.md)\)/g;
  const chapters = [];
  let m;
  while ((m = re.exec(md)) !== null) {
    const mdPath = m[1].trim();
    const htmlRel = mdPath.replace(/\.md$/, '.html');
    chapters.push(htmlRel);
  }
  return [...new Set(chapters)];
}

// =========================================================================
// ビルド（必要時）
// =========================================================================
function ensureBuilt() {
  const built =
    fs.existsSync(outDir) &&
    fs.existsSync(path.join(outDir, 'index.html'));
  if (NO_BUILD) {
    if (!built) {
      log('FATAL: --no-build 指定だが出力 (book/book) が未生成。');
      process.exit(1);
    }
    log('Using existing output (--no-build):', outDir);
    return;
  }
  log('Running `mdbook build book` for fresh static output...');
  try {
    execFileSync('mdbook', ['build', bookDir], {
      stdio: 'inherit',
      cwd: repoRoot,
    });
  } catch (e) {
    log('FATAL: mdbook build に失敗しました:', e.message);
    process.exit(1);
  }
}

// =========================================================================
// コア検証ロジック（自己テストからも再利用するため関数化）。
//   root を対象に検証し、{ staticOnly, brokenRefs, ... } を返す。
//   ここでは「失敗を検出できるか」だけを純粋に返し、check() は run() 側で行う。
// =========================================================================
function analyze(root) {
  const files = listFiles(root);
  const relSet = new Set(files.map((f) => f.rel.toLowerCase()));

  // (1) 静的アセットのみか
  const nonStatic = [];
  const serverFiles = [];
  for (const f of files) {
    const base = path.basename(f.rel);
    const ext = path.extname(base).toLowerCase();
    if (SERVER_EXTS.has(ext)) serverFiles.push(f.rel);
    const allowed =
      ALLOWED_EXTS.has(ext) ||
      ALLOWED_NOEXT_NAMES.has(base) ||
      (ext === '' && ALLOWED_NOEXT_NAMES.has(base));
    if (!allowed) nonStatic.push(f.rel);
  }

  // (2) 全 HTML の参照健全性
  //   404.html は「Web サーバーが 404 応答時に返す」サーバー専用ページであり、
  //   file:// のオフライン閲覧グラフには属さない（どの本文ページからもリンクされない）。
  //   site-url ベース ("/pasta/") のルート絶対リンクを含むのも mdBook の仕様どおりなので、
  //   オフライン参照解決の対象からは除外する。代わりに「本文ページが file:// で
  //   解決不能なルート絶対参照を含まない」ことは absoluteRefsInContent で別途検証する。
  const SERVER_ONLY_HTML = new Set(['404.html']);
  const htmlFiles = files.filter((f) => f.rel.endsWith('.html'));
  const contentHtml = htmlFiles.filter(
    (f) => !SERVER_ONLY_HTML.has(f.rel.toLowerCase())
  );
  const brokenRefs = [];
  const absoluteRefsInContent = []; // file:// で解決不能なルート絶対参照（本文ページ）
  let checkedRefCount = 0;
  for (const hf of contentHtml) {
    const html = fs.readFileSync(hf.abs, 'utf8');
    const fromDir = path.posix.dirname(hf.rel) === '.' ? '' : path.posix.dirname(hf.rel);
    for (const ref of extractRefs(html)) {
      if (!isLocalRef(ref)) continue;
      const p = refPath(ref);
      if (p === '') continue; // href="#..." 等
      checkedRefCount++;
      const resolved = resolveRef(fromDir, ref, root);
      if (!resolved) continue;
      if (resolved.root === false) {
        // ルート絶対参照（"/..."）。file:// では解決できない。
        absoluteRefsInContent.push({ from: hf.rel, ref });
        brokenRefs.push({ from: hf.rel, ref, resolved: '(root-absolute)' });
        continue;
      }
      const relResolved = path
        .relative(root, resolved.abs)
        .split(path.sep)
        .join('/')
        .toLowerCase();
      // 出力外へ出ている、または存在しない参照はリンク切れ。
      const escapes = relResolved.startsWith('..');
      const exists = !escapes && relSet.has(relResolved);
      if (!exists) {
        brokenRefs.push({ from: hf.rel, ref, resolved: relResolved });
      }
    }
  }

  return {
    files,
    relSet,
    nonStatic,
    serverFiles,
    htmlFiles,
    contentHtml,
    brokenRefs,
    absoluteRefsInContent,
    checkedRefCount,
  };
}

// =========================================================================
// 自己テスト: 検証ロジックが「本物」であることを保証する。
//   実出力を一時ディレクトリへコピーし、故意にリンクを壊して analyze() が
//   それを検出すること、無傷ならクリーンであることを確認する。
//   実出力 (book/book) は一切改変しない。
// =========================================================================
function runSelfTest() {
  log('');
  log('=== SELF-TEST (検証ロジックの健全性) ===');
  const tmp = fs.mkdtempSync(path.join(fs.realpathSync(require_os_tmpdir()), 'verify-static-'));
  try {
    copyDir(outDir, tmp);

    // (a) 無傷コピーはクリーンであるべき
    const clean = analyze(tmp);
    check('self-test: 無傷コピーは静的アセットのみ', clean.nonStatic.length === 0,
      JSON.stringify(clean.nonStatic.slice(0, 5)));
    check('self-test: 無傷コピーはリンク切れ無し', clean.brokenRefs.length === 0,
      JSON.stringify(clean.brokenRefs.slice(0, 3)));
    check('self-test: 参照を実際に解析している (>50件)', clean.checkedRefCount > 50,
      `checked=${clean.checkedRefCount}`);

    // (b) リンクを壊すと検出されるべき
    const idx = path.join(tmp, 'index.html');
    let html = fs.readFileSync(idx, 'utf8');
    html = html.replace(/href="grammar\/index\.html"/g,
      'href="grammar/__does_not_exist__.html"');
    fs.writeFileSync(idx, html);
    const broken = analyze(tmp);
    check('self-test: 故意のリンク切れを検出する',
      broken.brokenRefs.some((b) => b.ref.includes('__does_not_exist__')),
      JSON.stringify(broken.brokenRefs.slice(0, 3)));

    // (c) サーバー実行物拡張子を検出するべき
    fs.writeFileSync(path.join(tmp, 'evil.php'), '<?php echo 1; ?>');
    const withServer = analyze(tmp);
    check('self-test: サーバー実行物 (.php) を非静的として検出する',
      withServer.nonStatic.includes('evil.php') &&
        withServer.serverFiles.includes('evil.php'),
      JSON.stringify(withServer.nonStatic.slice(0, 5)));
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

function require_os_tmpdir() {
  // import を最小に保つためのローカルヘルパ。
  return process.env.TEMP || process.env.TMP || process.env.TMPDIR || '/tmp';
}

function copyDir(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const e of fs.readdirSync(src, { withFileTypes: true })) {
    const s = path.join(src, e.name);
    const d = path.join(dst, e.name);
    if (e.isDirectory()) copyDir(s, d);
    else if (e.isFile()) fs.copyFileSync(s, d);
  }
}

// =========================================================================
// 実行
// =========================================================================
function run() {
  ensureBuilt();
  log('');
  log('Static output verification target:', outDir);
  log('');

  const a = analyze(outDir);

  // --- (1) 静的アセットのみ ---
  log('--- (R1) 静的出力のみ・サーバー実行物が無い ---');
  log(`  files=${a.files.length}, html=${a.htmlFiles.length}`);
  check('R1.1: 出力配下が静的アセット拡張子のみ（サーバー実行物が無い）',
    a.nonStatic.length === 0,
    `non-static=${JSON.stringify(a.nonStatic.slice(0, 8))}`);
  check('R1.1: サーバーサイド実行拡張子 (.php/.cgi/.exe 等) が無い',
    a.serverFiles.length === 0,
    JSON.stringify(a.serverFiles.slice(0, 8)));
  check('R1.1: index.html が存在する（エントリポイント）',
    a.relSet.has('index.html'));

  // --- (2) オフライン参照の健全性 ---
  log('');
  log('--- (R1) file:// で解決可能な相対参照・リンク健全性 ---');
  check(`R1.2/R1.5: 本文ページの相対 href/src がすべて出力配下に実在する（解析${a.checkedRefCount}件）`,
    a.brokenRefs.length === 0,
    JSON.stringify(a.brokenRefs.slice(0, 6)));
  check('R1.2: 本文ページに file:// で解決不能なルート絶対参照が無い',
    a.absoluteRefsInContent.length === 0,
    JSON.stringify(a.absoluteRefsInContent.slice(0, 6)));
  check('R1.2: 実際に十分な参照を解析している（>50件、モックでない）',
    a.checkedRefCount > 50, `checked=${a.checkedRefCount}`);

  // --- (3) SUMMARY 由来の全章が生成され相互リンクが解決する ---
  log('');
  log('--- (R1.5) SUMMARY 全章の生成・章間リンク ---');
  const chapters = summaryChapters();
  log(`  SUMMARY chapters=${chapters.length}: ${chapters.join(', ')}`);
  const missingChapters = chapters.filter(
    (c) => !a.relSet.has(c.toLowerCase())
  );
  check('R1.5: SUMMARY 由来の全章 HTML が生成されている',
    missingChapters.length === 0,
    `missing=${JSON.stringify(missingChapters)}`);

  // next/prev ナビゲーションリンクの存在（任意ページで前後章へ遷移できる）。
  let navPages = 0;
  for (const hf of a.htmlFiles) {
    const html = fs.readFileSync(hf.abs, 'utf8');
    if (/class="[^"]*nav-chapters[^"]*"/.test(html) ||
        /<a\s+rel="(?:next|prev)"/.test(html)) {
      navPages++;
    }
  }
  check('R1.5: 章ページに前後ナビゲーション (nav-chapters / rel=next|prev) がある',
    navPages >= chapters.length - 1, `navPages=${navPages}`);

  // --- (4) 目次 (toc) が全章を含む ---
  log('');
  log('--- (R1.5) 目次 (toc) の網羅性 ---');
  const tocFile = path.join(outDir, 'toc.html');
  check('R1.5: toc.html が生成されている', fs.existsSync(tocFile));
  if (fs.existsSync(tocFile)) {
    const tocHtml = fs.readFileSync(tocFile, 'utf8');
    const tocRefs = extractRefs(tocHtml)
      .filter((r) => isLocalRef(r) && refPath(r).endsWith('.html'))
      .map((r) => refPath(r).toLowerCase());
    const tocSet = new Set(tocRefs);
    const tocMissing = chapters.filter((c) => !tocSet.has(c.toLowerCase()));
    check('R1.5: 目次 (toc) が SUMMARY 全章へのリンクを含む',
      tocMissing.length === 0,
      `missing-in-toc=${JSON.stringify(tocMissing)}`);
  }

  // --- (5) コードブロック生成 + ハイライト資材同梱（R3.3） ---
  log('');
  log('--- (R3.3) コードブロック・シンタックスハイライト ---');
  let codeBlockPages = 0;
  for (const hf of a.htmlFiles) {
    const html = fs.readFileSync(hf.abs, 'utf8');
    if (/<pre>[\s\S]*?<code[^>]*>/.test(html)) codeBlockPages++;
  }
  check('R3.3: コードブロック (<pre><code>) が生成されている',
    codeBlockPages > 0, `pages-with-code=${codeBlockPages}`);

  // language-xxx クラス（ハイライト対象指定）が少なくとも1つある。
  let langClassFound = false;
  for (const hf of a.htmlFiles) {
    const html = fs.readFileSync(hf.abs, 'utf8');
    if (/<code[^>]*class="[^"]*language-[^"]*"/.test(html)) {
      langClassFound = true;
      break;
    }
  }
  check('R3.3: コードブロックに language-* クラス（ハイライト対象）が付与されている',
    langClassFound);

  // ハイライト用 CSS / JS が同梱されている。
  const hasHlCss = a.files.some((f) => /(^|\/)highlight-[^/]*\.css$/.test(f.rel));
  const hasHlJs = a.files.some((f) => /(^|\/)highlight-[^/]*\.js$/.test(f.rel));
  check('R3.3: ハイライト用 CSS (highlight-*.css) が同梱されている', hasHlCss);
  check('R3.3: ハイライト用 JS (highlight-*.js) が同梱されている', hasHlJs);

  // 各章 HTML がハイライト CSS/JS を実際に参照していること（オフラインで効く）。
  let refHlCss = 0;
  let refHlJs = 0;
  for (const hf of a.htmlFiles) {
    const html = fs.readFileSync(hf.abs, 'utf8');
    if (/highlight-[^"']*\.css/.test(html)) refHlCss++;
    if (/highlight-[^"']*\.js/.test(html)) refHlJs++;
  }
  check('R3.3: HTML がハイライト CSS を参照している', refHlCss >= chapters.length,
    `pages=${refHlCss}`);
  check('R3.3: HTML がハイライト JS を参照している', refHlJs >= chapters.length,
    `pages=${refHlJs}`);

  // --- (6) フォント等のアセットも同梱（オフライン整合の補強） ---
  log('');
  log('--- (補強) オフライン同梱アセット ---');
  const hasFont = a.files.some((f) => /\.woff2?$/.test(f.rel));
  check('R3.4: Web フォント (woff/woff2) が同梱されている（外部依存なし表示）', hasFont);

  if (SELF_TEST) runSelfTest();

  log('');
  log(`RESULT: ${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}

run();
