// scope-map-test.mjs — ScopeClassMapper（タスク 2.1 / 要件 1.2, 1.3, 2.1, 2.2, 2.3, 6.1）の
// 自動検証。依存ゼロの自前 assert ハーネス（book/tools/bigram-index/build-index-test.mjs の
// check(name, cond, detail) 様式）に倣う。
//
// 目的:
//   scopesToClass(scopes) が TextMate スコープスタックを highlight.js 互換クラス
//   （両テーマ light/navy で確実に着色され色衝突しない6色スロット）へ写像する純関数
//   であることを、代表スコープ・lua スコープ・未マッチ・末尾優先・禁止クラス非使用・
//   決定論の観点で実機検証する。
//
//   - RED  : scope-map.mjs 未実装ゆえ import が解決できず即失敗。
//   - GREEN: scope-map.mjs 実装後、全ケース PASS で exit 0。
//
// 確定マッピングの根拠: research §8.2（実 CSS 色グループ検証済み）・design L292。

import { scopesToClass, SCOPE_CLASS_TABLE } from './scope-map.mjs';

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

// テスト対象が公開すべき禁止クラス（navy 無色 / 同色衝突）。
const FORBIDDEN_CLASSES = ['hljs-symbol', 'hljs-section', 'hljs-name'];

// 採用が許される確実着色6スロット（design ScopeClassMapper / research §8.2）。
const ALLOWED_CLASSES = new Set([
  'hljs-comment',  // gray
  'hljs-keyword',  // purple
  'hljs-title',    // blue/aqua
  'hljs-built_in', // orange
  'hljs-literal',  // orange
  'hljs-number',   // orange
  'hljs-string',   // green
  'hljs-variable', // red
  'hljs-attr',     // red
  'hljs-tag',      // red
]);

// =========================================================================
// 1. 代表 pasta スコープ → 期待クラス（6色グループ・research §8.2 確定表）
// =========================================================================
log('--- pasta 代表スコープ → 期待クラス ---');
// 各ケースは [説明, スコープ単体, 期待クラス]。tokenizeLine 由来は外→内のスタックだが
// ここでは「末尾最特定」の写像挙動を単一スコープで検証する（複数スコープは後段）。
const PASTA_CASES = [
  // comment（gray）
  ['行コメント', 'comment.line.pasta', 'hljs-comment'],
  // keyword（purple）
  ['マーカー', 'keyword.other.marker.pasta', 'hljs-keyword'],
  ['呼び出し（＞）', 'keyword.control.pasta', 'hljs-keyword'],
  // title（blue/aqua）— 名前系。keyword.control.scene は keyword より特定的で title になる。
  ['シーン名', 'keyword.control.scene.pasta', 'hljs-title'],
  ['アクター（％）', 'entity.name.class.pasta', 'hljs-title'],
  ['アクション行アクター', 'entity.name.type.actor.pasta', 'hljs-title'],
  // built_in（orange）
  ['関数呼出', 'entity.name.function.call.pasta', 'hljs-built_in'],
  ['キュー呼出', 'entity.name.function.cue.pasta', 'hljs-built_in'],
  // literal / number（orange）
  ['エスケープ', 'constant.character.escape.pasta', 'hljs-literal'],
  ['数値', 'constant.numeric.pasta', 'hljs-number'],
  // string（green）
  ['単語（＠ markup）', 'markup.inline.raw.string.pasta', 'hljs-string'],
  ['さくらスクリプト', 'string.other.sakura-script.pasta', 'hljs-string'],
  ['引用文字列', 'string.quoted.other.pasta', 'hljs-string'],
  // variable / attr / tag（red）
  ['変数定義（＄）', 'variable.other.pasta', 'hljs-variable'],
  ['変数参照', 'variable.other.reference.pasta', 'hljs-variable'],
  ['属性名（＆）', 'entity.other.attribute-name.pasta', 'hljs-attr'],
  ['タグ語', 'entity.name.tag.pasta', 'hljs-tag'],
];
for (const [desc, scope, expected] of PASTA_CASES) {
  const got = scopesToClass([scope]);
  check(`pasta: ${desc}「${scope}」→ ${expected}`, got === expected,
    `got=${JSON.stringify(got)}`);
}
log('');

