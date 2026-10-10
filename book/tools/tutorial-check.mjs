// tutorial-check.mjs — 入門の章の作例 ↔ hello-pasta 実ファイル 一致ガード
// （要件 9 / design「ツール層 > TutorialCheck」「System Flows > 作例の照合」）。
//
// 目的:
//   入門の章 book/src/getting-started/*.md は、起動可能な最小ゴースト hello-pasta の
//   dic/*.pasta を、辞書と同じ名前の章（dic/X.pasta ↔ X.md）に「逐語転記」している。
//   本スクリプトは、その逐語転記が崩れていないことを CI で機械的にガードする。
//   崩れていれば exit 1。
//
//   一致が保証されれば、構文妥当性は既存の `cargo test -p pasta_sample_ghost`
//   （self_deploy_integration_test.rs が本物の PastaLoader::load で hello-pasta の
//    .pasta を実際に読み込み・トランスパイルして検証）が transitively 担保する。
//   ＝章の作例が「起動可能な最小セット一式（hello-pasta 由来）」と
//   一致することの機械的ガード（フル SSP 実行なし）。
//
// 検証方式（2 段ガードの第 1 段。第 2 段の構文検証は manual.yml の cargo test ステップ）:
//   pasta_check CLI には .pasta 構文検証サブコマンドが無い（release のみ）ため、
//   本スクリプトは「逐語一致」検証に専念する。照合は 2 つの向きで行う。
//   1. 辞書から章へ: 辞書 dic/X.pasta があれば、章 X.md があり、その章に辞書の全体と
//      一致する ```pasta ブロックがある（無ければ no-chapter / no-matching-block）。
//   2. 章から辞書へ: 入門のどの章の ```pasta ブロックも、同じ名前の辞書の全体か、
//      連続した行の抜き出しである（違えば not-in-dic。辞書の無い章なら no-dic）。
//      字下げ・引用の中の ```pasta フェンスは抽出できないので indented-fence で失敗にする。
//
// 照合方式（堅牢性のため正規化して比較）:
//   - 情報文字列が pasta のフェンスだけを見る（text・toml の転記は照合しない）。
//   - 改行コード（CRLF/LF/CR）と末尾の余分な空白行のみ正規化（内容・全角空白等は不変）。
//   - 辞書も章も列挙から導く（固定一覧なし。辞書を足せば自動で照合対象に入る）。
//
// 冪等・決定論的。book/src・crates は読み取りのみ（書き込みなし）。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
// book/tools/ から 2 つ上がリポジトリルート。
export const REPO_ROOT = path.resolve(here, '../..');

// 入門の章のフォルダ（直下の *.md が章）。
export const GUIDE_REL = 'book/src/getting-started';

// hello-pasta の SSOT（dist-src は廃止）。入門の章が逐語転記している dic 群。
export const HELLO_DIC_REL = 'crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic';

// ---- 入門の章: GUIDE_REL 直下の *.md を辞書順に（ファイル名のみ） ----
// フォルダが無ければ空配列。
export function listGuideChapters(repoRoot = REPO_ROOT) {
  const dir = path.resolve(repoRoot, GUIDE_REL);
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith('.md'))
    .map((e) => e.name)
    .sort();
}

// ---- 照合対象: HELLO_DIC_REL 直下の *.pasta を辞書順に（ファイル名のみ） ----
// 固定一覧を持たないので、dic を足せば自動で照合対象に入る。dic/ が無ければ空配列。
export function listDicFiles(repoRoot = REPO_ROOT) {
  const dir = path.resolve(repoRoot, HELLO_DIC_REL);
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith('.pasta'))
    .map((e) => e.name)
    .sort();
}

