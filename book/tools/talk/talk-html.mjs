// talk-html.mjs — mdBook の出力 HTML の台詞引用ブロックを、顔アイコン付きの吹き出し HTML に置き換える。
// manual-claudia-theme タスク 3.1（要件 3.1, 3.3, 3.6, 3.9, 3.10, 3.11, 8.4, 8.6, 10.7 / design「TalkHtmlTransform」）。
//
// 対象: 出力フォルダの *.html すべて（print.html を含む・名前順）。
//   属性の無い <blockquote> の直後の最初の <p> が「【」で始まるものだけを置き換え、それ以外は 1 バイトも変えない。
//   mdBook 0.5.x の描画形: `> 【表情】本文` → `<blockquote>\n<p>【表情】本文</p>\n</blockquote>`。
//   注記（`> [!NOTE]`）は `<blockquote class="blockquote-tag …">` で最初の <p> が見出しなので対象外。
// 出力: design「データモデル / 出力 HTML 契約」の構造。本文はタグを除いた残りの HTML をそのまま移す。
//   顔画像の src は、その出力ファイルの位置から出力フォルダの根への相対パス（深さ 1 なら ../img/claudia/…）。
// 失敗: 未知のタグ・入れ子の台詞（引用やリストの中の台詞・入れ子の引用を含む台詞）を全件集め、
//   出力ファイルとタグを示して exit 1。このとき、どのファイルにも書き込まない（全ファイルを検査してから書く）。
// 依存: talk/talk.mjs だけ（design「Allowed Dependencies」）。Node 標準ライブラリのみ。
//
// CLI: node book/tools/talk/talk-html.mjs [book-out-dir]（既定 book/book）

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseTalkTag, faceFilePath } from './talk.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_OUT_DIR = path.resolve(here, '../../book');

const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ESC[c]);

// 台詞 1 件の HTML（出力 HTML 契約）。bodyHtml はタグを除いた本文の HTML（末尾改行込み）。
export function renderTalk(tag, bodyHtml, rootPrefix) {
  const sp = tag.speaker;
  return `<div class="talk talk-${esc(sp.id)} talk-${esc(sp.side)}">\n`
    + `<img class="talk-face" src="${esc(rootPrefix + faceFilePath(sp, tag.face))}" alt="${esc(`${sp.name}（${tag.face}）`)}" width="56" height="56">\n`
    + '<div class="talk-bubble">\n'
    + `<span class="talk-name" aria-hidden="true">${esc(sp.latin)}</span>\n`
    + bodyHtml
    + '</div>\n'
    + '</div>';
}

// 引用とリスト項目の開閉だけを拾う（入れ子の判定に使う）。
const TOKEN_RE = /<(\/?)(blockquote|li)\b[^>]*>/gi;
// 台詞の開始: 属性の無い <blockquote> の直後の最初の <p> が「【」で始まる。
const TALK_HEAD_RE = /[ \t\r\n]*<p>【/y;
// 内側の先頭のタグ（】の欠落・タグ内のマークアップは malformed-tag）。
const TAG_RE = /^[ \t\r\n]*<p>【([^】<\n]*)】/;
const LEAD_WS_RE = /^[ \t\r\n]+/;

// 内側の HTML からタグを取り除いた本文を作る。失敗なら { error }。
function splitTag(inner) {
  const m = inner.match(TAG_RE);
  if (!m) {
    const head = inner.replace(/^[ \t\r\n]*<p>/, '').split(/<|\n/)[0];
    return { tag: head, error: { kind: 'malformed-tag', detail: 'タグの閉じ括弧「】」がないか、タグの中に書式があります' } };
  }
  const r = parseTalkTag(m[1]);
  if (!r.ok) return { tag: m[1], error: { kind: r.kind, detail: r.detail } };
  let rest = inner.slice(m[0].length).replace(LEAD_WS_RE, '');
  // タグだけの段落は空の <p> を残さない。
  rest = rest.startsWith('</p>') ? rest.slice(4).replace(LEAD_WS_RE, '') : `<p>${rest}`;
  if (rest.trim() === '') return { tag: m[1], error: { kind: 'empty-body', detail: '台詞の本文が空です' } };
  if (!rest.endsWith('\n')) rest += '\n';
  return { tag: m[1], parsed: r.tag, body: rest };
}

