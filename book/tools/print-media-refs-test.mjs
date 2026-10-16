// print-media-refs-test.mjs — print.html のメディア要素の参照を直すツールの自己テスト
// （manual-print-media-refs タスク 1.3 / 要件 2.9, 2.10 / design「PrintMediaRefsTest」「Testing Strategy / Unit Tests」1〜8）。
//
// 観測する完了条件:
//   - 深さの違う章（根・1 段下・2 段下）で video の src・poster、source・track・audio の src が章の場所から直る。../ は正規化。
//   - 同じ名前の動画を持つ 2 つの章が、それぞれの章のファイルへ直る。
//   - 対象外（<a href>・<img src>・外部 URL・#…・/…・data:・本文・ほかのファイル）は 1 バイトも変わらない。
//     ?・# の残りは保つ。print.html で付け直された id があっても直る。
//   - 対象 0 件で成功（toc.html が無くても）。直せない場合は全ファイルが変わらず、文面に章と参照が出る。
//   - 出力のフォルダ・print.html が無いと失敗。2 回目は fixed 0。CLI の終了コードと出力。
//   フィクスチャは mdBook 0.5.4 の実際の出力の形（book/book の print.html・toc.html・debug/dev-actions.html）を写している:
//   print.html の <main> は章のページの <main> の中身を改ページの印でつないだもの。<main> の外に iframe・script がある。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { fixPrintMediaRefs } from './print-media-refs.mjs';

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
const show = (s) => JSON.stringify(s);

// --- mdBook 0.5.4 の形を写したフィクスチャ ---
const PAGE_BREAK = '<div style="break-before: page; page-break-before: always;"></div>';
const shell = (main) => '<!DOCTYPE HTML>\n<html lang="ja" class="light" dir="ltr">\n<body>\n'
  + '        <iframe class="sidebar-iframe-outer" src="toc.html"></iframe>\n'
  + '                <div id="mdbook-content" class="content">\n'
  + `                    <main>\n${main}\n\n                    </main>\n`
  + '                </div>\n        <script src="book-609e4cb8.js"></script>\n</body>\n</html>\n';
const tocHtml = (files) => '<!DOCTYPE HTML>\n<html lang="ja">\n<body>\n<ol class="chapter">'
  + files.map((f) => `<li class="chapter-item "><span class="chapter-link-wrapper"><a href="${f}" target="_parent">${f}</a></span></li>`).join('')
  + '</ol>\n</body>\n</html>\n';
const printHtml = (bodies) => shell(bodies.join(`\n${PAGE_BREAK}\n`));

function write(dir, rel, text) {
  const p = path.join(dir, rel);
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, text);
}

// chapters: [{ file, page, print?, want? }]
//   page: 章のページの <main> の中身 / print: print.html での中身（既定 page）/ want: 直した後の print.html での中身（既定 print）
// files: 出力にある写し先などのファイル。toc: false なら toc.html を作らない。
function build(dir, chapters, files = [], { toc = true } = {}) {
  fs.mkdirSync(dir, { recursive: true });
  for (const c of chapters) write(dir, c.file, shell(c.page));
  if (toc) write(dir, 'toc.html', tocHtml(chapters.map((c) => c.file)));
  write(dir, 'print.html', printHtml(chapters.map((c) => c.print ?? c.page)));
  for (const f of files) write(dir, f, `fixture ${f}\n`);
  return printHtml(chapters.map((c) => c.want ?? c.print ?? c.page));
}

function snapshot(dir) {
  const out = {};
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else out[path.relative(dir, p).split(path.sep).join('/')] = fs.readFileSync(p).toString('base64');
    }
  };
  walk(dir);
  return out;
}
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const without = (snap, key) => { const s = { ...snap }; delete s[key]; return s; };
const readPrint = (dir) => fs.readFileSync(path.join(dir, 'print.html'), 'utf8');
function attempt(fn) {
  try { return { result: fn(), error: null }; } catch (e) { return { result: null, error: e }; }
}

// 対象外の値（直さない・1 バイトも変えない）
const UNTOUCHED = '<p><a href="media/a.mp4">動画へのリンク</a> <img src="media/i.png" alt="図"></p>\n'
  + '<video src="https://example.com/x.mp4"></video>\n'
  + '<video src="#frag"></video>\n'
  + '<video src="/abs/x.mp4"></video>\n'
  + '<video src="data:video/mp4;base64,AAAA"></video>\n'
  + '<video data-src="media/a.mp4"></video>\n'
  + '<p>本文 src="media/a.mp4" と <code>&lt;video src=&quot;media/a.mp4&quot;&gt;</code></p>';
