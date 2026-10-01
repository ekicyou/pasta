// link-check.mjs — マニュアル（book/src）のリンク切れ検出（git 非依存）
// manual-ssot-authority タスク 3.1（要件 8.1, 8.4 / design「LinkCheck」）。
// 旧 drift-check.mjs からドリフト検出（doc/spec ハッシュ）・未マップ検出・
// manual-sources.toml 解析を撤去し、リンク切れ検出だけを残した縮小版。
//
// 検出（book/src/**/*.md を走査）:
//   (a) book 内相対 .md リンクが実在ファイルを指すか、
//   (b) リポジトリ内を指す GitHub blob/tree URL が
//       ローカルに実在するか（オフラインでローカル照合）、
//   (c) (a)(b) とも repoRoot 外へ脱出するパス（トラバーサル）は実在しても違反。
//
// 共用 export（gen-skill-refs.mjs が import する）:
//   LINK_RE    … インラインリンク `[text](target)` の正規表現（先頭キャプチャ=リンク先）
//   maskFences … CommonMark 準拠でコードフェンス内の行を空行に置換（行数は保つ）
//
// 終了コード: 違反あり → exit 1 / 無し → exit 0 / 予期しない例外 → exit 2。
// 冪等・決定論的（同一入力 → 同一結果）。book/src は読み取りのみ。

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
// book/tools/ から 2 つ上がリポジトリルート。
export const REPO_ROOT = path.resolve(here, '../..');

// このリポジトリの GitHub slug（blob/tree URL のローカル照合に使う）。
export const REPO_SLUG = 'ekicyou/pasta';

// Markdown のインラインリンク `[text](target)`。
// target は丸括弧を含まない範囲。タイトル付き `(url "title")` も先頭トークンを取る。
// 山括弧 `<...>` は外さずにキャプチャする（extractLinks が外す）。
export const LINK_RE = /\]\(\s*(<[^>]+>|[^()\s]+)/g;

// ---- コードフェンス判定（CommonMark 準拠） ----
// 開き: 0〜3 スペースのインデント＋ 3 個以上の ` または ~（` の場合、情報文字列に ` を含まない）。
// 閉じ: 0〜3 スペースのインデント＋開きと同じ文字を開き以上の個数＋末尾空白のみ。
// 閉じないフェンスは文書末尾まで続く。フェンス内の行（開閉行を含む）を空行に置換し、
// 行数は保つ。改行は LF に正規化して返す。
const FENCE_RE = /^ {0,3}(`{3,}|~{3,})(.*)$/;
export function maskFences(markdown) {
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  let fence = null; // 開きフェンスの { ch, len }
  return lines.map((line) => {
    const m = line.match(FENCE_RE);
    if (fence === null) {
      if (m && !(m[1][0] === '`' && m[2].includes('`'))) {
        fence = { ch: m[1][0], len: m[1].length };
        return '';
      }
      return line;
    }
    if (m && m[1][0] === fence.ch && m[1].length >= fence.len && m[2].trim() === '') {
      fence = null;
    }
    return '';
  }).join('\n');
}

// ---- リンク切れ検出（book/src 対象） ----
// book/src/**/*.md を走査し、相対 .md リンクと GitHub blob/tree URL を検証する。
export function listMarkdownFiles(dir) {
  const out = [];
  if (!fs.existsSync(dir)) return out;
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) out.push(...listMarkdownFiles(full));
    else if (ent.isFile() && ent.name.endsWith('.md')) out.push(full);
  }
  return out;
}

// Markdown のインラインリンク `[text](target)` を抽出する。
// 画像 `![...]` も同形式なので拾うが、リンク切れ判定上は同じ扱いでよい。
export function extractLinks(markdown) {
  const links = [];
  // matchAll は正規表現を複製するので共有 LINK_RE の lastIndex に依存しない。
  for (const m of markdown.matchAll(LINK_RE)) {
    let target = m[1];
    if (target.startsWith('<') && target.endsWith('>')) {
      target = target.slice(1, -1);
    }
    links.push(target);
  }
  return links;
}

// リンクからフラグメント（#...）とクエリ（?...）を落として実体パス部を返す。
function stripFragment(target) {
  let t = target;
  const h = t.indexOf('#');
  if (h >= 0) t = t.slice(0, h);
  const q = t.indexOf('?');
  if (q >= 0) t = t.slice(0, q);
  return t;
}

// GitHub blob/tree URL がこのリポジトリ内を指す場合、対応するローカルパスを返す。
// 対象外（別リポ・外部サイト・branch 名にスラッシュ等）は null。
export function githubUrlToRepoPath(url) {
  // 例: https://github.com/ekicyou/pasta/blob/main/doc/spec/02-markers.md
  //     https://github.com/ekicyou/pasta/tree/main/doc/spec
  const re = new RegExp(
    `^https?://github\\.com/${REPO_SLUG}/(?:blob|tree)/[^/]+/(.+)$`,
  );
  const m = url.match(re);
  if (!m) return null;
  return m[1];
}

