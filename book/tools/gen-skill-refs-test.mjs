// gen-skill-refs-test.mjs — gen-skill-refs の自動検証（manual-ssot-authority タスク 3.3・3.4 /
// 要件 5.1–5.9, 1.7, 7.1–7.4, 7.7）。
//
// 検証方針（design「Testing Strategy / Unit Tests」）:
//   - outName と実物の GENERATION_MAP（21 エントリ・(skill, outName) 重複なし）。
//   - findVoice: 口調マーカーの検出と、普通文体と衝突する 3 語（否定先読み）の非検出。
//   - extractBody: 導入・締めの除去、本文内 `---` の保持、bad-structure、voice-in-body（行番号つき）、
//     フェンス・表の行・インラインコード内の無視（4 連バッククォートのフェンスを含む）、LF/CRLF の同一性。
//   - readChapter: 章欠落で missing-chapter。
//   - rewriteLinks: 同一スキル宛て → 兄弟ファイル名、非生成章・別スキル宛て → 公開 URL、絶対 URL・#anchor 不変、
//     画像・非 .md・book/src 外 → unresolvable-link、フェンス内・インラインコード内は不変。
//   - renderEntry: 固定 2 行ヘッダ・LF・末尾改行 1 つ・BOM なし、LF/CRLF 入力でバイト一致。
//   - generateAll / writeAll / checkAll（tmp サンドボックス）: stale（章だけ変更・生成物だけ手編集）、
//     CRLF 化だけは一致、orphan、部分書き出しなし、同一内容は書き換えない。
//   - CLI（サンドボックスへツールを複製）: 書き出し直後の --check が exit 0、STALE/ORPHAN 報告と exit 1、
//     GenError で exit 1。
//   - 実リポジトリ: 21 章すべてが抽出・口調判定・リンク書き換えを通る（メモリ上のみ）。
//   実リポジトリの book/src・スキルには書き込まない。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  GENERATION_MAP,
  VOICE_MARKERS,
  MANUAL_BASE_URL,
  GenError,
  outName,
  findVoice,
  extractBody,
  readChapter,
  rewriteLinks,
  renderEntry,
  generateAll,
  writeAll,
  checkAll,
} from './gen-skill-refs.mjs';
import { REPO_ROOT, LINK_RE, maskFences } from './link-check.mjs';

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
// fn が投げた GenError を返す（投げなければ null）。
function thrown(fn) {
  try {
    fn();
    return null;
  } catch (e) {
    return e;
  }
}

// ============================================================
log('\n[A] outName / GENERATION_MAP');
check('A-1 grammar/markers.md → markers.md', outName('grammar/markers.md') === 'markers.md');
check('A-2 grammar/index.md → grammar-index.md', outName('grammar/index.md') === 'grammar-index.md');
check('A-3 lua/modules/index.md → modules-index.md', outName('lua/modules/index.md') === 'modules-index.md');
check('A-4 reference/pasta-toml.md → pasta-toml.md', outName('reference/pasta-toml.md') === 'pasta-toml.md');

const EXPECTED_MAP = [
  ['grammar/index.md', 'pasta-ghost-authoring'],
  ['grammar/markers.md', 'pasta-ghost-authoring'],
  ['grammar/block-structure.md', 'pasta-ghost-authoring'],
  ['grammar/call-jump.md', 'pasta-ghost-authoring'],
  ['grammar/literals.md', 'pasta-ghost-authoring'],
  ['grammar/action-line.md', 'pasta-ghost-authoring'],
  ['grammar/sakura-script.md', 'pasta-ghost-authoring'],
  ['grammar/variables.md', 'pasta-ghost-authoring'],
  ['grammar/words.md', 'pasta-ghost-authoring'],
  ['grammar/actor-dictionary.md', 'pasta-ghost-authoring'],
  ['reference/pasta-toml.md', 'pasta-ghost-authoring'],
  ['lua/modules/index.md', 'pasta-lua-coding'],
  ['lua/modules/pasta-search.md', 'pasta-lua-coding'],
  ['lua/modules/pasta-persistence.md', 'pasta-lua-coding'],
  ['lua/modules/pasta-config.md', 'pasta-lua-coding'],
  ['lua/modules/pasta-sakura-script.md', 'pasta-lua-coding'],
  ['lua/modules/enc.md', 'pasta-lua-coding'],
  ['lua/modules/pasta-log.md', 'pasta-lua-coding'],
  ['lua/modules/mlua-stdlib.md', 'pasta-lua-coding'],
  ['lua/shiori-events.md', 'pasta-lua-coding'],
  ['reference/startup.md', 'pasta-lua-coding'],
];
check('A-5 GENERATION_MAP は 21 エントリ', GENERATION_MAP.length === 21, String(GENERATION_MAP.length));
check('A-6 GENERATION_MAP は design の確定表と順序・内容一致',
  JSON.stringify(GENERATION_MAP.map((e) => [e.chapter, e.skill])) === JSON.stringify(EXPECTED_MAP));