// ---- ```pasta フェンス内コードブロックを全抽出 ----
// 言語注記 `pasta`（行末まで他の語が無い）の、3 つ以上のバッククォートで始まるフェンスのみ対象。
// 閉じは開きと同じ長さ以上のバッククォートだけの行（CommonMark のフェンス規則）。
// よって ````pasta の中の ```lua … ``` は閉じにならず中身として残る。未閉鎖のフェンスは捨てる。
export function extractPastaBlocks(markdown) {
  const blocks = [];
  let fence = null; // 開いているフェンスのバッククォート列
  let body = [];
  for (const line of markdown.split('\n')) {
    const text = line.replace(/\r$/, '');
    if (fence === null) {
      const open = /^(`{3,})pasta[^\S\r\n]*$/.exec(text);
      if (open) {
        fence = open[1];
        body = [];
      }
    } else if (/^`+[^\S\r\n]*$/.test(text) && text.trimEnd().length >= fence.length) {
      blocks.push(body.map((l) => l + '\n').join(''));
      fence = null;
    } else {
      body.push(line);
    }
  }
  return blocks;
}

// ---- 比較用正規化: 改行を LF へ、末尾の空白行を除去 ----
// 内容（全角空白・コメント・セリフ等）は一切変更しない。
// 改行は CRLF だけでなく単独 CR（旧 Mac 形式）も LF へ正規化する
// （gen-skill-refs.mjs の /\r\n?/g 正規化と対称・改行コード差を内容差としない）。
export function normalizeForCompare(text) {
  return text.replace(/\r\n?/g, '\n').replace(/\s+$/, '');
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

// ---- ブロックが辞書の連続した行の抜き出し（全体を含む）かを判定 ----
// 行ごとの完全一致で見る。行の途中からの一致・飛び飛びの行・順序の入れ替えは false。
// 正規化後のブロックが空なら false（空のブロックはどの辞書の抜き出しでもない）。
export function isExcerptOf(dicContent, block) {
  const want = normalizeForCompare(block);
  if (want === '') return false;
  // 前後を改行で挟んで探すと、行の境界でしか一致しない。
  return `\n${normalizeForCompare(dicContent)}\n`.includes(`\n${want}\n`);
}

// ---- 字下げ・引用の中にある ```pasta フェンスの開き行を拾う ----
// extractPastaBlocks は行頭のフェンスしか抽出しないので、字下げ・引用・リストの中の
// フェンスは照合をすり抜ける。ここで開き行を拾って失敗にする。
// 行頭のフェンス（言語を問わない）の中は中身なので見ない。
function findIndentedFences(markdown) {
  const heads = [];
  let fence = null; // 開いている行頭のフェンスのバッククォート列
  for (const line of markdown.split('\n')) {
    const text = line.replace(/\r$/, '');
    if (fence === null) {
      const open = /^(`{3,})[^`]*$/.exec(text);
      if (open) {
        fence = open[1];
      } else if (/^(?:[\s>]|[-*+]\s|\d+[.)]\s)+`{3,}pasta\s*$/.test(text)) {
        heads.push(text);
      }
    } else if (/^`+\s*$/.test(text) && text.trimEnd().length >= fence.length) {
      fence = null;
    }
  }
  return heads;
}

// ---- 台詞以外の段落のうち、指示の一文でないもの（説明の地の文）を拾う ----
// 本文検査（verify-content.mjs）の C-prose の判定（要件 10.8）。本文検査は import した時点で
// 検査が走り、中の関数を単体テストできないので、純粋な関数としてここに置く。
// 行頭のフェンスの中は見ない。段落は空行・フェンス・見出し・区切りで区切る
// （見出しと区切りは 1 行で終わるので、直後の行は別の段落として見る）。
// 先頭行が引用（台詞部品）・箇条書き・表・字下げ（箇条書きの続き）で始まる段落は対象にしない。
// 残った段落のうち「1 行だけ・末尾が句点・インラインコードの外の句点が 1 つ」でないものを返す。
/** @returns {ProseProblem[]} */
export function findProseParagraphs(markdown) {
  const paragraphs = []; // { line, lines }
  let fence = null; // 開いている行頭のフェンスのバッククォート列
  let current = null; // 行を足している段落
  markdown.split(/\r?\n/).forEach((text, i) => {
    if (fence !== null) {
      if (/^`+\s*$/.test(text) && text.trimEnd().length >= fence.length) fence = null;
      return;
    }
    const open = /^(`{3,})[^`]*$/.exec(text);
    if (open) fence = open[1];
    if (open || text.trim() === '' || /^#+\s/.test(text) || /^-{3,}\s*$/.test(text)) {
      current = null;
      return;
    }
    if (current === null) {
      current = { line: i + 1, lines: [] };
      paragraphs.push(current);
    }
    current.lines.push(text);
  });
  const isInstruction = (lines) => {
    const text = lines[0].trimEnd();
    // インラインコード（同じ本数のバッククォートで挟んだ範囲）を除いて句点を数える。
    const outside = text.replace(/(`+).*?\1/g, '');
    return lines.length === 1 && text.endsWith('。') && outside.split('。').length === 2;
  };
  return paragraphs
    .filter(({ lines }) => !/^(?:>|\||[ \t]|[-*+]\s|\d+\.\s)/.test(lines[0]) && !isInstruction(lines))
    .map(({ line, lines }) => ({ line, head: lines[0] }));
}

/**
 * @typedef {object} ProseProblem 指示の一文でない段落（説明の地の文）
 * @property {number} line 段落の先頭行（1 始まり）
 * @property {string} head 段落の先頭行の文字列（名指し用）
 *
 * @typedef {object} DicResult 辞書 1 ファイルの結果（辞書から章へ）
 * @property {string} file 辞書（リポジトリルートからの相対）
 * @property {string} chapter 対応する章（同上。無くても期待するパスを入れる）
 * @property {boolean} matched
 * @property {'verbatim-match' | 'no-chapter' | 'no-matching-block'} reason
 *
 * @typedef {object} BlockProblem 章のブロックの問題（章から辞書へ）
 * @property {string} chapter 章（リポジトリルートからの相対）
 * @property {string} head ブロックの先頭行（名指し用。indented-fence はフェンスの開き行）
 * @property {'not-in-dic' | 'no-dic' | 'indented-fence'} reason
 *
 * @typedef {object} TutorialCheckResult
 * @property {boolean} ok fatal が無く、results がすべて matched で、problems が空
 * @property {string | null} fatal 入門の章のフォルダが無い・辞書が 1 件も無い
 * @property {number} chapters 見た章の数
 * @property {number} blocks 見た pasta ブロックの数
 * @property {DicResult[]} results 辞書 1 ファイルにつき 1 件
 * @property {BlockProblem[]} problems
 */

// ---- オーケストレーション ----
/** @returns {TutorialCheckResult} */
export function runTutorialCheck(repoRoot = REPO_ROOT) {
  const empty = { ok: false, chapters: 0, blocks: 0, results: [], problems: [] };
  if (!fs.existsSync(path.resolve(repoRoot, GUIDE_REL))) {
    return { ...empty, fatal: `入門の章のフォルダが見つからない: ${GUIDE_REL}` };
  }
  const dicNames = listDicFiles(repoRoot);
  if (dicNames.length === 0) {
    // 空集合で every() が真になり素通りするのを防ぐ。
    return { ...empty, fatal: `照合対象の dic ファイルが無い: ${HELLO_DIC_REL}/*.pasta` };
  }
  const read = (rel, name) => fs.readFileSync(path.resolve(repoRoot, rel, name), 'utf8');
  const dics = new Map(dicNames.map((name) => [name, read(HELLO_DIC_REL, name)]));
  const chapters = new Map(
    listGuideChapters(repoRoot).map((name) => [name, read(GUIDE_REL, name)]),
  );

  // 辞書から章へ: 同じ名前の章があり、辞書の全体と一致するブロックがあるか。
  const results = [];
  for (const [name, content] of dics) {
    const chapterName = name.replace(/\.pasta$/, '.md');
    const markdown = chapters.get(chapterName);
    let reason = 'no-chapter';
    if (markdown !== undefined) {
      reason = matchDicFile(content, extractPastaBlocks(markdown))
        ? 'verbatim-match'
        : 'no-matching-block';
    }
    results.push({
      file: `${HELLO_DIC_REL}/${name}`,
      chapter: `${GUIDE_REL}/${chapterName}`,
      matched: reason === 'verbatim-match',
      reason,
    });
  }

  // 章から辞書へ: どのブロックも、同じ名前の辞書の全体か連続した行の抜き出しか。
  const problems = [];
  let blocks = 0;
  for (const [name, markdown] of chapters) {
    const chapter = `${GUIDE_REL}/${name}`;
    const dic = dics.get(name.replace(/\.md$/, '.pasta'));
    for (const block of extractPastaBlocks(markdown)) {
      blocks++;
      if (dic !== undefined && isExcerptOf(dic, block)) continue;
      problems.push({
        chapter,
        head: normalizeForCompare(block).split('\n')[0],
        reason: dic === undefined ? 'no-dic' : 'not-in-dic',
      });
    }
    for (const head of findIndentedFences(markdown)) {
      problems.push({ chapter, head, reason: 'indented-fence' });
    }
  }

  const ok = results.every((r) => r.matched) && problems.length === 0;
  return { ok, fatal: null, chapters: chapters.size, blocks, results, problems };
}

// ---- レポート ----
export function reportTutorialCheck(result) {
  const out = [];
  out.push('tutorial-check (入門の章 ↔ hello-pasta 実ファイル 逐語一致ガード)');
  if (result.fatal) {
    out.push(`  FATAL: ${result.fatal}`);
    out.push('RESULT: FAIL');
    return out.join('\n');
  }
  out.push(`  入門の章: ${result.chapters} 枚`);
  out.push(`  抽出 pasta コードブロック: ${result.blocks} 件`);
  out.push(`  検証対象 dic ファイル: ${result.results.length} 件`);
  out.push('');
  for (const r of result.results) {
    const tag = r.matched ? 'MATCH   ' : 'MISMATCH';
    out.push(`  ${tag}  ${r.file}  →  ${r.chapter}  [${r.reason}]`);
  }
  for (const p of result.problems) {
    out.push(`  PROBLEM   ${p.chapter}  「${p.head}」  [${p.reason}]`);
  }
  out.push('');
  if (result.ok) {
    out.push('RESULT: OK（全 dic が同じ名前の章と逐語一致・章の pasta ブロックはすべて辞書の全体か抜き出し）');
  } else {
    out.push('RESULT: FAIL（章の作例と実ファイルの逐語転記が崩れている → 公開中断）');
    out.push('  対処: 章の ```pasta ブロックを辞書の現内容に合わせること（辞書の全体か、連続した行の抜き出し）。');
    out.push(`        辞書と同じ名前の章が無いときは ${GUIDE_REL}/<辞書の名前>.md を書くこと。`);
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
