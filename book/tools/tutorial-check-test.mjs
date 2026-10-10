// tutorial-check-test.mjs — tutorial-check の自動検証（要件 9.8 / design「ツール層 > TutorialCheck」）。
//
// 検証方針:
//   book/src・crates を恒久変更しないため、検証は
//   (A) 実リポジトリ現状でのクリーン判定（exit 0 相当・全 dic が同じ名前の章と逐語一致）と、
//   (B) 一時サンドボックス（tmp）へ「辞書 1 つにつき章 1 枚」のフィクスチャを構築し、
//       食い違いを注入して失敗を確認、で行う。
//   サンドボックスは runTutorialCheck(repoRoot) の repoRoot を差し替えて使う
//   （本物の book/src・crates には一切書き込まない）。
//   実リポジトリの場合（A・B-9）は、入門の章と辞書がそろっていることが前提（そろう前は落ちる）。
//
// 観測する完了条件（design「自己テスト」の表）:
//   - 実リポジトリ: 全 dic が verbatim-match・problems が空 → ok=true。
//   - 段ごとの照合の成功 → ok=true。
//   - 章の作例の不一致 → その辞書が no-matching-block・そのブロックが not-in-dic。
//   - 章の欠落（章の無い辞書）→ 列挙で拾われ no-chapter。
//   - 辞書の欠落（辞書の無い章のブロック）→ no-dic。
//   - 旧方式（全部のブロックを 1 枚の章に集める）→ ほかの辞書が no-chapter・余分なブロックが not-in-dic。
//   - 抜き出しの一致 → ok=true ／ 抜き出しの不一致 → not-in-dic。
//   - 字下げ・引用の中のフェンス → indented-fence。
//   - 致命的な失敗（入門のフォルダ無し・辞書 0 件）→ fatal。
//   - ユニット: extractPastaBlocks / normalizeForCompare / matchDicFile / listDicFiles
//               / listGuideChapters / isExcerptOf。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  REPO_ROOT,
  GUIDE_REL,
  HELLO_DIC_REL,
  listDicFiles,
  listGuideChapters,
  extractPastaBlocks,
  normalizeForCompare,
  matchDicFile,
  isExcerptOf,
  runTutorialCheck,
  reportTutorialCheck,
} from './tutorial-check.mjs';

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

function writeFile(root, rel, content) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, content, 'utf8');
  return abs;
}

function rmrf(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

// 辞書のファイル名から、辞書と章のパス（リポジトリルートからの相対）を作る。
const dicRel = (name) => `${HELLO_DIC_REL}/${name}`;
const chapterRel = (name) => `${GUIDE_REL}/${name.replace(/\.pasta$/, '.md')}`;
const realDic = (name) => fs.readFileSync(path.resolve(REPO_ROOT, HELLO_DIC_REL, name), 'utf8');

// 中身を ```pasta フェンスで包む。
// 中に ``` を含むとき（12 段目の ```lua）は ````pasta で包む（章と同じ書き方）。
function fenced(body) {
  let text = body.replace(/\r\n/g, '\n');
  if (!text.endsWith('\n')) text += '\n';
  const fence = text.includes('```') ? '````' : '```';
  return fence + 'pasta\n' + text + fence + '\n';
}

// 実リポジトリの dic 内容から「辞書 1 つにつき章 1 枚」を機械生成して最小フィクスチャを作る。
// （逐語転記そのものを再現するため、本物の dic を章のコードブロックに埋め込む）。
// 辞書の無い章（index.md・13-nar.md）も pasta ブロック無しで置く。
// 食い違いは、返したサンドボックスのファイルを各場合が書き換えて注入する。
const DIC = listDicFiles();
function makeSandbox() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  writeFile(root, `${GUIDE_REL}/index.md`, '# 入口\n\n辞書を持たない章。\n');
  writeFile(root, `${GUIDE_REL}/13-nar.md`, '# 13 段目\n\n辞書を足さない段。\n');
  for (const name of DIC) {
    const content = realDic(name);
    writeFile(root, dicRel(name), content);
    writeFile(root, chapterRel(name), `# ${name} の章\n\n本文。\n\n` + fenced(content));
  }
  return root;
}