const keys = GENERATION_MAP.map((e) => `${e.skill}/${outName(e.chapter)}`);
check('A-7 (skill, outName) に重複なし', new Set(keys).size === keys.length, keys.join(', '));
const HANDWRITTEN = ['authoring-patterns.md', 'internal-modules.md', 'coding-conventions.md', 'testing-lint.md'];
check('A-8 出力名が手書きファイル名と重ならない',
  GENERATION_MAP.every((e) => !HANDWRITTEN.includes(outName(e.chapter))));
check('A-9 GENERATION_MAP は凍結（順序固定の定数）', Object.isFrozen(GENERATION_MAP) && GENERATION_MAP.every(Object.isFrozen));
check('A-10 MANUAL_BASE_URL', MANUAL_BASE_URL === 'https://ekicyou.github.io/pasta/');

// ============================================================
log('\n[B] VOICE_MARKERS / findVoice');
check('B-1 わたくし を検出', findVoice('わたくしが案内する').includes('わたくし'));
check('B-2 文末 ですわ を検出', findVoice('これは規則ですわ。').includes('ですわ'));
check('B-3 「書かなくてよい」は非検出', findVoice('このキーは書かなくてよい。').length === 0, JSON.stringify(findVoice('このキーは書かなくてよい。')));
check('B-4 「ですので」は非検出', findVoice('既定値ですので省略できる。').length === 0);
check('B-5 「ますので」は非検出', findVoice('読み込まれますので注意する。').length === 0);
check('B-6 くてよ（文末）は検出', findVoice('よろしくなくてよ。').includes('くてよ'));
check('B-7 ですの（文末）は検出', findVoice('これが規則ですの？').includes('ですの'));
check('B-8 ますの（文末）は検出', findVoice('こう書きますの。').includes('ますの'));
check('B-9 普通文体は空', findVoice('シーンは `＊` で始める。').length === 0);
const regexMarkers = VOICE_MARKERS.filter((m) => m instanceof RegExp).map((m) => m.source).sort();
check('B-10 否定先読みは衝突 3 語だけ',
  JSON.stringify(regexMarkers) === JSON.stringify(['くてよ(?!い)', 'ですの(?!で)', 'ますの(?!で)'].sort()),
  JSON.stringify(regexMarkers));
for (const s of ['ですわ', 'ますわ', 'ませんわ', 'おほほ', 'フンッ', 'わたくし', 'ごきげんよう', 'なさいまし',
  'くださいまし', 'まし。', 'まし、', '参りましょう', 'まいりましょう', 'よろしくて', 'ですこと']) {
  check(`B-11 文字列マーカー ${s} を保持`, VOICE_MARKERS.includes(s));
}
check('B-12 VOICE_MARKERS は 18 語', VOICE_MARKERS.length === 18, String(VOICE_MARKERS.length));

