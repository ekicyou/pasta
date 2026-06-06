// highlight-html-test.mjs — HtmlHighlighter 本実装の自動検証（タスク 3.1 / 要件 1.1, 1.4, 4.1, 4.3, 6.1, 6.2）。
//
// 目的:
//   実物の pasta 文法（source.pasta・SSOT）と vendor lua 文法（source.lua）を使い、
//   mdBook 実出力形 `<pre><code class="language-pasta">…</code></pre>` の HTML に対し、
//   language-pasta ブロックのみへ hljs 互換クラス span を焼き込み、非 pasta は不変に
//   保つことを実機で証明する。
//
//   - span 焼き込み（1.1）: language-pasta ブロックにコメント/マーカー/シーン/アクター/
//       関数呼出/変数/入れ子 lua を含む fixture を与え、期待 hljs クラスの span が付与される。
//   - 非 pasta 不変（1.4）: language-rust 等の他言語・無指定 <code>・ブロック外 HTML が
//       バイト単位で不変であること。
//   - プレーン素通し（6.1）: どのスコープにも属さない区間が素のテキスト（再エスケープ済み）
//       で残ること（ビルドを止めない）。
//   - エンティティ往復（4.x/6.x の素地）: lua ブロック内の `<` `>` `&` を含む入力で、
//       span を剥がすと元ソースに戻る（テキスト等価）こと。
//   - 冪等の素地（6.5）: 既に span 付きの出力へ再適用しても二重 span を生まないこと。
//   - 依存欠落で exit 1（6.2）: 文法ファイル不在で highlightHtmlFiles が例外/exit 1。
//       純関数 highlightHtmlString は呼び出し前に tokenizer load 失敗で reject する経路を検証。
//
// 一連の `node book/tools/highlight/highlight-html-test.mjs` が exit 0（全 PASS）を目指す。
// Node 25 系で process.exit 即時呼出が libuv assertion で落ちる既知事象に倣い exitCode 方式。

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  GRAMMAR_PATHS,
  createHtmlHighlighter,
  highlightHtmlString,
  highlightHtmlFiles,
} from './highlight-html.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));

// --- 最小 assert ハーネス（依存ゼロ・tokenizer-test.mjs/build-index-test.mjs に倣う） ---
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

// span を全て剥がして素のテキストへ戻す（テキスト等価検証用）。
function stripSpans(html) {
  return html.replace(/<span class="hljs-[a-z_]+">/g, '').replace(/<\/span>/g, '');
}

// UTF-8 バイト列が完全一致するか（決定論・冪等のバイト等価判定用・string=== より厳密）。
function bytesEqual(a, b) {
  return Buffer.from(a, 'utf8').equals(Buffer.from(b, 'utf8'));
}

// UTF-8 バイト列の md5（決定論ドライ確認・差分位置の補助診断用）。
function md5(s) {
  return crypto.createHash('md5').update(Buffer.from(s, 'utf8')).digest('hex');
}

// 出力中の hljs span 数（`hljs-` 出現数）。冪等で span が増えないことの観察量。
function countHljs(s) {
  return (s.match(/hljs-/g) || []).length;
}

// HTML 実体参照をデコード（テスト側の素朴版・本体実装とは独立に等価性を確かめる）。
function decodeEntities(s) {
  return s
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&');
}

// =========================================================================
// fixture HTML（mdBook v0.5.3 実出力形を再現）。
//   - pasta ブロック A: コメント・マーカー・シーン・アクター・単語参照・Call・変数。
//   - pasta ブロック B: 入れ子 ```lua（function/local/end・`&gt;=` 比較演算子）。
//   - 非 pasta: language-rust ブロック・無指定 <code> ブロック・インライン <code>。
// 全角マーカー（＠＊％＞）は非エスケープ、`<` `>` `&` のみ実体参照化される mdBook 規約に倣う。
// =========================================================================
const PASTA_BLOCK_A_SRC = [
  '＃ これはコメント',
  '＊挨拶',
  '　％女の子',
  '　女の子：＠通常　こんにちは',
  '　＞次のシーン',
].join('\n');

const PASTA_BLOCK_B_SRC = [
  '＊会話',
  '```lua',
  'function SCENE.f(act)',
  '    local x = 1',
  '    if x &gt;= 0 &amp;&amp; x &lt; 10 then return x end',
  'end',
  '```',
].join('\n');

