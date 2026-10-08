// gen-skill-refs.mjs — マニュアル章からスキル references/ を生成する（manual-ssot-authority）
// タスク 3.3（要件 5.1, 5.2, 5.8, 5.9, 1.7 / design「GenSkillRefs」）: 対応表・出力名・口調判定・本文抽出。
//
// export:
//   GENERATION_MAP  … 対応表（23 エントリ・順序固定）。入力の定義はこれだけ（設定ファイルなし）。
//   outName         … 章パス → references/ 内の出力名（index.md は直近の親ディレクトリ名付き）。
//   VOICE_MARKERS   … 口調マーカー（verify-content.mjs から移設）。普通文体と衝突する 3 語だけ否定先読み。
//   findVoice       … 一致したマーカーの語を返す（空なら口調なし）。
//   chapterRegions  … 章テキスト → { lines, intro, body, outro }（0 始まり・end 排他）。構造違反で bad-structure。
//                     （manual-claudia-theme タスク 3.2・要件 4.3）
//   extractBody     … 章テキスト → { title, body }。構造違反・本文の台詞・本文散文の口調で GenError を投げる。
//   readChapter     … book/src から章を読む。無ければ missing-chapter。
//   GenError        … kind: missing-chapter / bad-structure / talk-in-body / voice-in-body / unresolvable-link。
// タスク 3.4（要件 5.3–5.7, 7.1–7.4, 7.7）:
//   rewriteLinks    … フェンス外・インラインコード外のインラインリンクを書き換える（同一スキル宛て → 兄弟ファイル名、
//                     非生成章・別スキル宛て → 公開 URL、絶対 URL・#anchor 不変、画像・非 .md・book/src 外 → unresolvable-link）。
//   renderEntry     … 固定 2 行ヘッダ＋H1＋本文（LF・末尾改行 1 つ・BOM なし・時刻／環境値なし）。
//   generateAll     … 全エントリをメモリ上で生成（リポジトリ相対パス → 内容）。最初の GenError で投げる。
//   writeAll        … 全生成の成功後に書き出す。LF 正規化後に同一なら書き換えない。
//   checkAll        … { stale, orphans, fixCommand }（LF 正規化して比較・孤立は 1 行目の固定ヘッダで判定）。
//
// CLI: node book/tools/gen-skill-refs.mjs        … 書き出し（exit 0）
//      node book/tools/gen-skill-refs.mjs --check … 照合（不一致・孤立を全件列挙して exit 1／一致で exit 0）
//      GenError → 標準エラーへ章パス付きで exit 1 ／ 予期しない例外・不明な引数 → exit 2。
//
// 依存: link-check.mjs の LINK_RE・maskFences（規則を二重化しない）、talk/talk.mjs の TALK_LINE_RE・scanTalk
// （台詞の判定を二重化しない。talk.mjs は本ファイルを import しない）と Node 標準のみ。
// import しただけでは何も実行しない。

import fs from 'node:fs';
import path from 'node:path';
import { LINK_RE, REPO_ROOT, maskFences } from './link-check.mjs';
import { TALK_LINE_RE, scanTalk } from './talk/talk.mjs';

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
  ['lua/script-api.md', LC],
  ['internals/internal-modules.md', LC],
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

// 章を導入・本文・締めの領域に分ける（manual-claudia-theme タスク 3.2 / design「ChapterStructure」）。
// 範囲はすべて 0 始まり・end 排他の行番号。lines は LF に直した全行。
//   intro … H1 の次行〜最初のフェンス外の `---`
//   body  … 最初と最後のフェンス外の `---` の間から、前後の空行を除いた範囲（extractBody の本文）
//   outro … 最後のフェンス外の `---` の次行〜末尾
// 構造（H1・区切り 2 本以上）だけを見て bad-structure を投げる。台詞・口調は見ない（非生成章にも使うため）。
// 区切りの規則を変えるときは、talk/talk.mjs の --stats にある複製（循環 import 回避）も合わせる。
export function chapterRegions(chapterText, chapter) {
  const lines = chapterText.replace(/\r\n?/g, '\n').split('\n');
  const masked = maskFences(chapterText).split('\n');
  const bad = (detail) => new GenError('bad-structure', chapter, { detail }, detail);

  if (!lines[0].startsWith('# ')) throw bad('先頭行が H1（`# `）でない');
  const seps = [];
  masked.forEach((l, i) => { if (l === '---') seps.push(i); });
  if (seps.length < 2) throw bad(`フェンス外の区切り行 \`---\` が 2 本未満（${seps.length} 本）`);

  const last = seps[seps.length - 1];
  let start = seps[0] + 1;
  let end = last; // 排他
  while (start < end && lines[start].trim() === '') start++;
  while (end > start && lines[end - 1].trim() === '') end--;
  return {
    lines,
    intro: { start: 1, end: seps[0] },
    body: { start, end },
    outro: { start: last + 1, end: lines.length },
  };
}

