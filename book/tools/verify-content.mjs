// verify-content.mjs — コンテンツ整合・網羅レビューの機械検証（タスク 7.4）
//
// 目的（requirements.md R4–R9 / design「Testing Strategy / コンテンツ整合・編集レビュー」）:
//   本書 book/src/** の本文が、要件 4/5/6/7/8/9 のコンテンツ受入基準を満たすことを、
//   実ファイル走査により機械的にアサートする。各検証は「実物の book/src・book/tools を読む」ことで
//   成立し、固定文字列の自己満足チェックではない。
//   manual-ssot-authority（タスク 3.5）でマニュアルが唯一の権威になったため、doc/spec 権威リンク必須・
//   manual-sources 整合の検査を外し、doc/spec・GRAMMAR.md を案内しないこと（A-nospec）と
//   スキル生成対象章の目次到達（A-summary）を検査する。対応表と口調判定は gen-skill-refs.mjs から import する。
//
// 検証範囲（7.4 = コンテンツの網羅・整合・ボイスに集中。検索 7.2 / 静的 7.1 / ドリフト 7.3 とは重複しない）:
//   A. 文法網羅・権威（R4.1 / ssot 1.5, 4.2, 9.7） — grammar 全実装章の存在・本文、
//      book/src に doc/spec・GRAMMAR.md の記述なし（A-nospec）、生成対象章が SUMMARY から到達可能（A-summary）
//   B. Lua 網羅（R5.1, R5.5）   — 公開モジュール名の登場・LuaJIT 2.1 明示
//   C. チュートリアル（R6.1, R6.2） — 前提環境/手順/UTF-8 注意・tutorial-check 逐語一致
//   D. ボイス（R7.1, R7.2, R7.4 / ssot 1.7） — 導入/締めのキャラ口調（判定は findVoice）・コードフェンス内に口調なし
//   E. 外部参照（R8.2, R8.3）   — milkpot(lua51/lua52)＋luajit.org 絶対 URL・lua55 不採用明記
//   F. バージョン（R9.1, R9.3, R9.4） — introduction に対象系列・LuaJIT 2.1・将来変更注記
//
// 成功で exit 0、失敗（1 件でも）で exit 1。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runTutorialCheck } from './tutorial-check.mjs';
import { GENERATION_MAP, findVoice } from './gen-skill-refs.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(here, '../..');

const SRC = 'book/src';

// ---- ユーティリティ ----
function abs(rel) {
  return path.resolve(REPO_ROOT, rel);
}
function read(rel) {
  return fs.readFileSync(abs(rel), 'utf8');
}
function exists(rel) {
  return fs.existsSync(abs(rel));
}

