// talk.mjs — 台詞部品（Claudia とアンソニーの掛け合い）の登録簿と記法の解析。
// manual-claudia-theme タスク 1.1（要件 3.2, 3.3, 3.4, 3.8, 4.2, 4.8, 10.7 / design「TalkCore」）。
//
// 登録簿（SPEAKERS）は話し手と表情の唯一の定義。話し手や表情の追加はこの配列の変更と
// 顔画像（book/src/img/claudia/）の追加で完結する。
//
// 記法（design「データモデル / 記法の文法」）:
//   > 【表情】本文            … 既定の話し手（Claudia）の表情
//   > 【話し手】本文          … その話し手の「素」
//   > 【話し手：表情】本文    … 区切りは全角コロン「：」のみ
//   話し手名はカタカナの日本語名だけ（別名・欧文名なし）。
//
// 共用 export（talk-html.mjs・gen-skill-refs.mjs・verify-content.mjs などが import する）:
//   SPEAKERS / DEFAULT_SPEAKER_ID / DEFAULT_FACE / FACE_DIR / MAX_FACE_BYTES / TALK_LINE_RE
//   parseTalkTag(inner) … 【】の内側を解析
//   faceFilePath(speaker, face) … 顔画像への相対パス（book/src と出力根から）
//
//   scanTalk(markdown) … フェンス外の台詞ブロックの列挙と不正の全件列挙
//   checkRegion(blocks, lines, range, opts) … 導入・締めの範囲の検査
//   faceStats(blocksByChapter, regionsByChapter) … 表情の集計表
//
// 純関数のみ・入出力なし（--stats CLI を除く）。

import fs from 'node:fs';
import path from 'node:path';
import { REPO_ROOT, listMarkdownFiles, maskFences } from '../link-check.mjs';

const speaker = (id, name, latin, side, faces) =>
  Object.freeze({ id, name, latin, side, faces: Object.freeze(faces) });

export const SPEAKERS = Object.freeze([
  speaker('claudia', 'クローディア', 'Claudia', 'left', {
    素: 'f0', 照れ: 'f1', 驚き: 'f2', 不安: 'f3', 落胆: 'f4', 高笑い: 'f5', 目閉じ: 'f6',
    不機嫌: 'f7', 冷笑: 'f8', 照れ怒り: 'f9', にっこり: 'f25', したり顔: 'f26', 考え中: 'f27', お辞儀: 'f28',
  }),
  speaker('anthony', 'アンソニー', 'Anthony', 'right', { 素: 'f10', 刮目: 'f11' }),
]);

export const DEFAULT_SPEAKER_ID = 'claudia';
export const DEFAULT_FACE = '素';
export const FACE_DIR = 'img/claudia';
export const MAX_FACE_BYTES = 20480;

// 台詞の開始行（字下げも拾う。入れ子の引用・リスト内の判定は走査側で行う）。
export const TALK_LINE_RE = /^\s*>\s*【/;

const SEP = '：';
const DEFAULT_SPEAKER = SPEAKERS.find((s) => s.id === DEFAULT_SPEAKER_ID);
const byName = (name) => SPEAKERS.find((s) => s.name === name);
const candidates = (names) => `（候補: ${names.join('・')}）`;

const fail = (kind, detail) => ({ ok: false, kind, detail });

function resolveFace(sp, face) {
  if (!Object.hasOwn(sp.faces, face)) {
    return fail('unknown-expression',
      `${sp.name} に表情「${face}」はありません${candidates(Object.keys(sp.faces))}`);
  }
  return { ok: true, tag: { speaker: sp, face, file: sp.faces[face] } };
}

// 【】の内側を解析する。成功なら { ok: true, tag }、失敗なら { ok: false, kind, detail }。
export function parseTalkTag(inner) {
  if (inner === '') return fail('malformed-tag', '空のタグです');
  if (/[\s【】:]/.test(inner)) {
    return fail('malformed-tag',
      `タグ「${inner}」に空白・括弧・半角コロンは使えません（区切りは全角コロン「${SEP}」）`);
  }
  const parts = inner.split(SEP);
  if (parts.length > 2) return fail('malformed-tag', `タグ「${inner}」の区切り「${SEP}」は 1 つまでです`);
  if (parts.some((p) => p === '')) return fail('malformed-tag', `タグ「${inner}」の話し手か表情が空です`);

  if (parts.length === 2) {
    const sp = byName(parts[0]);
    if (!sp) {
      return fail('unknown-speaker',
        `話し手「${parts[0]}」はいません${candidates(SPEAKERS.map((s) => s.name))}`);
    }
    return resolveFace(sp, parts[1]);
  }

  // 単独の名前: 話し手名ならその話し手の「素」、そうでなければ既定の話し手の表情。
  const sp = byName(inner);
  if (sp) return resolveFace(sp, DEFAULT_FACE);
  if (Object.hasOwn(DEFAULT_SPEAKER.faces, inner)) return resolveFace(DEFAULT_SPEAKER, inner);
  return fail('unknown-expression',
    `「${inner}」は ${DEFAULT_SPEAKER.name} の表情にも話し手にもありません`
    + candidates([...Object.keys(DEFAULT_SPEAKER.faces), ...SPEAKERS.map((s) => s.name)]));
}

