// link-check.mjs — マニュアル（book/src）のリンク切れ検出（git 非依存）
// manual-ssot-authority タスク 3.1（要件 8.1, 8.4 / design「LinkCheck」）。
//
// 検出（book/src/**/*.md を走査）:
//   (a) book 内相対 .md リンクが実在ファイルを指すか、
//   (b) リポジトリ内を指す GitHub blob/tree URL が
//       ローカルに実在するか（オフラインでローカル照合）、
//   (c) (a)(b) とも repoRoot 外へ脱出するパス（トラバーサル）は実在しても違反。
//
// 内部設計章のパス実在検査（pasta-runtime-internals-doc タスク 1.1・要件 3.7）:
//   book/src/internals/**/*.md のフェンス外インラインコードに書いたリポジトリ内パスを
//   checkInternalsPaths で検査し、[1] の区分に internals-path として報告する。
//
// スキル自己完結検査（タスク 3.2・要件 5.3, 6.1, 6.4, 6.5, 10.2, 2.4）:
//   .claude/skills/{pasta-ghost-authoring,pasta-lua-coding}/**/*.md を checkSkillSelfContained で
//   検査する（skill-escape / skill-missing / skill-anchor / skill-forbidden-ref / skill-unlisted）。
//   アンカーは GitHub 方式の headingSlug（生成対象 21 章で mdBook の id と一致を確認済み）と
//   `<a id|name>` の明示アンカーで照合する。
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
  // 例: https://github.com/ekicyou/pasta/blob/main/book/src/introduction.md
  //     https://github.com/ekicyou/pasta/tree/main/book/src
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

