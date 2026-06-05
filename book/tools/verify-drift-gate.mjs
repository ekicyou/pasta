// verify-drift-gate.mjs — ドリフト検出・完了ゲートの検証（タスク 7.3）
//
// 役割（requirements.md Requirement 10 / design「Drift Detection & Gate」）:
//   drift-check.mjs の検出挙動と、完了ゲート（DoD / Manual Sync Gate）の結線が
//   要件どおり成立していることを、機械的に検証する。本物の検証であること。
//
// 検証対象（観測可能な完了条件）:
//   ① doc/spec 章を改変し記録ハッシュと不一致にすると drift-check が失敗（10.2）
//   ② manual-sources.toml に未マップの doc/spec 章を追加すると未マップ警告（10.2）
//   ③ マニュアル→doc/spec のリンク切れ・自リポ blob URL 切れを検出（10.4）
//   ④ 完了承認（DoD）実行時、未解決ドリフトがあれば完了中断（10.3）／
//      doc/spec・book いずれにも触れない変更ではゲートがスキップ（10.5）
//
// 方針:
//   ①②③ … drift-check.mjs の純関数（runDriftCheck 等）を import し、tmp に作った
//           最小サンドボックス（book/manual-sources.toml ＋ doc/spec ＋ book/src の
//           最小コピー）を repoRoot として渡して検証する。
//           drift-check.mjs / 本物の doc/spec・book/src・manual-sources.toml は
//           一切恒久改変しない（読み取り import のみ／サンドボックスで実施）。
//   ④   … プロセス文書の結線（workflow.md の DoD と SKILL.md のステップ1）を
//           文字列/構造でアサートし、「ゲート統合が成立している」機械的証跡とする。
//
// 終了コード: 全ケース期待どおり → exit 0、いずれか不一致 → exit 1。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

import {
  runDriftCheck,
  sha256File,
} from './drift-check.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(here, '../..');

// ---- ミニアサーション ----
const results = [];
function check(name, cond, detail = '') {
  results.push({ name, ok: !!cond, detail });
  const tag = cond ? 'PASS' : 'FAIL';
  console.log(`  [${tag}] ${name}${detail ? `  — ${detail}` : ''}`);
}

function sha256(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

// ---- サンドボックス構築 ----
// 最小の repoRoot を tmp に作る。drift-check は repoRoot 配下の
//   book/manual-sources.toml / doc/spec / book/src
// だけを参照するため、それらの最小コピーで十分。
function makeSandbox() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drift-sandbox-'));
  const specDir = path.join(root, 'doc', 'spec');
  const srcDir = path.join(root, 'book', 'src', 'grammar');
  fs.mkdirSync(specDir, { recursive: true });
  fs.mkdirSync(srcDir, { recursive: true });

  // doc/spec の最小章を 2 本作る。
  const specA = path.join(specDir, '01-grammar-model.md');
  const specB = path.join(specDir, '02-markers.md');
  fs.writeFileSync(specA, '# 文法モデル\n\n本文 A\n');
  fs.writeFileSync(specB, '# マーカー\n\n本文 B\n');
  const hashA = sha256File(specA);
  const hashB = sha256File(specB);

  // book/src/grammar のマニュアル章。doc/spec への自リポ blob URL リンクを含む。
  // REPO_SLUG は ekicyou/pasta。実在パスへのリンクと、わざと切れたリンクは
  // 各テストケース側で書き換える。クリーン版では実在パスのみ。
  fs.writeFileSync(
    path.join(srcDir, 'index.md'),
    [
      '# 文法',
      '',
      '由来: [01](https://github.com/ekicyou/pasta/blob/main/doc/spec/01-grammar-model.md)',
      '関連: [markers](./markers.md)',
      '',
    ].join('\n'),
  );
  fs.writeFileSync(
    path.join(srcDir, 'markers.md'),
    [
      '# マーカー',
      '',
      '由来: [02](https://github.com/ekicyou/pasta/blob/main/doc/spec/02-markers.md)',
      '',
    ].join('\n'),
  );

  // manual-sources.toml（クリーン: 記録ハッシュ = 現値）。
  const toml = [
    'algorithm = "sha256"',
    '',
    '[[mapping]]',
    'chapter = "book/src/grammar/index.md"',
    'source  = "doc/spec/01-grammar-model.md"',
    `hash    = "${hashA}"`,
    '',
    '[[mapping]]',
    'chapter = "book/src/grammar/markers.md"',
    'source  = "doc/spec/02-markers.md"',
    `hash    = "${hashB}"`,
    '',
  ].join('\n');
  fs.writeFileSync(path.join(root, 'book', 'manual-sources.toml'), toml);

  return { root, specA, specB, srcDir, tomlPath: path.join(root, 'book', 'manual-sources.toml') };
}