// ============================================================
log('\n[C] extractBody');
const GOOD = [
  '# 章タイトル',            // 1
  '',                        // 2
  'ごきげんよう、導入ですわ。', // 3
  '',                        // 4
  '---',                     // 5
  '',                        // 6
  '## 規則',                 // 7
  '',                        // 8
  '本文の一行目。',          // 9
  '',                        // 10
  '---',                     // 11 本文内の区切り（保持）
  '',                        // 12
  '本文の二行目。',          // 13
  '',                        // 14
  '---',                     // 15
  '',                        // 16
  '締めですわ。',            // 17
  '',                        // 18
  '> 締め後の引用ですわ。',  // 19
  '',
].join('\n');
{
  const r = extractBody(GOOD, 'grammar/x.md');
  check('C-1 タイトルは H1 の文字列', r.title === '章タイトル', r.title);
  check('C-2 本文は最初と最後の区切りの間（前後空行除去・内側 --- 保持）',
    r.body === '## 規則\n\n本文の一行目。\n\n---\n\n本文の二行目。', JSON.stringify(r.body));
  check('C-3 導入・締め・締め後の引用を含まない', !/ですわ|導入|締め/.test(r.body));
  const crlf = extractBody(GOOD.replace(/\n/g, '\r\n'), 'grammar/x.md');
  check('C-4 CRLF 入力でも LF 入力と同一', crlf.title === r.title && crlf.body === r.body);
}
{
  const e = thrown(() => extractBody('# T\n\n導入\n\n---\n\n本文のみ\n', 'grammar/one.md'));
  check('C-5 区切り 1 本 → bad-structure', e instanceof GenError && e.kind === 'bad-structure' && e.chapter === 'grammar/one.md', String(e));
  const e2 = thrown(() => extractBody('導入\n\n---\n\n本文\n\n---\n\n締め\n', 'grammar/noh1.md'));
  check('C-6 先頭行が H1 でない → bad-structure', e2 instanceof GenError && e2.kind === 'bad-structure', String(e2));
  const e3 = thrown(() => extractBody('## T\n\n---\n\n本文\n\n---\n', 'grammar/h2.md'));
  check('C-7 先頭行が H2 → bad-structure', e3 instanceof GenError && e3.kind === 'bad-structure', String(e3));
  const e4 = thrown(() => extractBody('# T\n\n---\n\n```text\n---\n```\n\n本文\n', 'grammar/fence.md'));
  check('C-8 フェンス内の --- は区切りに数えない → bad-structure', e4 instanceof GenError && e4.kind === 'bad-structure', String(e4));
  check('C-9 エラーメッセージに章パス', e && String(e.message).includes('grammar/one.md'), e && e.message);
}
{
  const src = [
    '# T',          // 1
    '導入',         // 2
    '---',          // 3
    '普通の文。',   // 4
    'これは規則ですわ。', // 5
    '## わたくしの見出し', // 6
    '> 引用ですの。',     // 7
    '---',          // 8
    '締め',         // 9
  ].join('\n');
  const e = thrown(() => extractBody(src, 'grammar/voice.md'));
  check('C-10 本文散文の口調 → voice-in-body', e instanceof GenError && e.kind === 'voice-in-body', String(e));
  const hits = e && e.hits ? e.hits : [];
  check('C-11 行番号（章内）と語を報告（散文・見出し・引用）',
    JSON.stringify(hits) === JSON.stringify([
      { line: 5, marker: 'ですわ' },
      { line: 6, marker: 'わたくし' },
      { line: 7, marker: 'ですの' },
    ]), JSON.stringify(hits));
  check('C-12 メッセージに章パス・行番号・語', e && /grammar\/voice\.md/.test(e.message) && /5/.test(e.message) && /ですわ/.test(e.message), e && e.message);
}
{
  const src = [
    '# T',
    '',
    'ごきげんよう、導入ですわ。',
    '',
    '---',
    '',
    '```pasta',
    '＊会話',
    '  さくら：わたくしですわ。',
    '```',
    '',
    '| 列 | 説明 |',
    '|----|------|',
    '| a | ですわ |',
    '  | 字下げ表 | ですわ |',
    '',
    '例: `ですわ` と `` わたくし ` `` はコード。',
    '',
    '````markdown',
    '```pasta',
    'わたくしですわ',
    '---',
    '```',
    '````',
    '',
    '~~~',
    'ですわ',
    '~~~',
    '',
    'このキーは書かなくてよい。既定値ですので、読み込まれますので。',
    '',
    '---',
    '',
    '締めですわ。',
  ].join('\n');
  const e = thrown(() => extractBody(src, 'grammar/ok.md'));
  check('C-13 フェンス・表・インラインコード・4 連フェンス・衝突 3 語は口調として扱わない', e === null, e && e.message);
  const r = e ? null : extractBody(src, 'grammar/ok.md');
  check('C-14 4 連フェンス内の --- は区切りにならず本文に残る', r !== null && r.body.includes('````markdown') && r.body.endsWith('このキーは書かなくてよい。既定値ですので、読み込まれますので。'),
    r && JSON.stringify(r.body.slice(-80)));
}

