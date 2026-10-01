// gen-skill-refs.mjs — マニュアル章からスキル references/ を生成する（manual-ssot-authority）
// タスク 3.3（要件 5.1, 5.2, 5.8, 5.9, 1.7 / design「GenSkillRefs」）: 対応表・出力名・口調判定・本文抽出。
//
// export:
//   GENERATION_MAP  … 対応表（21 エントリ・順序固定）。入力の定義はこれだけ（設定ファイルなし）。
//   outName         … 章パス → references/ 内の出力名（index.md は直近の親ディレクトリ名付き）。
//   VOICE_MARKERS   … 口調マーカー（verify-content.mjs から移設）。普通文体と衝突する 3 語だけ否定先読み。
//   findVoice       … 一致したマーカーの語を返す（空なら口調なし）。
//   extractBody     … 章テキスト → { title, body }。構造違反・本文散文の口調で GenError を投げる。
//   readChapter     … book/src から章を読む。無ければ missing-chapter。
//   GenError        … kind: missing-chapter / bad-structure / voice-in-body / unresolvable-link。
//
// 依存: link-check.mjs の maskFences（フェンス判定の規則を二重化しない）と Node 標準のみ。
// import しただけでは何も実行しない。

import fs from 'node:fs';
import path from 'node:path';
import { maskFences } from './link-check.mjs';

export const MANUAL_BASE_URL = 'https://ekicyou.github.io/pasta/';

const GA = 'pasta-ghost-authoring';
const LC = 'pasta-lua-coding';
export const GENERATION_MAP = Object.freeze([
  ['grammar/index.md', GA],
  ['grammar/markers.md', GA],
  ['grammar/block-structure.md', GA],
  ['grammar/call-jump.md', GA],
  ['grammar/literals.md', GA],
  ['grammar/action-line.md', GA],
  ['grammar/sakura-script.md', GA],
  ['grammar/variables.md', GA],
  ['grammar/words.md', GA],
  ['grammar/actor-dictionary.md', GA],
  ['reference/pasta-toml.md', GA],
  ['lua/modules/index.md', LC],
  ['lua/modules/pasta-search.md', LC],
  ['lua/modules/pasta-persistence.md', LC],
  ['lua/modules/pasta-config.md', LC],
  ['lua/modules/pasta-sakura-script.md', LC],
  ['lua/modules/enc.md', LC],
  ['lua/modules/pasta-log.md', LC],
  ['lua/modules/mlua-stdlib.md', LC],
  ['lua/shiori-events.md', LC],
  ['reference/startup.md', LC],
].map(([chapter, skill]) => Object.freeze({ chapter, skill })));

// grammar/markers.md → markers.md、grammar/index.md → grammar-index.md、lua/modules/index.md → modules-index.md
export function outName(chapter) {
  const parts = chapter.split('/');
  const base = parts[parts.length - 1];
  return base === 'index.md' ? `${parts[parts.length - 2]}-index.md` : base;
}

// Claudia 令嬢ボイスのマーカー（部分一致）。衝突確認済みの 3 語だけ否定先読み:
// 「書かなくてよい」「ですので」「ますので」を拾わない。新たな衝突も同じ方法で集合側を直す。
export const VOICE_MARKERS = Object.freeze([
  'ですわ', 'ますわ', 'ませんわ', /ますの(?!で)/, /ですの(?!で)/, 'おほほ', 'フンッ',
  'わたくし', 'ごきげんよう', 'なさいまし', 'くださいまし', 'まし。', 'まし、',
  '参りましょう', 'まいりましょう', 'よろしくて', 'ですこと', /くてよ(?!い)/,
]);

export function findVoice(text) {
  const hits = [];
  for (const mk of VOICE_MARKERS) {
    if (typeof mk === 'string') {
      if (text.includes(mk)) hits.push(mk);
    } else {
      const m = text.match(mk);
      if (m) hits.push(m[0]);
    }
  }
  return hits;
}

export class GenError extends Error {
  constructor(kind, chapter, fields, message) {
    super(`${kind}: ${chapter}: ${message}`);
    this.name = 'GenError';
    this.kind = kind;
    this.chapter = chapter;
    Object.assign(this, fields);
  }
}

export function readChapter(chapter, repoRoot) {
  const file = path.resolve(repoRoot, 'book/src', chapter);
  if (!fs.existsSync(file)) {
    throw new GenError('missing-chapter', chapter, {}, `対応章が存在しない（book/src/${chapter}）`);
  }
  return fs.readFileSync(file, 'utf8');
}

// インラインコード `…`（同数のバッククォートで閉じる）。
const INLINE_CODE_RE = /(`+)(?!`)[\s\S]*?(?<!`)\1(?!`)/g;

export function extractBody(chapterText, chapter) {
  const lines = chapterText.replace(/\r\n?/g, '\n').split('\n');
  const masked = maskFences(chapterText).split('\n');
  const bad = (detail) => new GenError('bad-structure', chapter, { detail }, detail);

  if (!lines[0].startsWith('# ')) throw bad('先頭行が H1（`# `）でない');
  const seps = [];
  masked.forEach((l, i) => { if (l === '---') seps.push(i); });
  if (seps.length < 2) throw bad(`フェンス外の区切り行 \`---\` が 2 本未満（${seps.length} 本）`);

  let start = seps[0] + 1;
  let end = seps[seps.length - 1]; // 排他
  while (start < end && lines[start].trim() === '') start++;
  while (end > start && lines[end - 1].trim() === '') end--;

  // 散文部 = フェンス・表の行・インラインコードを除いた残り（見出し・引用は含む）。
  const hits = [];
  for (let i = start; i < end; i++) {
    const l = masked[i];
    if (l.trim().startsWith('|')) continue;
    for (const marker of findVoice(l.replace(INLINE_CODE_RE, ''))) hits.push({ line: i + 1, marker });
  }
  if (hits.length > 0) {
    throw new GenError('voice-in-body', chapter, { hits },
      `本文散文に口調マーカー: ${hits.map((h) => `L${h.line} ${h.marker}`).join(', ')}`);
  }
  return { title: lines[0].slice(2).trim(), body: lines.slice(start, end).join('\n') };
}