// 話し手と表情から顔画像への相対パスを組み立てる（例: 'img/claudia/f5.png'）。
export function faceFilePath(sp, face) {
  if (!Object.hasOwn(sp.faces, face)) throw new Error(`${sp.name} に表情「${face}」はありません`);
  return `${FACE_DIR}/${sp.faces[face]}.png`;
}

// ---- 章の Markdown の走査（タスク 1.2 / 要件 3.7, 4.1, 4.3, 4.7, 10.3） ----
// フェンス外の台詞ブロックを列挙し、不正を行番号付きで全件返す（最初の 1 件で止めない）。
// フェンス判定は link-check の maskFences（フェンス内の行を空行にする・行数は保つ・LF 正規化）。
// design「台詞ブロックの判定（Markdown 側）」の流れに従う。

// 引用の行: 字下げ・リスト記号・引用記号の並び・内容に分ける。
const QUOTE_RE = /^(\s*)((?:[-*+]|\d{1,9}[.)])\s+)?((?:>\s?)+)(.*)$/;
const INNER_FENCE_RE = /^ {0,3}(`{3,}|~{3,})/;
// 台詞の直前に空行が無くてもよい行（区切り・見出し）。
const EDGE_RE = /^(-{3,}|#{1,6}(\s.*)?)\s*$/;

export function scanTalk(markdown) {
  const lines = maskFences(markdown).split('\n');
  const blocks = [];
  const errors = [];
  const err = (kind, line, detail) => errors.push({ kind, line, detail });
  let cur = null; // 解析中のブロック { line, endLine, tag, parts, fence, table, noBracket }

  const close = () => {
    if (cur === null) return;
    const body = cur.parts.join('\n').trim();
    // 】欠落のタグは本文の境界が分からないので empty-body を重ねて出さない。
    if (body === '' && !cur.noBracket) err('empty-body', cur.line, '台詞の本文が空です');
    else if (cur.tag) blocks.push({ line: cur.line, endLine: cur.endLine, tag: cur.tag, body });
    cur = null;
  };

  lines.forEach((text, i) => {
    const no = i + 1;
    if (text.trim() === '') { close(); return; }
    const q = text.match(QUOTE_RE);
    if (!q) {
      // 台詞の直後の空行なしの行は遅延継続（引用に吸い込まれる）。
      if (cur) { err('missing-blank-line', no, '台詞の後ろに空行が必要です（遅延継続）'); close(); }
      return;
    }
    const [, indent, list, marks, rest] = q;
    const depth = (marks.match(/>/g) || []).length;
    const content = rest.trimStart();
    const isTag = content.startsWith('【');

    if (indent !== '' || list || depth > 1) {
      if (isTag) {
        err('nested-talk', no, '台詞は字下げ・リスト・引用の中に置けません（行頭の引用ブロックだけ）');
      } else if (cur && !cur.fence) {
        err('unsupported-content', no, '台詞の中に入れ子の引用・リストは書けません');
      }
      return;
    }

    if (isTag) {
      const prev = i > 0 ? lines[i - 1] : '';
      if (cur || (prev.trim() !== '' && !EDGE_RE.test(prev))) {
        err('missing-blank-line', no, '台詞の前に空行が必要です');
      }
      close();
      cur = { line: no, endLine: no, tag: null, parts: [], fence: false, table: false };
      const end = content.indexOf('】');
      if (end < 0) {
        err('malformed-tag', no, 'タグの閉じ括弧「】」がありません');
        cur.noBracket = true;
        return;
      }
      const r = parseTalkTag(content.slice(1, end));
      if (r.ok) cur.tag = r.tag;
      else err(r.kind, no, r.detail);
      cur.parts.push(content.slice(end + 1));
      return;
    }

    if (!cur) return; // 台詞でない通常の引用
    cur.endLine = no;
    if (INNER_FENCE_RE.test(rest)) {
      if (!cur.fence) err('unsupported-content', no, '台詞の中にコードブロックは書けません');
      cur.fence = !cur.fence;
      return;
    }
    if (cur.fence) return;
    const isTable = content.startsWith('|');
    if (isTable && !cur.table) err('unsupported-content', no, '台詞の中に表は書けません');
    cur.table = isTable;
    if (!isTable) cur.parts.push(rest);
  });
  close();
  return { blocks, errors };
}

// ---- 領域の検査（導入・締め） ----
// 範囲（0 始まり・end 排他）の非空行がすべて台詞ブロックに属し、各話し手の台詞が 1 つ以上あるか。
// cover: true（表紙）では、扉 <section class="claudia-hero">〜</section> の内側と HTML ブロックも許す。
const HERO_OPEN_RE = /^\s*<section\s[^>]*class="claudia-hero"/;
const HERO_CLOSE_RE = /<\/section>/;

export function checkRegion(blocks, markdownLines, range, opts) {
  const errors = [];
  const inRange = blocks.filter((b) => b.line - 1 >= range.start && b.line - 1 < range.end);
  const talkLine = (i) => inRange.some((b) => i + 1 >= b.line && i + 1 <= b.endLine);
  let hero = false;
  let html = false;
  let prevBad = false;
  for (let i = range.start; i < range.end; i++) {
    const text = markdownLines[i] ?? '';
    if (text.trim() === '') { html = false; prevBad = false; continue; }
    let ok = talkLine(i);
    if (opts.cover && !ok) {
      if (HERO_OPEN_RE.test(text)) hero = true;
      if (/^ {0,3}</.test(text)) html = true; // HTML ブロックは空行まで続く
      ok = hero || html;
      if (hero && HERO_CLOSE_RE.test(text)) hero = false;
    }
    if (!ok && !prevBad) errors.push({ kind: 'prose-in-region', line: i + 1, detail: '導入・締めには台詞以外を置けません' });
    prevBad = !ok;
  }
  for (const sp of SPEAKERS) {
    if (!inRange.some((b) => b.tag.speaker === sp)) {
      errors.push({ kind: 'missing-speaker', line: range.start + 1, detail: `${sp.name} の台詞がありません` });
    }
  }
  return errors;
}

// ---- 表情の集計（--stats / 要件 4.6 の人のレビューを支える表示。合否は出さない） ----
// blocksByChapter: 章 → 台詞ブロック。regionsByChapter: 章 → { intro, outro }（0 始まり・end 排他）。
const faceLabel = (b) => `${b.tag.speaker.name}：${b.tag.face}`;

export function faceStats(blocksByChapter, regionsByChapter = new Map()) {
  const all = [...blocksByChapter.values()].flat();
  const out = ['## 表情の出現数', '', '| 話し手 | 表情 | 出現数 |', '| --- | --- | --- |'];
  for (const sp of SPEAKERS) {
    for (const face of Object.keys(sp.faces)) {
      const n = all.filter((b) => b.tag.speaker === sp && b.tag.face === face).length;
      out.push(`| ${sp.name} | ${face} | ${n} |`);
    }
  }
  out.push('', '## 章ごとの導入・締めの表情', '', '| 章 | 導入 | 締め |', '| --- | --- | --- |');
  for (const [chapter, blocks] of blocksByChapter) {
    const reg = regionsByChapter.get(chapter);
    const faces = (r) => {
      const hit = reg ? blocks.filter((b) => b.line - 1 >= r.start && b.line - 1 < r.end) : [];
      return hit.length ? hit.map(faceLabel).join(', ') : '—';
    };
    out.push(`| ${chapter} | ${faces(reg?.intro)} | ${faces(reg?.outro)} |`);
  }
  return out.join('\n');
}

// CLI: node book/tools/talk/talk.mjs --stats（全章 = book/src 配下の SUMMARY.md 以外の .md。常に exit 0）
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1])) && process.argv.includes('--stats')) {
  const src = path.join(REPO_ROOT, 'book/src');
  const blocksByChapter = new Map();
  const regionsByChapter = new Map();
  for (const file of listMarkdownFiles(src).sort()) {
    const chapter = path.relative(src, file).split(path.sep).join('/');
    if (chapter === 'SUMMARY.md') continue;
    const text = fs.readFileSync(file, 'utf8');
    blocksByChapter.set(chapter, scanTalk(text).blocks);
    // 導入=H1 の次行〜最初の ---・締め=最後の --- の次行〜末尾。区切りの規則は gen-skill-refs の extractBody と同じ。
    // gen-skill-refs が talk.mjs を import するため、循環を避けてここに複製している。
    const masked = maskFences(text).split('\n');
    const seps = masked.flatMap((l, i) => (l === '---' ? [i] : []));
    if (seps.length >= 2) {
      regionsByChapter.set(chapter, {
        intro: { start: 1, end: seps[0] },
        outro: { start: seps[seps.length - 1] + 1, end: masked.length },
      });
    }
  }
  console.log(faceStats(blocksByChapter, regionsByChapter));
  process.exit(0);
}
