// drift-check.mjs — マニュアル↔doc/spec ドリフト検出（マーカー方式・git 非依存）
// 本実装 / タスク 4.1（要件 10.2, 10.4 / design「Drift Detection & Gate」）。
//
// 役割（design.md「Drift Detection & Gate」Batch Contract）:
//   Input  = book/manual-sources.toml（記録ハッシュ＋マッピング）
//            ＋ doc/spec/ 現状 ＋ book/src/ 現状。git 差分には依存しない。
//   Output = OK / ドリフト・未マップ・リンク切れ一覧（標準出力に分類表示）。
//            ゲート文脈では非ゼロ終了で完了中断。
//
// 検出は 3 種:
//   ① ドリフト（10.2）: manual-sources.toml の各 mapping について、source
//      （doc/spec/*.md）の現値 sha256 を算出し、記録 hash と比較。不一致＝ドリフト。
//      ＝「参照元が変わったのにマニュアル章が追従していない」状態。git の分岐点・
//      diff base に依存せず、CI でも完了ゲートでも同一に動作する。
//   ② 未マップ（10.2）: doc/spec/*.md を列挙し、マッピングに無い章を警告する
//      （マッピング漏れによる検出すり抜け防止）。ただし網羅対象外
//      （未実装 ch08 / 将来 ch12 / README）は除外する（manual-sources.toml 注記準拠）。
//   ③ リンク切れ（10.4）: book/src/**/*.md を走査し、
//      (a) book 内相対 .md リンクが実在ファイルを指すか、
//      (b) リポジトリ内を指す GitHub blob/tree URL（doc/spec・crates 等）が
//          ローカルに実在するか（オフラインでローカル照合）を検証する。
//
// 終了コード（design「非ゼロ終了で完了中断」/ 実装方針）:
//   - ドリフト ① あり          → exit 1（完了中断対象）
//   - リンク切れ ③ あり        → exit 1（壊れた参照は中断対象）
//   - 未マップ ② のみ          → warning。検出漏れ防止のため必ず report するが、
//                                既定では exit コードに寄与しない（網羅対象外章の
//                                増加だけで完了をブロックしないため）。
//                                ※環境変数 DRIFT_STRICT=1 で未マップも exit 1 にできる。
//   - いずれも無し             → exit 0
//
// 冪等・決定論的（同一入力 → 同一結果）。doc/spec・book/src は読み取りのみ。

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
// book/tools/ から 2 つ上がリポジトリルート。
export const REPO_ROOT = path.resolve(here, '../..');

// このリポジトリの GitHub slug（blob/tree URL のローカル照合に使う）。
export const REPO_SLUG = 'ekicyou/pasta';

// 未マップ検出の網羅対象外ファイル（manual-sources.toml 注記 / 要件 4.6）。
//   - 08-attributes.md : 属性=未実装
//   - 12-future.md     : 将来仕様
//   - README.md        : 章ではなく目次/索引
export const UNMAPPED_EXCLUDE = new Set([
  '08-attributes.md',
  '12-future.md',
  'README.md',
]);

// ---- sha256（改行正規化後の内容ハッシュ。manual-sources.toml の方式に一致） ----
// doc/spec は UTF-8 テキスト。CRLF↔LF の改行差はドリフト（内容変化）ではないため、
// ハッシュ前に改行を LF へ正規化する。これにより core.autocrlf=true の Windows
// 作業コピー（CRLF）でも、git 格納・CI チェックアウト（LF）でも同一ハッシュになり、
// 改行コード差による誤検出（ローカル成功・CI 失敗の乖離）を防ぐ。
export function sha256File(absPath) {
  const text = fs.readFileSync(absPath, 'utf8').replace(/\r\n?/g, '\n');
  return crypto.createHash('sha256').update(text, 'utf8').digest('hex');
}