// サンドボックスを作って fn に渡し、終わったら消す。
function inSandbox(fn) {
  const root = makeSandbox();
  try {
    fn(root);
  } finally {
    rmrf(root);
  }
}

// 章の末尾に追記する（空行を 1 つ挟む）。
function append(root, rel, text) {
  fs.appendFileSync(path.join(root, rel), '\n' + text, 'utf8');
}

// 抜き出しの場合で使う、1 段目の辞書の行（正規化後）。
const BOOT = '01-boot.pasta';
const BOOT_LINES = normalizeForCompare(realDic(BOOT)).split('\n');

// ============================================================
log('\n== (A) 実リポジトリ現状: 全 dic が同じ名前の章と逐語一致 ==');
{
  const result = runTutorialCheck(REPO_ROOT);
  check('実リポジトリ: ok=true（exit 0 相当）', result.ok === true,
    JSON.stringify(result.results.filter((r) => !r.matched)));
  check(`実リポジトリ: 列挙した ${DIC.length} 件の dic を検証`,
    DIC.length > 0 && result.results.length === DIC.length,
    `listed=${DIC.length} checked=${result.results.length}`);
  check('実リポジトリ: 全 dic が verbatim-match',
    result.results.every((r) => r.matched && r.reason === 'verbatim-match'),
    JSON.stringify(result.results.filter((r) => !r.matched)));
  check('実リポジトリ: problems が空', result.problems.length === 0,
    JSON.stringify(result.problems));
  check('実リポジトリ: reportTutorialCheck が文字列を返す',
    typeof reportTutorialCheck(result) === 'string');
}

// ============================================================
log('\n== (B-1) 段ごとの照合の成功（サンドボックスのまま） ==');
inSandbox((root) => {
  const result = runTutorialCheck(root);
  check('一致: ok=true', result.ok === true,
    JSON.stringify([result.fatal, result.results, result.problems]));
  check('一致: 全 dic が verbatim-match',
    result.results.length === DIC.length
      && result.results.every((r) => r.matched && r.reason === 'verbatim-match'));
  check('一致: 各 dic の章は同じ名前の .md',
    result.results.every((r, i) => r.file === dicRel(DIC[i]) && r.chapter === chapterRel(DIC[i])),
    JSON.stringify(result.results));
  check('一致: 辞書の無い章（index.md・13-nar.md）は pasta ブロックが無ければ記録されない',
    result.problems.length === 0, JSON.stringify(result.problems));
  check('一致: 見た章と pasta ブロックの数',
    result.chapters === DIC.length + 2 && result.blocks === DIC.length,
    `chapters=${result.chapters} blocks=${result.blocks}`);
});

// ============================================================
log('\n== (B-2) 章の作例の不一致（1 章の全体ブロックに 1 行足す） ==');
inSandbox((root) => {
  writeFile(root, chapterRel(BOOT), '# 章\n\n'
    + fenced(BOOT_LINES.join('\n') + '\n＃ 章の側にだけ混入した余計な行\n'));
  const result = runTutorialCheck(root);
  check('改変で ok=false（exit 1 相当）', result.ok === false);
  const bad = result.results.find((r) => r.file === dicRel(BOOT));
  check('01-boot.pasta が no-matching-block',
    bad && bad.matched === false && bad.reason === 'no-matching-block'
      && bad.chapter === chapterRel(BOOT),
    JSON.stringify(bad));
  check('そのブロックが not-in-dic（章と先頭行を名指し）',
    result.problems.length === 1 && result.problems[0].reason === 'not-in-dic'
      && result.problems[0].chapter === chapterRel(BOOT)
      && result.problems[0].head === BOOT_LINES[0],
    JSON.stringify(result.problems));
  check('他の dic は依然 match',
    result.results.filter((r) => r.file !== dicRel(BOOT)).every((r) => r.matched));
});