// ---- 見出し slug（GitHub 方式） ----
// 見出し記号と前後空白を除いたテキストについて、(1) インラインコードのバッククォートを外し
// リンクは表示テキストへ、(2) 小文字化、(3) 文字・結合文字・数字と `_`・`-`・空白以外を除去、
// (4) 空白 1 文字を `-` 1 文字へ（連続空白は畳まない）。重複の付番は headingSlugs が行う。
export function headingSlug(heading) {
  return heading
    .trim()
    .replace(/^#{1,6}(?:\s+|$)/, '')
    .replace(/\s+#+\s*$/, '')
    .trim()
    .replace(/`+/g, '')
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1')
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N}_\- ]/gu, '')
    .replace(/ /g, '-');
}

// 文書のフェンス外 ATX 見出し（#〜######）の slug を出現順に返す。
// 同一文書内の重複には出現順に -1・-2 … を付ける（GitHub・mdBook 共通の付番）。
const HEADING_RE = /^ {0,3}#{1,6}(?:[ \t]+|$)/;
export function headingSlugs(markdown) {
  const seen = new Map();
  const out = [];
  for (const line of maskFences(markdown).split('\n')) {
    if (!HEADING_RE.test(line)) continue;
    const base = headingSlug(line);
    let slug = base;
    let n = seen.get(base) || 0;
    while (seen.has(slug)) slug = `${base}-${++n}`;
    seen.set(base, n);
    if (slug !== base) seen.set(slug, 0);
    out.push(slug);
  }
  return out;
}

// ---- スキル自己完結検査（2 スキル対象） ----
export const CHECKED_SKILLS = Object.freeze(['pasta-ghost-authoring', 'pasta-lua-coding']);
export const FORBIDDEN_SKILL_TOKENS = Object.freeze(['doc/spec', 'GRAMMAR.md', 'book/src', 'crates/']);

// アンカー集合: フェンス外見出しの slug ∪ `<a id|name="…">` の明示アンカー。
function anchorSet(markdown) {
  const masked = maskFences(markdown);
  const set = new Set(headingSlugs(markdown));
  for (const m of masked.matchAll(/<a\s[^>]*\b(?:id|name)\s*=\s*["']([^"']+)["']/gi)) set.add(m[1]);
  return set;
}

function decodeAnchor(a) {
  try {
    return decodeURIComponent(a);
  } catch {
    return a;
  }
}

// 規則（リンク抽出はフェンス外のみ）:
//   (a) 相対リンクはスキルディレクトリ内に解決され（skill-escape）、実在する（skill-missing）。
//   (b) *.md#anchor・#anchor はリンク先のアンカー集合に含まれる（skill-anchor）。
//   (c) ファイル全文（HTML コメント・フェンス内を含む）に禁止トークンが無い（skill-forbidden-ref）。
//   (d) references/*.md は同じスキルの SKILL.md から 1 回以上リンクされる（skill-unlisted）。
// スキルディレクトリが無ければ検査しない。
export function checkSkillSelfContained(repoRoot = REPO_ROOT) {
  const broken = [];
  const rel = (abs) => path.relative(repoRoot, abs).split(path.sep).join('/');
  const anchorCache = new Map();
  const anchorsOf = (abs) => {
    if (!anchorCache.has(abs)) anchorCache.set(abs, anchorSet(fs.readFileSync(abs, 'utf8')));
    return anchorCache.get(abs);
  };

  for (const skill of CHECKED_SKILLS) {
    const skillDir = path.resolve(repoRoot, '.claude/skills', skill);
    if (!fs.existsSync(skillDir)) continue;
    const skillMd = path.join(skillDir, 'SKILL.md');
    const linkedFromSkillMd = new Set();

    for (const file of listMarkdownFiles(skillDir).sort()) {
      const text = fs.readFileSync(file, 'utf8');
      const relFile = rel(file);

      // (c) 全文検査（行ごと・語ごとに 1 件）。
      text.replace(/\r\n?/g, '\n').split('\n').forEach((line, i) => {
        for (const tok of FORBIDDEN_SKILL_TOKENS) {
          if (line.includes(tok)) {
            broken.push({ file: relFile, target: tok, kind: 'skill-forbidden-ref',
              detail: `L${i + 1}: スキル外（リポジトリ内）参照の語 "${tok}" を含む` });
          }
        }
      });

      for (const rawTarget of extractLinks(maskFences(text))) {
        const t = rawTarget.trim();
        if (t === '' || /^[a-z][a-z0-9+.-]*:/i.test(t) || t.startsWith('//')) continue; // 絶対 URL
        const h = t.indexOf('#');
        const pathPart = stripFragment(t);
        const anchor = h >= 0 ? decodeAnchor(t.slice(h + 1)) : '';
        const abs = pathPart === '' ? file : path.resolve(path.dirname(file), decodeAnchor(pathPart));

        // (a)
        if (!isWithinRoot(skillDir, abs)) {
          broken.push({ file: relFile, target: rawTarget, kind: 'skill-escape',
            detail: `スキルディレクトリ外を指す: ${pathPart}` });
          continue;
        }
        if (!fs.existsSync(abs)) {
          broken.push({ file: relFile, target: rawTarget, kind: 'skill-missing',
            detail: `リンク先が存在しない: ${pathPart}` });
          continue;
        }
        if (file === skillMd) linkedFromSkillMd.add(abs);

        // (b)
        if (anchor !== '' && abs.endsWith('.md') && !anchorsOf(abs).has(anchor)) {
          broken.push({ file: relFile, target: rawTarget, kind: 'skill-anchor',
            detail: `リンク先 ${rel(abs)} に見出し／明示アンカー "${anchor}" が無い` });
        }
      }
    }

    // (d)
    const refDir = path.join(skillDir, 'references');
    if (fs.existsSync(refDir)) {
      for (const name of fs.readdirSync(refDir).sort()) {
        const abs = path.join(refDir, name);
        if (!name.endsWith('.md') || !fs.statSync(abs).isFile()) continue;
        if (!linkedFromSkillMd.has(abs)) {
          broken.push({ file: rel(abs), target: '', kind: 'skill-unlisted',
            detail: `${rel(skillMd)} からリンクされていない（区分表への記載漏れ／削除し忘れ）` });
        }
      }
    }
  }
  return broken;
}

// ---- 内部設計章のリポジトリ内パス実在検査（internals-path） ----
// pasta-runtime-internals-doc タスク 1.1（要件 3.2, 3.7, 1.6）。
// book/src/internals/**/*.md のフェンス外インラインコード（同数のバッククォートで閉じる区間）の
// 中身（前後空白除去）のうち、空白を含まず REPO_PATH_PREFIXES で始まるものを repoRoot 基準で照合する。
// 違反: repoRoot 外へ解決される（トラバーサル）、実在しない（末尾 `/` はディレクトリとして実在すること）。
// `:行番号` 付き・ワイルドカード入りは OS（Windows の代替データストリーム等）に依らず実在しない扱い。
export const INTERNALS_DIR = 'book/src/internals';
export const REPO_PATH_PREFIXES = Object.freeze(['crates/', 'book/', '.github/', '.cargo/', '.kiro/', '.claude/']);
const CODE_SPAN_RE = /(?<!`)(`+)(?!`)(.+?)(?<!`)\1(?!`)/g;

export function checkInternalsPaths(repoRoot = REPO_ROOT) {
  const broken = [];
  for (const file of listMarkdownFiles(path.resolve(repoRoot, INTERNALS_DIR)).sort()) {
    const relFile = path.relative(repoRoot, file).split(path.sep).join('/');
    maskFences(fs.readFileSync(file, 'utf8')).split('\n').forEach((line, i) => {
      for (const m of line.matchAll(CODE_SPAN_RE)) {
        const p = m[2].trim();
        if (/\s/.test(p) || !REPO_PATH_PREFIXES.some((pre) => p.startsWith(pre))) continue;
        const abs = path.resolve(repoRoot, p);
        let problem = null;
        if (!isWithinRoot(repoRoot, abs)) problem = 'リポジトリ外を指すパス（トラバーサル）';
        else if (/[:*?]/.test(p) || !fs.existsSync(abs)) problem = 'リポジトリ内に実在しない';
        else if (p.endsWith('/') && !fs.statSync(abs).isDirectory()) problem = '末尾 / だがディレクトリでない';
        if (problem) {
          broken.push({ file: relFile, target: p, kind: 'internals-path', detail: `L${i + 1}: ${problem}: ${p}` });
        }
      }
    });
  }
  return broken;
}

// ---- オーケストレーション ----
const BOOK_KINDS = new Set(['internal-md', 'github-repo-path', 'internals-path']);

export function runLinkCheck(repoRoot = REPO_ROOT) {
  const broken = [
    ...detectBrokenLinks(repoRoot),
    ...checkInternalsPaths(repoRoot),
    ...checkSkillSelfContained(repoRoot),
  ];
  return { broken, failed: broken.length > 0 };
}

// 結果を標準出力へ分類表示する（[1] book 内リンク切れ・[2] スキル自己完結を種別ごと）。
export function reportLinkCheck(result) {
  const { broken } = result;
  const book = broken.filter((b) => BOOK_KINDS.has(b.kind));
  const skill = broken.filter((b) => !BOOK_KINDS.has(b.kind));
  const out = [];
  const list = (items) => {
    for (const b of items) {
      out.push(`  BROKEN  ${b.file}${b.target ? `  ->  ${b.target}` : ''}`);
      out.push(`          [${b.kind}] ${b.detail}`);
    }
  };
  out.push('link-check (git 非依存)');
  out.push('');
  out.push(`[1] リンク切れ（book 内 .md / リポジトリ内 GitHub URL / 内部設計章のパス）: ${book.length} 件`);
  list(book);
  out.push('');
  out.push(`[2] スキル自己完結（${CHECKED_SKILLS.join(', ')}）: ${skill.length} 件`);
  for (const kind of ['skill-escape', 'skill-missing', 'skill-anchor', 'skill-forbidden-ref', 'skill-unlisted']) {
    const items = skill.filter((b) => b.kind === kind);
    if (items.length === 0) continue;
    out.push(` ${kind}: ${items.length} 件`);
    list(items);
  }
  out.push('');
  out.push(result.failed ? 'RESULT: FAIL（違反あり）' : 'RESULT: OK');
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