// コードフェンス（```...```）の中身を全抽出。言語注記は問わない。
function extractCodeFences(markdown) {
  const fences = [];
  const re = /```[^\n]*\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(markdown)) !== null) {
    fences.push(m[1]);
  }
  return fences;
}

// コードフェンスを除いた本文（=散文部）を返す。ボイス検査の対象。
function stripCodeFences(markdown) {
  return markdown.replace(/```[^\n]*\n[\s\S]*?```/g, '');
}

// キャラ口調（Claudia 令嬢ボイス）の存在判定。マーカー集合（VOICE_MARKERS）と判定は
// gen-skill-refs.mjs の findVoice に一本化する（生成器の voice-in-body と同じ規則）。
function hasVoice(text) {
  return findVoice(text).length > 0;
}

// コードフェンス内の混入検査に使う「狭い」集合（R7.4）。
// R7.4 が禁じるのは *解説者 Claudia の地の文（ナレーション）* がコード中へ漏れること。
// 一方、サンプル `.pasta` 内のキャラクター台詞（例: アクター「ラザニア」が「〜ですわ」と話す）は
// 正当な作例コンテンツであり、お嬢様口調の語尾を含み得る。これを「混入」と誤検出しないため、
// コード検査では *Claudia の一人称・読者への語りかけに固有* のマーカーのみを対象にする。
// （「わたくし」「おほほ」「フンッ」「ごきげんよう」「ですこと」等。一般的な丁寧語尾は除外）
// 注意: 「よろしくて」「ですわ」等の丁寧語尾はサンプルキャラ台詞にも現れ得るため除外する。
// ここに残すのは「解説の地の文でしか使われない」一人称・固有マーカーに限定する。
const NARRATION_MARKERS = [
  'わたくし', 'おほほ', 'フンッ', 'ごきげんよう Claudia',
];

// 本文がプレースホルダでなく実体を持つか（最低文字数＋見出しの存在）。
function isSubstantive(markdown, minChars = 800) {
  const stripped = markdown.trim();
  if (stripped.length < minChars) return false;
  // プレースホルダ常套句を弾く。
  if (/TODO|プレースホルダ|placeholder|ここに本文/i.test(stripped)) return false;
  return true;
}

// ---- 検証結果の収集 ----
const checks = [];
function ok(id, msg) {
  checks.push({ id, pass: true, msg });
}
function fail(id, msg) {
  checks.push({ id, pass: false, msg });
}
function assert(id, cond, passMsg, failMsg) {
  if (cond) ok(id, passMsg);
  else fail(id, failMsg);
}

// ============================================================
// A. 文法網羅（R4.1 全実装章の網羅）・権威（ssot 1.5, 9.7 doc/spec を案内しない / 4.2 目次到達）
// ============================================================
const GRAMMAR_CHAPTERS = [
  'index', 'markers', 'block-structure', 'call-jump', 'literals',
  'action-line', 'sakura-script', 'variables', 'words', 'actor-dictionary',
];

for (const ch of GRAMMAR_CHAPTERS) {
  const rel = `${SRC}/grammar/${ch}.md`;
  if (!exists(rel)) {
    fail('A-exist', `文法章が存在しない: ${rel}`);
    continue;
  }
  const md = read(rel);
  assert(
    `A-body:${ch}`,
    isSubstantive(md),
    `文法章 ${ch}.md が本文を持つ`,
    `文法章 ${ch}.md が本文不足/プレースホルダ`,
  );
}

// book/src 配下の全 .md（再帰）。
function listMarkdown(relDir) {
  const out = [];
  for (const ent of fs.readdirSync(abs(relDir), { withFileTypes: true })) {
    const rel = `${relDir}/${ent.name}`;
    if (ent.isDirectory()) out.push(...listMarkdown(rel));
    else if (ent.isFile() && ent.name.endsWith('.md')) out.push(rel);
  }
  return out;
}

// A-nospec: マニュアルが廃止済みの doc/spec・GRAMMAR.md を（権威としても参照先としても）案内しない。
{
  const FORBIDDEN = ['doc/spec', 'GRAMMAR.md'];
  const hits = [];
  for (const rel of listMarkdown(SRC)) {
    read(rel).split(/\r?\n/).forEach((line, i) => {
      for (const tok of FORBIDDEN) if (line.includes(tok)) hits.push(`${rel}:${i + 1} (${tok})`);
    });
  }
  assert(
    'A-nospec',
    hits.length === 0,
    `book/src に doc/spec・GRAMMAR.md の記述が無い`,
    `book/src に doc/spec・GRAMMAR.md の記述が残っている: ${hits.join(', ')}`,
  );
}

// A-summary: スキル生成対象の全章が目次（SUMMARY.md）からリンクされている。
{
  const summary = read(`${SRC}/SUMMARY.md`);
  const linked = new Set(
    [...summary.matchAll(/\]\(([^)\s#]+\.md)(?:#[^)]*)?\)/g)].map((m) => m[1].replace(/^\.\//, '')),
  );
  for (const { chapter } of GENERATION_MAP) {
    assert(
      `A-summary:${chapter}`,
      linked.has(chapter),
      `生成対象章 ${chapter} が SUMMARY.md からリンクされている`,
      `生成対象章 ${chapter} が SUMMARY.md からリンクされていない`,
    );
  }
}

// ============================================================
// B. Lua 網羅（R5.1 公開モジュール網羅 / R5.5 LuaJIT 2.1 明示）
// ============================================================
const LUA_MODULES = [
  '@pasta_search', '@pasta_persistence', '@pasta_config',
  '@pasta_sakura_script', '@enc', '@pasta_log',
];
{
  // lua・lua/modules 配下の全 md を結合して網羅判定（各ディレクトリは非再帰）。
  const luaDir = `${SRC}/lua`;
  const allLua = [luaDir, `${SRC}/lua/modules`]
    .flatMap((d) => fs.readdirSync(abs(d)).filter((f) => f.endsWith('.md')).map((f) => read(`${d}/${f}`)))
    .join('\n');
  for (const mod of LUA_MODULES) {
    assert(
      `B-mod:${mod}`,
      allLua.includes(mod),
      `Lua 章に公開モジュール ${mod} が登場する`,
      `Lua 章に公開モジュール ${mod} が登場しない`,
    );
  }
  assert(
    'B-luajit',
    /LuaJIT 2\.1/.test(allLua),
    `Lua 章に LuaJIT 2.1 明示がある`,
    `Lua 章に LuaJIT 2.1 明示が無い`,
  );
  // 基礎入口・外部参照リンクの存在（R5.6 / R8.2 連携の最低限）。
  assert(
    'B-basics',
    exists(`${luaDir}/basics.md`) && isSubstantive(read(`${luaDir}/basics.md`), 500),
    `Lua 基礎入口 basics.md が本文を持つ`,
    `Lua 基礎入口 basics.md が不足`,
  );
}

// ============================================================
// C. チュートリアル（R6.1 前提環境/手順/UTF-8 / R6.2 起動可能な最小一式に一致）
// ============================================================
{
  const gsDir = `${SRC}/getting-started`;
  const gsFiles = fs.readdirSync(abs(gsDir)).filter((f) => f.endsWith('.md'));
  const allGs = gsFiles.map((f) => read(`${gsDir}/${f}`)).join('\n');
  assert(
    'C-utf8',
    /UTF-8/.test(allGs),
    `チュートリアルに UTF-8 保存の注意がある`,
    `チュートリアルに UTF-8 注意が無い`,
  );
  assert(
    'C-sjis',
    /Shift_JIS/.test(allGs),
    `チュートリアルに Shift_JIS 辞書移行注意がある`,
    `チュートリアルに Shift_JIS 移行注意が無い`,
  );
  assert(
    'C-env',
    /Windows/.test(allGs) && /SSP/.test(allGs),
    `チュートリアルに前提環境（Windows / SSP）の記載がある`,
    `チュートリアルに前提環境（Windows / SSP）の記載が無い`,
  );
  assert(
    'C-steps',
    exists(`${gsDir}/first-ghost.md`) && isSubstantive(read(`${gsDir}/first-ghost.md`), 3000),
    `first-ghost.md が完結した手順本文を持つ`,
    `first-ghost.md の手順本文が不足`,
  );
  // 起動可能な最小一式に一致 = tutorial-check 逐語一致が成立すること（実走査）。
  const tut = runTutorialCheck(REPO_ROOT);
  assert(
    'C-tutorial-check',
    tut.ok,
    `tutorial-check が成立（チュートリアル末成果物が hello-pasta 最小一式と逐語一致）`,
    `tutorial-check が失敗: ${tut.fatal || tut.results.filter((r) => !r.matched).map((r) => r.file).join(', ')}`,
  );
}

// ============================================================
// D. ボイス（R7.1 導入/締めキャラ口調 / R7.2 本体普通文体 / R7.4 コード内に口調なし）
// ============================================================
{
  const contentFiles = [];
  for (const dir of ['', 'grammar', 'lua', 'lua/modules', 'getting-started', 'reference']) {
    const d = dir ? `${SRC}/${dir}` : SRC;
    for (const f of fs.readdirSync(abs(d))) {
      if (f === 'SUMMARY.md') continue;
      const rel = `${d}/${f}`;
      if (fs.statSync(abs(rel)).isFile() && f.endsWith('.md')) contentFiles.push(rel);
    }
  }

  for (const rel of contentFiles) {
    const md = read(rel);
    // D1: 散文部（コードフェンス除去後）にキャラ口調がある = 導入/締めボイスの存在。
    const prose = stripCodeFences(md);
    assert(
      `D-voice:${rel}`,
      hasVoice(prose),
      `${rel} の散文部にキャラ口調（導入/締め）がある`,
      `${rel} の散文部にキャラ口調が見当たらない`,
    );
    // D2: コードフェンス内に *解説者ナレーション* が無い（R7.4）。
    // 作例 .pasta のキャラ台詞（丁寧語尾を含み得る）は対象外。Claudia 固有の地の文マーカーのみ検出。
    const fences = extractCodeFences(md);
    for (let i = 0; i < fences.length; i++) {
      const body = fences[i];
      const leaked = NARRATION_MARKERS.filter((mk) => body.includes(mk));
      if (leaked.length > 0) {
        fail(
          `D-codevoice:${rel}#${i}`,
          `${rel} のコードフェンス内に解説ナレーションが混入: ${leaked.join(', ')}`,
        );
      }
    }
  }
  // コードフェンス内ボイス混入が 1 件も無ければまとめて PASS を 1 件記録。
  if (!checks.some((c) => c.id.startsWith('D-codevoice:'))) {
    ok('D-codevoice', `全章のコードフェンス内にキャラ口調の混入なし`);
  }
}