// ---- manual-sources.toml の最小パーサ ----
// 外部依存を増やさないため、本ファイルの構造（algorithm 行 ＋ [[mapping]] 配列で
// chapter/source/hash の各キー = "値"）に特化した最小 TOML パーサを自前実装する。
// 文字列値はダブルクォート囲み。コメント（# 始まり）・空行は無視する。
export function parseManualSources(tomlText) {
  const lines = tomlText.split(/\r?\n/);
  let algorithm = null;
  const mappings = [];
  let current = null;

  const flush = () => {
    if (current) {
      mappings.push(current);
      current = null;
    }
  };

  for (const raw of lines) {
    const line = stripComment(raw).trim();
    if (line === '') continue;

    if (line === '[[mapping]]') {
      flush();
      current = {};
      continue;
    }

    const m = line.match(/^([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (!m) continue;
    const key = m[1];
    const value = parseTomlValue(m[2]);

    if (current) {
      current[key] = value;
    } else if (key === 'algorithm') {
      algorithm = value;
    }
  }
  flush();

  return { algorithm, mappings };
}

// 行末コメントを除去する。ただしダブルクォート文字列内の # は保持する。
function stripComment(line) {
  let inStr = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') inStr = !inStr;
    else if (c === '#' && !inStr) return line.slice(0, i);
  }
  return line;
}

// TOML 値（本ファイルではダブルクォート文字列のみ）をデコードする。
function parseTomlValue(token) {
  const t = token.trim();
  if (t.startsWith('"') && t.endsWith('"') && t.length >= 2) {
    return t
      .slice(1, -1)
      .replace(/\\"/g, '"')
      .replace(/\\\\/g, '\\');
  }
  return t;
}

// ---- ドリフト検出（①・要件 10.2） ----
// 各 mapping の source 現値ハッシュと記録 hash を比較する。
export function detectDrift(manualSources, repoRoot = REPO_ROOT) {
  const drift = [];
  for (const map of manualSources.mappings) {
    const src = map.source;
    const recorded = map.hash;
    if (!src || !recorded) {
      drift.push({
        chapter: map.chapter,
        source: src,
        reason: 'incomplete-mapping',
        detail: 'mapping に source / hash が欠落',
      });
      continue;
    }
    const abs = path.resolve(repoRoot, src);
    if (!fs.existsSync(abs)) {
      drift.push({
        chapter: map.chapter,
        source: src,
        reason: 'missing-source',
        detail: `source ファイルが存在しない: ${src}`,
      });
      continue;
    }
    const current = sha256File(abs);
    if (current !== recorded) {
      drift.push({
        chapter: map.chapter,
        source: src,
        reason: 'hash-mismatch',
        recorded,
        current,
        detail: '由来 doc/spec が変更されたのに章が未追従（ドリフト）',
      });
    }
  }
  return drift;
}

// ---- 未マップ検出（②・要件 10.2） ----
// doc/spec/*.md を列挙し、mapping の source 集合に無く、かつ網羅対象外でもない
// 章を未マップとして報告する。
export function detectUnmapped(manualSources, repoRoot = REPO_ROOT) {
  const specDir = path.resolve(repoRoot, 'doc/spec');
  if (!fs.existsSync(specDir)) return [];

  const mapped = new Set(
    manualSources.mappings
      .map((m) => m.source && path.normalize(m.source))
      .filter(Boolean),
  );

  const unmapped = [];
  for (const name of fs.readdirSync(specDir).sort()) {
    if (!name.endsWith('.md')) continue;
    if (UNMAPPED_EXCLUDE.has(name)) continue;
    const rel = path.normalize(path.join('doc/spec', name));
    if (!mapped.has(rel)) {
      unmapped.push(rel.split(path.sep).join('/'));
    }
  }
  return unmapped;
}

// ---- リンク切れ検出（③・要件 10.4） ----
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
  // target は丸括弧を含まない範囲。タイトル付き `(url "title")` も先頭トークンを取る。
  const re = /\]\(\s*(<[^>]+>|[^()\s]+)/g;
  let m;
  while ((m = re.exec(markdown)) !== null) {
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
export function runDriftCheck(repoRoot = REPO_ROOT, { strict = false } = {}) {
  const tomlPath = path.resolve(repoRoot, 'book/manual-sources.toml');
  const manualSources = parseManualSources(fs.readFileSync(tomlPath, 'utf8'));

  const drift = detectDrift(manualSources, repoRoot);
  const unmapped = detectUnmapped(manualSources, repoRoot);
  const broken = detectBrokenLinks(repoRoot);

  // ドリフト or リンク切れがあれば失敗。未マップは strict 指定時のみ失敗に寄与。
  const failed = drift.length > 0 || broken.length > 0 || (strict && unmapped.length > 0);

  return { manualSources, drift, unmapped, broken, failed };
}

// 結果を標準出力へ分類表示する。
export function reportDriftCheck(result) {
  const { manualSources, drift, unmapped, broken } = result;
  const out = [];
  out.push('drift-check (マーカー方式・git 非依存)');
  out.push(`  mappings: ${manualSources.mappings.length} 件 / algorithm=${manualSources.algorithm}`);

  out.push('');
  out.push(`[1] ドリフト（記録ハッシュ vs doc/spec 現値）: ${drift.length} 件`);
  for (const d of drift) {
    out.push(`  DRIFT  ${d.chapter || '(章不明)'} <- ${d.source || '(source 不明)'}`);
    out.push(`         ${d.detail}`);
    if (d.reason === 'hash-mismatch') {
      out.push(`         recorded=${d.recorded}`);
      out.push(`         current =${d.current}`);
    }
  }

  out.push('');
  out.push(`[2] 未マップ doc/spec 章（網羅対象外を除く）: ${unmapped.length} 件`);
  for (const u of unmapped) {
    out.push(`  UNMAPPED  ${u}`);
  }

  out.push('');
  out.push(`[3] リンク切れ（book 内 .md / リポジトリ内 GitHub URL）: ${broken.length} 件`);
  for (const b of broken) {
    out.push(`  BROKEN  ${b.file}  ->  ${b.target}`);
    out.push(`          [${b.kind}] ${b.detail}`);
  }

  out.push('');
  if (result.failed) {
    out.push('RESULT: FAIL（未解決ドリフトまたはリンク切れあり → 完了中断）');
  } else if (unmapped.length > 0) {
    out.push('RESULT: OK（ただし未マップ章の警告あり。検出漏れ防止のため確認推奨）');
  } else {
    out.push('RESULT: OK');
  }
  return out.join('\n');
}

// CLI: node drift-check.mjs
//   DRIFT_STRICT=1 で未マップも失敗扱いにする。
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  const strict = process.env.DRIFT_STRICT === '1';
  try {
    const result = runDriftCheck(REPO_ROOT, { strict });
    console.log(reportDriftCheck(result));
    process.exit(result.failed ? 1 : 0);
  } catch (e) {
    console.error(`drift-check failed: ${e && e.stack ? e.stack : e}`);
    process.exit(2);
  }
}