function rmSandbox(root) {
  try {
    fs.rmSync(root, { recursive: true, force: true });
  } catch {
    /* best effort */
  }
}

// ================================================================
// (0) クリーンなサンドボックス → exit 0 相当（failed=false）
// ================================================================
function testClean() {
  console.log('\n[0] クリーン状態は OK（failed=false）');
  const sb = makeSandbox();
  try {
    const r = runDriftCheck(sb.root);
    check('クリーン: ドリフト 0 件', r.drift.length === 0, `drift=${r.drift.length}`);
    check('クリーン: リンク切れ 0 件', r.broken.length === 0, `broken=${r.broken.length}`);
    check('クリーン: failed=false', r.failed === false, `failed=${r.failed}`);
  } finally {
    rmSandbox(sb.root);
  }
}

// ================================================================
// (①) doc/spec を改変しハッシュ不一致 → ドリフト検出・failed=true（10.2）
// ================================================================
function testDrift() {
  console.log('\n[1] doc/spec 改変でハッシュ不一致 → ドリフト検出 failed=true（10.2）');
  const sb = makeSandbox();
  try {
    // 記録ハッシュは旧版のまま、source 本文を変更する＝マニュアル未追従。
    fs.writeFileSync(sb.specA, '# 文法モデル\n\n本文 A（改訂・追記あり）\n');
    const r = runDriftCheck(sb.root);
    const hit = r.drift.find(
      (d) => d.reason === 'hash-mismatch' && d.source === 'doc/spec/01-grammar-model.md',
    );
    check('ドリフト hash-mismatch を検出', !!hit, hit ? hit.detail : '未検出');
    check('failed=true（完了中断対象）', r.failed === true, `failed=${r.failed}`);
  } finally {
    rmSandbox(sb.root);
  }
}

// ================================================================
// (②) 未マップの doc/spec 章を追加 → 未マップ警告（10.2）
// ================================================================
function testUnmapped() {
  console.log('\n[2] 未マップ doc/spec 章を追加 → 未マップ警告（10.2）');
  const sb = makeSandbox();
  try {
    // マッピングに無い章を doc/spec に追加（網羅対象外名ではない）。
    fs.writeFileSync(
      path.join(sb.root, 'doc', 'spec', '03-block-structure.md'),
      '# ブロック構造\n\n本文 C\n',
    );
    const r = runDriftCheck(sb.root);
    const hit = r.unmapped.includes('doc/spec/03-block-structure.md');
    check('未マップ章を報告', hit, `unmapped=[${r.unmapped.join(', ')}]`);
    // 既定では未マップのみは failed に寄与しない（警告）。
    check('未マップのみでは failed=false（既定・警告扱い）', r.failed === false, `failed=${r.failed}`);
    // DRIFT_STRICT 相当（strict:true）では failed=true。
    const rs = runDriftCheck(sb.root, { strict: true });
    check('strict 時は未マップで failed=true', rs.failed === true, `failed=${rs.failed}`);
  } finally {
    rmSandbox(sb.root);
  }
}

// ================================================================
// (③) リンク切れ検出（10.4）
//   (a) book 内相対 .md リンク切れ
//   (b) 自リポ blob URL が実在しない
// ================================================================
function testBrokenLinks() {
  console.log('\n[3] リンク切れ検出（10.4）');
  const sb = makeSandbox();
  try {
    // (a) 存在しない相対 .md リンクを追加。
    fs.appendFileSync(
      path.join(sb.srcDir, 'index.md'),
      '\n参照: [missing](./does-not-exist.md)\n',
    );
    // (b) 実在しない自リポ blob URL を追加。
    fs.appendFileSync(
      path.join(sb.srcDir, 'markers.md'),
      '\n壊れURL: [x](https://github.com/ekicyou/pasta/blob/main/doc/spec/99-nope.md)\n',
    );
    const r = runDriftCheck(sb.root);
    const internal = r.broken.find(
      (b) => b.kind === 'internal-md' && b.target.includes('does-not-exist.md'),
    );
    const repoUrl = r.broken.find(
      (b) => b.kind === 'github-repo-path' && b.target.includes('99-nope.md'),
    );
    check('book 内相対 .md リンク切れを検出', !!internal, internal ? internal.detail : '未検出');
    check('自リポ blob URL 切れを検出', !!repoUrl, repoUrl ? repoUrl.detail : '未検出');
    check('failed=true（壊れた参照は中断対象）', r.failed === true, `failed=${r.failed}`);
  } finally {
    rmSandbox(sb.root);
  }
}