// ============================================================
log('\n== (B-2b) 全体ブロックの欠落（章に辞書の先頭 2 行の抜き出ししか無い） ==');
inSandbox((root) => {
  // 抜き出しは章から辞書への向きでは通る。辞書から章への向きは全体のブロックが要る。
  writeFile(root, chapterRel(BOOT), '# 章\n\n' + fenced(BOOT_LINES.slice(0, 2).join('\n')));
  const result = runTutorialCheck(root);
  check('抜き出しだけの章は ok=false', result.ok === false);
  const bad = result.results.find((r) => r.file === dicRel(BOOT));
  check('抜き出しだけ: 01-boot.pasta が no-matching-block',
    bad && bad.matched === false && bad.reason === 'no-matching-block',
    JSON.stringify(bad));
  check('抜き出しだけ: 抜き出しそのものは問題にしない（problems は空）',
    result.problems.length === 0, JSON.stringify(result.problems));
});

// ============================================================
log('\n== (B-3)章の欠落（章の無い辞書を dic/ に足す・列挙で拾われる） ==');
inSandbox((root) => {
  writeFile(root, dicRel('14-extra.pasta'), '＊追加\n　章の無い辞書。\n');
  const result = runTutorialCheck(root);
  check('追加 dic で ok=false', result.ok === false);
  check('追加 dic も検証対象に入る', result.results.length === DIC.length + 1,
    `checked=${result.results.length}`);
  const extra = result.results.find((r) => r.file === dicRel('14-extra.pasta'));
  check('14-extra.pasta が no-chapter（期待する章のパス付き）',
    extra && extra.matched === false && extra.reason === 'no-chapter'
      && extra.chapter === `${GUIDE_REL}/14-extra.md`,
    JSON.stringify(extra));
  check('他の dic は依然 match・problems は空',
    result.results.filter((r) => r !== extra).every((r) => r.matched)
      && result.problems.length === 0);
});

// ============================================================
log('\n== (B-3b) 辞書の欠落（章を残して辞書を 1 つ消す） ==');
inSandbox((root) => {
  fs.rmSync(path.join(root, dicRel('02-talk.pasta')));
  const result = runTutorialCheck(root);
  check('辞書の欠落で ok=false', result.ok === false);
  check('残った dic は全件 match',
    result.results.length === DIC.length - 1 && result.results.every((r) => r.matched),
    JSON.stringify(result.results.filter((r) => !r.matched)));
  check('その章のブロックが no-dic',
    result.problems.length === 1 && result.problems[0].reason === 'no-dic'
      && result.problems[0].chapter === chapterRel('02-talk.pasta'),
    JSON.stringify(result.problems));
});

// ============================================================
log('\n== (B-3c) 旧方式（全部の辞書のブロックを 1 枚の章に集める） ==');
inSandbox((root) => {
  for (const name of DIC) fs.rmSync(path.join(root, chapterRel(name)));
  writeFile(root, chapterRel(BOOT), '# 旧方式\n\n' + DIC.map((n) => fenced(realDic(n))).join('\n'));
  const result = runTutorialCheck(root);
  check('旧方式で ok=false', result.ok === false);
  check('集めた章と同じ名前の辞書だけ verbatim-match・ほかの辞書は no-chapter',
    result.results.every((r) => (r.file === dicRel(BOOT)
      ? r.reason === 'verbatim-match'
      : r.matched === false && r.reason === 'no-chapter')),
    JSON.stringify(result.results));
  check('集めた章の余分なブロックが not-in-dic',
    result.problems.length === DIC.length - 1
      && result.problems.every((p) => p.reason === 'not-in-dic' && p.chapter === chapterRel(BOOT)),
    JSON.stringify(result.problems));
});

// ============================================================
log('\n== (B-3d) 抜き出しの一致（章に辞書の連続した 2 行のブロックを足す） ==');
inSandbox((root) => {
  append(root, chapterRel(BOOT), fenced(BOOT_LINES.slice(1, 3).join('\n')));
  const result = runTutorialCheck(root);
  check('連続した 2 行の抜き出しは ok=true', result.ok === true,
    JSON.stringify(result.problems));
  check('抜き出しのブロックも数に入る', result.blocks === DIC.length + 1,
    `blocks=${result.blocks}`);
});