export function extractBody(chapterText, chapter) {
  const { lines, body: { start, end } } = chapterRegions(chapterText, chapter);
  const masked = maskFences(chapterText).split('\n');

  // 本文（フェンス外）の台詞の開始行（manual-claudia-theme タスク 3.2 / 要件 4.4）。生成対象章と内部設計章
  // （verify-content の I-structure）は、台詞を導入・締めにだけ置ける。口調より先に見る（台詞は口調を含むため）。
  // TALK_LINE_RE は行頭・字下げの台詞だけを拾い、入れ子の引用・リストの中（`> > 【`・`- > 【`）は拾わない。
  // そこは scanTalk の nested-talk（字下げ・リスト・入れ子の台詞の開始行で必ず出る）で補う。
  const talk = new Set();
  for (let i = start; i < end; i++) if (TALK_LINE_RE.test(masked[i])) talk.add(i + 1);
  for (const e of scanTalk(chapterText).errors) {
    if (e.kind === 'nested-talk' && e.line - 1 >= start && e.line - 1 < end) talk.add(e.line);
  }
  if (talk.size > 0) {
    const talkHits = [...talk].sort((a, b) => a - b).map((line) => ({ line }));
    throw new GenError('talk-in-body', chapter, { hits: talkHits },
      `${talkHits.map((h) => `L${h.line}`).join(', ')}（生成対象章・内部設計章の台詞は導入・締めにだけ置ける）`);
  }

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

// ---- タスク 3.4: リンク書き換え・ヘッダ・書き出し・照合 ----

export const GENERATED_MARK = '<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->';
export const FIX_COMMAND = 'node book/tools/gen-skill-refs.mjs';
const SKILLS = [...new Set(GENERATION_MAP.map((e) => e.skill))];
const BY_CHAPTER = new Map(GENERATION_MAP.map((e) => [e.chapter, e]));
const toHtml = (chapter) => chapter.replace(/\.md$/, '.html');
const relOutPath = (e) => `.claude/skills/${e.skill}/references/${outName(e.chapter)}`;
const lf = (s) => s.replace(/\r\n?/g, '\n');

// 1 つのリンク先を書き換える（design「リンク書き換え規則」1–3）。
function rewriteTarget(target, entry) {
  if (/^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith('#')) return target;
  const hash = target.indexOf('#');
  const file = hash < 0 ? target : target.slice(0, hash);
  const anchor = hash < 0 ? '' : target.slice(hash);
  const resolved = path.posix.normalize(path.posix.join(path.posix.dirname(entry.chapter), file));
  if (file.startsWith('/') || !file.endsWith('.md') || resolved === '..' || resolved.startsWith('../')) {
    throw new GenError('unresolvable-link', entry.chapter, { target },
      `解決できないリンク（画像・非 .md・book/src 外）: ${target}`);
  }
  const dest = BY_CHAPTER.get(resolved);
  if (dest && dest.skill === entry.skill) return outName(resolved) + anchor;
  return MANUAL_BASE_URL + toHtml(resolved) + anchor;
}

// フェンス外・インラインコード外のインラインリンクだけを書き換える。
export function rewriteLinks(body, entry) {
  const masked = maskFences(body).split('\n');
  return lf(body).split('\n').map((line, i) => {
    if (masked[i] === '') return line; // フェンス内（または空行）
    let out = '';
    let last = 0;
    for (const code of line.matchAll(INLINE_CODE_RE)) {
      out += rewriteSegment(line.slice(last, code.index), entry) + code[0];
      last = code.index + code[0].length;
    }
    return out + rewriteSegment(line.slice(last), entry);
  }).join('\n');
}

function rewriteSegment(text, entry) {
  return text.replace(LINK_RE, (m, raw) => {
    const angled = raw.startsWith('<');
    const t = rewriteTarget(angled ? raw.slice(1, -1) : raw, entry);
    return m.slice(0, m.length - raw.length) + (angled ? `<${t}>` : t);
  });
}

export function renderEntry(entry, repoRoot) {
  const { title, body } = extractBody(readChapter(entry.chapter, repoRoot), entry.chapter);
  return [
    GENERATED_MARK,
    `<!-- このファイルは pasta マニュアル「${title}」（${MANUAL_BASE_URL}${toHtml(entry.chapter)}）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->`,
    '',
    `# ${title}`,
    '',
    rewriteLinks(body, entry),
    '',
  ].join('\n');
}

// 全エントリをメモリ上で生成する（最初の GenError で投げる）。リポジトリ相対パス → 内容。
export function generateAll(repoRoot) {
  return new Map(GENERATION_MAP.map((e) => [relOutPath(e), renderEntry(e, repoRoot)]));
}

const readIfExists = (file) => (fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : null);

// 書き出しモード。全生成が成功してから書く。LF 正規化後に同一なら書き換えない。
export function writeAll(repoRoot) {
  const all = generateAll(repoRoot);
  const written = [];
  const unchanged = [];
  for (const [rel, content] of all) {
    const file = path.join(repoRoot, rel);
    const cur = readIfExists(file);
    if (cur !== null && lf(cur) === content) {
      unchanged.push(rel);
      continue;
    }
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content);
    written.push(rel);
  }
  return { written, unchanged };
}

