// highlight-html.mjs — HtmlHighlighter（タスク 3.1 / 要件 1.1, 1.4, 4.1, 4.3, 6.1, 6.2, 6.4, 6.5, 7.1）。
//
// 役割（design.md「HtmlHighlighter」Batch / Job Contract）:
//   - mdbook build 後段で動作。`<book-out-dir>/**/*.html` をグロブ列挙し、
//     `<code class="language-pasta">…</code>` ブロックのみを処理する。
//     注: 抽出は `<code>` のクラス基準であり、外側の `<pre>` は要求しない。mdBook が
//     language-pasta クラスを与えるのはフェンスブロック（`<pre>` 内）のみだが、もし
//     インライン `<code class="language-pasta">` が現れた場合も同様に着色対象となる
//     （現 book 出力では非発生・テストで挙動固定済み）。
//   - 各ブロック内容をタグ除去＋実体参照デコードでソース復元 → 行分割 →
//     PastaTokenizer でトークナイズ → scopesToClass で hljs 互換クラスへ写像 →
//     非 null はそのクラスの <span> で包み、null は素のテキスト（プレーン＝6.1）→
//     `& < > "` を再エスケープして emit。
//   - 他言語（language-rust 等）・無指定 <code>・インライン <code>・ブロック外 HTML は
//     バイト単位で不変（1.4）。
//   - 純静的出力のみ（4.1/4.3）。成果物に WASM・文法・node_modules を出さない。
//   - 失敗（6.2）: 文法ロード/トークナイズ例外/必須 build-time 依存欠落で process.exit(1)
//     ＋原因診断を stderr。部分書き込みを避けるためファイル単位で全変換後に書き込む。
//
// 冪等の素地（6.5・research §8.4）:
//   ソース復元時に既存 hljs <span> をタグ除去するため、再実行は常にプレーンテキストから
//   再生成する＝二重 span を生まない構造。厳密なバイト等価冪等テストは後続 3.2 の領分。
//
// 決定論（6.4）: 同一入力 HTML → 同一トークン列 → 同一バイト出力（bigram ツールと同契約）。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createPastaTokenizer } from './tokenizer.mjs';
import { scopesToClass } from './scope-map.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const bookDir = path.resolve(here, '../..'); // book/
const repoRoot = path.resolve(bookDir, '..'); // リポジトリルート

// 文法パス（スクリプト位置基準で解決・既存 tokenizer-test.mjs の repoRoot 解決に倣う）。
//   pasta: SSOT（読み取り専用・改変禁止）。lua: vendor（入れ子 lua の二段化用）。
export const GRAMMAR_PATHS = Object.freeze({
  pasta: path.resolve(repoRoot, 'editors/vscode/syntaxes/pasta.tmLanguage.json'),
  lua: path.resolve(here, 'grammars/lua.tmLanguage.json'),
});

// language-pasta コードブロックを抽出する正規表現。
//   - `<code ... class="… language-pasta …">…</code>` を捕捉（外側 `<pre>` は不問。
//     ヘッダコメント参照 — インライン <code> でも同クラスなら処理対象）。
//   - class 属性内で language-pasta が他クラスと併記される形にも頑健（\b 区切り）。
//   - `</code>` がブロック内に実体参照化されずに出現しない前提で安全に区切る
//     （design Batch Contract）。中身は最短一致で取る。
//   捕捉群: [1]=開始 <code ...> タグ全体, [2]=ブロック本文, [3]=閉じ <code> 直前まで不要。
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

// プレーンテキストを HTML テキストノード用に再エスケープする。
//   `&` を最初に処理（先に変換した実体参照を二重エスケープしないため）。
//   `"` も再エスケープして属性混入を防ぐ（research §8.4 の「必要なら "」）。
function escapeText(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ブロック本文（<code>…</code> の内側 HTML）からソース文字列を復元する。
//   1) 既存タグ（hljs <span> 等）を除去 ＝ 再実行でも常にプレーンから再生成（冪等の素地）。
//   2) 実体参照をデコードして元ソースのテキストへ戻す。
function blockInnerToSource(innerHtml) {
  const noTags = innerHtml.replace(/<[^>]*>/g, '');
  return decodeEntities(noTags);
}

// トークナイズ済みの 1 行（TokenSpan[]）を hljs span 付き HTML 断片へ整形する。
//   - scopesToClass が非 null のトークンは <span class="hljs-…">…</span>。
//   - null のトークンは素のテキスト（プレーン＝6.1）。
//   - 隣接する同一クラス（または共に null）の断片は 1 つの span/テキストへ結合する
//     （トークン境界に沿った結合＝決定論的・出力最小化）。
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
    // 不在・不正は tokenizer.load 内で例外（6.2: 文法ロード/WASM/依存欠落）。
    await tokenizer.load(paths);
    loaded = true;
  }

  function ensureLoaded() {
    if (!loaded) {
      throw new Error('HtmlHighlighter: load() を先に await してください');
    }
  }

  // 1 つの language-pasta ブロック本文を着色済み HTML 本文へ変換する。
  // ソース復元（タグ除去＋デコード）→ tokenizeText → renderLine を行連結。
  function highlightBlockInner(innerHtml) {
    const source = blockInnerToSource(innerHtml);
    const lines = source.split('\n');
    const tokenized = tokenizer.tokenizeText(source); // TokenSpan[][]（行数一致）
    const rendered = lines.map((line, i) => renderLine(tokenized[i] || [], line));
    return rendered.join('\n');
  }

  // HTML 文字列中の language-pasta ブロックを着色し、{ html, blocksColored } を返す内部関数。
  // 非 pasta・ブロック外はバイト不変（1.4）。決定論的（6.4）。
  function colorize(html) {
    ensureLoaded();
    let blocksColored = 0;
    const out = html.replace(PASTA_BLOCK_RE, (_m, openTag, inner, closeTag) => {
      blocksColored++;
      return openTag + highlightBlockInner(inner) + closeTag;
    });
    return { html: out, blocksColored };
  }

  // HTML 文字列中の language-pasta ブロックのみを着色して返す（純粋・同期・string→string）。
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
// node_modules / grammars 等は book 出力には存在しない前提だが、安全に走査する。
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
// load 済み highlighter を内部生成して 1 回限りの変換を行う非同期版。
// （主用途はテストの利便。実運用は createHtmlHighlighter を使う。）

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
    await hl.load(GRAMMAR_PATHS); // 文法/WASM/依存欠落はここで例外（6.2）
    const { filesProcessed, blocksColored, files } = hl.highlightHtmlFiles(dir);
    console.log(`highlight-html: scanned ${files.length} html file(s) under ${dir}`);
    console.log(`highlight-html: colored ${blocksColored} language-pasta block(s) across ${filesProcessed} file(s)`);
  })().catch((e) => {
    console.error(`highlight-html failed: ${e && e.stack ? e.stack : e}`);
    process.exitCode = 1;
  });
}