// ============================================================
log('\n== (B-3e) 抜き出しの不一致（飛び飛びの 2 行・1 文字違い） ==');
inSandbox((root) => {
  append(root, chapterRel(BOOT), fenced([BOOT_LINES[0], BOOT_LINES[2]].join('\n')));
  append(root, chapterRel(BOOT), fenced([BOOT_LINES[1], BOOT_LINES[2] + '！'].join('\n')));
  const result = runTutorialCheck(root);
  check('抜き出しの不一致で ok=false', result.ok === false);
  check('辞書の全体のブロックは残っているので全 dic は match',
    result.results.every((r) => r.matched));
  check('飛び飛びの 2 行・1 文字違いがどちらも not-in-dic',
    result.problems.length === 2
      && result.problems.every((p) => p.reason === 'not-in-dic' && p.chapter === chapterRel(BOOT))
      && result.problems[0].head === BOOT_LINES[0] && result.problems[1].head === BOOT_LINES[1],
    JSON.stringify(result.problems));
  const rep = reportTutorialCheck(result);
  check('レポートに章と先頭行',
    rep.includes(chapterRel(BOOT)) && rep.includes(BOOT_LINES[0]) && rep.includes(BOOT_LINES[1]),
    rep);
});

// ============================================================
log('\n== (B-3f) 辞書の無い章のブロック（index.md に pasta ブロックを置く） ==');
inSandbox((root) => {
  append(root, `${GUIDE_REL}/index.md`, fenced('＊OnBoot\n　女の子：やっほー。\n'));
  const result = runTutorialCheck(root);
  check('辞書の無い章のブロックで ok=false', result.ok === false);
  check('index.md のブロックが no-dic（章と先頭行を名指し）',
    result.problems.length === 1 && result.problems[0].reason === 'no-dic'
      && result.problems[0].chapter === `${GUIDE_REL}/index.md`
      && result.problems[0].head === '＊OnBoot',
    JSON.stringify(result.problems));
});

// ============================================================
log('\n== (B-3g) 字下げのフェンス（リスト・引用の中の pasta フェンス） ==');
inSandbox((root) => {
  append(root, chapterRel(BOOT), '- 項目\n\n  ```pasta\n  ＊OnBoot\n  ```\n');
  append(root, chapterRel('02-talk.pasta'), '> ```pasta\n> ＊会話\n> ```\n');
  append(root, chapterRel('04-variety.pasta'), '- ```pasta\n  ＊会話\n  ```\n');
  // ```text の中に書いた字下げの ```pasta は例示であり、フェンスの開きではない。
  append(root, chapterRel('03-face.pasta'), '```text\n  ```pasta\n  例\n  ```\n```\n');
  const result = runTutorialCheck(root);
  check('字下げのフェンスで ok=false', result.ok === false);
  check('リストの中のフェンスが indented-fence（章とその行を名指し）',
    result.problems.some((p) => p.reason === 'indented-fence'
      && p.chapter === chapterRel(BOOT) && p.head === '  ```pasta'),
    JSON.stringify(result.problems));
  check('引用の中のフェンスが indented-fence',
    result.problems.some((p) => p.reason === 'indented-fence'
      && p.chapter === chapterRel('02-talk.pasta') && p.head === '> ```pasta'),
    JSON.stringify(result.problems));
  check('リストの印の直後のフェンスが indented-fence',
    result.problems.some((p) => p.reason === 'indented-fence'
      && p.chapter === chapterRel('04-variety.pasta') && p.head === '- ```pasta'),
    JSON.stringify(result.problems));
  check('別の言語のフェンスの中の字下げ ```pasta は記録しない',
    result.problems.length === 3, JSON.stringify(result.problems));
});