// =========================================================================
// 2. 最長前方一致: keyword.control.scene は keyword/keyword.control より特定的
// =========================================================================
log('--- 最長前方一致（scene が keyword に負けない） ---');
{
  const got = scopesToClass(['keyword.control.scene.pasta']);
  check('keyword.control.scene.pasta は hljs-keyword でなく hljs-title',
    got === 'hljs-title', `got=${JSON.stringify(got)}`);
  // keyword.control 単体は keyword 系（purple）であること（scene への誤吸着がない）。
  check('keyword.control.pasta は hljs-keyword（scene へ吸着しない）',
    scopesToClass(['keyword.control.pasta']) === 'hljs-keyword');
}
log('');

// =========================================================================
// 3. 文法非依存: lua スコープも同一テーブルで6色へ写る（前方一致）
// =========================================================================
log('--- 文法非依存（lua スコープ・実 grammars 由来） ---');
const LUA_CASES = [
  ['行コメント', 'comment.line.double-dash.lua', 'hljs-comment'],
  ['ブロックコメント', 'comment.block.lua', 'hljs-comment'],
  ['local キーワード', 'keyword.local.lua', 'hljs-keyword'],
  ['制御キーワード', 'keyword.control.lua', 'hljs-keyword'],
  ['演算子', 'keyword.operator.lua', 'hljs-keyword'],
  ['二重引用文字列', 'string.quoted.double.lua', 'hljs-string'],
  ['単一引用文字列', 'string.quoted.single.lua', 'hljs-string'],
  ['浮動小数', 'constant.numeric.float.lua', 'hljs-number'],
  ['整数', 'constant.numeric.integer.lua', 'hljs-number'],
  ['文字エスケープ', 'constant.character.escape.lua', 'hljs-literal'],
  ['変数', 'variable.other.lua', 'hljs-variable'],
];
for (const [desc, scope, expected] of LUA_CASES) {
  const got = scopesToClass([scope]);
  check(`lua: ${desc}「${scope}」→ ${expected}`, got === expected,
    `got=${JSON.stringify(got)}`);
}
log('');

// =========================================================================
// 4. 未マッチ → null（要件 6.1 プレーン）
// =========================================================================
log('--- 未マッチ → null（プレーン継続・6.1） ---');
const NULL_CASES = [
  ['句読点', 'punctuation.definition.code.begin.pasta'],
  ['区切り', 'punctuation.separator.pasta'],
  ['meta コンテナ', 'meta.talk.pasta'],
  ['meta 埋め込み', 'meta.embedded.block.lua'],
  ['source.pasta ルート', 'source.pasta'],
  ['source.lua ルート', 'source.lua'],
  ['未知スコープ', 'totally.unknown.scope.xyz'],
];
for (const [desc, scope] of NULL_CASES) {
  const got = scopesToClass([scope]);
  check(`null: ${desc}「${scope}」→ null`, got === null, `got=${JSON.stringify(got)}`);
}
// 空配列も null（着色対象なし）。
check('null: 空配列 [] → null', scopesToClass([]) === null,
  `got=${JSON.stringify(scopesToClass([]))}`);
log('');

// =========================================================================
// 5. 末尾最特定の採用（複数スコープ配列で末尾＝最特定が優先）
// =========================================================================
log('--- 末尾最特定の採用（複数スコープスタック） ---');
{
  // 実 tokenizeLine 相当スタック（外→内）。末尾が着色対象。
  const stack1 = ['source.pasta', 'meta.talk.pasta', 'entity.name.function.call.pasta'];
  check('スタック末尾 entity.name.function.call → hljs-built_in',
    scopesToClass(stack1) === 'hljs-built_in', `got=${JSON.stringify(scopesToClass(stack1))}`);

  // 末尾が未着色（meta）で、その手前に着色スコープがある場合は手前を採用する
  // （末尾→先頭走査・最初にヒットしたものを返す）。
  const stack2 = ['source.pasta', 'variable.other.pasta', 'meta.variable.pasta'];
  check('末尾 meta は無着色だが手前 variable.other を採用 → hljs-variable',
    scopesToClass(stack2) === 'hljs-variable', `got=${JSON.stringify(scopesToClass(stack2))}`);

  // どのスコープも未マッチなら null。
  const stack3 = ['source.pasta', 'meta.talk.pasta', 'punctuation.separator.pasta'];
  check('全て無着色のスタック → null',
    scopesToClass(stack3) === null, `got=${JSON.stringify(scopesToClass(stack3))}`);

  // 末尾優先: scene（title）が手前にある keyword.control を上書きしないこと。
  // 末尾 keyword.control.scene を最特定として採用する。
  const stack4 = ['source.pasta', 'keyword.control.pasta', 'keyword.control.scene.pasta'];
  check('末尾 keyword.control.scene を採用 → hljs-title（手前の keyword.control に勝つ）',
    scopesToClass(stack4) === 'hljs-title', `got=${JSON.stringify(scopesToClass(stack4))}`);
}
log('');