// print.html では mdBook が <a href>・<img src> を章の場所に合わせて書き換える（ツールは触らない）
const UNTOUCHED_PRINT = UNTOUCHED.replace('href="media/a.mp4"', 'href="debug/media/a.mp4"').replace('src="media/i.png"', 'src="debug/media/i.png"');

const debugPage = (v) => '<h1 id="x"><a class="header" href="#x">X</a></h1>\n'
  + `<video controls="" width="100%" preload="metadata" src="${v.a}" poster="${v.p}">\n`
  + `<source src="${v.webm}" type="video/webm">\n`
  + `<track src="${v.vtt}" kind="captions">\n`
  + '</video>\n'
  + `<audio controls src="${v.mp3}"></audio>\n`
  + `<video src="${v.up}"></video>\n`
  + `<video src="${v.q}"></video>\n`
  + `<video id="${v.id}" src="${v.a}"></video>\n`;
const DEBUG_BEFORE = { a: 'media/a.mp4', p: 'media/p.png', webm: 'media/a.webm', vtt: 'media/a.vtt', mp3: 'media/s.mp3', up: '../img/b.mp4', q: 'media/a.mp4?v=1#t=2', id: 'demo' };

// 正常系のフィクスチャ（対象 13 件・うち根の章の 2 件は値が変わらない）
function goodChapters() {
  return [
    { file: 'intro.html', page: '<h1 id="i"><a class="header" href="#i">I</a></h1>\n<video controls src="media/a.mp4" poster="media/a.png"></video>' },
    {
      file: 'debug/x.html',
      page: debugPage(DEBUG_BEFORE) + UNTOUCHED,
      // print.html では id が付け直される（demo → demo-1）
      print: debugPage({ ...DEBUG_BEFORE, id: 'demo-1' }) + UNTOUCHED_PRINT,
      want: debugPage({
        a: 'debug/media/a.mp4', p: 'debug/media/p.png', webm: 'debug/media/a.webm', vtt: 'debug/media/a.vtt',
        mp3: 'debug/media/s.mp3', up: 'img/b.mp4', q: 'debug/media/a.mp4?v=1#t=2', id: 'demo-1',
      }) + UNTOUCHED_PRINT,
    },
    { file: 'a/b/y.html', page: '<video src="media/c.mp4"></video>', want: '<video src="a/b/media/c.mp4"></video>' },
    { file: 'ch1/v.html', page: '<video src="media/demo.mp4"></video>', want: '<video src="ch1/media/demo.mp4"></video>' },
    { file: 'ch2/v.html', page: '<video src="media/demo.mp4"></video>', want: '<video src="ch2/media/demo.mp4"></video>' },
  ];
}
const GOOD_FILES = [
  'media/a.mp4', 'media/a.png', 'debug/media/a.mp4', 'debug/media/p.png', 'debug/media/a.webm', 'debug/media/a.vtt',
  'debug/media/s.mp3', 'img/b.mp4', 'a/b/media/c.mp4', 'ch1/media/demo.mp4', 'ch2/media/demo.mp4',
  'other.html', // ほかのファイル（変わらないこと）
];