// ============================================================
// E. 外部参照（R8.2 milkpot lua51/lua52 + luajit.org / R8.3 lua55 不採用）
// ============================================================
{
  const rel = `${SRC}/reference/external-links.md`;
  assert('E-exist', exists(rel), `external-links.md が存在する`, `external-links.md が無い`);
  if (exists(rel)) {
    const md = read(rel);
    assert(
      'E-lua51',
      /milkpot[^\n]*lua51_manual_ja|lua51_manual_ja[^\n]*milkpot|http:\/\/milkpot\.sakura\.ne\.jp\/lua\/lua51_manual_ja/.test(md),
      `日本語 Lua 5.1 リファレンス（milkpot 版・絶対 URL）がある`,
      `Lua 5.1 milkpot 版リンクが無い`,
    );
    assert(
      'E-lua52',
      /milkpot\.sakura\.ne\.jp\/lua\/lua52_manual_ja/.test(md),
      `日本語 Lua 5.2 リファレンス（milkpot 版・絶対 URL）がある`,
      `Lua 5.2 milkpot 版リンクが無い`,
    );
    assert(
      'E-luajit',
      /https?:\/\/luajit\.org/.test(md),
      `LuaJIT 公式（luajit.org・絶対 URL）がある`,
      `luajit.org リンクが無い`,
    );
    // lua55 を「不採用」と明記していること（言語リファレンスとして案内しない）。
    assert(
      'E-lua55',
      /lua\s*5\.5|lua55/i.test(md) && /不採用|案内しない/.test(md),
      `lua55 系を言語リファレンスとして不採用と明記している`,
      `lua55 系の不採用明記が無い`,
    );
  }
}

