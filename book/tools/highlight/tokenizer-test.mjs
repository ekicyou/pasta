// tokenizer-test.mjs — PastaTokenizer 本実装の自動検証（タスク 2.2 / 要件 1.1, 1.3, 5.1, 5.2, 5.3, 6.2）。
//
// 目的:
//   pasta 文法（source.pasta・SSOT・読み取り専用）と vendor lua 文法（source.lua）を
//   実物の oniguruma WASM でロードし、実テキストを行単位トークナイズすることを実機で証明する。
//
//   - 基本トークナイズ（1.1）: 実在の pasta 構文 fixture を行単位でトークナイズし、
//       行数＝返り配列長／各行被覆／代表トークンが期待 pasta スコープを持つ。
//   - 入れ子 lua の二段化（核心・1.3）: pasta パス単独では lua 本体が
//       meta.embedded.block.lua.content 止まりだが、二段化後は lua スコープ
//       （keyword.*.lua / string.*.lua / comment.*.lua / constant.*.lua 等）が付く。
//       この「差」を検証する。
//   - 複数行 lua ブロック跨ぎ: lua 側の行間 ruleStack 継続（複数行文字列）を検証。
//   - オフセット正規化: lua 区間トークンの startIndex/endIndex が元行内の正しい位置を指す。
//   - 文法不在で例外（6.2）: 存在しないパスで load() がリジェクトされる。
//   - 決定論（6.4 を支える）: 同一 text を 2 回トークナイズして同一結果。
//
// 一連の `node book/tools/highlight/tokenizer-test.mjs` が exit 0（全 PASS）を目指す。
// 本テストは ScopeClassMapper（scope-map.mjs・タスク 2.1）に依存しない。トークナイズ結果の
// スコープ検証はスコープ文字列を直接 assert する（境界は PastaTokenizer のみ）。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createPastaTokenizer } from './tokenizer.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const bookDir = path.resolve(here, '../..'); // book/
const repoRoot = path.resolve(bookDir, '..');

const PASTA_GRAMMAR = path.resolve(
  repoRoot,
  'editors/vscode/syntaxes/pasta.tmLanguage.json',
);
const LUA_GRAMMAR = path.resolve(here, 'grammars/lua.tmLanguage.json');

// --- 最小 assert ハーネス（依存ゼロ・bigram build-index-test.mjs に倣う） ---
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

// 行を被覆する TokenSpan 列か（各 span が隣接し、行全体 [0, line.length] を覆う）。
function coversLine(spans, line) {
  if (spans.length === 0) return line.length === 0;
  if (spans[0].startIndex !== 0) return false;
  for (let i = 1; i < spans.length; i++) {
    if (spans[i].startIndex !== spans[i - 1].endIndex) return false;
  }
  return spans[spans.length - 1].endIndex === line.length;
}

// span 列に、末尾（最特定）スコープが指定プレフィックスで始まるトークンが、
// 期待テキスト片を含む形で存在するか。
function hasLeafScopeForText(spans, line, scopePrefix, expectText) {
  return spans.some((s) => {
    const leaf = s.scopes[s.scopes.length - 1] || '';
    if (!leaf.startsWith(scopePrefix)) return false;
    if (expectText == null) return true;
    return line.slice(s.startIndex, s.endIndex).includes(expectText);
  });
}

// span 列のどこかに（末尾でなくても）指定プレフィックスを含むスコープがあるか。
function hasAnyScope(spans, scopePrefix) {
  return spans.some((s) => s.scopes.some((sc) => sc.startsWith(scopePrefix)));
}

// =========================================================================
// 実在 pasta 構文の fixture（crates/pasta_sample_ghost の boot.pasta/talk.pasta 由来）。
// コメント・グローバルシーン・アクター・アクション行（単語参照）・Call を含む。
// =========================================================================
const PASTA_FIXTURE = [
  '＃ boot.pasta - 起動/終了イベント用シーン定義', // 0 comment
  '＠終了挨拶：またね～！、お疲れ様！',              // 1 word def
  '＊OnClose',                                       // 2 global scene
  '　％女の子、男の子',                              // 3 actor
  '　女の子：＠通常　おやすみなさい...',            // 4 action-line + word ref
  '　＞ゴースト終了（３００）',                      // 5 call
].join('\n');

