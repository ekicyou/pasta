// tutorial-check.mjs — チュートリアル成果物 ↔ hello-pasta 実ファイル 一致ガード
// 本実装 / タスク 5.2（要件 6.2 / design「Content: Getting Started」「Publish Pipeline」）。
//
// 目的（design「Testing/コンテンツ整合」）:
//   チュートリアル book/src/getting-started/first-ghost.md は、起動可能な最小ゴースト
//   hello-pasta の dic/*.pasta を「逐語転記」している。本スクリプトは、その逐語転記が
//   崩れていないことを CI で機械的にガードする。崩れていれば exit 1。
//
//   一致が保証されれば、構文妥当性は既存の `cargo test -p pasta_sample_ghost`
//   （self_deploy_integration_test.rs が本物の PastaLoader::load で hello-pasta の
//    .pasta を実際に読み込み・トランスパイルして検証）が transitively 担保する。
//   ＝チュートリアル末の成果物が「起動可能な最小セット一式（hello-pasta 由来）」と
//   一致することの機械的ガード（フル SSP 実行なし）。
//
// 検証方式（2 段ガードの第 1 段。第 2 段の構文検証は manual.yml の cargo test ステップ）:
//   pasta_check CLI には .pasta 構文検証サブコマンドが無い（release のみ）ため、
//   本スクリプトは「逐語一致」検証に専念する。各 hello-pasta dic ファイルの内容が、
//   チュートリアル本文の ```pasta コードブロックのいずれかと「逐語一致」するかを確認する。
//
// 照合方式（堅牢性のため正規化して比較）:
//   - チュートリアルの ```pasta フェンス内テキストを全抽出。
//   - 各 dic/*.pasta 実ファイルについて、コードブロック群のいずれかと一致するか判定。
//   - 改行コード（CRLF/LF）と末尾の余分な空白行のみ正規化（内容・全角空白等は不変）。
//   - 一致しない dic があれば報告して exit 1。
//
// 冪等・決定論的。book/src・crates は読み取りのみ（書き込みなし）。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
// book/tools/ から 2 つ上がリポジトリルート。
export const REPO_ROOT = path.resolve(here, '../..');

// チュートリアル本文（逐語転記元）。
export const TUTORIAL_REL = 'book/src/getting-started/first-ghost.md';

// hello-pasta の SSOT（dist-src は廃止）。チュートリアルが逐語転記している dic 群。
export const HELLO_DIC_REL = 'crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic';
export const DIC_FILES = [
  'actors.pasta',
  'boot.pasta',
  'talk.pasta',
  'click.pasta',
  'choice.pasta',
];

// ---- ```pasta フェンス内コードブロックを全抽出 ----
export function extractPastaBlocks(markdown) {
  const blocks = [];
  // 言語注記 `pasta`（行末まで他の語が無い）のフェンスのみ対象。
  const re = /```pasta[^\S\r\n]*\r?\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(markdown)) !== null) {
    blocks.push(m[1]);
  }
  return blocks;
}

// ---- 比較用正規化: 改行を LF へ、末尾の空白行を除去 ----
// 内容（全角空白・コメント・セリフ等）は一切変更しない。
export function normalizeForCompare(text) {
  return text.replace(/\r\n/g, '\n').replace(/\s+$/, '');
}

// ---- 1 ファイルの逐語一致を判定 ----
export function matchDicFile(dicContent, blocks) {
  const target = normalizeForCompare(dicContent);
  for (const block of blocks) {
    if (normalizeForCompare(block) === target) {
      return true;
    }
  }
  return false;
}

// ---- オーケストレーション ----
export function runTutorialCheck(repoRoot = REPO_ROOT) {
  const tutorialAbs = path.resolve(repoRoot, TUTORIAL_REL);
  if (!fs.existsSync(tutorialAbs)) {
    return {
      ok: false,
      fatal: `チュートリアルが見つからない: ${TUTORIAL_REL}`,
      blocks: [],
      results: [],
    };
  }
  const markdown = fs.readFileSync(tutorialAbs, 'utf8');
  const blocks = extractPastaBlocks(markdown);

  const results = [];
  for (const name of DIC_FILES) {
    const rel = `${HELLO_DIC_REL}/${name}`;
    const abs = path.resolve(repoRoot, HELLO_DIC_REL, name);
    if (!fs.existsSync(abs)) {
      results.push({ file: rel, matched: false, reason: 'missing-source' });
      continue;
    }
    const content = fs.readFileSync(abs, 'utf8');
    const matched = matchDicFile(content, blocks);
    results.push({
      file: rel,
      matched,
      reason: matched ? 'verbatim-match' : 'no-matching-block',
    });
  }

  const ok = results.every((r) => r.matched);
  return { ok, fatal: null, blocks, results };
}

// ---- レポート ----
export function reportTutorialCheck(result) {
  const out = [];
  out.push('tutorial-check (チュートリアル ↔ hello-pasta 実ファイル 逐語一致ガード)');
  if (result.fatal) {
    out.push(`  FATAL: ${result.fatal}`);
    out.push('RESULT: FAIL');
    return out.join('\n');
  }
  out.push(`  抽出 pasta コードブロック: ${result.blocks.length} 件`);
  out.push(`  検証対象 dic ファイル: ${result.results.length} 件`);
  out.push('');
  for (const r of result.results) {
    const tag = r.matched ? 'MATCH ' : 'MISMATCH';
    out.push(`  ${tag}  ${r.file}  [${r.reason}]`);
  }
  out.push('');
  if (result.ok) {
    out.push('RESULT: OK（全 dic がチュートリアル本文と逐語一致）');
  } else {
    out.push('RESULT: FAIL（チュートリアルと実ファイルの逐語転記が崩れている → 公開中断）');
    out.push('  対処: book/src/getting-started/first-ghost.md の該当 ```pasta ブロックを');
    out.push(`        ${HELLO_DIC_REL}/*.pasta の現内容に合わせて更新すること。`);
  }
  return out.join('\n');
}

// CLI: node book/tools/tutorial-check.mjs
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  try {
    const result = runTutorialCheck(REPO_ROOT);
    console.log(reportTutorialCheck(result));
    process.exit(result.ok ? 0 : 1);
  } catch (e) {
    console.error(`tutorial-check failed: ${e && e.stack ? e.stack : e}`);
    process.exit(2);
  }
}