// ============================================================
log('\n== (B-4) ユニット: extractPastaBlocks ==');
{
  const md = [
    '# t', '',
    '```pasta', '＃ A', '```', '',
    '```text', 'これは pasta ブロックではない', '```', '',
    '```pasta', '＃ B', '＃ B2', '```', '',
  ].join('\n');
  const blocks = extractPastaBlocks(md);
  check('pasta ブロックのみ 2 件抽出', blocks.length === 2, JSON.stringify(blocks));
  check('内容 A を含む', /＃ A/.test(blocks[0]));
  check('text ブロックは拾わない', !blocks.some((b) => /pasta ブロックではない/.test(b)));
}

// ============================================================
log('\n== (B-5) ユニット: normalizeForCompare / matchDicFile ==');
{
  check('CRLF と LF を同一視',
    normalizeForCompare('a\r\nb\r\n') === normalizeForCompare('a\nb\n'));
  check('末尾空白行を無視',
    normalizeForCompare('a\nb\n\n\n') === normalizeForCompare('a\nb'));
  check('全角空白は保持（正規化しない）',
    normalizeForCompare('　＠笑顔') === '　＠笑顔');

  const dic = '＃ x\n％女の子\n　＠笑顔：\\s[0]\n';
  check('matchDicFile: CRLF ブロックでも一致',
    matchDicFile(dic, ['＃ x\r\n％女の子\r\n　＠笑顔：\\s[0]\r\n']) === true);
  check('matchDicFile: 1 文字違いは不一致',
    matchDicFile(dic, ['＃ x\n％男の子\n　＠笑顔：\\s[0]\n']) === false);

  // ハードニング境界（cell 3.59）: 単独 CR（旧 Mac 改行）も LF と同一視する
  // （gen-skill-refs.mjs の /\r\n?/g 正規化と対称。改行コード差は内容差でない）。
  check('単独 CR と LF を同一視（\\r\\n? 正規化・gen-skill-refs と対称）',
    normalizeForCompare('a\rb\r') === normalizeForCompare('a\nb\n'));
  check('matchDicFile: 単独 CR の dic でも LF ブロックと一致',
    matchDicFile('＃ x\r％女の子\r　＠笑顔：\\s[0]\r', [dic]) === true);
}