// ============================================================
// F. バージョン（R9.1 対象系列 / R9.3 LuaJIT 2.1 / R9.4 将来変更注記）
// ============================================================
{
  const rel = `${SRC}/introduction.md`;
  assert('F-exist', exists(rel), `introduction.md が存在する`, `introduction.md が無い`);
  if (exists(rel)) {
    const md = read(rel);
    assert(
      'F-version',
      /v0\.\d+\s*系列|バージョン[^\n]*系列|対象[^\n]*pasta[^\n]*バージョン/.test(md),
      `introduction に対象 pasta バージョン系列の明示がある`,
      `introduction に対象バージョン系列の明示が無い`,
    );
    assert(
      'F-luajit',
      /LuaJIT 2\.1/.test(md),
      `introduction に LuaJIT 2.1 方言の明示がある`,
      `introduction に LuaJIT 2.1 明示が無い`,
    );
    assert(
      'F-future',
      /将来変更あり/.test(md),
      `introduction に「将来変更あり」注記（流動部の区別）がある`,
      `introduction に将来変更注記が無い`,
    );
  }
}

// ============================================================
// G. デバッグ章（R7.1 導入/締めボイス / R7.3 コード内に口調なし / R8.4 存在・本文・ボイス検査可能）
// ============================================================
// isSubstantive / hasVoice / stripCodeFences / extractCodeFences / NARRATION_MARKERS を再利用し、
// 文法章ループ（GRAMMAR_CHAPTERS）と同型のイディオムでアサートする。
const DEBUG_CHAPTERS = ['index', 'vscode-setup', 'source-level', 'constraints', 'troubleshooting'];