const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'print-media-refs-test-'));
const cli = path.join(path.dirname(fileURLToPath(import.meta.url)), 'print-media-refs.mjs');
try {
  // ============================================================
  log('\n== (A) 直す規則（深さ・要素・正規化・同名の動画・対象外） ==');
  {
    const dir = path.join(tmpRoot, 'good');
    const want = build(dir, goodChapters(), GOOD_FILES);
    const before = snapshot(dir);

    const dry = attempt(() => fixPrintMediaRefs(dir, { write: false }));
    check('A-1 write: false は件数を返し何も書かない', dry.result && dry.result.targets === 13 && dry.result.fixed === 11 && same(snapshot(dir), before), show(dry.result) + (dry.error ? dry.error.message : ''));

    const r = attempt(() => fixPrintMediaRefs(dir));
    check('A-2 成功・対象 13・直した 11（根の章の 2 件は値が同じ）', r.result && r.result.targets === 13 && r.result.fixed === 11, show(r.result) + (r.error ? r.error.message : ''));
    const got = readPrint(dir);
    check('A-3 print.html が期待とバイト一致（深さ・poster・source・track・audio・../・?#・付け直された id・対象外）', got === want);
    if (got !== want) {
      const i = [...got].findIndex((ch, k) => ch !== want[k]);
      log(`        差の位置 ${i}: got ${show(got.slice(i - 40, i + 60))}\n                      want ${show(want.slice(i - 40, i + 60))}`);
    }
    check('A-4 根の章はそのまま（media/a.mp4・media/a.png）', got.includes('<video controls src="media/a.mp4" poster="media/a.png">'));
    check('A-5 1 段下: debug/media/…（src・poster・source・track・audio）', got.includes('src="debug/media/a.mp4" poster="debug/media/p.png"')
      && got.includes('<source src="debug/media/a.webm"') && got.includes('<track src="debug/media/a.vtt"') && got.includes('<audio controls src="debug/media/s.mp3"'));
    check('A-6 ../ を含む値は正規化（../img/b.mp4 → img/b.mp4）', got.includes('<video src="img/b.mp4">') && !got.includes('debug/../'));
    check('A-7 ?・# の残りを保つ', got.includes('src="debug/media/a.mp4?v=1#t=2"'));
    check('A-8 付け直された id="demo-1" でも直る', got.includes('<video id="demo-1" src="debug/media/a.mp4">'));
    check('A-9 2 段下: a/b/media/c.mp4', got.includes('<video src="a/b/media/c.mp4">'));
    check('A-10 同名の動画はそれぞれの章へ（ch1・ch2）', got.includes('<video src="ch1/media/demo.mp4">') && got.includes('<video src="ch2/media/demo.mp4">'));
    check('A-11 対象外（a・img・外部・#・/・data:・data-src・本文）は変わらない', got.includes(UNTOUCHED_PRINT));
    check('A-12 <main> の外（iframe・script）は変わらない', got.includes('<iframe class="sidebar-iframe-outer" src="toc.html">') && got.includes('<script src="book-609e4cb8.js">'));
    check('A-13 print.html のほかのファイル（章のページ・toc・メディア・other.html）は 1 バイトも変わらない', same(without(snapshot(dir), 'print.html'), without(before, 'print.html')));

    const afterFirst = snapshot(dir);
    const r2 = attempt(() => fixPrintMediaRefs(dir));
    check('A-14 2 回目は fixed 0・対象 13', r2.result && r2.result.fixed === 0 && r2.result.targets === 13, show(r2.result) + (r2.error ? r2.error.message : ''));
    check('A-15 2 回目は全ファイル不変', same(snapshot(dir), afterFirst));
  }

  // ============================================================
  log('\n== (B) 対象 0 件 ==');
  {
    const dir = path.join(tmpRoot, 'zero');
    build(dir, [
      { file: 'intro.html', page: '<p>本文だけ <a href="media/a.mp4">a</a> <img src="img/x.png" alt=""></p>' },
      { file: 'debug/x.html', page: '<video src="https://example.com/x.mp4"></video>\n<video src="#t"></video>' },
    ], [], { toc: false });
    const before = snapshot(dir);
    const r = attempt(() => fixPrintMediaRefs(dir));
    check('B-1 toc.html が無くても成功・targets 0・fixed 0', r.result && r.result.targets === 0 && r.result.fixed === 0, show(r.result) + (r.error ? r.error.message : ''));
    check('B-2 何も変わらない', same(snapshot(dir), before));
  }

  // ============================================================
  log('\n== (C) 直せないとき何も書かずに失敗 ==');
  // mutate(dir) で正常系のフィクスチャを壊し、全ファイル不変と文面を確かめる。
  const unfixable = (key, name, mutate, patterns, chapters = goodChapters()) => {
    const dir = path.join(tmpRoot, key);
    build(dir, chapters, GOOD_FILES);
    mutate(dir);
    const before = snapshot(dir);
    const r = attempt(() => fixPrintMediaRefs(dir));
    const msg = r.error ? r.error.message : '';
    check(`${name}: 例外`, r.error !== null, show(r.result));
    check(`${name}: 全ファイル不変`, same(snapshot(dir), before));
    for (const p of patterns) check(`${name}: 文面に ${p}`, msg.includes(p), msg);
    return dir;
  };

  unfixable('c-missing', 'C-1 写し先なし（2 章・全件）', (d) => {
    fs.rmSync(path.join(d, 'debug/media/a.webm'));
    fs.rmSync(path.join(d, 'a/b/media/c.mp4'));
  }, ['debug/x.html', 'source.src="media/a.webm"', 'debug/media/a.webm', 'a/b/y.html', 'video.src="media/c.mp4"', '2 件']);

  {
    const cs = goodChapters();
    cs[0] = { file: 'intro.html', page: '<video src="../out.mp4"></video>' };
    unfixable('c-escape', 'C-2 根の外', (d) => write(d, '../out.mp4', 'x'), ['intro.html', 'video.src="../out.mp4"', '根の外'], cs);
  }

  unfixable('c-count', 'C-3 区切りと目次の数が違う', (d) => {
    write(d, 'toc.html', tocHtml([...goodChapters().map((c) => c.file), 'extra.html']));
    write(d, 'extra.html', shell('<p>追加</p>'));
  }, ['改ページで分けた部分 5', 'toc.html の章の数（6）']);

  {
    const cs = goodChapters();
    // print.html の debug/x.html の部分から track が抜けている
    cs[1] = { ...cs[1], print: cs[1].print.replace(/<track [^>]*>\n/, '') };
    unfixable('c-seq', 'C-4 部分と章のページの並びが違う', () => {}, ['debug/x.html', '並び', 'track.src="media/a.vtt"'], cs);
  }

  unfixable('c-nopage', 'C-5 章のページが無い', (d) => fs.rmSync(path.join(d, 'a/b/y.html')), ['a/b/y.html', '章のページがありません']);

  unfixable('c-outside', 'C-6 <main> の外の対象', (d) => {
    const p = path.join(d, 'print.html');
    fs.writeFileSync(p, fs.readFileSync(p, 'utf8').replace('<main>', '<video src="media/z.mp4"></video>\n<main>'));
  }, ['<main> の外', 'video.src="media/z.mp4"']);

  // ============================================================
  log('\n== (D) 出力のフォルダ・print.html が無い ==');
  {
    const r = attempt(() => fixPrintMediaRefs(path.join(tmpRoot, 'nope')));
    check('D-1 出力のフォルダが無いと例外', r.error !== null && /出力フォルダがありません/.test(r.error.message), r.error && r.error.message);
    const dir = path.join(tmpRoot, 'noprint');
    write(dir, 'intro.html', shell('<video src="media/a.mp4"></video>'));
    const r2 = attempt(() => fixPrintMediaRefs(dir));
    check('D-2 print.html が無いと例外', r2.error !== null && /print\.html がありません/.test(r2.error.message), r2.error && r2.error.message);
  }

  // ============================================================
  log('\n== (E) CLI ==');
  {
    const dir = path.join(tmpRoot, 'cli-good');
    const want = build(dir, goodChapters(), GOOD_FILES);
    const ok = spawnSync(process.execPath, [cli, dir], { encoding: 'utf8' });
    check('E-1 成功は exit 0・件数の 1 行', ok.status === 0
      && ok.stdout === 'print-media-refs: fixed 11 media reference(s) in print.html (13 target(s))\n' && ok.stderr === '', show(ok.stdout + ok.stderr));
    check('E-2 CLI で書き換わる', readPrint(dir) === want);

    const bad = path.join(tmpRoot, 'cli-bad');
    build(bad, goodChapters(), GOOD_FILES);
    fs.rmSync(path.join(bad, 'debug/media/a.webm'));
    const before = snapshot(bad);
    const ng = spawnSync(process.execPath, [cli, bad], { encoding: 'utf8' });
    check('E-3 失敗は exit 1・stderr に print-media-refs failed: と章と参照・書き込みなし', ng.status === 1
      && ng.stderr.startsWith('print-media-refs failed: ') && ng.stderr.includes('debug/x.html') && ng.stderr.includes('media/a.webm')
      && ng.stdout === '' && same(snapshot(bad), before), show(ng.stdout + ng.stderr));

    const nd = spawnSync(process.execPath, [cli, path.join(tmpRoot, 'nope')], { encoding: 'utf8' });
    check('E-4 出力のフォルダが無いと exit 1', nd.status === 1 && nd.stderr.startsWith('print-media-refs failed: '), show(nd.stderr));
  }
} finally {
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}

log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
