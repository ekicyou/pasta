// print-media-refs.mjs — mdBook の出力の print.html で、メディア要素の参照を章の場所から出力の根から見た場所へ直す。
// manual-print-media-refs タスク 1.1（要件 1.1, 1.4, 1.5, 1.6, 2.1〜2.7, 6.2 / design「PrintMediaRefs」）。
//
// mdBook は print.html で <a href>・<img src> を章の場所に合わせて書き換えるが、メディア要素の参照は書き換えない。
// 対象: video の src・poster、audio・source・track の src（属性名の前が空白のときだけ。data-src などは拾わない）。
//   空・#…・スキーム付き・//…・/… の値は直さず、実在も見ない。
// 章の決め方: print.html の <main> の中を改ページの印で分け（<main> の外の対象は問題）、toc.html の章のリンクと出現の順に対応させる。
//   章ごとに、対象の並び（要素・属性・値）が章のページの <main> と一致することを確かめる。
//   print.html の値が章のページの値そのもの（未修正）か、直した値（修正済み）なら一致とみなす（冪等）。
// 失敗: 問題を全件集めて 1 つの Error（1 行 1 件）で投げ、何も書かない。書くのは print.html だけ・変わったときだけ 1 回。
// 依存: Node 標準ライブラリのみ（design「Allowed Dependencies」）。
//
// CLI: node book/tools/print-media-refs.mjs [book-out-dir]（既定 book/book。タスク 1.2・要件 2.5, 2.6, 2.8）

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_OUT_DIR = path.resolve(here, '../book');

const PAGE_BREAK = '<div style="break-before: page; page-break-before: always;"></div>';
// 対象の表（要素 → 属性）。
const TARGETS = { video: ['src', 'poster'], audio: ['src'], source: ['src'], track: ['src'] };
const OPEN_TAG_RE = /<(video|audio|source|track)\b[^>]*>/gi;
const ATTR_RE = /\s(src|poster)\s*=\s*"([^"]*)"/gi;
const SCHEME_RE = /^[a-z][a-z0-9+.-]*:/i;

// 直さない値: 空・ページの中の見出し・スキーム付き・//…・根からの絶対パス。
const isSkipped = (v) => v === '' || v.startsWith('#') || v.startsWith('/') || SCHEME_RE.test(v);

// html の [from, to) の範囲の対象を出現の順に返す。index は値の先頭の html 上の位置。
function collectTargets(html, from = 0, to = html.length) {
  const out = [];
  const part = html.slice(from, to);
  for (const tag of part.matchAll(OPEN_TAG_RE)) {
    const el = tag[1].toLowerCase();
    for (const a of tag[0].matchAll(ATTR_RE)) {
      const attr = a[1].toLowerCase();
      const value = a[2];
      if (!TARGETS[el].includes(attr) || isSkipped(value)) continue;
      const index = from + tag.index + a.index + a[0].length - 1 - value.length;
      out.push({ el, attr, value, index });
    }
  }
  return out;
}

// <main> から最後の </main> までの範囲。無ければ null。
function mainRange(html) {
  const s = html.indexOf('<main>');
  const e = html.lastIndexOf('</main>');
  return s < 0 || e < s ? null : [s + '<main>'.length, e];
}