// 解決済み絶対パスが repoRoot 配下に留まるか（`..` 等による境界脱出の検出）。
// repoRoot 自身は配下とみなす。脱出していれば false。
function isWithinRoot(repoRoot, absPath) {
  const rel = path.relative(path.resolve(repoRoot), absPath);
  return rel === '' || (!rel.startsWith('..') && !path.isAbsolute(rel));
}

export function detectBrokenLinks(repoRoot = REPO_ROOT) {
  const srcDir = path.resolve(repoRoot, 'book/src');
  const files = listMarkdownFiles(srcDir);
  const broken = [];

  for (const file of files) {
    const text = fs.readFileSync(file, 'utf8');
    const relFile = path.relative(repoRoot, file).split(path.sep).join('/');
    for (const rawTarget of extractLinks(text)) {
      const target = stripFragment(rawTarget).trim();
      if (target === '') continue;

      // (b) このリポジトリ内を指す GitHub blob/tree URL → ローカル照合。
      const repoPath = githubUrlToRepoPath(target);
      if (repoPath !== null) {
        const abs = path.resolve(repoRoot, repoPath);
        // ハードニング: `..` 等で repoRoot 外へ脱出するパスは、実在有無に
        // かかわらずリンク切れとして報告する（リポジトリ外の存在プローブ防止）。
        if (!isWithinRoot(repoRoot, abs)) {
          broken.push({
            file: relFile,
            target: rawTarget,
            kind: 'github-repo-path',
            detail: `リポジトリ外を指すパス（トラバーサル）: ${repoPath}`,
          });
          continue;
        }
        if (!fs.existsSync(abs)) {
          broken.push({
            file: relFile,
            target: rawTarget,
            kind: 'github-repo-path',
            detail: `リポジトリ内に実在しない: ${repoPath}`,
          });
        }
        continue;
      }

      // その他の絶対 URL（外部サイト・別リポ）はオフライン照合対象外＝スキップ。
      if (/^[a-z][a-z0-9+.-]*:\/\//i.test(target) || target.startsWith('//')) {
        continue;
      }
      // mailto: 等のスキームもスキップ。
      if (/^[a-z][a-z0-9+.-]*:/i.test(target)) continue;

      // (a) book 内相対リンク。.md（または .md 配下のアンカー）の実在を確認する。
      //     book 内資産は .md リンクのみが移動対象。画像等は対象外として .md に限定。
      if (target.endsWith('.md')) {
        const abs = path.resolve(path.dirname(file), target);
        // ハードニング: 相対リンクが repoRoot 外へ脱出する場合も、実在有無に
        // かかわらずリンク切れとして報告する（repoRoot 内の `..` 参照は従来どおり許容）。
        if (!isWithinRoot(repoRoot, abs)) {
          broken.push({
            file: relFile,
            target: rawTarget,
            kind: 'internal-md',
            detail: `リポジトリ外を指すリンク（トラバーサル）: ${target}`,
          });
          continue;
        }
        if (!fs.existsSync(abs)) {
          broken.push({
            file: relFile,
            target: rawTarget,
            kind: 'internal-md',
            detail: `book 内リンク先が存在しない: ${target}`,
          });
        }
      }
    }
  }
  return broken;
}

// ---- オーケストレーション ----
export function runLinkCheck(repoRoot = REPO_ROOT) {
  const broken = detectBrokenLinks(repoRoot);
  return { broken, failed: broken.length > 0 };
}

// 結果を標準出力へ分類表示する。
export function reportLinkCheck(result) {
  const { broken } = result;
  const out = [];
  out.push('link-check (git 非依存)');
  out.push('');
  out.push(`[1] リンク切れ（book 内 .md / リポジトリ内 GitHub URL）: ${broken.length} 件`);
  for (const b of broken) {
    out.push(`  BROKEN  ${b.file}  ->  ${b.target}`);
    out.push(`          [${b.kind}] ${b.detail}`);
  }
  out.push('');
  out.push(result.failed ? 'RESULT: FAIL（リンク切れあり）' : 'RESULT: OK');
  return out.join('\n');
}

// CLI: node link-check.mjs
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  try {
    const result = runLinkCheck(REPO_ROOT);
    console.log(reportLinkCheck(result));
    process.exit(result.failed ? 1 : 0);
  } catch (e) {
    console.error(`link-check failed: ${e && e.stack ? e.stack : e}`);
    process.exit(2);
  }
}