// =========================================================================
// 6. 禁止クラス非使用（hljs-symbol / hljs-section / hljs-name は決して出力しない）
// =========================================================================
log('--- 禁止クラス非使用 ---');
{
  // (a) マッピングテーブルを直接走査し、禁止クラスを含まないことを保証。
  const tableClasses = Object.values(SCOPE_CLASS_TABLE);
  for (const forbidden of FORBIDDEN_CLASSES) {
    check(`テーブルに ${forbidden} を含まない`, !tableClasses.includes(forbidden),
      `table=${JSON.stringify(tableClasses)}`);
  }
  // (b) テーブルの全クラスが許可6スロットのいずれかであること（採用範囲の封じ込め）。
  for (const cls of tableClasses) {
    check(`テーブルクラス「${cls}」は許可6スロット内`, ALLOWED_CLASSES.has(cls),
      `cls=${cls}`);
  }
  // (c) 代表入力でも禁止クラスが現れないこと（pasta + lua の全ケースを走査）。
  const allScopes = [...PASTA_CASES, ...LUA_CASES].map((c) => c[1]);
  let leaked = false;
  for (const scope of allScopes) {
    const got = scopesToClass([scope]);
    if (got !== null && FORBIDDEN_CLASSES.includes(got)) leaked = true;
  }
  check('代表入力の出力に禁止クラスが現れない', !leaked);
}
log('');

// =========================================================================
// 7. 純粋性・決定論（同一入力 → 同一出力）
// =========================================================================
log('--- 純粋性・決定論 ---');
{
  const sample = ['source.pasta', 'meta.talk.pasta', 'string.other.sakura-script.pasta'];
  const a = scopesToClass(sample);
  const b = scopesToClass(sample);
  check('同一入力 → 同一出力（決定論）', a === b && a === 'hljs-string',
    `a=${JSON.stringify(a)} b=${JSON.stringify(b)}`);
  // 副作用なし: 入力配列を変更しないこと。
  const frozen = Object.freeze(['comment.line.pasta']);
  let threw = false;
  try {
    scopesToClass(frozen);
  } catch (e) {
    threw = true;
  }
  check('凍結入力でも例外を出さない（入力非破壊）', threw === false);
}
log('');

// =========================================================================
// 8. 不正入力ガード（非配列・非文字列要素・空文字列要素 — 例外を出さず null/スキップ）
// =========================================================================
log('--- 不正入力ガード ---');
{
  // (a) 非配列入力は null（Array.isArray ガード）。
  const NON_ARRAYS = [
    ['null', null],
    ['undefined', undefined],
    ['文字列', 'comment.line.pasta'],
    ['オブジェクト', { 0: 'comment.line.pasta', length: 1 }],
    ['数値', 42],
  ];
  for (const [desc, input] of NON_ARRAYS) {
    let got;
    let threw = false;
    try {
      got = scopesToClass(input);
    } catch (e) {
      threw = true;
    }
    check(`guard: 非配列（${desc}）→ null（例外なし）`, !threw && got === null,
      threw ? 'threw' : `got=${JSON.stringify(got)}`);
  }

  // (b) 配列内の非文字列・空文字列要素は安全にスキップされ、全て未マッチなら null。
  let got;
  let threw = false;
  try {
    got = scopesToClass([123, '', null, undefined]);
  } catch (e) {
    threw = true;
  }
  check('guard: 非文字列/空文字列のみの配列 → null（例外なし）', !threw && got === null,
    threw ? 'threw' : `got=${JSON.stringify(got)}`);

  // (c) 非文字列要素が混在しても、走査は継続し有効スコープでヒットする。
  // 末尾の 123 は classForScope ガードで null → 手前の comment.line.pasta を採用。
  check('guard: 非文字列要素を飛ばして手前の有効スコープを採用',
    scopesToClass(['comment.line.pasta', 123]) === 'hljs-comment',
    `got=${JSON.stringify(scopesToClass(['comment.line.pasta', 123]))}`);
}
log('');

log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