// 章のページ chapter（根からの相対）の値 value を、根から見た値へ直す。?…・#… の残りは保つ。
function fixValue(chapter, value) {
  const cut = value.search(/[?#]/);
  const p = cut < 0 ? value : value.slice(0, cut);
  const rest = cut < 0 ? '' : value.slice(cut);
  return { path: path.posix.normalize(path.posix.join(path.posix.dirname(chapter), p)), rest };
}

// toc.html の章のリンクを出現の順に返す。
function tocChapters(toc) {
  return [...toc.matchAll(/<a\s[^>]*?href="([^"]*)"/gi)]
    .map((m) => m[1])
    .filter((h) => h.endsWith('.html') && !SCHEME_RE.test(h) && !/\/\/|[#?]/.test(h) && !h.startsWith('/'));
}

/**
 * print.html のメディア要素の参照を直す。
 * @param {string} outDir mdBook の HTML 出力のフォルダ
 * @param {{ write?: boolean }} [options] write: false なら何も書かない
 * @returns {{ targets: number, fixed: number }} targets: 対象の数 / fixed: 書き換えた数
 * @throws {Error} 問題があるとき（全件・1 行 1 件。何も書かない）
 */
export function fixPrintMediaRefs(outDir, { write = true } = {}) {
  if (!fs.existsSync(outDir) || !fs.statSync(outDir).isDirectory()) {
    throw new Error(`出力フォルダがありません: ${outDir}`);
  }
  const printFile = path.join(outDir, 'print.html');
  if (!fs.existsSync(printFile)) throw new Error(`print.html がありません: ${printFile}`);
  const html = fs.readFileSync(printFile, 'utf8');
  const targets = collectTargets(html).length;
  if (targets === 0) return { targets: 0, fixed: 0 };

  const fail = (problems) => {
    throw new Error(`print.html のメディアの参照を直せません（${problems.length} 件・何も書き込んでいません）:\n${problems.join('\n')}`);
  };
  const problems = [];
  const range = mainRange(html);
  if (!range) problems.push(`print.html に <main> がありません: ${printFile}`);
  else {
    // <main> の外の対象は章に対応させられないので、黙って残さず問題にする。
    for (const t of [...collectTargets(html, 0, range[0]), ...collectTargets(html, range[1])]) {
      problems.push(`print.html の <main> の外にメディアの参照があります: ${t.el}.${t.attr}="${t.value}"`);
    }
  }
  const tocFile = path.join(outDir, 'toc.html');
  const chapters = fs.existsSync(tocFile) ? tocChapters(fs.readFileSync(tocFile, 'utf8')) : null;
  if (!chapters) problems.push(`toc.html がありません: ${tocFile}`);
  else if (chapters.length === 0) problems.push(`toc.html に章のリンクがありません: ${tocFile}`);
  if (problems.length > 0) fail(problems);

  // 改ページの印で分けた部分の範囲。
  const segments = [];
  let pos = range[0];
  for (;;) {
    const next = html.indexOf(PAGE_BREAK, pos);
    if (next < 0 || next >= range[1]) { segments.push([pos, range[1]]); break; }
    segments.push([pos, next]);
    pos = next + PAGE_BREAK.length;
  }
  if (segments.length !== chapters.length) {
    fail([`print.html の章の数（改ページで分けた部分 ${segments.length}）と toc.html の章の数（${chapters.length}）が違います`]);
  }

  const replacements = [];
  const seq = (ts) => ts.map((t) => `${t.el}.${t.attr}="${t.value}"`).join(', ') || '（なし）';
  chapters.forEach((chapter, i) => {
    const printTs = collectTargets(html, ...segments[i]);
    const chapterFile = path.join(outDir, chapter);
    if (!fs.existsSync(chapterFile)) { problems.push(`${chapter}: 章のページがありません`); return; }
    const page = fs.readFileSync(chapterFile, 'utf8');
    const r = mainRange(page);
    if (!r) { problems.push(`${chapter}: 章のページに <main> がありません`); return; }
    const pageTs = collectTargets(page, ...r);
    const fixes = pageTs.map((t) => fixValue(chapter, t.value));
    const matches = printTs.length === pageTs.length && printTs.every((t, k) => {
      const c = pageTs[k];
      return t.el === c.el && t.attr === c.attr && (t.value === c.value || t.value === fixes[k].path + fixes[k].rest);
    });
    if (!matches) {
      problems.push(`${chapter}: print.html の対象の並びが章のページと合いません（print.html: ${seq(printTs)} / 章: ${seq(pageTs)}）`);
      return;
    }
    printTs.forEach((t, k) => {
      const f = fixes[k];
      const where = `${chapter}: ${t.el}.${t.attr}="${pageTs[k].value}" → ${f.path}`;
      if (f.path === '..' || f.path.startsWith('../')) { problems.push(`${where}: 出力の根の外を指します`); return; }
      let decoded;
      try { decoded = decodeURIComponent(f.path); } catch { problems.push(`${where}: パスを復号できません`); return; }
      if (!fs.existsSync(path.join(outDir, decoded))) { problems.push(`${where}: 出力にファイルがありません`); return; }
      const value = f.path + f.rest;
      if (value !== t.value) replacements.push({ index: t.index, length: t.value.length, value });
    });
  });
  if (problems.length > 0) fail(problems);

  // 出現の順に置き換える（対象の属性の値のほかは変えない）。
  let out = '';
  let last = 0;
  for (const rep of replacements) {
    out += html.slice(last, rep.index) + rep.value;
    last = rep.index + rep.length;
  }
  out += html.slice(last);
  if (write && out !== html) fs.writeFileSync(printFile, out, 'utf8');
  return { targets, fixed: replacements.length };
}

if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  const dir = process.argv[2] || DEFAULT_OUT_DIR;
  try {
    const { targets, fixed } = fixPrintMediaRefs(dir);
    console.log(`print-media-refs: fixed ${fixed} media reference(s) in print.html (${targets} target(s))`);
  } catch (e) {
    console.error(`print-media-refs failed: ${e.message}`);
    process.exitCode = 1;
  }
}
