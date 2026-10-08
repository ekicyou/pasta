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
//   node book/tools/verify-static.mjs            # mdbook build → talk-html の変換をしてから book/book を検証
//   node book/tools/verify-static.mjs --no-build # 既存出力をそのまま検証
//   node book/tools/verify-static.mjs --self-test # 検証ロジック自身の健全性テストも実行

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { transformDir } from './talk/talk-html.mjs';
import { SPEAKERS } from './talk/talk.mjs';

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
  '.mp4',
]);
// 拡張子を持たないが許可する固定ファイル名（mdBook/GitHub Pages 由来）。
const ALLOWED_NOEXT_NAMES = new Set(['.nojekyll', 'CNAME']);
// 明らかにサーバー実行物・非静的を示す拡張子（検出したら即失敗）。診断強化用。
const SERVER_EXTS = new Set([
  '.php', '.asp', '.aspx', '.jsp', '.cgi', '.py', '.rb', '.pl',
  '.exe', '.dll', '.sh', '.bat', '.ps1', '.wasm', '.node',
]);

// 出力配下の全ファイルを列挙（rel は POSIX 区切りで返す）。
// 旧版（classic/。manual.yml の旧版生成段が新版の検査の後に作る）は列挙しない。
// 旧版は着せ替え前の版を手を加えずに残したもので、その実在は旧版生成段が確かめる（要件 11.3・11.4）。
function listFiles(root) {
  const out = [];
  function walk(dir) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, e.name);
      if (e.isDirectory() && dir === root && e.name === 'classic') continue;
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
  // 台詞の吹き出し変換（CI の build → highlight → talk → index の talk に当たる）。
  // 台詞部品の検査は変換後の出力を見るため、ビルドモードではここで変換しておく。
  log('Running talk-html.mjs to render talk components...');
  try {
    const { converted, filesChanged } = transformDir(outDir);
    log(`talk-html: converted ${converted} talk block(s) across ${filesChanged} file(s)`);
  } catch (e) {
    log('FATAL: talk-html の変換に失敗しました:', e.message);
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
    const allowed = ALLOWED_EXTS.has(ext) || ALLOWED_NOEXT_NAMES.has(base);
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
    let html = fs.readFileSync(hf.abs, 'utf8');
    // print.html: mdBook は <a href> を章基準へ書き換えるが <video>/<source> の src は
    // 書き換えない（既知制約）。印刷ビューで動画は用途外のため、これらのタグの参照のみ対象外とする。
    if (hf.rel === 'print.html') html = html.replace(/<(?:video|source)\b[^>]*>/gi, '');
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
// テーマ資材・台詞出力・外部参照・メニュー id の検査（manual-claudia-theme タスク 3.5 /
// 要件 1.7, 8.4, 8.5, 8.6, 8.9, 10.4 / design「StaticVerifier」）。
//   pages は SUMMARY 全章と print.html（出力根からの相対）。lua/modules.html（リダイレクト）と
//   toc.html（JS 無効時の目次フレーム）は章ではなくテーマ資材を読まないので pages に含めない。
//   mdBook 0.5.x は追加 CSS・JS を theme/claudia-<hash>.css のようにハッシュ付きで出すため、
//   固定のファイル名では探さない。参照先の実在は analyze() の相対参照検査が見る。
// =========================================================================
const ALLOWED_EXTERNAL_ORIGINS = new Set(['https://fonts.googleapis.com', 'https://fonts.gstatic.com']);
const MENU_IDS = ['light', 'rust', 'coal', 'navy', 'ayu'].map((t) => `mdbook-theme-${t}`);
const THEME_CSS_RE = /(^|\/)theme\/claudia-[^/]*\.css$/;
const THEME_JS_RE = /(^|\/)theme\/claudia-[^/]*\.js$/;
// 外部参照の検査対象は link・script・img だけ（<a href> は通信しないので対象外）。
const ASSET_TAG_RE = /<(?:link|script|img)\b[^>]*>/gi;
// 未変換の台詞: 属性の無い <blockquote> の直後の最初の <p> が「【」で始まる（talk-html の判定と同じ形）。
const UNTRANSFORMED_TALK_RE = /<blockquote>\s*<p>【/;

// link・script・img の中の href・src・srcset の値。extractRefs（二重引用符だけ）とは別に、
// 単引用符・引用符なしの値と srcset の各候補も拾う（外部参照の取りこぼしを防ぐ）。
const ASSET_ATTR_RE = /\b(href|src|srcset)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))/gi;
function assetRefs(html) {
  const refs = [];
  for (const tag of html.match(ASSET_TAG_RE) || []) {
    for (const m of tag.matchAll(ASSET_ATTR_RE)) {
      const value = m[2] ?? m[3] ?? m[4];
      if (m[1].toLowerCase() === 'srcset') {
        // 候補は「URL 記述子」をコンマで区切ったもの。URL 部分だけを取る。
        for (const c of value.split(',')) {
          const url = c.trim().split(/\s+/)[0];
          if (url) refs.push(url);
        }
      } else {
        refs.push(value);
      }
    }
  }
  return refs;
}
const isAbsoluteUrl = (ref) => /^[a-z][a-z0-9+.-]*:/i.test(ref.trim()) || ref.trim().startsWith('//');
function externalOrigin(ref) {
  try {
    return new URL(ref.trim(), 'https://invalid.invalid/').origin;
  } catch {
    return null;
  }
}