// ============================================================
log('\n[D] readChapter');
{
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'gen-skill-refs-'));
  try {
    fs.mkdirSync(path.join(root, 'book/src/grammar'), { recursive: true });
    fs.writeFileSync(path.join(root, 'book/src/grammar/a.md'), 'abc\r\n');
    check('D-1 存在する章を読む', readChapter('grammar/a.md', root) === 'abc\r\n');
    const e = thrown(() => readChapter('grammar/nope.md', root));
    check('D-2 章欠落 → missing-chapter', e instanceof GenError && e.kind === 'missing-chapter' && e.chapter === 'grammar/nope.md', String(e));
    check('D-3 メッセージに章パス', e && e.message.includes('grammar/nope.md'));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

// ============================================================
log('\n[E] 実リポジトリ: 21 章の抽出・口調判定');
for (const entry of GENERATION_MAP) {
  const e = thrown(() => extractBody(readChapter(entry.chapter, REPO_ROOT), entry.chapter));
  check(`E ${entry.chapter}`, e === null, e && e.message);
}

// ============================================================
log('\n[F] rewriteLinks');
const GA = 'pasta-ghost-authoring';
const LC = 'pasta-lua-coding';
const URL_BASE = 'https://ekicyou.github.io/pasta/';
{
  const search = { chapter: 'lua/modules/pasta-search.md', skill: LC };
  const rw = (body, entry = search) => rewriteLinks(body, entry);
  const cases = [
    ['F-1 同一ディレクトリの index.md → modules-index.md', '[a](index.md)', '[a](modules-index.md)'],
    ['F-2 ../shiori-events.md（同一スキル）→ 兄弟ファイル名＋アンカー', '[a](../shiori-events.md#onhour)', '[a](shiori-events.md#onhour)'],
    ['F-3 ../patterns.md（非生成章）→ 公開 URL', '[a](../patterns.md)', `[a](${URL_BASE}lua/patterns.html)`],
    ['F-4 別スキル宛て → 公開 URL＋アンカー', '[a](../../grammar/words.md#スコープと優先順位)', `[a](${URL_BASE}grammar/words.html#スコープと優先順位)`],
    ['F-5 ./ 付き同一スキル', '[a](./enc.md)', '[a](enc.md)'],
    ['F-6 絶対 URL は不変', '[a](https://example.com/x.md#y)', '[a](https://example.com/x.md#y)'],
    ['F-7 mailto は不変', '[a](mailto:a@example.com)', '[a](mailto:a@example.com)'],
    ['F-8 #anchor のみは不変', '[a](#savedata)', '[a](#savedata)'],
    ['F-9 1 行に複数リンク', '[a](index.md) と [b](../patterns.md#p)', `[a](modules-index.md) と [b](${URL_BASE}lua/patterns.html#p)`],
    ['F-10 フェンス内は不変', '```text\n[a](../patterns.md)\n```\n[b](index.md)', '```text\n[a](../patterns.md)\n```\n[b](modules-index.md)'],
    ['F-11 インラインコード内は不変・外は書き換え', '`[a](../patterns.md)` と [b](index.md)', '`[a](../patterns.md)` と [b](modules-index.md)'],
    ['F-12 画像の絶対 URL は不変', '![i](https://example.com/i.png)', '![i](https://example.com/i.png)'],
    ['F-13 タイトル付きリンク', '[a](index.md "t")', '[a](modules-index.md "t")'],
    ['F-14 山括弧のリンク先', '[a](<index.md>)', '[a](<modules-index.md>)'],
  ];
  for (const [name, input, want] of cases) {
    let got;
    try { got = rw(input); } catch (e) { got = `THROW ${e.message}`; }
    check(name, got === want, JSON.stringify(got));
  }
  const startup = { chapter: 'reference/startup.md', skill: LC };
  check('F-15 startup: ../lua/modules/pasta-persistence.md → pasta-persistence.md',
    rw('[a](../lua/modules/pasta-persistence.md)', startup) === '[a](pasta-persistence.md)');
  check('F-16 startup: ../debug/troubleshooting.md → 公開 URL',
    rw('[a](../debug/troubleshooting.md)', startup) === `[a](${URL_BASE}debug/troubleshooting.html)`);
  const vars = { chapter: 'grammar/variables.md', skill: GA };
  check('F-17 variables: ../lua/patterns.md → 公開 URL', rw('[a](../lua/patterns.md)', vars) === `[a](${URL_BASE}lua/patterns.html)`);
  check('F-18 variables: literals.md#x → literals.md#x', rw('[a](literals.md#x)', vars) === '[a](literals.md#x)');
  check('F-19 variables: ../reference/pasta-toml.md（同一スキル GA）→ pasta-toml.md',
    rw('[a](../reference/pasta-toml.md#y)', vars) === '[a](pasta-toml.md#y)');
  check('F-20 pasta-toml（GA）→ lua/modules/pasta-config.md（LC）は公開 URL',
    rw('[a](../lua/modules/pasta-config.md)', { chapter: 'reference/pasta-toml.md', skill: GA })
      === `[a](${URL_BASE}lua/modules/pasta-config.html)`);
  check('F-21 index.md 宛ての公開 URL は index.html', rw('[a](../lua/index.md)', vars) === `[a](${URL_BASE}lua/index.html)`);
  check('F-22 リンクの無い本文は不変', rw('本文。\n\n| a | b |') === '本文。\n\n| a | b |');

  const bad = [
    ['F-23 画像の相対リンク', '![i](img.png)', 'img.png'],
    ['F-24 非 .md の相対リンク', '[a](foo.txt)', 'foo.txt'],
    ['F-25 ディレクトリへの相対リンク', '[a](../)', '../'],
    ['F-26 book/src 外へ出る相対リンク', '[a](../../../README.md)', '../../../README.md'],
    ['F-27 ルート相対リンク', '[a](/grammar/words.md)', '/grammar/words.md'],
  ];
  for (const [name, input, target] of bad) {
    const e = thrown(() => rw(input));
    check(`${name} → unresolvable-link`,
      e instanceof GenError && e.kind === 'unresolvable-link' && e.chapter === search.chapter && e.target === target
        && e.message.includes(search.chapter) && e.message.includes(target),
      e ? `${e.kind} ${e.target} ${e.message}` : 'no throw');
  }
}

// ============================================================
// サンドボックス: 21 章すべてを合成した最小の book/src を持つ tmp リポジトリ。
const HEADER1 = '<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->';
const header2 = (title, chapter) =>
  `<!-- このファイルは pasta マニュアル「${title}」（${URL_BASE}${chapter.replace(/\.md$/, '.html')}）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->`;
const relOut = (e) => `.claude/skills/${e.skill}/references/${outName(e.chapter)}`;
function synthChapter(entry, i) {
  const extra = entry.chapter === 'lua/modules/pasta-search.md'
    ? '\n\n[一覧](index.md)・[パターン](../patterns.md#x)・[語](../../grammar/words.md)'
    : '';
  return `# 章${i}\n\nごきげんよう、導入ですわ。\n\n---\n\n## 規則 ${i}\n\n本文 ${i}。${extra}\n\n---\n\n締めですわ。\n`;
}
function makeSandbox({ withTools = false } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'gen-skill-refs-sb-'));
  GENERATION_MAP.forEach((e, i) => {
    const f = path.join(root, 'book/src', e.chapter);
    fs.mkdirSync(path.dirname(f), { recursive: true });
    fs.writeFileSync(f, synthChapter(e, i));
  });
  if (withTools) {
    const toolsDir = path.dirname(fileURLToPath(import.meta.url));
    fs.mkdirSync(path.join(root, 'book/tools'), { recursive: true });
    for (const f of ['gen-skill-refs.mjs', 'link-check.mjs']) {
      fs.copyFileSync(path.join(toolsDir, f), path.join(root, 'book/tools', f));
    }
  }
  return root;
}
const rmrf = (root) => fs.rmSync(root, { recursive: true, force: true });
const readOut = (root, rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const writeOut = (root, rel, text) => fs.writeFileSync(path.join(root, rel), text);

// ============================================================
log('\n[G] renderEntry（ヘッダ・決定性）');
{
  const root = makeSandbox();
  try {
    const entry = GENERATION_MAP.find((e) => e.chapter === 'lua/modules/pasta-search.md');
    const i = GENERATION_MAP.indexOf(entry);
    const out = renderEntry(entry, root);
    const want = [
      HEADER1,
      header2(`章${i}`, entry.chapter),
      '',
      `# 章${i}`,
      '',
      `## 規則 ${i}`,
      '',
      `本文 ${i}。`,
      '',
      `[一覧](modules-index.md)・[パターン](${URL_BASE}lua/patterns.html#x)・[語](${URL_BASE}grammar/words.html)`,
      '',
    ].join('\n');
    check('G-1 ヘッダ 2 行＋空行＋H1＋空行＋本文（リンク書き換え済み）＋末尾改行 1 つ', out === want, JSON.stringify(out));
    check('G-2 BOM なし・CR なし・末尾改行はちょうど 1 つ',
      !out.startsWith('﻿') && !out.includes('\r') && out.endsWith('\n') && !out.endsWith('\n\n'));
    check('G-3 導入・締めを含まない', !/ですわ|ごきげんよう/.test(out));
    check('G-4 ヘッダにリポジトリ内パス（book/src・.claude）を含まない',
      !out.split('\n').slice(0, 2).some((l) => l.includes('book/src') || l.includes('.claude')));

    const f = path.join(root, 'book/src', entry.chapter);
    fs.writeFileSync(f, fs.readFileSync(f, 'utf8').replace(/\n/g, '\r\n'));
    const crlf = renderEntry(entry, root);
    check('G-5 CRLF の章から生成しても LF 版とバイト一致', Buffer.from(crlf).equals(Buffer.from(out)));
    check('G-6 同一入力の再生成はバイト一致', renderEntry(entry, root) === crlf);

    fs.writeFileSync(f, '# T\n\n---\n\n![図](fig.png)\n\n---\n');
    const e = thrown(() => renderEntry(entry, root));
    check('G-7 解決不能リンクは renderEntry でも GenError', e instanceof GenError && e.kind === 'unresolvable-link', String(e));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n[H] generateAll / writeAll / checkAll（サンドボックス）');
{
  const root = makeSandbox();
  try {
    const all = generateAll(root);
    check('H-1 generateAll は 21 エントリ（リポジトリ相対パス → 内容）',
      all instanceof Map && all.size === 21 && GENERATION_MAP.every((e) => all.has(relOut(e))), [...all.keys()].join(', '));
    check('H-2 全エントリが固定 1 行目で始まる', [...all.values()].every((c) => c.startsWith(HEADER1 + '\n')));

    const before = checkAll(root);
    check('H-3 未生成なら全 21 件が stale・orphan なし・fixCommand',
      before.stale.length === 21 && before.orphans.length === 0 && before.fixCommand === 'node book/tools/gen-skill-refs.mjs',
      JSON.stringify(before));

    // 部分書き出しなし: 最後のエントリだけ壊すと何も書かれない。
    const last = GENERATION_MAP[GENERATION_MAP.length - 1];
    const lastFile = path.join(root, 'book/src', last.chapter);
    const lastText = fs.readFileSync(lastFile, 'utf8');
    fs.writeFileSync(lastFile, '# T\n\n---\n\nわたくしの本文。\n\n---\n');
    const e = thrown(() => writeAll(root));
    check('H-4 1 章でも GenError なら writeAll は投げる', e instanceof GenError && e.kind === 'voice-in-body' && e.chapter === last.chapter, String(e));
    check('H-5 失敗時はファイルを 1 つも書かない', GENERATION_MAP.every((x) => !fs.existsSync(path.join(root, relOut(x)))));
    fs.writeFileSync(lastFile, lastText);

    // 手書きの同名ファイル（ヘッダなし）は上書きされる。
    const ga0 = GENERATION_MAP[0];
    fs.mkdirSync(path.dirname(path.join(root, relOut(ga0))), { recursive: true });
    writeOut(root, relOut(ga0), '# 手書き\n');
    const w = writeAll(root);
    check('H-6 書き出しは 21 件（未生成 20＋手書き同名 1 を上書き）', w.written.length === 21 && w.unchanged.length === 0, JSON.stringify(w));
    check('H-7 書き出した内容は generateAll と一致', GENERATION_MAP.every((x) => readOut(root, relOut(x)) === all.get(relOut(x))));
    const after = checkAll(root);
    check('H-8 書き出し直後の checkAll は stale・orphan とも空', after.stale.length === 0 && after.orphans.length === 0, JSON.stringify(after));

    // 生成物を CRLF 化しただけ → 一致。writeAll も書き換えない。
    const target = relOut(GENERATION_MAP[3]);
    writeOut(root, target, readOut(root, target).replace(/\n/g, '\r\n'));
    check('H-9 生成物を CRLF 化しただけなら一致', checkAll(root).stale.length === 0);
    const w2 = writeAll(root);
    check('H-10 LF 正規化後に同一なら書き換えない（CRLF のまま残る）',
      w2.written.length === 0 && w2.unchanged.length === 21 && readOut(root, target).includes('\r\n'), JSON.stringify(w2));

    // 生成物だけ手編集 → stale。
    writeOut(root, target, readOut(root, target) + '手で追記。\n');
    const s1 = checkAll(root);
    check('H-11 生成物だけ手編集 → その 1 件だけ stale', JSON.stringify(s1.stale) === JSON.stringify([target]), JSON.stringify(s1));
    writeAll(root);

    // 章だけ変更 → stale。
    const ch = GENERATION_MAP[12];
    const chFile = path.join(root, 'book/src', ch.chapter);
    fs.writeFileSync(chFile, fs.readFileSync(chFile, 'utf8').replace('## 規則', '## 規則（改）'));
    const s2 = checkAll(root);
    check('H-12 章だけ変更 → 対応する生成物 1 件だけ stale', JSON.stringify(s2.stale) === JSON.stringify([relOut(ch)]), JSON.stringify(s2));
    writeAll(root);

    // 欠落 → stale。
    fs.rmSync(path.join(root, relOut(GENERATION_MAP[5])));
    check('H-13 生成物の欠落 → stale', JSON.stringify(checkAll(root).stale) === JSON.stringify([relOut(GENERATION_MAP[5])]));
    writeAll(root);

    // orphan: ヘッダ付きの対応表外ファイル（CRLF でも）。ヘッダなしの手書きは対象外。
    const ghostRefs = `.claude/skills/${GA}/references`;
    const luaRefs = `.claude/skills/${LC}/references`;
    writeOut(root, `${ghostRefs}/call-spec-old.md`, `${HEADER1}\n<!-- x -->\n\n# 古い\n`);
    writeOut(root, `${luaRefs}/stale-gen.md`, `${HEADER1}\r\n# 古い\r\n`);
    writeOut(root, `${luaRefs}/coding-conventions.md`, '# 手書き\n');
    writeOut(root, `${luaRefs}/mentions.md`, `# 手書き\n${HEADER1}\n`);
    const s3 = checkAll(root);
    check('H-14 ヘッダ付きの対応表外ファイルだけ orphan（全件・ソート済み）',
      JSON.stringify(s3.orphans) === JSON.stringify([`${ghostRefs}/call-spec-old.md`, `${luaRefs}/stale-gen.md`]) && s3.stale.length === 0,
      JSON.stringify(s3));

    // 全件列挙: stale 複数＋orphan。
    for (const x of GENERATION_MAP.slice(0, 3)) writeOut(root, relOut(x), 'x\n');
    const s4 = checkAll(root);
    check('H-15 stale を全件列挙する', s4.stale.length === 3 && s4.orphans.length === 2, JSON.stringify(s4));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n[I] CLI（サンドボックスへツールを複製して実行）');
{
  const root = makeSandbox({ withTools: true });
  const run = (...args) => {
    const r = spawnSync(process.execPath, [path.join(root, 'book/tools/gen-skill-refs.mjs'), ...args], { encoding: 'utf8' });
    return { code: r.status, out: r.stdout, err: r.stderr };
  };
  try {
    const c0 = run('--check');
    check('I-1 未生成の --check は exit 1', c0.code === 1, `${c0.code} ${c0.err}`);
    check('I-2 STALE を全件・再生成コマンドを表示',
      GENERATION_MAP.every((e) => c0.out.includes(`STALE ${relOut(e)}`))
        && c0.out.includes('再生成: `node book/tools/gen-skill-refs.mjs` を実行してコミット'), c0.out);
    const w = run();
    check('I-3 書き出しモードは exit 0', w.code === 0, `${w.code} ${w.out} ${w.err}`);
    const c1 = run('--check');
    check('I-4 書き出し直後の --check は exit 0', c1.code === 0, `${c1.code} ${c1.out} ${c1.err}`);
    const w2 = run();
    check('I-5 再実行の書き出しは冪等（exit 0）', w2.code === 0 && checkAll(root).stale.length === 0);

    writeOut(root, `.claude/skills/${LC}/references/orphan.md`, `${HEADER1}\n`);
    const c2 = run('--check');
    check('I-6 orphan で --check は exit 1・ORPHAN と対処を表示',
      c2.code === 1 && c2.out.includes(`ORPHAN .claude/skills/${LC}/references/orphan.md`)
        && c2.out.includes('孤立ファイルは削除するか対応表へ追加'), `${c2.code} ${c2.out}`);
    fs.rmSync(path.join(root, `.claude/skills/${LC}/references/orphan.md`));

    const ch = GENERATION_MAP[7];
    fs.writeFileSync(path.join(root, 'book/src', ch.chapter), '# T\n\n本文だけ\n');
    const before = readOut(root, relOut(GENERATION_MAP[0]));
    const w3 = run();
    check('I-7 GenError は exit 1・章パスを標準エラーへ', w3.code === 1 && w3.err.includes(ch.chapter) && w3.err.includes('bad-structure'),
      `${w3.code} ${w3.err}`);
    check('I-8 GenError 時は他の生成物も書き換えない', readOut(root, relOut(GENERATION_MAP[0])) === before);
    const c3 = run('--check');
    check('I-9 --check でも GenError は exit 1', c3.code === 1 && c3.err.includes(ch.chapter), `${c3.code} ${c3.err}`);
    const u = run('--bogus');
    check('I-10 不明な引数は exit 2', u.code === 2, `${u.code} ${u.err}`);
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n[J] 実リポジトリ: 21 章のメモリ上全生成（書き出さない）');
{
  let all = null;
  const e = thrown(() => { all = generateAll(REPO_ROOT); });
  check('J-1 generateAll が生成エラーなし', e === null, e && e.message);
  if (all) {
    check('J-2 21 エントリ', all.size === 21, String(all.size));
    // 生成物内の相対リンクは同一スキルの生成ファイル（兄弟）か #anchor だけ。
    const siblings = (skill) => new Set(GENERATION_MAP.filter((x) => x.skill === skill).map((x) => outName(x.chapter)));
    for (const entry of GENERATION_MAP) {
      const content = all.get(relOut(entry));
      const bad = [];
      for (const line of maskFences(content).split('\n')) {
        for (const m of line.replace(/(`+)(?!`)[\s\S]*?(?<!`)\1(?!`)/g, '').matchAll(LINK_RE)) {
          const t = m[1].replace(/^<|>$/g, '');
          if (/^[a-z][a-z0-9+.-]*:/i.test(t) || t.startsWith('#')) continue;
          if (!siblings(entry.skill).has(t.split('#')[0])) bad.push(t);
        }
      }
      check(`J ${entry.chapter} のリンクはスキル内で閉じる`, bad.length === 0, bad.join(', '));
    }
  }
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
