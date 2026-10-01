// gen-skill-refs-test.mjs — gen-skill-refs の自動検証（manual-ssot-authority タスク 3.3 / 要件 5.1, 5.2, 5.8, 5.9, 1.7）。
//
// 検証方針（design「Testing Strategy / Unit Tests」）:
//   - outName と実物の GENERATION_MAP（21 エントリ・(skill, outName) 重複なし）。
//   - findVoice: 口調マーカーの検出と、普通文体と衝突する 3 語（否定先読み）の非検出。
//   - extractBody: 導入・締めの除去、本文内 `---` の保持、bad-structure、voice-in-body（行番号つき）、
//     フェンス・表の行・インラインコード内の無視（4 連バッククォートのフェンスを含む）、LF/CRLF の同一性。
//   - readChapter: 章欠落で missing-chapter。
//   - 実リポジトリ: 21 章すべてが抽出・口調判定を通る。
//   book/src・スキルには書き込まない。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  GENERATION_MAP,
  VOICE_MARKERS,
  MANUAL_BASE_URL,
  GenError,
  outName,
  findVoice,
  extractBody,
  readChapter,
} from './gen-skill-refs.mjs';
import { REPO_ROOT } from './link-check.mjs';

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
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