function analyzeTheme(root, pages) {
  const files = listFiles(root);
  const themeCss = files.filter((f) => /^theme\/claudia-[^/]*\.css$/.test(f.rel)).map((f) => f.rel);
  const themeJs = files.filter((f) => /^theme\/claudia-[^/]*\.js$/.test(f.rel)).map((f) => f.rel);

  const noCssRef = [];
  const noJsRef = [];
  const missingTalk = []; // { page, speakers: 欠けている話し手 id }
  const missingMenuIds = []; // { page, ids: 欠けている id }
  for (const page of pages) {
    const abs = path.join(root, page);
    if (!fs.existsSync(abs)) continue; // 章の欠落そのものは「全章の生成」の検査が報告する
    const html = fs.readFileSync(abs, 'utf8');
    // refPath は decodeURIComponent するので、壊れた % を含み得る絶対 URL には使わない。
    const refs = assetRefs(html).filter(isLocalRef).map(refPath);
    if (!refs.some((r) => THEME_CSS_RE.test(r))) noCssRef.push(page);
    if (!refs.some((r) => THEME_JS_RE.test(r))) noJsRef.push(page);
    const speakers = SPEAKERS.map((s) => s.id).filter((id) => !html.includes(`class="talk talk-${id} `));
    if (speakers.length > 0) missingTalk.push({ page, speakers });
    const ids = MENU_IDS.filter((id) => !html.includes(`id="${id}"`));
    if (ids.length > 0) missingMenuIds.push({ page, ids });
  }

  // 未変換の台詞と外部参照は、章に限らず出力のすべての HTML を見る（404.html なども外部へ通信し得る）。
  const untransformedTalk = [];
  const externalRefs = []; // { from, ref }
  for (const f of files.filter((x) => x.rel.endsWith('.html'))) {
    const html = fs.readFileSync(f.abs, 'utf8');
    if (UNTRANSFORMED_TALK_RE.test(html)) untransformedTalk.push(f.rel);
    for (const ref of assetRefs(html)) {
      if (isAbsoluteUrl(ref) && !ALLOWED_EXTERNAL_ORIGINS.has(externalOrigin(ref))) {
        externalRefs.push({ from: f.rel, ref });
      }
    }
  }

  return { themeCss, themeJs, noCssRef, noJsRef, missingTalk, untransformedTalk, externalRefs, missingMenuIds };
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
  const tmp = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'verify-static-'));
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

    // (a2) 旧版（classic/）は検査しない。旧版の 404.html の /pasta/ や本文の参照は旧版の生成段が扱う（要件 11.3）
    const classicDir = path.join(tmp, 'classic');
    fs.mkdirSync(classicDir, { recursive: true });
    fs.writeFileSync(path.join(classicDir, '404.html'),
      '<a href="/pasta/">top</a><img src="https://example.com/x.png"><blockquote>\n<p>【得意】未変換</p>\n</blockquote>');
    fs.writeFileSync(path.join(classicDir, 'print.html'),
      '<video src="missing.mp4"></video><img src="missing.png"><script src="https://example.com/x.js"></script>');
    fs.writeFileSync(path.join(classicDir, 'evil.php'), '<?php echo 1; ?>');
    const withClassic = analyze(tmp);
    const themeWithClassic = analyzeTheme(tmp, [...summaryChapters(), 'print.html']);
    const classicHits = [
      ...withClassic.brokenRefs.map((b) => JSON.stringify(b)),
      ...withClassic.absoluteRefsInContent.map((b) => JSON.stringify(b)),
      ...withClassic.nonStatic,
      ...themeWithClassic.externalRefs.map((b) => JSON.stringify(b)),
      ...themeWithClassic.untransformedTalk,
    ];
    check('self-test: classic/ の下の壊れた参照・外部参照・未変換の台詞は報告しない',
      classicHits.length === 0, classicHits.slice(0, 5).join(' / '));
    fs.rmSync(classicDir, { recursive: true, force: true });

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

    // (d) テーマ資材・台詞出力・外部参照・メニュー id の検査（manual-claudia-theme タスク 3.5）。
    //   無傷コピーで資材・外部参照・メニュー id・未変換の台詞が合格し、
    //   1 か所ずつ壊すと、壊したページと中身を正しく報告することを確かめる。
    const pages = [...summaryChapters(), 'print.html'];
    const target = 'grammar/index.html';
    const themeClean = analyzeTheme(tmp, pages);
    check('self-test: 無傷コピーにテーマ CSS・JS が 1 つずつある',
      themeClean.themeCss.length === 1 && themeClean.themeJs.length === 1,
      JSON.stringify({ css: themeClean.themeCss, js: themeClean.themeJs }));
    check('self-test: 無傷コピーは全章と print.html がテーマ資材を参照している',
      themeClean.noCssRef.length === 0 && themeClean.noJsRef.length === 0,
      JSON.stringify({ css: themeClean.noCssRef, js: themeClean.noJsRef }));
    check('self-test: 無傷コピーは外部参照が Google Fonts だけ',
      themeClean.externalRefs.length === 0, JSON.stringify(themeClean.externalRefs.slice(0, 3)));
    check('self-test: 無傷コピーはメニュー id がすべてある',
      themeClean.missingMenuIds.length === 0, JSON.stringify(themeClean.missingMenuIds.slice(0, 3)));
    check('self-test: 無傷コピーに未変換の台詞が無い',
      themeClean.untransformedTalk.length === 0, JSON.stringify(themeClean.untransformedTalk));

    const targetFile = path.join(tmp, target);
    const original = fs.readFileSync(targetFile, 'utf8')
      .replace(/class="talk talk-/g, 'class="x-talk x-talk-'); // 既存の台詞部品を無効化して起点をそろえる
    const withTarget = (mutated) => {
      fs.writeFileSync(targetFile, mutated);
      return analyzeTheme(tmp, pages);
    };
    const talkOf = (r) => r.missingTalk.find((m) => m.page === target);

    // 期待値は登録簿（talk.mjs の SPEAKERS）から組み立てる。
    const ids = SPEAKERS.map((s) => s.id);
    const talkDiv = (s) => `<div class="talk talk-${s.id} talk-${s.side}"></div>`;
    let r = withTarget(original);
    check('self-test: 台詞部品の無い章を、全話し手の欠落として報告する',
      JSON.stringify(talkOf(r)?.speakers) === JSON.stringify(ids), JSON.stringify(talkOf(r)));
    r = withTarget(original.replace('<main>', `<main>${talkDiv(SPEAKERS[0])}`));
    check(`self-test: ${ids[0]} だけの章を、残りの話し手の欠落として報告する`,
      JSON.stringify(talkOf(r)?.speakers) === JSON.stringify(ids.slice(1)), JSON.stringify(talkOf(r)));
    r = withTarget(original.replace('<main>', `<main>${SPEAKERS.map(talkDiv).join('')}`));
    check('self-test: 全話し手がいる章は報告しない', talkOf(r) === undefined, JSON.stringify(talkOf(r)));

    r = withTarget(original.replace('</head>', '<script src="https://example.com/x.js"></script></head>'));
    check('self-test: Google Fonts 以外の外部 script を検出する',
      r.externalRefs.some((e) => e.from === target && e.ref === 'https://example.com/x.js'),
      JSON.stringify(r.externalRefs.slice(0, 3)));
    r = withTarget(original.replace('</head>', '<img src="//cdn.example.com/a.png"></head>'));
    check('self-test: プロトコル相対の外部 img を検出する',
      r.externalRefs.some((e) => e.from === target && e.ref === '//cdn.example.com/a.png'),
      JSON.stringify(r.externalRefs.slice(0, 3)));
    // 引用符の形と srcset の違いで取りこぼさないこと。
    for (const [label, tag, ref] of [
      ['単引用符の img src', `<img src='https://evil.example/a.png'>`, 'https://evil.example/a.png'],
      ['引用符なしの script src', '<script src=https://evil.example/b.js></script>', 'https://evil.example/b.js'],
      ['img srcset の 2 つめの候補', '<img src="img/x.png" srcset="img/x.png 1x, https://evil.example/c.png 2x">',
        'https://evil.example/c.png'],
    ]) {
      r = withTarget(original.replace('</head>', `${tag}</head>`));
      check(`self-test: ${label} の外部 URL を検出する`,
        r.externalRefs.some((e) => e.from === target && e.ref === ref),
        JSON.stringify(r.externalRefs.slice(0, 3)));
    }
    r = withTarget(original.replace('</head>', '<script src="https://evil.example/%E0%A4%A.js"></script></head>'));
    check('self-test: 壊れた % を含む外部 URL でも落ちずに検出する',
      r.externalRefs.some((e) => e.from === target && e.ref.includes('%E0%A4%A')),
      JSON.stringify(r.externalRefs.slice(0, 3)));
    r = withTarget(original + '<a href="https://example.com/">ok</a>');
    check('self-test: <a href> の外部 URL は検査の対象外', r.externalRefs.length === 0,
      JSON.stringify(r.externalRefs.slice(0, 3)));

    r = withTarget(original.replace('id="mdbook-theme-ayu"', 'id="mdbook-theme-ayu2"'));
    check('self-test: 欠けたメニュー id を検出する',
      r.missingMenuIds.some((m) => m.page === target && m.ids.join() === 'mdbook-theme-ayu'),
      JSON.stringify(r.missingMenuIds.slice(0, 3)));

    r = withTarget(original.replace(/<link rel="stylesheet" href="[^"]*theme\/claudia-[^"]*\.css">/, ''));
    check('self-test: テーマ CSS を読まない章を検出する', r.noCssRef.includes(target),
      JSON.stringify(r.noCssRef));
    r = withTarget(original.replace(/<script src="[^"]*theme\/claudia-[^"]*\.js"><\/script>/, ''));
    check('self-test: テーマ JS を読まない章を検出する', r.noJsRef.includes(target),
      JSON.stringify(r.noJsRef));

    r = withTarget(original.replace('<main>', '<main><blockquote>\n<p>【得意】未変換です。</p>\n</blockquote>'));
    check('self-test: 未変換の台詞を検出する', r.untransformedTalk.includes(target),
      JSON.stringify(r.untransformedTalk));
    r = withTarget(original.replace('<main>', '<main><blockquote>\n<p>ふつうの引用【注】</p>\n</blockquote>'));
    check('self-test: 【で始まらない引用は未変換の台詞としない', r.untransformedTalk.length === 0,
      JSON.stringify(r.untransformedTalk));
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
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

  // --- (7) テーマ資材・台詞出力・外部参照・メニュー id（manual-claudia-theme） ---
  log('');
  log('--- (1.7/8.4–8.6/8.9/10.4) テーマ資材・台詞出力・外部参照・メニュー id ---');
  const pages = [...chapters, 'print.html'];
  const t = analyzeTheme(outDir, pages);
  check('10.4: テーマ CSS (theme/claudia-*.css) が出力にある', t.themeCss.length > 0,
    JSON.stringify(t.themeCss));
  check('10.4: テーマ JS (theme/claudia-*.js) が出力にある', t.themeJs.length > 0,
    JSON.stringify(t.themeJs));
  check(`1.7: 全章と print.html（${pages.length} ページ）がテーマ CSS を参照している`,
    t.noCssRef.length === 0, `missing=${JSON.stringify(t.noCssRef)}`);
  check(`1.7: 全章と print.html（${pages.length} ページ）がテーマ JS を参照している`,
    t.noJsRef.length === 0, `missing=${JSON.stringify(t.noJsRef)}`);
  check(`8.4/8.6: 全章と print.html に両方の話し手の台詞部品がある（欠落 ${t.missingTalk.length} ページ）`,
    t.missingTalk.length === 0);
  for (const m of t.missingTalk) log(`        ${m.page}: 台詞部品が無い話し手 ${m.speakers.join(', ')}`);
  check('8.4: 変換されずに残った台詞（<blockquote> 直後の <p>【）が無い',
    t.untransformedTalk.length === 0, `files=${JSON.stringify(t.untransformedTalk)}`);
  check('8.5: link・script・img の絶対 URL が Google Fonts の 2 オリジンだけ',
    t.externalRefs.length === 0, JSON.stringify(t.externalRefs.slice(0, 6)));
  check(`8.4: テーマメニューの id（${MENU_IDS.join(', ')}）が全章と print.html にある`,
    t.missingMenuIds.length === 0, JSON.stringify(t.missingMenuIds.slice(0, 6)));

  if (SELF_TEST) runSelfTest();

  log('');
  log(`RESULT: ${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}

run();