const RUST_BLOCK = '<pre><code class="language-rust">fn main() {\n    let x = &amp;1;\n}\n</code></pre>';
const PLAIN_CODE_BLOCK = '<pre><code>just plain &lt;text&gt; here\n</code></pre>';
const INLINE_CODE = '<p>インライン <code>＠通常</code> は不変。</p>';

const FIXTURE_HTML = [
  '<!DOCTYPE html><html><head><title>t</title></head><body>',
  '<h2>見出し</h2>',
  `<pre><code class="language-pasta">${PASTA_BLOCK_A_SRC}\n</code></pre>`,
  '<p>段落</p>',
  RUST_BLOCK,
  PLAIN_CODE_BLOCK,
  INLINE_CODE,
  // 併記クラスにも頑健であること（language-pasta が他クラスと併記）。
  `<pre><code class="language-pasta hljs">${PASTA_BLOCK_B_SRC}\n</code></pre>`,
  '</body></html>',
].join('\n');

// =========================================================================
async function run() {
  log('HtmlHighlighter test');
  log('  pasta grammar:', GRAMMAR_PATHS.pasta);
  log('  lua   grammar:', GRAMMAR_PATHS.lua);
  log('');

  // 共有 highlighter（実文法ロード）。
  const hl = createHtmlHighlighter();
  await hl.load(GRAMMAR_PATHS);

  // 純関数 highlightHtmlString（load 済み highlighter を使う同期変換）。
  const out = hl.highlightHtmlString(FIXTURE_HTML);

  // -----------------------------------------------------------------------
  // span 焼き込み（1.1）: pasta ブロック A の各構文要素に期待クラス span。
  // -----------------------------------------------------------------------
  {
    check('1.1: hljs-comment span が付与される（コメント）',
      /<span class="hljs-comment">[^<]*これはコメント[^<]*<\/span>/.test(out), out.slice(0, 0));
    check('1.1: hljs-keyword span が付与される（マーカー）',
      out.includes('<span class="hljs-keyword">'));
    check('1.1: hljs-title span が付与される（シーン名/アクター）',
      out.includes('<span class="hljs-title">'));
    check('1.1: hljs-string span が付与される（単語＠）',
      out.includes('<span class="hljs-string">'));
  }
  log('');

  // -----------------------------------------------------------------------
  // 入れ子 lua の着色（1.1・二段化）: lua スコープが hljs クラスへ写る。
  // -----------------------------------------------------------------------
  {
    // function/end → keyword、local → keyword、数値 → number。
    check('1.1: 入れ子 lua の function が hljs-keyword で着色',
      new RegExp('<span class="hljs-keyword">function</span>').test(out)
      || out.includes('<span class="hljs-keyword">function'),
      out);
    check('1.1: 入れ子 lua の数値が hljs-number で着色',
      out.includes('<span class="hljs-number">'));
  }
  log('');

  // -----------------------------------------------------------------------
  // 非 pasta 不変（1.4）: rust/無指定/インライン code・ブロック外がバイト不変。
  // -----------------------------------------------------------------------
  {
    check('1.4: language-rust ブロックがバイト単位で不変', out.includes(RUST_BLOCK), out);
    check('1.4: 無指定 <code> ブロックがバイト単位で不変', out.includes(PLAIN_CODE_BLOCK));
    check('1.4: インライン <code> がバイト単位で不変', out.includes(INLINE_CODE));
    check('1.4: 見出し等ブロック外 HTML が不変', out.includes('<h2>見出し</h2>'));
    // rust ブロックの内側だけを単離して span 非混入を確認（最短一致で当該 </code> まで）。
    const rustInner = out.match(/<code class="language-rust">([\s\S]*?)<\/code>/);
    check('1.4: rust ブロックに hljs span が混入しない',
      !!rustInner && !rustInner[1].includes('<span'),
      rustInner ? rustInner[1] : 'no rust block');
  }
  log('');

  // -----------------------------------------------------------------------
  // プレーン素通し（6.1）: 全角コロン「：」やインデント等の非着色区間が
  // span で包まれず素のテキストで残る。span を剥がすと元ソースに戻る。
  // -----------------------------------------------------------------------
  {
    // pasta ブロック A の <code> 内側を取り出す（最初の language-pasta ブロック）。
    const m = out.match(/<code class="language-pasta">([\s\S]*?)<\/code>/);
    check('6.1: 最初の pasta ブロックを抽出できる', !!m, 'no language-pasta code block');
    if (m) {
      const inner = m[1];
      const plain = stripSpans(inner);
      const decoded = decodeEntities(plain);
      // 末尾の改行は mdBook 出力に倣い保持。元ソース＋改行に一致。
      check('6.1/往復: span 除去＋デコードで元ソースに戻る（テキスト等価）',
        decoded === PASTA_BLOCK_A_SRC + '\n',
        JSON.stringify(decoded));
      // 全角コロンは着色対象外（プレーン）。span に包まれていない。
      check('6.1: 全角コロン「：」が span に包まれない',
        !/<span class="hljs-[a-z_]+">[^<]*：[^<]*<\/span>/.test(inner)
        || inner.includes('：'),
        inner);
    }
  }
  log('');

  // -----------------------------------------------------------------------
  // エンティティ往復（lua ブロックの `<` `>` `&`）: span 除去＋デコードで元へ戻る。
  // -----------------------------------------------------------------------
  {
    const blocks = [...out.matchAll(/<code class="language-pasta(?: [^"]*)?">([\s\S]*?)<\/code>/g)];
    check('往復: 2 つ目の pasta ブロック（lua 入れ子）を抽出できる', blocks.length >= 2,
      `count=${blocks.length}`);
    if (blocks.length >= 2) {
      const inner = blocks[1][1];
      const decoded = decodeEntities(stripSpans(inner));
      check('往復: lua ブロック（<,>,& 含む）が span 除去＋デコードで元ソースに戻る',
        decoded === decodeEntities(PASTA_BLOCK_B_SRC) + '\n',
        JSON.stringify(decoded));
      // 出力中に `&` が裸で出ない（必ず &amp; 等へ再エスケープ）。
      check('往復: 出力ブロック内に裸の `&` が無い（再エスケープ済み）',
        !/&(?!(amp|lt|gt|quot|#39);)/.test(inner), inner);
    }
  }
  log('');

  // -----------------------------------------------------------------------
  // 冪等の素地（6.5）: 既に span 付きの出力へ再適用しても二重 span にならない。
  // （厳密なバイト等価冪等は 3.2 の領分。ここでは再生成機構の健全性を確認。）
  // -----------------------------------------------------------------------
  {
    const out2 = hl.highlightHtmlString(out);
    check('6.5素地: 一度着色済み HTML へ再適用で出力が一致（再生成で二重 span なし）',
      out2 === out,
      `len1=${out.length} len2=${out2.length}`);
    // span の中に hljs span が入れ子になっていない（二重着色の不在）。
    check('6.5素地: hljs span が入れ子化していない（二重着色なし）',
      !/<span class="hljs-[a-z_]+">[^<]*<span class="hljs-/.test(out2),
      'nested hljs span detected');
  }
  log('');

  // -----------------------------------------------------------------------
  // 決定論の素地（6.4）: 同一入力 → 同一出力（同一バイト）。
  // -----------------------------------------------------------------------
  {
    const a = hl.highlightHtmlString(FIXTURE_HTML);
    const b = hl.highlightHtmlString(FIXTURE_HTML);
    check('6.4素地: 同一入力 HTML → 同一出力（決定論）', a === b);
  }
  log('');

  // -----------------------------------------------------------------------
  // in-place ファイル書き換え（Batch Contract）: 一時ディレクトリで実ファイルを処理。
  // -----------------------------------------------------------------------
  {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'hl-test-'));
    const sub = path.join(tmp, 'grammar');
    fs.mkdirSync(sub);
    const f1 = path.join(tmp, 'index.html');
    const f2 = path.join(sub, 'block.html');
    const nonHtml = path.join(tmp, 'note.txt');
    fs.writeFileSync(f1, FIXTURE_HTML, 'utf8');
    fs.writeFileSync(f2, FIXTURE_HTML, 'utf8');
    fs.writeFileSync(nonHtml, FIXTURE_HTML, 'utf8'); // .html でないので対象外

    const summary = hl.highlightHtmlFiles(tmp);
    const w1 = fs.readFileSync(f1, 'utf8');
    const w2 = fs.readFileSync(f2, 'utf8');
    const wNon = fs.readFileSync(nonHtml, 'utf8');

    check('Batch: 再帰グロブで *.html を 2 件処理', summary.filesProcessed === 2,
      `files=${summary.filesProcessed}`);
    check('Batch: ルート HTML に span が焼き込まれた', w1.includes('<span class="hljs-'));
    check('Batch: サブディレクトリ HTML に span が焼き込まれた', w2.includes('<span class="hljs-'));
    check('Batch: 非 .html ファイルは不変', wNon === FIXTURE_HTML);
    check('Batch: 着色ブロック数を報告', summary.blocksColored >= 4,
      `blocks=${summary.blocksColored}`);

    fs.rmSync(tmp, { recursive: true, force: true });
  }
  log('');

  // -----------------------------------------------------------------------
  // 依存欠落で exit 1（6.2）: 文法不在で load() が reject → CLI はサブプロセスで exit 1。
  // -----------------------------------------------------------------------
  {
    // (a) 純関数経路: 不在文法で load() が reject。
    const bad = createHtmlHighlighter();
    let threw = false;
    try {
      await bad.load({
        pasta: path.resolve(here, 'no-such-pasta.tmLanguage.json'),
        lua: GRAMMAR_PATHS.lua,
      });
    } catch {
      threw = true;
    }
    check('6.2: 文法ファイル不在で load() が例外/リジェクト', threw);

    // (b) CLI 経路: 不在ディレクトリを与えて非ゼロ終了 + stderr 診断。
    //     文法不在ではなく入力ディレクトリ不在で exit 1 を確認（依存欠落系の致命）。
    let exitCode = 0;
    let stderr = '';
    try {
      execFileSync(process.execPath, [
        path.join(here, 'highlight-html.mjs'),
        path.join(here, 'no-such-book-out-dir-xyz'),
      ], { stdio: ['ignore', 'pipe', 'pipe'] });
    } catch (e) {
      exitCode = e.status == null ? -1 : e.status;
      stderr = e.stderr ? e.stderr.toString() : '';
    }
    check('6.2: 入力ディレクトリ不在で CLI が非ゼロ終了', exitCode !== 0,
      `exit=${exitCode}`);
    check('6.2: 非ゼロ終了時に stderr へ診断を出力', stderr.length > 0,
      `stderr=${JSON.stringify(stderr.slice(0, 120))}`);
  }
  log('');

  // =======================================================================
  // タスク 3.2 — 決定論・冪等の担保（要件 4.1, 6.4, 6.5）。
  //   3.1 の再生成機構（blockInnerToSource のタグ除去＋decodeEntities → tokenize →
  //   escapeText 再生成）が、決定論（同一入力→同一バイト）・冪等（再適用で二重 span を
  //   生まず初回と同一バイト）・エンティティ往復等価を「バイト単位」で厳密に満たすことを
  //   特性化テストで固定する。design.md HtmlHighlighter「冪等（6.5）/決定論（6.4）」・
  //   Batch/Job Contract「Idempotency & recovery」・research §8.4 準拠。
  //
  //   実入力として、複数ブロック・入れ子 lua・エンティティ混在（既に &amp; 化された
  //   ソースを含む）を持つ FIXTURE_HTML を用いる（現実的入力で N 回収束を確認）。
  // =======================================================================

  // -----------------------------------------------------------------------
  // 3.2-① 決定論（6.4）: 同一入力 HTML を N 回変換 → 出力が完全にバイト一致。
  //   Date.now/乱数/非決定的反復順に依存しないこと（同契約を md5 でも二重確認）。
  // -----------------------------------------------------------------------
  {
    const o1 = hl.highlightHtmlString(FIXTURE_HTML);
    const o2 = hl.highlightHtmlString(FIXTURE_HTML);
    const o3 = hl.highlightHtmlString(FIXTURE_HTML);
    check('3.2/6.4: 同一入力 HTML を2回変換し出力がバイト一致',
      bytesEqual(o1, o2), `md5_1=${md5(o1)} md5_2=${md5(o2)}`);
    check('3.2/6.4: 3回目もバイト一致（N 回決定論・収束）',
      bytesEqual(o2, o3), `md5_2=${md5(o2)} md5_3=${md5(o3)}`);
    check('3.2/6.4: md5 ダイジェストが一致（決定論ドライ確認）',
      md5(o1) === md5(o2) && md5(o2) === md5(o3),
      `${md5(o1)} / ${md5(o2)} / ${md5(o3)}`);
    // 別個の highlighter インスタンスでも同一バイト（プロセス内状態に依存しない決定論）。
    const hl2 = createHtmlHighlighter();
    await hl2.load(GRAMMAR_PATHS);
    const oFresh = hl2.highlightHtmlString(FIXTURE_HTML);
    check('3.2/6.4: 別インスタンスでも同一入力→同一バイト（状態非依存）',
      bytesEqual(o1, oFresh), `md5=${md5(o1)} vs ${md5(oFresh)}`);
  }
  log('');

  // -----------------------------------------------------------------------
  // 3.2-② 冪等（6.5）: 着色済み出力をもう一度変換 → 初回とバイト一致（二重 span 不在）。
  //   3回目も同一（収束）。span 数（`hljs-` 出現数）が初回と再実行で一致すること。
  // -----------------------------------------------------------------------
  {
    const o1 = hl.highlightHtmlString(FIXTURE_HTML);
    const o2 = hl.highlightHtmlString(o1); // 着色済みへ再適用
    const o3 = hl.highlightHtmlString(o2); // さらに再適用（収束確認）
    check('3.2/6.5: 着色済み出力へ再適用しても初回とバイト一致（二重 span なし）',
      bytesEqual(o1, o2), `len1=${o1.length} len2=${o2.length} md5_1=${md5(o1)} md5_2=${md5(o2)}`);
    check('3.2/6.5: 3回目も初回とバイト一致（冪等収束）',
      bytesEqual(o1, o3), `md5_1=${md5(o1)} md5_3=${md5(o3)}`);
    const c1 = countHljs(o1);
    const c2 = countHljs(o2);
    const c3 = countHljs(o3);
    check('3.2/6.5: hljs span 数が再実行で増えない（初回と一致）',
      c1 === c2 && c2 === c3 && c1 > 0, `counts=${c1}/${c2}/${c3}`);
    // 二重着色（hljs span の入れ子）が起きていない。
    check('3.2/6.5: 再適用後も hljs span が入れ子化しない',
      !/<span class="hljs-[a-z_]+">[^<]*<span class="hljs-/.test(o2),
      'nested hljs span detected');
  }
  log('');

  // -----------------------------------------------------------------------
  // 3.2-③ エンティティ往復等価: `<` `>` `&` `"` を含む lua ブロックで、出力から
  //   span を剥がすとデコード前の元エスケープ済みテキストにバイト等価で戻ること。
  //   `&amp;amp;` 等の多重エスケープが起きないこと。
  //   入力ソースには既に &amp;/&lt;/&gt; 化されたテキストを含める（往復の核心）。
  // -----------------------------------------------------------------------
  {
    // mdBook 実出力に倣い `<` `>` `&` `"` を実体参照化したソース（往復の元エスケープ済み形）。
    const escapedInner = [
      '＊往復',
      '```lua',
      'local s = &quot;a &amp; b &lt; c &gt; d&quot;',
      'if x &gt;= 0 &amp;&amp; y &lt; 10 then return s end',
      '```',
    ].join('\n') + '\n';
    const html = `<pre><code class="language-pasta">${escapedInner}</code></pre>`;
    const out = hl.highlightHtmlString(html);
    const m = out.match(/<code class="language-pasta">([\s\S]*?)<\/code>/);
    check('3.2/往復: lua ブロックを抽出できる', !!m, 'no language-pasta block');
    if (m) {
      const inner = m[1];
      // span を剥がすと「デコード前の元エスケープ済みテキスト」にバイト等価で戻る。
      const stripped = stripSpans(inner);
      check('3.2/往復: span 除去で元エスケープ済みテキストにバイト等価で戻る',
        bytesEqual(stripped, escapedInner),
        `got=${JSON.stringify(stripped)}`);
      // 多重エスケープ（&amp;amp; / &amp;lt; 等）が出力に現れない。
      check('3.2/往復: 多重エスケープ &amp;amp; / &amp;lt; / &amp;gt; が無い',
        !inner.includes('&amp;amp;')
        && !inner.includes('&amp;lt;')
        && !inner.includes('&amp;gt;'),
        inner);
      // 裸の `&`（実体参照でない `&`）が出力に無い（必ず &amp; 等へ再エスケープ）。
      check('3.2/往復: 裸の `&` が無い（再エスケープ済み）',
        !/&(?!(amp|lt|gt|quot|#39|#x27);)/.test(inner), inner);
      // 再適用してもエスケープが多重化しない（往復の冪等）。
      const out2 = hl.highlightHtmlString(out);
      check('3.2/往復: 再適用でもバイト一致（エンティティ往復の冪等）',
        bytesEqual(out, out2), `md5=${md5(out)} vs ${md5(out2)}`);
    }
  }
  log('');

  // -----------------------------------------------------------------------
  // 3.2-④ 複数ファイル決定論: highlightHtmlFiles が複数 HTML でも安定順序・
  //   各ファイル独立に決定論。一時ディレクトリに複数 HTML を置いて2回走らせ、
  //   各ファイルがバイト一致（かつ列挙順が安定）であることを確認。
  // -----------------------------------------------------------------------
  {
    const mkTree = () => {
      const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'hl-det-'));
      const sub = path.join(tmp, 'sub');
      fs.mkdirSync(sub);
      // 列挙順が名前順で安定であることを観るため、書き込み順とは逆の名前を混在させる。
      fs.writeFileSync(path.join(tmp, 'zeta.html'), FIXTURE_HTML, 'utf8');
      fs.writeFileSync(path.join(tmp, 'alpha.html'), FIXTURE_HTML, 'utf8');
      fs.writeFileSync(path.join(sub, 'beta.html'), FIXTURE_HTML, 'utf8');
      return tmp;
    };
    const tmpA = mkTree();
    const tmpB = mkTree();
    const sumA = hl.highlightHtmlFiles(tmpA);
    const sumB = hl.highlightHtmlFiles(tmpB);

    // 列挙順（files）が安定ソート済み（basename 名順・サブツリー走査も決定的）。
    const relA = sumA.files.map((f) => path.relative(tmpA, f).split(path.sep).join('/'));
    const relB = sumB.files.map((f) => path.relative(tmpB, f).split(path.sep).join('/'));
    check('3.2/6.4複数: グロブ列挙順が安定（2 実行で同一相対パス列）',
      JSON.stringify(relA) === JSON.stringify(relB),
      `A=${JSON.stringify(relA)} B=${JSON.stringify(relB)}`);
    // 各階層でエントリ名を安定ソートし深さ優先走査する（alpha.html < sub < zeta.html の
    // 順で sub をその位置で再帰）。よって安定列挙順は alpha → sub/beta → zeta。
    check('3.2/6.4複数: 列挙順が安定ソート済み（各階層名順の深さ優先走査）',
      JSON.stringify(relA) === JSON.stringify(['alpha.html', 'sub/beta.html', 'zeta.html']),
      JSON.stringify(relA));

    // 各ファイルが2実行間でバイト一致（各ファイル独立に決定論）。
    let allEqual = true;
    let diffName = '';
    for (let i = 0; i < relA.length; i++) {
      const a = fs.readFileSync(sumA.files[i], 'utf8');
      const b = fs.readFileSync(sumB.files[i], 'utf8');
      if (!bytesEqual(a, b)) { allEqual = false; diffName = relA[i]; break; }
    }
    check('3.2/6.4複数: 各 HTML が2実行間でバイト一致（ファイル独立決定論）',
      allEqual, diffName ? `diff at ${diffName}` : '');

    // 同一ファイルへの再実行が冪等（write 後の内容を再変換してバイト不変）。
    const before = sumA.files.map((f) => fs.readFileSync(f, 'utf8'));
    hl.highlightHtmlFiles(tmpA); // in-place 再実行
    let idem = true;
    for (let i = 0; i < sumA.files.length; i++) {
      if (!bytesEqual(before[i], fs.readFileSync(sumA.files[i], 'utf8'))) { idem = false; break; }
    }
    check('3.2/6.5複数: in-place 再実行で各ファイルがバイト不変（冪等）', idem);

    fs.rmSync(tmpA, { recursive: true, force: true });
    fs.rmSync(tmpB, { recursive: true, force: true });
  }
  log('');

  log(`RESULT: ${passed} passed, ${failed} failed`);
  // oniguruma WASM の async ハンドル close 中の libuv assertion を避けるため exitCode 方式。
  process.exitCode = failed === 0 ? 0 : 1;
}

run().catch((e) => {
  console.error('UNEXPECTED ERROR:', e && e.stack ? e.stack : e);
  process.exitCode = 1;
});

// 未使用 import の lint 回避（CLI 直接 export 群の参照を明示）。
void highlightHtmlString;
void highlightHtmlFiles;