for (const ch of DEBUG_CHAPTERS) {
  const rel = `${SRC}/debug/${ch}.md`;
  // G-exist: 各ページが存在する。
  if (!exists(rel)) {
    fail(`G-exist:${ch}`, `デバッグ章が存在しない: ${rel}`);
    continue;
  }
  ok(`G-exist:${ch}`, `デバッグ章 ${ch}.md が存在する`);
  const md = read(rel);
  // G-body: 本文実体を持つ（プレースホルダでない）。
  assert(
    `G-body:${ch}`,
    isSubstantive(md, 800),
    `デバッグ章 ${ch}.md が本文を持つ`,
    `デバッグ章 ${ch}.md が本文不足/プレースホルダ`,
  );
  // G-voice: 散文部（コードフェンス除去後）に導入/締めのキャラ口調がある（R7.1）。
  const prose = stripCodeFences(md);
  assert(
    `G-voice:${ch}`,
    hasVoice(prose),
    `デバッグ章 ${ch}.md の散文部にキャラ口調（導入/締め）がある`,
    `デバッグ章 ${ch}.md の散文部にキャラ口調が見当たらない`,
  );
  // G-codevoice: コードフェンス内に解説ナレーションが混入していない（R7.3）。
  const fences = extractCodeFences(md);
  const leaked = [];
  for (const body of fences) {
    for (const mk of NARRATION_MARKERS) {
      if (body.includes(mk)) leaked.push(mk);
    }
  }
  assert(
    `G-codevoice:${ch}`,
    leaked.length === 0,
    `デバッグ章 ${ch}.md のコードフェンス内にナレーション混入なし`,
    `デバッグ章 ${ch}.md のコードフェンス内に解説ナレーションが混入: ${leaked.join(', ')}`,
  );
}

// G-fact: 主要事実が章全体のどこかに登場する（取りこぼし防止・R8.1 の最低限）。
{
  const allDebug = DEBUG_CHAPTERS
    .map((ch) => `${SRC}/debug/${ch}.md`)
    .filter((rel) => exists(rel))
    .map((rel) => read(rel))
    .join('\n');
  const FACTS = ['9276', 'PASTA_DEBUG', 'attach', '.pasta', 'sourcePresentation'];
  const missing = FACTS.filter((kw) => !allDebug.includes(kw));
  assert(
    'G-fact',
    missing.length === 0,
    `デバッグ章に主要事実が登場する (${FACTS.join(', ')})`,
    `デバッグ章に未登場の主要事実: ${missing.join(', ')}`,
  );
}

// ============================================================
// レポート
// ============================================================
const passed = checks.filter((c) => c.pass);
const failed = checks.filter((c) => !c.pass);

console.log('verify-content (コンテンツ整合・網羅レビュー / タスク 7.4)');
console.log(`  検証項目: ${checks.length} 件  PASS: ${passed.length}  FAIL: ${failed.length}`);
console.log('');
for (const c of checks) {
  console.log(`  ${c.pass ? 'PASS' : 'FAIL'}  [${c.id}]  ${c.msg}`);
}
console.log('');
if (failed.length === 0) {
  console.log('RESULT: OK（R4–R9 のコンテンツ受入基準を全項目で満たす）');
  process.exit(0);
} else {
  console.log(`RESULT: FAIL（${failed.length} 件の未充足項目あり）`);
  process.exit(1);
}