// 1 ファイル分の変換（純関数）。rootPrefix はそのファイルから出力根への相対（'' / '../' / '../../' …）。
// 戻り値: { html, converted, errors: [{ kind, tag, detail }] }。errors があっても html は部分変換を返すが、書き込み側は使わない。
export function transformHtml(html, rootPrefix) {
  const stack = []; // { name, talk, start, innerStart, nested }
  const errors = [];
  const edits = [];
  TOKEN_RE.lastIndex = 0;
  for (let m; (m = TOKEN_RE.exec(html));) {
    const name = m[2].toLowerCase();
    if (m[1] === '') {
      const end = m.index + m[0].length;
      TALK_HEAD_RE.lastIndex = end;
      const talk = name === 'blockquote' && m[0] === '<blockquote>' && TALK_HEAD_RE.test(html);
      if (name === 'blockquote') {
        const outerTalk = stack.find((e) => e.talk);
        if (outerTalk) outerTalk.nested = true; // 入れ子の引用を含む台詞（報告は外側の 1 件だけ）
        else if (talk && stack.length > 0) {
          errors.push({ kind: 'nested-talk', tag: splitTag(html.slice(end)).tag, detail: '台詞は引用やリストの中に置けません' });
        }
      }
      stack.push({ name, talk: talk && stack.length === 0, start: m.index, innerStart: end, nested: false });
      continue;
    }
    // 閉じタグ: 対応する開きまで戻す（対応しない閉じタグは無視）。
    const at = stack.map((e) => e.name).lastIndexOf(name);
    if (at < 0) continue;
    const [e] = stack.splice(at);
    if (!e.talk) continue;
    const s = splitTag(html.slice(e.innerStart, m.index));
    if (e.nested) errors.push({ kind: 'nested-talk', tag: s.tag, detail: '台詞の中に入れ子の引用は書けません' });
    else if (s.error) errors.push({ tag: s.tag, ...s.error });
    else edits.push({ start: e.start, end: m.index + m[0].length, text: renderTalk(s.parsed, s.body, rootPrefix) });
  }
  let out = html;
  for (const ed of edits.reverse()) out = out.slice(0, ed.start) + ed.text + out.slice(ed.end);
  return { html: out, converted: edits.length, errors };
}

// 出力フォルダを変換する。全ファイルを検査し、1 件でも不正があれば何も書かずに例外を投げる。
// 戻り値: { files（出力根からの相対・名前順）, converted, filesChanged }。
export function transformDir(outDir, { write = true } = {}) {
  if (!fs.existsSync(outDir) || !fs.statSync(outDir).isDirectory()) {
    throw new Error(`出力フォルダがありません: ${outDir}`);
  }
  const files = fs.readdirSync(outDir, { recursive: true })
    .map((p) => p.split(path.sep).join('/'))
    .filter((p) => p.toLowerCase().endsWith('.html'))
    .sort();
  const pending = [];
  const problems = [];
  let converted = 0;
  for (const rel of files) {
    const file = path.join(outDir, rel);
    const html = fs.readFileSync(file, 'utf8');
    const r = transformHtml(html, '../'.repeat(rel.split('/').length - 1));
    for (const e of r.errors) problems.push(`${rel}: ${e.kind} 【${e.tag}】 ${e.detail}`);
    converted += r.converted;
    if (r.html !== html) pending.push([file, r.html]);
  }
  if (problems.length > 0) {
    throw new Error(`台詞を変換できません（${problems.length} 件・何も書き込んでいません）:\n${problems.join('\n')}`);
  }
  if (write) for (const [file, html] of pending) fs.writeFileSync(file, html, 'utf8');
  return { files, converted, filesChanged: pending.length };
}

if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  const dir = process.argv[2] || DEFAULT_OUT_DIR;
  try {
    const { files, converted, filesChanged } = transformDir(dir);
    console.log(`talk-html: scanned ${files.length} html file(s) under ${dir}`);
    console.log(`talk-html: converted ${converted} talk block(s) across ${filesChanged} file(s)`);
  } catch (e) {
    console.error(`talk-html failed: ${e.message}`);
    process.exitCode = 1;
  }
}