// ================================================================
// (③-b) 実在する自リポ blob URL はリンク切れにならない（偽陽性なし）
// ================================================================
function testValidLinks() {
  console.log('\n[3b] 実在リンクは検出されない（偽陽性なし）');
  const sb = makeSandbox();
  try {
    const r = runDriftCheck(sb.root);
    // クリーン版は実在 blob URL ＋ 実在相対リンクのみ。
    check('実在リンクは broken に含まれない', r.broken.length === 0, `broken=${r.broken.length}`);
  } finally {
    rmSandbox(sb.root);
  }
}

// ================================================================
// (④) 完了ゲート統合の結線検証（10.3 / 10.5）
//   workflow.md の DoD「Manual Sync Gate（条件付き）」と
//   SKILL.md ステップ1の drift-check 発火を、文字列/構造でアサート。
// ================================================================
function testGateWiring() {
  console.log('\n[4] 完了ゲート統合の結線（10.3 / 10.5）');

  const workflowPath = path.join(REPO_ROOT, '.kiro', 'steering', 'workflow.md');
  const skillPath = path.join(
    REPO_ROOT,
    '.claude',
    'skills',
    'kiro-spec-complete',
    'SKILL.md',
  );

  check('workflow.md が存在', fs.existsSync(workflowPath), workflowPath);
  check('SKILL.md が存在', fs.existsSync(skillPath), skillPath);
  if (!fs.existsSync(workflowPath) || !fs.existsSync(skillPath)) return;

  const wf = fs.readFileSync(workflowPath, 'utf8');
  const sk = fs.readFileSync(skillPath, 'utf8');

  // --- workflow.md: DoD に Manual Sync Gate がある（権威ルール本体） ---
  check(
    'workflow.md: DoD に「Manual Sync Gate」記載',
    /完了基準（DoD）/.test(wf) && /Manual Sync Gate/.test(wf),
  );
  // 条件付き発火（doc/spec か book に触れる時のみ）— 10.3 前提条件。
  check(
    'workflow.md: 条件付き発火（doc/spec か book 変更時のみ）',
    /発火条件/.test(wf) && /doc\/spec\//.test(wf) && /book\//.test(wf),
  );
  // 無関係変更はスキップ — 10.5。
  check(
    'workflow.md: 無関係変更はスキップ（10.5）',
    /スキップ/.test(wf) &&
      /(にも|どちらにも|いずれにも)?.*book\/.*(にも|も触れない|触れない)/s.test(wf),
  );
  // 非ゼロ終了で完了中断 — 10.3。
  check(
    'workflow.md: 非ゼロ終了で完了中断（10.3）',
    /非ゼロ終了/.test(wf) && /(中断)/.test(wf),
  );
  // 判定で drift-check.mjs を実行する。
  check(
    'workflow.md: drift-check.mjs を判定に使用',
    /node book\/tools\/drift-check\.mjs/.test(wf),
  );

  // --- SKILL.md: ステップ1で同ゲートの drift-check を発火する（結線） ---
  check(
    'SKILL.md: ステップ1（DoD ゲート検証）',
    /ステップ1/.test(sk) && /DoD/.test(sk),
  );
  check(
    'SKILL.md: Manual Sync Gate を発火',
    /Manual Sync Gate/.test(sk),
  );
  check(
    'SKILL.md: drift-check.mjs を実行する',
    /node book\/tools\/drift-check\.mjs/.test(sk),
  );
  // 非ゼロで中断する結線。
  check(
    'SKILL.md: 非ゼロ終了で中断',
    /非ゼロ終了/.test(sk) && /中断/.test(sk),
  );
  // 無関係変更はスキップする結線（10.5）。
  check(
    'SKILL.md: 無関係変更はスキップ（10.5）',
    /スキップ/.test(sk) && /book\//.test(sk),
  );
  // ルール本体は workflow.md（権威）を参照し複製しない結線。
  check(
    'SKILL.md: 判定ルール本体は workflow.md（権威）参照',
    /workflow\.md/.test(sk),
  );
}

// ---- main ----
function main() {
  console.log('verify-drift-gate: ドリフト検出・完了ゲートの検証（タスク 7.3）');
  console.log(`REPO_ROOT = ${REPO_ROOT}`);

  testClean();
  testDrift();
  testUnmapped();
  testBrokenLinks();
  testValidLinks();
  testGateWiring();

  const failed = results.filter((r) => !r.ok);
  console.log('\n================ サマリ ================');
  console.log(`  合計 ${results.length} 件 / 失敗 ${failed.length} 件`);
  if (failed.length > 0) {
    console.log('  失敗:');
    for (const f of failed) console.log(`    - ${f.name} (${f.detail})`);
    console.log('\nRESULT: FAIL');
    process.exit(1);
  }
  console.log('\nRESULT: OK（全ケース期待どおり検出/中断/スキップ）');
  process.exit(0);
}

main();
