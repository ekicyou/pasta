// highlight-html.mjs — HtmlHighlighter。mdbook build の後段で `<book-out-dir>/**/*.html` の
// `<code class="language-pasta">…</code>` ブロックだけを着色する。
//
//   - 抽出は `<code>` のクラス基準で、外側の `<pre>` は要求しない（インライン `<code class="language-pasta">` も着色対象）。
//   - ブロック内容はタグ除去＋実体参照デコードでソースへ戻し、行分割 → PastaTokenizer → scopesToClass で hljs 互換クラスへ写し、
//     非 null はその <span> で包み、null は素のテキストのまま `& < > "` を再エスケープして出す。
//   - 他言語・無指定 <code>・インライン <code>・ブロック外 HTML はバイト単位で不変。
//   - 純静的出力のみ。成果物に WASM・文法・node_modules を出さない。
//   - 文法ロード/トークナイズ例外/必須 build-time 依存欠落は process.exit(1) ＋原因診断を stderr。
//     部分書き込みを避けるためファイル単位で全変換後に書き込む。
//   - 再実行は既存 hljs <span> をタグ除去したプレーンテキストから再生成するので、二重 span を生まない。
//   - 決定論的: 同一入力 HTML → 同一トークン列 → 同一バイト出力。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createPastaTokenizer } from './tokenizer.mjs';
import { scopesToClass } from './scope-map.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const bookDir = path.resolve(here, '../..'); // book/
const repoRoot = path.resolve(bookDir, '..'); // リポジトリルート

// 文法パス（スクリプト位置基準）。pasta は SSOT（読み取り専用）、lua は入れ子 lua の二段化用の vendor。
export const GRAMMAR_PATHS = Object.freeze({
  pasta: path.resolve(repoRoot, 'editors/vscode/syntaxes/pasta.tmLanguage.json'),
  lua: path.resolve(here, 'grammars/lua.tmLanguage.json'),
});

// language-pasta コードブロックを抽出する正規表現。外側 `<pre>` は不問（インライン <code> でも同クラスなら対象）。
// class 属性内で他クラスと併記されても拾う（\b 区切り）。`</code>` はブロック内で実体参照化される前提で、最短一致で区切る。
// 捕捉群: [1]=開始 <code ...> タグ全体, [2]=ブロック本文。
const PASTA_BLOCK_RE =
  /(<code\b[^>]*\bclass="[^"]*\blanguage-pasta\b[^"]*"[^>]*>)([\s\S]*?)(<\/code>)/g;

// HTML 実体参照のデコード（mdBook がコードブロックで施すエスケープの逆変換）。
//   mdBook は `&` `<` `>` を必ず、文脈により `"` `'` を実体参照化する。
//   全角マーカー（＠＊％ 等）は非エスケープのまま。
function decodeEntities(s) {
  return s
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&#x27;/g, "'")
    .replace(/&amp;/g, '&'); // &amp; は最後（多重デコード防止）
}

