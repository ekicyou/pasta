// verify-scripts-test.mjs — verify 系スクリプトの回帰スモークテスト
// （review-improvement-loop セル 3.58 / G1 テスト網羅・要件 2.2）。
//
// 背景:
//   verify-content.mjs（タスク 7.4）は
//   import 時に検証本体を即実行する構造のため、本体を改変せずに関数単体テストは
//   できない。代わりに子プロセスとして実行し、
//     - exit 0 で完走する（結線・構文・依存の回帰検出）
//     - 実際に多数のチェックを実行している（退化・空回りでない）
//     - 失敗 0 件で RESULT: OK を出力する
//   を機械的にアサートする。スクリプト本体や検証対象（book/src・steering）が
//   壊れればこのスモークが fail する。
//
// 対象外（根拠）:
//   verify-static.mjs / verify-search.mjs は mdbook ビルド（外部コマンド・
//   book/book 出力生成）が前提のため、高速・読み取り専用のスモーク対象から除外する。
//   verify-static.mjs は --self-test で検証ロジック自身の健全性テストを内蔵している。
//
// 実行: node book/tools/verify-scripts-test.mjs

import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

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

function runScript(name) {
  return spawnSync(process.execPath, [path.join(here, name)], {
    encoding: 'utf8',
    timeout: 120000,
  });
}

// ============================================================
log('\n== verify-content.mjs（コンテンツ整合・網羅レビュー検証） ==');
{
  const r = runScript('verify-content.mjs');
  const out = r.stdout || '';
  check('exit 0 で完走', r.status === 0, `status=${r.status} stderr=${(r.stderr || '').slice(0, 300)}`);
  const m = out.match(/検証項目:\s*(\d+) 件\s+PASS:\s*(\d+)\s+FAIL:\s*(\d+)/);
  check('サマリ行を出力（検証項目/PASS/FAIL）', !!m, out.slice(0, 300));
  check('実際に多数のチェックを実行（>= 50 件）', m && Number(m[1]) >= 50,
    m ? `検証項目=${m[1]}` : '(サマリなし)');
  check('FAIL 0 件', m && Number(m[3]) === 0, m ? `FAIL=${m[3]}` : '(サマリなし)');
  check('RESULT: OK を出力', /RESULT: OK/.test(out), out.slice(-200));
  // 内部設計パート（I 系）が空回りしていない: 10 章の構造検査・題材章 8 章の必須 H2・機構網羅が走る。
  const count = (re) => (out.match(re) || []).length;
  check('I-structure を内部設計 10 章に実行', count(/PASS {2}\[I-structure:/g) === 10,
    `I-structure PASS=${count(/PASS {2}\[I-structure:/g)}`);
  check('I-sections を題材章 8 章に実行', count(/PASS {2}\[I-sections:/g) === 8,
    `I-sections PASS=${count(/PASS {2}\[I-sections:/g)}`);
  check('I-fact（機構網羅・概要章）を実行', count(/PASS {2}\[I-fact:/g) === 2,
    `I-fact PASS=${count(/PASS {2}\[I-fact:/g)}`);
  check('口調検査の走査対象に内部設計パートを含む', /PASS {2}\[D-voice:book\/src\/internals\//.test(out));
}

// ============================================================
log(`\n結果: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