// 実在 pasta + 入れ子 lua（book/src/grammar/block-structure.md の実コードブロック由来）。
// lua 本体は function/local/end・メソッド呼出・= true（boolean）を含む。
const PASTA_WITH_LUA = [
  '＊会話',                                          // 0 global scene
  '```lua',                                          // 1 lua block begin
  'function SCENE.initialize(act)',                  // 2 lua: function/entity/params
  '    local save, var = act:init_scene(SCENE)',     // 3 lua: local/operator/method
  '    save.talked = true',                          // 4 lua: attribute/= true
  'end',                                             // 5 lua: end
  '```',                                             // 6 lua block end
  '    こんにちは',                                  // 7 plain talk
].join('\n');

// 複数行 lua 文字列（[[ ... ]]）が行を跨ぐ fixture。lua 側 ruleStack 継続の検証用。
const PASTA_WITH_MULTILINE_LUA = [
  '＊長文',                                          // 0 global scene
  '```lua',                                          // 1 lua begin
  'local s = [[',                                    // 2 lua: string begin
  'still inside the string',                         // 3 lua: string body (跨ぎ)
  ']]',                                              // 4 lua: string end
  'print(s)',                                        // 5 lua: function call
  '```',                                             // 6 lua end
].join('\n');

// =========================================================================
async function run() {
  log('PastaTokenizer test');
  log('  pasta grammar:', PASTA_GRAMMAR);
  log('  lua   grammar:', LUA_GRAMMAR);
  log('');

  // -----------------------------------------------------------------------
  // 文法不在で例外（6.2）— load() の前に検証（実物ロード前のガード）。
  // -----------------------------------------------------------------------
  {
    const t = createPastaTokenizer();
    let threw = false;
    let msg = '';
    try {
      await t.load({
        pasta: path.resolve(here, 'no-such-pasta.tmLanguage.json'),
        lua: LUA_GRAMMAR,
      });
    } catch (e) {
      threw = true;
      msg = String(e && e.message ? e.message : e);
    }
    check('6.2: pasta 文法ファイル不在で load() が例外/リジェクト', threw, msg);
  }
  {
    const t = createPastaTokenizer();
    let threw = false;
    try {
      await t.load({
        pasta: PASTA_GRAMMAR,
        lua: path.resolve(here, 'grammars/no-such-lua.tmLanguage.json'),
      });
    } catch (e) {
      threw = true;
    }
    check('6.2: lua 文法ファイル不在で load() が例外/リジェクト', threw);
  }
  log('');

  // 以降は正規ロード済みインスタンスを使う。
  const tok = createPastaTokenizer();
  await tok.load({ pasta: PASTA_GRAMMAR, lua: LUA_GRAMMAR });

  // -----------------------------------------------------------------------
  // 基本トークナイズ（1.1）: 行数一致・各行被覆・代表 pasta スコープ。
  // -----------------------------------------------------------------------
  {
    const lines = PASTA_FIXTURE.split('\n');
    const result = tok.tokenizeText(PASTA_FIXTURE);
    check('1.1: 返り配列長＝行数', result.length === lines.length,
      `got=${result.length} expected=${lines.length}`);

    let allCover = true;
    for (let i = 0; i < lines.length; i++) {
      if (!coversLine(result[i], lines[i])) {
        allCover = false;
        log(`        line ${i} not covered: ${JSON.stringify(lines[i])} spans=${JSON.stringify(result[i])}`);
      }
    }
    check('1.1: 各行が TokenSpan で隙間なく被覆される', allCover);

    // 代表 pasta スコープ（末尾最特定で前方一致）。
    check('1.1: コメント行が comment.line.pasta',
      hasLeafScopeForText(result[0], lines[0], 'comment.line.pasta', null));
    check('1.1: 単語定義のマーカーが keyword.other.marker.pasta',
      hasLeafScopeForText(result[1], lines[1], 'keyword.other.marker.pasta', '＠'));
    check('1.1: 単語定義本体が markup.inline.raw.string.pasta',
      hasLeafScopeForText(result[1], lines[1], 'markup.inline.raw.string.pasta', '終了挨拶'));
    check('1.1: グローバルシーン名が keyword.control.scene.pasta',
      hasLeafScopeForText(result[2], lines[2], 'keyword.control.scene.pasta', 'OnClose'));
    check('1.1: アクターが entity.name.class.pasta',
      hasLeafScopeForText(result[3], lines[3], 'entity.name.class.pasta', '女の子'));
    check('1.1: アクション行アクターが entity.name.type.actor.pasta',
      hasLeafScopeForText(result[4], lines[4], 'entity.name.type.actor.pasta', '女の子'));
    check('1.1: アクション行の単語参照が markup.inline.raw.string.pasta',
      hasLeafScopeForText(result[4], lines[4], 'markup.inline.raw.string.pasta', '通常'));
    check('1.1: Call 行が keyword.control.pasta',
      hasLeafScopeForText(result[5], lines[5], 'keyword.control.pasta', 'ゴースト終了'));
  }
  log('');

  // -----------------------------------------------------------------------
  // 入れ子 lua の二段化（核心・1.3）。
  // 単一パス（pasta のみ）では lua 本体は meta.embedded.block.lua.content 止まり。
  // 二段化後は lua スコープが付くことを検証（差を示す）。
  // -----------------------------------------------------------------------
  {
    const lines = PASTA_WITH_LUA.split('\n');
    const result = tok.tokenizeText(PASTA_WITH_LUA);

    check('1.3: 返り配列長＝行数（lua 入れ子込み）', result.length === lines.length,
      `got=${result.length} expected=${lines.length}`);

    let allCover = true;
    for (let i = 0; i < lines.length; i++) {
      if (!coversLine(result[i], lines[i])) {
        allCover = false;
        log(`        line ${i} not covered: ${JSON.stringify(lines[i])} spans=${JSON.stringify(result[i])}`);
      }
    }
    check('1.3: 各行被覆（lua 入れ子込み）', allCover);

    // pasta 側のフェンス・シーンは pasta スコープのまま。
    check('1.3: シーン名は pasta スコープ',
      hasLeafScopeForText(result[0], lines[0], 'keyword.control.scene.pasta', '会話'));
    check('1.3: フェンス言語名は entity.name.type.language.pasta',
      hasLeafScopeForText(result[1], lines[1], 'entity.name.type.language.pasta', 'lua'));

    // 核心: lua 本体行に lua スコープが付く（二段化されている証拠）。
    const luaFnLine = result[2]; // function SCENE.initialize(act)
    const luaLocalLine = result[3]; // local save, var = act:init_scene(SCENE)
    const luaBoolLine = result[4]; // save.talked = true
    const luaEndLine = result[5]; // end

    check('1.3核心: function が keyword.control.lua（lua スコープ）',
      hasLeafScopeForText(luaFnLine, lines[2], 'keyword.control.lua', 'function'));
    check('1.3核心: 関数名が entity.name.function.lua',
      hasAnyScope(luaFnLine, 'entity.name.function.lua'));
    check('1.3核心: local が keyword.local.lua',
      hasLeafScopeForText(luaLocalLine, lines[3], 'keyword.local.lua', 'local'));
    check('1.3核心: true が constant.language.lua',
      hasLeafScopeForText(luaBoolLine, lines[4], 'constant.language.lua', 'true'));
    check('1.3核心: end が keyword.control.lua',
      hasLeafScopeForText(luaEndLine, lines[5], 'keyword.control.lua', 'end'));

    // 差の明示: 二段化後、lua 本体行は meta.embedded.block.lua.content を末尾に持たない
    // （= lua スコープへ差し替わった）。少なくとも 1 つ lua スコープが含まれる。
    const luaScopedSomewhere = [luaFnLine, luaLocalLine, luaBoolLine, luaEndLine]
      .every((spans) => hasAnyScope(spans, 'source.lua') || spans.some((s) => s.scopes.some((sc) => sc.endsWith('.lua'))));
    check('1.3核心: lua 本体の全行に lua スコープが含まれる（二段化の差）', luaScopedSomewhere);

    // 単一パス（pasta のみ）では lua 本体に lua 構文スコープが付かないことを対比で確認。
    const single = tok.tokenizeTextSinglePass
      ? tok.tokenizeTextSinglePass(PASTA_WITH_LUA)
      : null;
    if (single) {
      const singleHasLuaSyntax = single.some((spans) =>
        spans.some((s) => s.scopes.some((sc) => /^keyword\.|^constant\.language|^string\.quoted/.test(sc) && sc.endsWith('.lua'))));
      check('1.3対比: 単一パスでは lua 構文スコープが付かない（content 止まり）',
        !singleHasLuaSyntax);
    } else {
      // 単一パス API が無くても核心検証は二段化側で成立しているため情報ログのみ。
      log('  INFO  tokenizeTextSinglePass 未提供のため単一パス対比はスキップ');
    }
  }
  log('');

  // -----------------------------------------------------------------------
  // 複数行 lua ブロック跨ぎ: lua 側 ruleStack 継続（[[...]] 複数行文字列）。
  // -----------------------------------------------------------------------
  {
    const lines = PASTA_WITH_MULTILINE_LUA.split('\n');
    const result = tok.tokenizeText(PASTA_WITH_MULTILINE_LUA);
    check('複数行: 返り配列長＝行数', result.length === lines.length);

    // 行 2（local s = [[）・行 3（still inside…）・行 4（]]）が string.quoted.*.lua を持つ。
    const strBegin = hasAnyScope(result[2], 'string.quoted');
    const strBody = result[3].every((s) =>
      s.scopes.some((sc) => sc.startsWith('string.quoted') && sc.endsWith('.lua')));
    const strEnd = hasAnyScope(result[4], 'string.quoted');
    check('複数行: lua 文字列開始行に string.quoted.*.lua', strBegin);
    check('複数行: 跨ぎ行(本体)全体が string.quoted.*.lua（ruleStack 継続）', strBody,
      `spans=${JSON.stringify(result[3])}`);
    check('複数行: lua 文字列終了行に string.quoted.*.lua', strEnd);
  }
  log('');

  // -----------------------------------------------------------------------
  // オフセット正規化: lua 区間トークンの startIndex/endIndex が元行内の
  // 正しい位置を指す（行テキストから切り出して妥当性確認）。
  // -----------------------------------------------------------------------
  {
    const lines = PASTA_WITH_LUA.split('\n');
    const result = tok.tokenizeText(PASTA_WITH_LUA);

    // 全 lua 本体行で span が行境界 [0, line.length] に収まり、切り出しが原文と一致。
    let offsetsOk = true;
    for (const i of [2, 3, 4, 5]) {
      const spans = result[i];
      const line = lines[i];
      // 連結したテキストが原行に一致＝オフセットが正しい。
      let rebuilt = '';
      for (const s of spans) {
        if (s.startIndex < 0 || s.endIndex > line.length || s.startIndex > s.endIndex) {
          offsetsOk = false;
          log(`        line ${i} span out of bounds: ${JSON.stringify(s)} len=${line.length}`);
        }
        rebuilt += line.slice(s.startIndex, s.endIndex);
      }
      if (rebuilt !== line) {
        offsetsOk = false;
        log(`        line ${i} rebuild mismatch: ${JSON.stringify(rebuilt)} != ${JSON.stringify(line)}`);
      }
    }
    check('オフセット正規化: lua 区間 span が元行内に収まり原文を再構成できる', offsetsOk);

    // 具体: "local" が行 3 の正しい部分文字列を指す。
    const localSpan = result[3].find((s) =>
      (s.scopes[s.scopes.length - 1] || '').startsWith('keyword.local.lua'));
    check('オフセット正規化: local トークンの切り出しが "local"',
      localSpan && lines[3].slice(localSpan.startIndex, localSpan.endIndex) === 'local',
      localSpan ? lines[3].slice(localSpan.startIndex, localSpan.endIndex) : 'not found');
  }
  log('');

  // -----------------------------------------------------------------------
  // 決定論（6.4 を支える）: 同一 text を 2 回 → 同一構造。
  // -----------------------------------------------------------------------
  {
    const a = tok.tokenizeText(PASTA_WITH_LUA);
    const b = tok.tokenizeText(PASTA_WITH_LUA);
    check('決定論: 同一 text を 2 回トークナイズして同一結果',
      JSON.stringify(a) === JSON.stringify(b));
  }
  log('');

  // -----------------------------------------------------------------------
  // load 前呼び出しガード: tokenizeText / tokenizeTextSinglePass は load 前に例外。
  // -----------------------------------------------------------------------
  {
    const fresh = createPastaTokenizer();
    let threw1 = false;
    try {
      fresh.tokenizeText('＊シーン');
    } catch (e) {
      threw1 = true;
    }
    check('ガード: load 前の tokenizeText が例外', threw1);
    let threw2 = false;
    try {
      fresh.tokenizeTextSinglePass('＊シーン');
    } catch (e) {
      threw2 = true;
    }
    check('ガード: load 前の tokenizeTextSinglePass が例外', threw2);
  }
  log('');

  // -----------------------------------------------------------------------
  // load 引数バリデーション: paths 欠落・非文字列パスで例外/リジェクト（6.2）。
  // -----------------------------------------------------------------------
  {
    const cases = [
      ['引数なし', undefined],
      ['空オブジェクト', {}],
      ['pasta が非文字列', { pasta: 123, lua: LUA_GRAMMAR }],
      ['lua が欠落', { pasta: PASTA_GRAMMAR }],
    ];
    for (const [desc, arg] of cases) {
      const t = createPastaTokenizer();
      let threw = false;
      try {
        await t.load(arg);
      } catch (e) {
        threw = true;
      }
      check(`6.2: load(${desc}) が例外/リジェクト`, threw);
    }
  }
  log('');

  // -----------------------------------------------------------------------
  // scopeName 欠落文法で例外（6.2）: JSON としては妥当だが TextMate 文法でない
  // ファイルを一時ディレクトリに置いて load がリジェクトされることを確認。
  // -----------------------------------------------------------------------
  {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'tok-test-'));
    const badGrammar = path.join(tmp, 'bad.tmLanguage.json');
    fs.writeFileSync(badGrammar, JSON.stringify({ patterns: [] }), 'utf8');
    const t = createPastaTokenizer();
    let threw = false;
    let msg = '';
    try {
      await t.load({ pasta: badGrammar, lua: LUA_GRAMMAR });
    } catch (e) {
      threw = true;
      msg = String(e && e.message ? e.message : e);
    }
    check('6.2: scopeName 欠落の文法 JSON で load() が例外/リジェクト',
      threw && msg.includes('Invalid TextMate grammar'), msg);
    fs.rmSync(tmp, { recursive: true, force: true });
  }
  log('');

  // -----------------------------------------------------------------------
  // 境界入力: 空テキスト・空行混在でも行数一致＋被覆（Post 条件の境界）。
  // -----------------------------------------------------------------------
  {
    const empty = tok.tokenizeText('');
    check('境界: 空テキスト("") → 1 行・空 span 配列',
      empty.length === 1 && Array.isArray(empty[0]) && empty[0].length === 0,
      JSON.stringify(empty));

    const text = '＊シーン\n\n　こんにちは';
    const lines = text.split('\n');
    const result = tok.tokenizeText(text);
    check('境界: 空行混在でも行数一致', result.length === lines.length,
      `got=${result.length}`);
    check('境界: 空行は空 span 配列（被覆と整合）', result[1].length === 0,
      JSON.stringify(result[1]));
    let allCover = true;
    for (let i = 0; i < lines.length; i++) {
      if (!coversLine(result[i], lines[i])) allCover = false;
    }
    check('境界: 空行混在でも各行被覆', allCover);
  }
  log('');

  // -----------------------------------------------------------------------
  // 複数 lua ブロック分離: pasta 行を挟む 2 つの lua ブロックがそれぞれ独立に
  // 二段化される（ブロック間 flush と lua ruleStack の分離）。
  // -----------------------------------------------------------------------
  {
    const text = [
      '＊二つ',            // 0 pasta
      '```lua',            // 1 fence
      'local a = 1',       // 2 lua block 1
      '```',               // 3 fence
      '　間のトーク',      // 4 pasta（ブロック分離の楔）
      '```lua',            // 5 fence
      'local b = [[',      // 6 lua block 2: 複数行文字列開始
      ']]',                // 7 lua block 2: 文字列終了
      '```',               // 8 fence
    ].join('\n');
    const lines = text.split('\n');
    const result = tok.tokenizeText(text);
    check('複数ブロック: 行数一致', result.length === lines.length);
    check('複数ブロック: 1 つ目のブロックに lua スコープ（local）',
      hasLeafScopeForText(result[2], lines[2], 'keyword.local.lua', 'local'));
    check('複数ブロック: 2 つ目のブロックにも lua スコープ（local）',
      hasLeafScopeForText(result[6], lines[6], 'keyword.local.lua', 'local'));
    check('複数ブロック: 間の pasta 行に lua スコープが漏れない',
      !hasAnyScope(result[4], 'source.lua') &&
        !result[4].some((s) => s.scopes.some((sc) => sc.endsWith('.lua'))),
      JSON.stringify(result[4]));
    // ブロック 2 内の複数行文字列が継続している（ブロック単位の ruleStack 引き回し）。
    check('複数ブロック: 2 つ目のブロック内で複数行文字列が継続',
      hasAnyScope(result[7], 'string.quoted'),
      JSON.stringify(result[7]));
  }
  log('');

  // -----------------------------------------------------------------------
  // 未終端 lua ブロック: 閉じフェンス無しで EOF に達しても例外を出さず、
  // 末尾までの lua 本体行が二段化される（末尾 flush 経路）。
  // -----------------------------------------------------------------------
  {
    const text = [
      '＊未終端',
      '```lua',
      'local x = 1',
      'return x', // EOF（閉じ ``` なし）
    ].join('\n');
    const lines = text.split('\n');
    let result = null;
    let threw = false;
    try {
      result = tok.tokenizeText(text);
    } catch (e) {
      threw = true;
    }
    check('未終端: 例外を出さず行数一致', !threw && result && result.length === lines.length);
    if (result) {
      check('未終端: EOF 直前の lua 行も二段化される（return が keyword.control.lua）',
        hasLeafScopeForText(result[3], lines[3], 'keyword.control.lua', 'return'),
        JSON.stringify(result[3]));
    }
  }
  log('');

  log(`RESULT: ${passed} passed, ${failed} failed`);
  // process.exit() を即時に呼ぶと oniguruma WASM の async ハンドル close 中に
  // libuv assertion で落ちる環境（Node 25 系）がある。exitCode を設定し、
  // イベントループを自然に drain させて確実な終了コードを返す。
  process.exitCode = failed === 0 ? 0 : 1;
}

run().catch((e) => {
  // 想定外の例外（load 後の処理中）はテスト失敗として扱う。
  console.error('UNEXPECTED ERROR:', e && e.stack ? e.stack : e);
  process.exitCode = 1;
});