// 照合モード。stale（不一致・欠落）と orphans（生成ヘッダを持つ対応表外ファイル）を全件返す。
export function checkAll(repoRoot) {
  const all = generateAll(repoRoot);
  const stale = [];
  for (const [rel, content] of all) {
    const cur = readIfExists(path.join(repoRoot, rel));
    if (cur === null || lf(cur) !== content) stale.push(rel);
  }
  const orphans = [];
  for (const skill of SKILLS) {
    const dir = path.join(repoRoot, '.claude/skills', skill, 'references');
    if (!fs.existsSync(dir)) continue;
    for (const name of fs.readdirSync(dir).filter((n) => n.endsWith('.md')).sort()) {
      const rel = `.claude/skills/${skill}/references/${name}`;
      if (all.has(rel)) continue;
      const first = fs.readFileSync(path.join(dir, name), 'utf8').split(/\r?\n/, 1)[0];
      if (first === GENERATED_MARK) orphans.push(rel);
    }
  }
  return { stale, orphans, fixCommand: FIX_COMMAND };
}

// CLI: node book/tools/gen-skill-refs.mjs [--check]
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  try {
    const args = process.argv.slice(2);
    if (args.length > 1 || (args.length === 1 && args[0] !== '--check')) {
      console.error(`usage: ${FIX_COMMAND} [--check]`);
      process.exit(2);
    }
    if (args[0] === '--check') {
      const r = checkAll(REPO_ROOT);
      if (r.stale.length === 0 && r.orphans.length === 0) {
        console.log(`gen-skill-refs --check: OK（${GENERATION_MAP.length} 件すべて最新・孤立なし）`);
        process.exit(0);
      }
      for (const p of r.stale) console.log(`STALE ${p}`);
      for (const p of r.orphans) console.log(`ORPHAN ${p}`);
      console.log(`再生成: \`${r.fixCommand}\` を実行してコミット`);
      console.log('孤立ファイルは削除するか対応表へ追加');
      process.exit(1);
    }
    const w = writeAll(REPO_ROOT);
    for (const p of w.written) console.log(`WROTE ${p}`);
    console.log(`gen-skill-refs: 書き出し ${w.written.length} 件・変更なし ${w.unchanged.length} 件`);
    process.exit(0);
  } catch (e) {
    if (e instanceof GenError) {
      console.error(`gen-skill-refs: ${e.message}`);
      process.exit(1);
    }
    console.error(`gen-skill-refs failed: ${e && e.stack ? e.stack : e}`);
    process.exit(2);
  }
}