// ============================================================
log('\n== (B-6) fatal 経路（入門の章のフォルダ不在） ==');
{
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  try {
    writeFile(root, dicRel('01-boot.pasta'), '＊OnBoot\n');
    const result = runTutorialCheck(root);
    check('入門のフォルダ不在で ok=false', result.ok === false);
    check('fatal メッセージに入門のフォルダのパスを含む',
      typeof result.fatal === 'string' && result.fatal.includes(GUIDE_REL),
      String(result.fatal));
    check('fatal 時は results・problems が空',
      result.results.length === 0 && result.problems.length === 0);
    const rep = reportTutorialCheck(result);
    check('レポートに FATAL 行', rep.includes('FATAL'));
    check('レポートが RESULT: FAIL', rep.includes('RESULT: FAIL'));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-7) レポート分岐（MISMATCH 表示・対処ガイダンス / OK 表示） ==');
inSandbox((root) => {
  const repGood = reportTutorialCheck(runTutorialCheck(root));
  check('一致レポートに RESULT: OK', repGood.includes('RESULT: OK'));
  check('一致レポートの MATCH 行に辞書と章',
    /MATCH\s+\S*01-boot\.pasta\s.*01-boot\.md/.test(repGood), repGood);

  const talkLines = normalizeForCompare(realDic('02-talk.pasta')).split('\n');
  writeFile(root, chapterRel('02-talk.pasta'), '# 章\n\n'
    + fenced(talkLines.join('\n') + '\n＃ 章の側にだけ混入した余計な行\n'));
  writeFile(root, dicRel('14-extra.pasta'), '＊追加\n　章の無い辞書。\n');
  const repBad = reportTutorialCheck(runTutorialCheck(root));
  check('MISMATCH 行に対象 dic と章と理由',
    /MISMATCH\s+\S*02-talk\.pasta\s.*02-talk\.md\s.*no-matching-block/.test(repBad), repBad);
  check('ブロックの問題の行に章・先頭行・理由',
    repBad.split('\n').some((l) => l.includes(chapterRel('02-talk.pasta'))
      && l.includes(talkLines[0]) && l.includes('not-in-dic')),
    repBad);
  check('章の無い辞書の行に、期待する章のパスと no-chapter',
    repBad.split('\n').some((l) => l.includes(dicRel('14-extra.pasta'))
      && l.includes(`${GUIDE_REL}/14-extra.md`) && l.includes('no-chapter')),
    repBad);
  check('失敗レポートに RESULT: FAIL と対処ガイダンス（章の作例を直す・章を書く）',
    repBad.includes('RESULT: FAIL') && repBad.includes('対処')
      && repBad.includes('連続した行の抜き出し')
      && repBad.includes(`${GUIDE_REL}/<辞書の名前>.md`),
    repBad);
  check('レポートに first-ghost.md の名指しが無い',
    !repGood.includes('first-ghost') && !repBad.includes('first-ghost'));
});

// ============================================================
log('\n== (B-8) extractPastaBlocks 端ケース（CRLF / 行内空白 / 類似言語名 / 未閉鎖） ==');
{
  check('CRLF フェンスを抽出',
    extractPastaBlocks('```pasta\r\n＃ X\r\n```\r\n').length === 1);
  check('```pasta 後の行内空白を許容',
    extractPastaBlocks('```pasta  \n＃ Y\n```\n').length === 1);
  check('pasta 始まりの別言語注記（```pastalang）は拾わない',
    extractPastaBlocks('```pastalang\n＃ Z\n```\n').length === 0);
  check('未閉鎖フェンスは抽出しない',
    extractPastaBlocks('```pasta\n＃ W\n').length === 0);

  // 4 バッククォートのフェンス: 内側の ```lua … ``` は閉じにならず中身として残る。
  const lua = ['＊会話', '```lua', 'function SCENE.f(act)', 'end', '```', '　＠f（）'].join('\n');
  const four = extractPastaBlocks('````pasta\n' + lua + '\n````\n\n```pasta\n＃ 次\n```\n');
  check('````pasta: 内側の ```lua を中身として残し 2 件抽出',
    four.length === 2 && normalizeForCompare(four[0]) === lua
      && normalizeForCompare(four[1]) === '＃ 次',
    JSON.stringify(four));
  check('開きより長いバッククォート行でも閉じる',
    extractPastaBlocks('```pasta\n＃ L\n`````\n').length === 1);
}

// ============================================================
log('\n== (B-8b) listDicFiles（dic/ 直下の *.pasta を辞書順・空なら fatal） ==');
{
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  try {
    writeFile(root, `${HELLO_DIC_REL}/b.pasta`, '＃ b\n');
    writeFile(root, `${HELLO_DIC_REL}/a.pasta`, '＃ a\n');
    writeFile(root, `${HELLO_DIC_REL}/note.txt`, 'x\n');
    writeFile(root, `${HELLO_DIC_REL}/sub/c.pasta`, '＃ c\n');
    const names = listDicFiles(root);
    check('*.pasta だけを辞書順に返す（サブディレクトリ・他拡張子は除外）',
      JSON.stringify(names) === JSON.stringify(['a.pasta', 'b.pasta']), JSON.stringify(names));
    check('実リポジトリの列挙は辞書順',
      JSON.stringify(DIC) === JSON.stringify([...DIC].sort()), JSON.stringify(DIC));
  } finally {
    rmrf(root);
  }
  const empty = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  try {
    writeFile(empty, `${GUIDE_REL}/01-boot.md`, '```pasta\n＃ x\n```\n');
    const result = runTutorialCheck(empty);
    check('dic が 1 件も無ければ fatal で ok=false（空集合で素通りしない）',
      result.ok === false && typeof result.fatal === 'string'
        && result.fatal.includes(HELLO_DIC_REL),
      String(result.fatal));
    check('dic 0 件のレポートに FATAL 行', reportTutorialCheck(result).includes('FATAL'));
  } finally {
    rmrf(empty);
  }
}

// ============================================================
log('\n== (B-8c) listGuideChapters（入門のフォルダ直下の *.md を辞書順） ==');
{
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  try {
    check('入門のフォルダが無ければ空配列', listGuideChapters(root).length === 0);
    writeFile(root, `${GUIDE_REL}/index.md`, '# i\n');
    writeFile(root, `${GUIDE_REL}/01-boot.md`, '# 1\n');
    writeFile(root, `${GUIDE_REL}/note.txt`, 'x\n');
    writeFile(root, `${GUIDE_REL}/sub/02-talk.md`, '# 2\n');
    const names = listGuideChapters(root);
    check('*.md だけを辞書順に返す（サブフォルダ・他拡張子は除外）',
      JSON.stringify(names) === JSON.stringify(['01-boot.md', 'index.md']), JSON.stringify(names));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-8d) ユニット: isExcerptOf（辞書の連続した行の抜き出し） ==');
{
  const dic = '＃ 見出し\n＊会話\n　女の子：やあ。\n　男の子：おう。\n';
  check('連続した 2 行は一致', isExcerptOf(dic, '＊会話\n　女の子：やあ。\n') === true);
  check('1 行だけでも一致', isExcerptOf(dic, '　女の子：やあ。\n') === true);
  check('辞書の全体は一致', isExcerptOf(dic, dic) === true);
  check('飛び飛びの行は不一致', isExcerptOf(dic, '＊会話\n　男の子：おう。\n') === false);
  check('順序の入れ替えは不一致', isExcerptOf(dic, '　女の子：やあ。\n＊会話\n') === false);
  check('行の途中からは不一致', isExcerptOf(dic, '会話\n　女の子：やあ。\n') === false);
  check('行の途中までは不一致', isExcerptOf(dic, '＊会話\n　女の子：やあ\n') === false);
  check('1 文字違いは不一致', isExcerptOf(dic, '＊会話\n　女の子：やあ！\n') === false);
  check('空のブロックは不一致', isExcerptOf(dic, '') === false && isExcerptOf(dic, '\n\n') === false);
  // 実際の辞書は空行を含む。空行のある辞書でも、空のブロックはその空行の抜き出しにしない。
  const blankDic = 'a\n\nb\n';
  check('空行を含む辞書でも空のブロックは不一致', isExcerptOf(blankDic, '') === false);
  check('空行を含む辞書でも空白行だけのブロックは不一致', isExcerptOf(blankDic, '\n\n') === false);
  check('CRLF のブロックでも一致', isExcerptOf(dic, '＊会話\r\n　女の子：やあ。\r\n') === true);
  check('単独 CR の辞書でも一致',
    isExcerptOf(dic.replace(/\n/g, '\r'), '＊会話\n　女の子：やあ。\n') === true);
  check('ブロックの末尾の空白行を無視', isExcerptOf(dic, '＊会話\n　女の子：やあ。\n\n\n') === true);

  // 4 本フェンスで囲んだ、```lua を含む抜き出し（12 段目の形）。
  const luaDic = ['＊会話', '```lua', 'function SCENE.f(act)', 'end', '```', '　＠f（）', ''].join('\n');
  const [luaBlock] = extractPastaBlocks('````pasta\n```lua\nfunction SCENE.f(act)\nend\n```\n````\n');
  check('````pasta で囲んだ ```lua を含む抜き出しが一致', isExcerptOf(luaDic, luaBlock) === true,
    JSON.stringify(luaBlock));
}

// ============================================================
log('\n== (B-9) CLI 結線（exit code / RESULT 出力） ==');
{
  const here = path.dirname(fileURLToPath(import.meta.url));
  const r = spawnSync(process.execPath, [path.join(here, 'tutorial-check.mjs')], {
    encoding: 'utf8',
    timeout: 60000,
  });
  check('CLI: 逐語一致の実リポジトリで exit 0', r.status === 0, `status=${r.status}`);
  check('CLI: RESULT: OK を出力', /RESULT: OK/.test(r.stdout || ''),
    (r.stdout || '').slice(-200));
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
