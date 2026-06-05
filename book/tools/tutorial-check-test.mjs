// tutorial-check-test.mjs — tutorial-check 本実装の自動検証（タスク 5.2 / 要件 6.2）。
//
// 検証方針:
//   book/src・crates を恒久変更しないため、検証は
//   (A) 実リポジトリ現状でのクリーン判定（exit 0 相当・全 dic 逐語一致）と、
//   (B) 一時サンドボックス（tmp）へ最小フィクスチャを構築し、逐語転記の崩れを
//       注入して MISMATCH→failed を確認、で行う。
//   サンドボックスは runTutorialCheck(repoRoot) の repoRoot を差し替えて使う
//   （本物の book/src・crates には一切書き込まない）。
//
// 観測する完了条件（design「Testing/コンテンツ整合」）:
//   - 実リポジトリ: 全 dic がチュートリアルと逐語一致 → ok=true。
//   - 一致サンドボックス → ok=true。
//   - 1 文字でも改変注入 → MISMATCH 検出＆ ok=false（exit 1 相当）。
//   - dic 欠落注入 → missing-source 検出＆ ok=false。
//   - ユニット: extractPastaBlocks / normalizeForCompare / matchDicFile。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  REPO_ROOT,
  TUTORIAL_REL,
  HELLO_DIC_REL,
  DIC_FILES,
  extractPastaBlocks,
  normalizeForCompare,
  matchDicFile,
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

// 実リポジトリの dic 内容から、チュートリアル本文を機械生成して最小フィクスチャを作る。
// （逐語転記そのものを再現するため、本物の dic をコードブロックに埋め込む）。
function makeSandbox({ corruptFile = null, dropFile = null } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tutorial-check-'));
  const tutorialParts = ['# サンドボックス チュートリアル\n\n本文。\n'];
  for (const name of DIC_FILES) {
    if (name === dropFile) continue; // dic ファイル自体を配置しない（欠落注入）
    const realAbs = path.resolve(REPO_ROOT, HELLO_DIC_REL, name);
    let content = fs.readFileSync(realAbs, 'utf8');
    writeFile(root, `${HELLO_DIC_REL}/${name}`, content);

    let blockBody = content.replace(/\r\n/g, '\n');
    if (!blockBody.endsWith('\n')) blockBody += '\n';
    if (name === corruptFile) {
      // チュートリアル側ブロックだけを改変（実ファイルとずれる → MISMATCH）。
      blockBody = blockBody + '＃ チュートリアル側にだけ混入した余計な行\n';
    }
    tutorialParts.push('```pasta\n' + blockBody + '```\n');
  }
  writeFile(root, TUTORIAL_REL, tutorialParts.join('\n'));
  return root;
}

// ============================================================
log('\n== (A) 実リポジトリ現状: 全 dic 逐語一致 ==');
{
  const result = runTutorialCheck(REPO_ROOT);
  check('実リポジトリで ok=true（exit 0 相当）', result.ok === true,
    JSON.stringify(result.results));
  check('5 件の dic を検証', result.results.length === DIC_FILES.length);
  check('全 dic が verbatim-match',
    result.results.every((r) => r.matched && r.reason === 'verbatim-match'),
    JSON.stringify(result.results.filter((r) => !r.matched)));
  check('reportTutorialCheck が文字列を返す', typeof reportTutorialCheck(result) === 'string');
}

// ============================================================
log('\n== (B-1) 一致サンドボックス ==');
{
  const root = makeSandbox();
  try {
    const result = runTutorialCheck(root);
    check('一致: ok=true', result.ok === true, JSON.stringify(result.results));
    check('一致: 全 dic match', result.results.every((r) => r.matched));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-2) 改変注入（チュートリアル側ブロックを 1 行追加） ==');
{
  const root = makeSandbox({ corruptFile: 'boot.pasta' });
  try {
    const result = runTutorialCheck(root);
    check('改変で ok=false（exit 1 相当）', result.ok === false);
    const bad = result.results.find((r) => r.file.endsWith('boot.pasta'));
    check('boot.pasta が no-matching-block',
      bad && bad.matched === false && bad.reason === 'no-matching-block',
      JSON.stringify(bad));
    check('他の dic は依然 match',
      result.results.filter((r) => !r.file.endsWith('boot.pasta')).every((r) => r.matched));
  } finally {
    rmrf(root);
  }
}

// ============================================================
log('\n== (B-3) dic 欠落注入 ==');
{
  const root = makeSandbox({ dropFile: 'choice.pasta' });
  try {
    const result = runTutorialCheck(root);
    check('欠落で ok=false', result.ok === false);
    const miss = result.results.find((r) => r.file.endsWith('choice.pasta'));
    check('choice.pasta が missing-source',
      miss && miss.matched === false && miss.reason === 'missing-source',
      JSON.stringify(miss));
  } finally {
    rmrf(root);
  }
}

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
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