// プレーンテキストを HTML テキストノード用に再エスケープする。`&` を最初に処理する（二重エスケープを避ける）。`"` も属性混入を防ぐため。
function escapeText(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ブロック本文（<code>…</code> の内側 HTML）からソース文字列を復元する（タグ除去 → 実体参照デコード）。
function blockInnerToSource(innerHtml) {
  const noTags = innerHtml.replace(/<[^>]*>/g, '');
  return decodeEntities(noTags);
}

// トークナイズ済みの 1 行（TokenSpan[]）を hljs span 付き HTML 断片へ整形する。隣接する同一クラス（または共に null）の断片は結合する。
function renderLine(spans, line) {
  let html = '';
  let runClass = undefined; // 現在の結合中ランのクラス（null=プレーン, string=span）
  let runText = '';

  const flush = () => {
    if (runText.length === 0 && runClass === undefined) return;
    if (runClass) {
      html += `<span class="${runClass}">${escapeText(runText)}</span>`;
    } else {
      html += escapeText(runText);
    }
    runText = '';
    runClass = undefined;
  };

  for (const s of spans) {
    const text = line.slice(s.startIndex, s.endIndex);
    if (text.length === 0) continue;
    const cls = scopesToClass(s.scopes); // string | null
    if (runClass === undefined) {
      runClass = cls;
      runText = text;
    } else if (cls === runClass) {
      runText += text;
    } else {
      flush();
      runClass = cls;
      runText = text;
    }
  }
  flush();
  return html;
}

/**
 * HtmlHighlighter を生成する factory。
 * `load(paths)` → `highlightHtmlString(html)` / `highlightHtmlFiles(dir)` を提供する。
 *
 * @returns {{
 *   load: (paths?: {pasta:string, lua:string}) => Promise<void>,
 *   highlightHtmlString: (html: string) => string,
 *   highlightHtmlFiles: (bookOutDir: string, opts?: {write?: boolean}) => {
 *     filesProcessed: number, blocksColored: number, files: string[]
 *   },
 * }}
 */
export function createHtmlHighlighter() {
  const tokenizer = createPastaTokenizer();
  let loaded = false;

  async function load(paths = GRAMMAR_PATHS) {
    await tokenizer.load(paths);
    loaded = true;
  }

  function ensureLoaded() {
    if (!loaded) {
      throw new Error('HtmlHighlighter: load() を先に await してください');
    }
  }

  // 1 つの language-pasta ブロック本文を着色済み HTML 本文へ変換する。
  function highlightBlockInner(innerHtml) {
    const source = blockInnerToSource(innerHtml);
    const lines = source.split('\n');
    const tokenized = tokenizer.tokenizeText(source); // TokenSpan[][]（行数一致）
    const rendered = lines.map((line, i) => renderLine(tokenized[i] || [], line));
    return rendered.join('\n');
  }

  // HTML 文字列中の language-pasta ブロックを着色し、{ html, blocksColored } を返す。非 pasta・ブロック外はバイト不変。
  function colorize(html) {
    ensureLoaded();
    let blocksColored = 0;
    const out = html.replace(PASTA_BLOCK_RE, (_m, openTag, inner, closeTag) => {
      blocksColored++;
      return openTag + highlightBlockInner(inner) + closeTag;
    });
    return { html: out, blocksColored };
  }

  function highlightHtmlString(html) {
    return colorize(html).html;
  }

  // book 出力ディレクトリ配下の *.html を再帰グロブし、language-pasta ブロックを
  // in-place 書き換えする（Batch Contract）。部分書き込み回避のためファイル単位で
  // 全変換後に書き込む。失敗時は例外を送出（CLI 層で exit 1）。
  function highlightHtmlFiles(bookOutDir, { write = true } = {}) {
    ensureLoaded();
    if (!fs.existsSync(bookOutDir) || !fs.statSync(bookOutDir).isDirectory()) {
      throw new Error(`book out dir not found or not a directory: ${bookOutDir}`);
    }
    const files = listHtmlFiles(bookOutDir);
    let filesProcessed = 0;
    let blocksColored = 0;
    for (const file of files) {
      const html = fs.readFileSync(file, 'utf8');
      const { html: out, blocksColored: colored } = colorize(html);
      if (colored > 0 && out !== html && write) {
        fs.writeFileSync(file, out, 'utf8');
      }
      filesProcessed++;
      blocksColored += colored;
    }
    return { filesProcessed, blocksColored, files };
  }

  return { load, highlightHtmlString, highlightHtmlFiles };
}

// <book-out-dir> 配下の *.html を再帰列挙する（決定論のためソート）。
function listHtmlFiles(dir) {
  const out = [];
  const walk = (d) => {
    const entries = fs.readdirSync(d, { withFileTypes: true })
      .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
    for (const ent of entries) {
      const full = path.join(d, ent.name);
      if (ent.isDirectory()) {
        walk(full);
      } else if (ent.isFile() && ent.name.toLowerCase().endsWith('.html')) {
        out.push(full);
      }
    }
  };
  walk(dir);
  return out;
}

// --- モジュール直 export（テスト用の薄い純関数ラッパ） ---------------------
// load 済み highlighter を内部生成して 1 回限りの変換を行う非同期版（主にテスト用）。

/**
 * 文字列 HTML を着色して返す（文法ロード込みの非同期ワンショット）。
 * @param {string} html
 * @param {{pasta:string, lua:string}} [paths]
 * @returns {Promise<string>}
 */
export async function highlightHtmlString(html, paths = GRAMMAR_PATHS) {
  const hl = createHtmlHighlighter();
  await hl.load(paths);
  return hl.highlightHtmlString(html);
}

/**
 * book 出力ディレクトリを着色する（文法ロード込みの非同期ワンショット）。
 * @param {string} bookOutDir
 * @param {{paths?:{pasta:string,lua:string}, write?:boolean}} [opts]
 * @returns {Promise<{filesProcessed:number, blocksColored:number, files:string[]}>}
 */
export async function highlightHtmlFiles(bookOutDir, opts = {}) {
  const { paths = GRAMMAR_PATHS, write = true } = opts;
  const hl = createHtmlHighlighter();
  await hl.load(paths);
  return hl.highlightHtmlFiles(bookOutDir, { write });
}

// CLI: node highlight-html.mjs <book-out-dir>（既定 book/book）
// 成功時 exit 0、失敗時は stderr へ診断 ＋ exit 1（mdbook build 後段でビルド失敗扱い）。
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  const dir = process.argv[2] || path.resolve(bookDir, 'book');
  (async () => {
    const hl = createHtmlHighlighter();
    await hl.load(GRAMMAR_PATHS); // 文法/WASM/依存欠落はここで例外
    const { filesProcessed, blocksColored, files } = hl.highlightHtmlFiles(dir);
    console.log(`highlight-html: scanned ${files.length} html file(s) under ${dir}`);
    console.log(`highlight-html: colored ${blocksColored} language-pasta block(s) across ${filesProcessed} file(s)`);
    // 0 件は入力の取り違えか出力形の変化で着色が空回りしている。
    if (blocksColored === 0) {
      console.error('highlight-html: language-pasta のブロックを 1 つも着色しなかった');
      process.exitCode = 1;
    }
  })().catch((e) => {
    console.error(`highlight-html failed: ${e && e.stack ? e.stack : e}`);
    process.exitCode = 1;
  });
}
